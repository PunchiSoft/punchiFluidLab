// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests for the flow marks: how many, where, which way and when the hose is still.
//!
//! The video of 2026-11-21 caught the real defect this file now guards: one mark alone jumps
//! from the sink back to the source and leaves the line empty in between, which reads as
//! flicker. The stream keeps a head on the hose at every frame while the air moves.

use super::flow::*;
use super::*;
use crate::orient::Orientation;
use crate::surface::{Call, Recording};
use punchi_fluid_catalog::Catalog;

const ROUTE_MM: f32 = 60.0;

fn placed(index: usize, catalog_id: &'static str, rect: Rect) -> Placed<'static> {
    Placed {
        index,
        catalog_id,
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
    }
}

fn pipe(from: usize, from_port: &'static str, to: usize, to_port: &'static str) -> Pipe<'static> {
    Pipe {
        from: PipeEnd {
            placement: from,
            port: from_port,
        },
        to: PipeEnd {
            placement: to,
            port: to_port,
        },
        offset_mm: 0.0,
        pressure_ratio: 0.0,
        class: LineClass::Working,
        tube_od_hundredths: Some(1000),
        flow_l_min: 0.0,
        phase_s: 0.0,
        animate: true,
    }
}

/// A straight hose 60 mm long, enough for a stream.
fn straight_route() -> [Pt; 2] {
    [Pt::new(0.0, 0.0), Pt::new(ROUTE_MM, 0.0)]
}

fn draw_marks(hose: &Pipe<'_>) -> Recording {
    let mut surface = Recording::default();
    draw_flow_marks(
        &mut surface,
        &straight_route(),
        hose,
        0.50,
        Ink::print().line,
    );
    surface
}

#[test]
fn the_stream_never_leaves_the_hose_empty() {
    // 60 frames per second over more than one full cycle of the stream: at no instant may
    // the hose be without a mark while the air moves. This is the anti-flicker rule, and it
    // comes straight from the recording that showed a single mark jumping from end to end.
    // Four seconds at 60 frames per second covers more than one cycle of the stream.
    let mut phase = 0.0_f32;
    while phase < 4.0 {
        let (_, count) = mark_positions(ROUTE_MM, 20.0, phase, true);
        phase += 1.0 / 60.0;
        assert!(
            count >= 1,
            "the hose is empty at phase {phase:.3}: the stream must always carry a mark"
        );
    }
}

#[test]
fn the_marks_are_ordered_inside_the_hose_and_do_not_overlap() {
    for phase in [0.0, 0.37, 0.9, 1.4, 2.0, 2.6] {
        let mut previous = 0.0;
        let (marks, count) = mark_positions(ROUTE_MM, 20.0, phase, true);
        assert!(count >= 2, "a 60 mm hose holds a stream, not one mark");
        for (index, start) in marks[..count].iter().enumerate() {
            assert!(
                *start >= PULSE_CLEAR_MM && start + MARK_MM <= ROUTE_MM - PULSE_CLEAR_MM,
                "a mark stays clear of the ports: {start:.2} at phase {phase}"
            );
            if index > 0 {
                assert!(
                    start - previous >= MARK_MM,
                    "two marks overlap: {previous:.2} then {start:.2}"
                );
            }
            previous = *start;
        }
    }
}

#[test]
fn the_stream_moves_with_the_phase() {
    let (first, first_count) = mark_positions(ROUTE_MM, 20.0, 0.5, true);
    let (second, second_count) = mark_positions(ROUTE_MM, 20.0, 0.75, true);
    assert!(first_count >= 2 && second_count >= 2);
    let moved = first[..first_count]
        .iter()
        .zip(second[..second_count].iter())
        .any(|(a, b)| (a - b).abs() > 0.01);
    assert!(moved, "the marks do not move with the phase");
}

