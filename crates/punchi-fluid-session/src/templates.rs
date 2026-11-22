// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The small, authored circuits offered from Inicio.
//!
//! These are compositions of catalog files, not another source of component behaviour or
//! geometry. The same documents are loaded into a new project and drawn for the start-page
//! thumbnails.

use punchi_fluid_catalog::Catalog;
use punchi_fluid_doc::{Document, Endpoint, Placement};
use punchi_fluid_draw::surface::{LineClass, Pt, Rect};

pub const COUNT: u32 = 4;

#[derive(Debug, Clone, Copy)]
pub enum Template {
    WorkedExample,
    DoubleActing,
    AirPreparation,
    ElectricalControl,
}

impl Template {
    #[must_use]
    pub const fn from_index(index: u32) -> Option<Self> {
        match index {
            0 => Some(Self::WorkedExample),
            1 => Some(Self::DoubleActing),
            2 => Some(Self::AirPreparation),
            3 => Some(Self::ElectricalControl),
            _ => None,
        }
    }
}

#[derive(Clone, Copy)]
struct Part {
    catalog_id: &'static str,
    centre: Pt,
    width: f32,
    height: f32,
}

impl Part {
    fn at(area: Rect, unit: f32, catalog_id: &'static str, x: f32, y: f32, w: f32, h: f32) -> Self {
        Self {
            catalog_id,
            centre: area.point_at(x, y),
            width: unit * w,
            height: unit * h,
        }
    }

    fn in_box(catalog_id: &'static str, rect: Rect) -> Self {
        Self {
            catalog_id,
            centre: rect.center(),
            width: rect.width(),
            height: rect.height(),
        }
    }
}

#[derive(Clone, Copy)]
struct Wire {
    from_part: usize,
    from_port: &'static str,
    to_part: usize,
    to_port: &'static str,
    class: LineClass,
}

const WORKING: LineClass = LineClass::Working;
const ELECTRICAL: LineClass = LineClass::Electrical;

const fn wire(
    source_component: usize,
    source_port: &'static str,
    target_component: usize,
    target_port: &'static str,
    class: LineClass,
) -> Wire {
    Wire {
        from_part: source_component,
        from_port: source_port,
        to_part: target_component,
        to_port: target_port,
        class,
    }
}

const WORKED_EXAMPLE_WIRES: [Wire; 3] = [
    wire(0, "1", 1, "1", WORKING),
    wire(1, "2", 2, "1", WORKING),
    wire(2, "2", 3, "1", WORKING),
];
const DOUBLE_ACTING_WIRES: [Wire; 3] = [
    wire(0, "1", 1, "1", WORKING),
    wire(1, "2", 2, "1", WORKING),
    wire(1, "4", 2, "2", WORKING),
];
const AIR_PREPARATION_WIRES: [Wire; 4] = [
    wire(0, "1", 1, "1", WORKING),
    wire(1, "2", 2, "1", WORKING),
    wire(2, "2", 3, "1", WORKING),
    wire(3, "2", 4, "1", WORKING),
];
const ELECTRICAL_CONTROL_WIRES: [Wire; 5] = [
    wire(0, "1", 1, "1", WORKING),
    wire(1, "2", 2, "1", WORKING),
    wire(2, "2", 3, "1", WORKING),
    wire(4, "L", 5, "1", ELECTRICAL),
    wire(5, "2", 2, "Y1", ELECTRICAL),
];

