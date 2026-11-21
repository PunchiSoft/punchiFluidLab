// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests for the shared drawing: the series, the connection circles, the readings and the bounds.

use super::*;
use crate::orient::Orientation;
use crate::surface::{Call, Recording};

#[test]
fn a_series_draws_a_bounded_curve_inside_its_box() {
    // A minute of samples is 600 values; the drawing keeps at most 64 vertices, inside
    // the box, and the newest sample is the right-hand end of the curve.
    // 600 samples fit in `u16`, so the conversion is exact and says so.
    let values: Vec<f32> = (0_u16..600).map(f32::from).collect();
    let catalog = Catalog::builtin();
    let rect = Rect::from_center_size(Pt::new(50.0, 50.0), 24.0, 24.0);
    let placed = [Placed {
        index: 0,
        catalog_id: "chart.recorder",
        rect,
        extension: 0.0,
        valve_position: 0,
        reading: None,
        series: Some(Series {
            values: &values,
            min: 0.0,
            max: 599.0,
        }),
        contact_closed: None,
        orientation: Orientation::NONE,
        designation: None,
        closures: &[],
        exhausts: &[],
    }];
    let mut surface = Recording::default();
    draw_placed(&mut surface, &placed, &[], &catalog, &Ink::print(), &[]);
    let curve = surface
        .calls()
        .iter()
        .filter_map(|call| match call {
            Call::Line { points, .. } => Some(points),
            _ => None,
        })
        .max_by_key(|points| points.len())
        .expect("the recorder draws a line");
    assert!(
        curve.len() <= SERIES_POINTS + 1,
        "the curve has {} vertices, more than the {} the drawing allows",
        curve.len(),
        SERIES_POINTS
    );
    assert!(curve.len() > 2, "a curve is more than one segment");
    assert!(
        curve.iter().all(|point| rect.expand(0.01).contains(*point)),
        "the curve left the recorder's box"
    );
    // The first sample sits on the bottom edge and the last in the top-right corner of
    // the plot, which is what makes the end of the curve the value the reading prints.
    let plot = rect.expand(-rect.height() * 0.14);
    let first = curve.first().expect("a first point");
    assert!(
        (first.y - plot.max.y).abs() < 0.01,
        "the oldest sample is at {first:?}, not on the bottom edge"
    );
    let last = curve.last().expect("a last point");
    assert!(
        (last.x - plot.max.x).abs() < 0.01 && (last.y - plot.min.y).abs() < 0.01,
        "the last sample is at {last:?}, not at the top-right corner of the plot"
    );
}

#[test]
fn every_port_carries_its_connection_circle() {
    // The small open circle is the convention that tells a reader where a hose
    // attaches, so it belongs to every port of every symbol — and it is drawn filled
    // with the paper colour, which is what hides the end of the hose behind it.
    let catalog = Catalog::builtin();
    let rect = Rect::from_center_size(Pt::new(60.0, 60.0), 34.0, 34.0);
    let definition = catalog
        .get("valve.3_2.spring_return")
        .expect("the valve exists");
    let placed = [Placed {
        index: 0,
        catalog_id: definition.id.as_str(),
        rect,
        extension: 0.0,
        valve_position: 0,
        reading: None,
        series: None,
        contact_closed: None,
        orientation: Orientation::NONE,
        designation: None,
        closures: &[],
        exhausts: &[],
    }];
    let mut surface = Recording::default();
    draw_placed(&mut surface, &placed, &[], &catalog, &Ink::print(), &[]);
    let ink = Ink::print();
    for port in &definition.port {
        let point = port_point(rect, Orientation::NONE, definition, &port.id)
            .expect("the port has a point");
        let found = surface.calls().iter().any(|call| match call {
            Call::Circle {
                centre,
                fill,
                stroke,
                ..
            } => {
                (centre.x - point.x).abs() < 0.01
                    && (centre.y - point.y).abs() < 0.01
                    && *fill == Some(ink.background)
                    && stroke.is_some()
            }
            _ => false,
        });
        assert!(
            found,
            "port {} has no connection circle at {point:?}",
            port.id
        );
    }
}

#[test]
fn a_reading_carries_the_unit_of_its_instrument() {
    // The drawing owns the unit and the precision, so every device prints the same
    // number and the front end cannot invent a second format.
    assert_eq!(Reading::Pressure(3.24).text(), "3.2 bar");
    assert_eq!(Reading::Flow(41.0).text(), "41.0 l/min");
    assert_eq!(Reading::Flow(-2.55).text(), "-2.5 l/min");
}

#[test]
fn the_bounds_cover_the_boxes_and_the_pipe_route() {
    let catalog = Catalog::builtin();
    let placements = [
        Placed {
            index: 0,
            catalog_id: "supply.compressor",
            rect: Rect::from_min_size(Pt::new(0.0, 0.0), 20.0, 20.0),
            extension: 0.0,
            valve_position: 0,
            reading: None,
            series: None,
            contact_closed: None,
            orientation: Orientation::NONE,
            designation: None,
            closures: &[],
            exhausts: &[],
        },
        Placed {
            index: 1,
            catalog_id: "cylinder.single_acting_20_100",
            rect: Rect::from_min_size(Pt::new(100.0, 0.0), 40.0, 20.0),
            extension: 0.0,
            valve_position: 0,
            reading: None,
            series: None,
            contact_closed: None,
            orientation: Orientation::NONE,
            designation: None,
            closures: &[],
            exhausts: &[],
        },
    ];
    let pipes = [Pipe {
        class: LineClass::Working,
        from: PipeEnd {
            placement: 0,
            port: "1",
        },
        to: PipeEnd {
            placement: 1,
            port: "1",
        },
        offset_mm: 10.0,
        pressure_ratio: 0.0,
        tube_od_hundredths: Some(1000),
        flow_l_min: 0.0,
        phase_s: 0.0,
        animate: false,
    }];

    let bounds = bounds_placed(&placements, &pipes, &catalog).expect("a drawing has bounds");
    assert!(bounds.contains(Pt::new(0.0, 0.0)), "{bounds:?}");
    assert!(bounds.contains(Pt::new(140.0, 20.0)), "{bounds:?}");
}