#[test]
fn the_sign_reverses_the_stream() {
    // The positions are measured from `from`; the drawing is what reverses them. The same
    // phase must draw the mirrored stream when the air runs the other way.
    let mut hose = pipe(0, "2", 1, "1");
    hose.flow_l_min = 20.0;
    hose.phase_s = 1.0;
    let ahead = stroke_spans(&draw_marks(&hose));
    hose.flow_l_min = -20.0;
    let back = stroke_spans(&draw_marks(&hose));
    assert_eq!(ahead.len(), back.len(), "the same number of marks");
    for ((a_start, a_end), (b_start, b_end)) in ahead.iter().zip(back.iter().rev()) {
        let mirrored_start = ROUTE_MM - a_end;
        let mirrored_end = ROUTE_MM - a_start;
        assert!(
            (b_start - mirrored_start).abs() < 0.05 && (b_end - mirrored_end).abs() < 0.05,
            "the reversed stream mirrors: ({b_start:.2}, {b_end:.2}) against ({mirrored_start:.2}, {mirrored_end:.2})"
        );
    }
}

/// The start and end of every thick stroke, in millimetres along a straight route.
fn stroke_spans(recording: &Recording) -> Vec<(f32, f32)> {
    let mut spans: Vec<(f32, f32)> = recording
        .calls()
        .iter()
        .filter_map(|call| match call {
            Call::Line { points, stroke }
                if (stroke.width - 0.50 * PULSE_WIDTH_FACTOR).abs() < 1.0e-6 =>
            {
                let start = points.first()?.x;
                let end = points.last()?.x;
                Some((start.min(end), start.max(end)))
            }
            _ => None,
        })
        .collect();
    spans.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    spans
}

#[test]
fn a_still_hose_draws_nothing() {
    let (marks, count) = mark_positions(ROUTE_MM, 0.0, 1.0, true);
    assert_eq!(count, 0, "no flow, no mark");
    let (below, below_count) = mark_positions(ROUTE_MM, 0.05, 1.0, true);
    assert_eq!(below_count, 0, "below the floor is still no reading");
    assert!(marks[0].abs() < f32::EPSILON);
    assert!(below[0].abs() < f32::EPSILON);
}

#[test]
fn reduced_motion_keeps_one_arrowhead_in_the_middle() {
    let (still, count) = mark_positions(ROUTE_MM, 20.0, 0.0, false);
    assert_eq!(
        count, 1,
        "a still sheet keeps the direction, not the travel"
    );
    assert!((still[0] - ROUTE_MM * 0.5).abs() < 1.0e-4);
    let (later, later_count) = mark_positions(ROUTE_MM, 20.0, 7.0, false);
    assert_eq!(later_count, 1);
    assert!(
        (later[0] - still[0]).abs() < 1.0e-4,
        "a frozen head does not follow the phase"
    );
}

#[test]
fn a_short_route_draws_nothing() {
    let short = MARK_MM + PULSE_CLEAR_MM * 2.0 - 0.5;
    let (_, count) = mark_positions(short, 20.0, 0.0, true);
    assert_eq!(count, 0, "no room for a mark, no mark");
}

#[test]
fn a_stronger_flow_moves_the_stream_faster() {
    // The air moves with the flow: a trickle drifts, a strong current runs, and both stay
    // inside the readable band. The scale is not the physical air speed (debt in ADR 0034),
    // but the ordering is the model's.
    let weak = pulse_speed(2.0);
    let strong = pulse_speed(60.0);
    assert!(strong > weak, "a stronger flow is faster");
    assert!(
        (weak - MIN_SPEED).abs() < 1.0e-4 || weak > MIN_SPEED,
        "the floor keeps a trickle visible: {weak:.2}"
    );
    assert!(
        pulse_speed(10_000.0) <= MAX_SPEED + 1.0e-4,
        "the cap keeps a burst readable"
    );
    let linear = pulse_speed(30.0);
    assert!(
        (linear - 30.0 * SPEED_PER_LPM).abs() < 1.0e-4,
        "between the clamps the speed follows the flow: {linear:.2}"
    );
}

#[test]
fn every_mark_draws_a_thick_stroke_with_its_head() {
    let hose = {
        let mut hose = pipe(0, "2", 1, "1");
        hose.flow_l_min = 20.0;
        hose.phase_s = 0.5;
        hose
    };
    let ink = Ink::print();
    let recording = draw_marks(&hose);
    let (_, count) = mark_positions(ROUTE_MM, hose.flow_l_min, hose.phase_s, true);
    let strokes = recording
        .calls()
        .iter()
        .filter(|call| {
            matches!(call, Call::Line { stroke, .. }
                if (stroke.width - 0.50 * PULSE_WIDTH_FACTOR).abs() < 1.0e-6)
        })
        .count();
    let heads = recording
        .calls()
        .iter()
        .filter(|call| {
            matches!(call, Call::Polygon { fill, stroke, .. }
            if *fill == Some(ink.line) && stroke.is_none())
        })
        .count();
    assert_eq!(strokes, count, "one stroke per mark");
    assert_eq!(heads, count, "one solid head per mark");
    for call in recording.calls() {
        match call {
            Call::Line { stroke, .. } => assert_ne!(
                stroke.colour, ink.accent,
                "the marks take the hose's colour, never the selection accent"
            ),
            Call::Polygon { fill, .. } => assert_ne!(*fill, Some(ink.accent)),
            _ => {}
        }
    }
}