type Composition = (Vec<Part>, &'static [Wire]);

fn worked_example(area: Rect) -> Option<Composition> {
    let layout = punchi_fluid_draw::layout(area)?;
    Some((
        vec![
            Part::in_box(punchi_fluid_draw::schematic::SUPPLY_ID, layout.supply),
            Part::in_box(
                punchi_fluid_draw::schematic::PREPARATION_ID,
                layout.preparation,
            ),
            Part::in_box(punchi_fluid_draw::schematic::VALVE_ID, layout.valve),
            Part::in_box(punchi_fluid_draw::schematic::CYLINDER_ID, layout.cylinder),
        ],
        &WORKED_EXAMPLE_WIRES,
    ))
}

fn double_acting(area: Rect) -> Composition {
    // The size is the project's, not a fraction of the paper: a symbol is read by shape.
    let unit = punchi_fluid_draw::SYMBOL_BOX_MM;
    (
        vec![
            Part::at(area, unit, "supply.compressor", 0.10, 0.76, 1.0, 1.0),
            Part::at(area, unit, "valve.5_2.spring_return", 0.43, 0.58, 1.0, 1.0),
            Part::at(
                area,
                unit,
                "cylinder.double_acting_32_200",
                0.80,
                0.27,
                2.2,
                1.0,
            ),
        ],
        &DOUBLE_ACTING_WIRES,
    )
}

fn air_preparation(area: Rect) -> Composition {
    // The size is the project's, not a fraction of the paper: a symbol is read by shape.
    let unit = punchi_fluid_draw::SYMBOL_BOX_MM;
    (
        vec![
            Part::at(area, unit, "supply.compressor", 0.10, 0.76, 1.0, 1.0),
            Part::at(area, unit, "filter.air", 0.29, 0.76, 1.0, 1.0),
            Part::at(area, unit, "preparation.service_unit", 0.48, 0.76, 1.0, 1.0),
            Part::at(area, unit, "valve.3_2.spring_return", 0.68, 0.53, 1.0, 1.0),
            Part::at(
                area,
                unit,
                "cylinder.single_acting_20_100",
                0.84,
                0.23,
                2.2,
                1.0,
            ),
        ],
        &AIR_PREPARATION_WIRES,
    )
}

fn electrical_control(area: Rect) -> Composition {
    // The size is the project's, not a fraction of the paper: a symbol is read by shape.
    let unit = punchi_fluid_draw::SYMBOL_BOX_MM;
    (
        vec![
            Part::at(area, unit, "supply.compressor", 0.10, 0.77, 1.0, 1.0),
            Part::at(area, unit, "preparation.service_unit", 0.30, 0.77, 1.0, 1.0),
            Part::at(area, unit, "valve.3_2.solenoid", 0.55, 0.77, 1.0, 1.0),
            Part::at(
                area,
                unit,
                "cylinder.single_acting_20_100",
                0.84,
                0.77,
                2.2,
                1.0,
            ),
            Part::at(area, unit, "electrical.supply", 0.15, 0.20, 1.0, 1.0),
            Part::at(area, unit, "electrical.contact", 0.43, 0.20, 1.0, 1.0),
        ],
        &ELECTRICAL_CONTROL_WIRES,
    )
}

fn composition(template: Template, area: Rect) -> Option<Composition> {
    match template {
        Template::WorkedExample => worked_example(area),
        Template::DoubleActing => Some(double_acting(area)),
        Template::AirPreparation => Some(air_preparation(area)),
        Template::ElectricalControl => Some(electrical_control(area)),
    }
}

/// Builds one of the four bounded, compiled-in circuits in the requested drawing area.
///
/// # Errors
///
/// Returns an error when the index is unknown, the area cannot hold the worked example, or
/// an effective catalog definition no longer provides one of the ports the template needs.
pub fn document(index: u32, catalog: &Catalog, area: Rect) -> Result<Document, String> {
    let template = Template::from_index(index)
        .ok_or_else(|| String::from("that start-page template does not exist"))?;
    let (parts, wires) = composition(template, area)
        .ok_or_else(|| String::from("the drawing area is too small for this template"))?;

    for part in &parts {
        if catalog.get(part.catalog_id).is_none() {
            return Err(String::from(
                "a component used by this template is unavailable",
            ));
        }
    }
    for wire in wires {
        let from = parts
            .get(wire.from_part)
            .and_then(|part| catalog.get(part.catalog_id));
        let to = parts
            .get(wire.to_part)
            .and_then(|part| catalog.get(part.catalog_id));
        let from_exists = from
            .is_some_and(|definition| definition.port.iter().any(|port| port.id == wire.from_port));
        let to_exists =
            to.is_some_and(|definition| definition.port.iter().any(|port| port.id == wire.to_port));
        if !from_exists || !to_exists {
            return Err(String::from(
                "a component used by this template has changed its ports",
            ));
        }
    }

    let mut document = Document::new();
    document.seed(
        parts
            .iter()
            .map(|part| Placement::new(part.catalog_id, part.centre, part.width, part.height))
            .collect(),
    );
    for wire in wires {
        let from = document.placements()[wire.from_part].id();
        let to = document.placements()[wire.to_part].id();
        if !document.seed_connect(
            Endpoint::new(from, wire.from_port),
            Endpoint::new(to, wire.to_port),
        ) {
            return Err(String::from("a connection in this template is invalid"));
        }
        let pipe = document.connections().len() - 1;
        if wire.class != LineClass::Working {
            let _ = document.seed_pipe_class(pipe, wire.class);
        }
        // A pneumatic pipe starts at the documented 10 mm; the electrical control
        // template's wire is not a tube and stays unspecified.
        if wire.class != LineClass::Electrical {
            let _ = document.seed_pipe_tube(pipe, Some(punchi_fluid_core::DEFAULT_TUBE_HUNDREDTHS));
        }
    }
    Ok(document)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_four_templates_are_bounded_connected_starting_documents() {
        let catalog = Catalog::builtin();
        let area = Rect::from_corners(Pt::new(10.0, 10.0), Pt::new(170.0, 100.0));
        let expected = [(4, 3), (3, 3), (5, 4), (6, 5)];

        for (index, (components, pipes)) in expected.into_iter().enumerate() {
            let index = u32::try_from(index).expect("four template ids fit in u32");
            let document = document(index, &catalog, area).expect("template should build");
            assert_eq!(document.placements().len(), components);
            assert_eq!(document.connections().len(), pipes);
            for part in document.placements() {
                assert!(
                    part.rect().min.x >= area.min.x
                        && part.rect().max.x <= area.max.x
                        && part.rect().min.y >= area.min.y
                        && part.rect().max.y <= area.max.y
                );
                // A box that is not the canonical size stretches the symbol: a 3/2 valve in
                // a 1.2 : 1 box is a drawing that is not the symbol. The cylinder is the
                // only rectangle, because its barrel is one.
                let ratio = part.width / part.height;
                if part.catalog_id.starts_with("cylinder.") {
                    assert!((ratio - punchi_fluid_draw::CYLINDER_RATIO).abs() < 0.01);
                    assert!((part.height - punchi_fluid_draw::SYMBOL_BOX_MM).abs() < 0.01);
                } else {
                    assert!(
                        (ratio - 1.0).abs() < 0.001,
                        "{} is not square: {} x {}",
                        part.catalog_id,
                        part.width,
                        part.height
                    );
                    assert!((part.width - punchi_fluid_draw::SYMBOL_BOX_MM).abs() < 0.01);
                }
            }
        }
    }

    #[test]
    fn an_unknown_template_is_refused() {
        let catalog = Catalog::builtin();
        let area = Rect::from_corners(Pt::new(0.0, 0.0), Pt::new(160.0, 90.0));
        assert!(document(COUNT, &catalog, area).is_err());
    }
}