#[test]
fn a_drawn_mark_keeps_the_line_class_and_the_numerals() {
    let placements = [
        placed(
            0,
            "supply.compressor",
            Rect::from_center_size(Pt::new(30.0, 40.0), 22.0, 22.0),
        ),
        placed(
            1,
            "valve.3_2.spring_return",
            Rect::from_center_size(Pt::new(90.0, 90.0), 22.0, 22.0),
        ),
    ];
    let mut hose = pipe(0, "1", 1, "1");
    hose.class = LineClass::Pilot;
    hose.flow_l_min = 20.0;
    hose.phase_s = 0.3;
    let ink = Ink::print();
    let catalog = Catalog::builtin();
    let mut surface = Recording::default();
    draw_placed(&mut surface, &placements, &[hose], &catalog, &ink, &[]);

    assert!(
        surface.calls().iter().any(|call| matches!(
            call,
            Call::Line { stroke, .. }
                if stroke.class == LineClass::Pilot && (stroke.width - 0.50).abs() < 1.0e-6
        )),
        "the pilot pattern stays under the marks"
    );
    let marks = surface
        .calls()
        .iter()
        .position(|call| {
            matches!(call, Call::Line { stroke, .. }
            if (stroke.width - 0.50 * PULSE_WIDTH_FACTOR).abs() < 1.0e-6)
        })
        .expect("the stream is drawn");
    // The symbols are drawn after the pipes, so a mark can never cover a port numeral.
    let numeral = surface
        .calls()
        .iter()
        .position(|call| matches!(call, Call::Text { text, .. } if text == "1"))
        .expect("the valve prints its numerals");
    assert!(
        numeral > marks,
        "the numerals are drawn on top of the stream"
    );
}

#[test]
fn a_page_without_flow_draws_no_mark() {
    let mut hose = pipe(0, "2", 1, "1");
    hose.animate = false;
    let recording = draw_marks(&hose);
    assert!(recording.calls().is_empty());
}

#[test]
fn an_exhaust_port_draws_the_air_leaving_it() {
    // The jet is outside the symbol, on the port's edge: the drawing places it, the solver
    // owns how much. A plugged port has no reading, so its slice is empty and nothing draws.
    let catalog = Catalog::builtin();
    let ink = Ink::print();
    let valve = placed(
        0,
        "valve.3_2.spring_return",
        Rect::from_center_size(Pt::new(40.0, 40.0), 20.0, 20.0),
    );
    let mark = ExhaustMark {
        port: "3",
        flow_l_min: 20.0,
    };
    let mut venting = valve;
    venting.exhausts = std::slice::from_ref(&mark);
    let mut surface = Recording::default();
    draw_placed(&mut surface, &[venting], &[], &catalog, &ink, &[]);
    let outside = |surface: &Recording| {
        surface
            .calls()
            .iter()
            .filter(|call| match call {
                Call::Polygon { points, fill, .. } => {
                    *fill == Some(ink.line)
                        && !points.is_empty()
                        && points
                            .iter()
                            .all(|point| point.y > venting.rect.max.y + 0.5)
                }
                _ => false,
            })
            .count()
    };
    assert!(
        outside(&surface) >= 2,
        "the air leaving port 3 draws below the valve"
    );

    let still = ExhaustMark {
        port: "3",
        flow_l_min: 0.0,
    };
    let mut quiet = valve;
    quiet.exhausts = std::slice::from_ref(&still);
    let mut surface = Recording::default();
    draw_placed(&mut surface, &[quiet], &[], &catalog, &ink, &[]);
    assert_eq!(outside(&surface), 0, "no flow, no jet");
}
