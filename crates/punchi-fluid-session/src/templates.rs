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
    /// A part at a normalised position of the area, sized by the box its own file
    /// declares. The catalog is the source of the size: a template does not choose it.
    fn at(area: Rect, catalog: &Catalog, catalog_id: &'static str, x: f32, y: f32) -> Self {
        let (width, height) = catalog.get(catalog_id).map_or(
            (
                punchi_fluid_draw::SYMBOL_BOX_MM,
                punchi_fluid_draw::SYMBOL_BOX_MM,
            ),
            punchi_fluid_draw::symbol_box_mm,
        );
        Self {
            catalog_id,
            centre: area.point_at(x, y),
            width,
            height,
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

const WORKED_EXAMPLE_WIRES: [Wire; 6] = [
    wire(0, "2", 1, "1", WORKING),
    wire(1, "2", 2, "1", WORKING),
    wire(1, "2", 3, "1", WORKING),
    wire(2, "2", 4, "1a", WORKING),
    wire(3, "2", 4, "1b", WORKING),
    wire(4, "2", 5, "1", WORKING),
];
const DOUBLE_ACTING_WIRES: [Wire; 3] = [
    wire(0, "2", 1, "1", WORKING),
    wire(1, "2", 2, "1", WORKING),
    wire(1, "4", 2, "2", WORKING),
];
const AIR_PREPARATION_WIRES: [Wire; 4] = [
    wire(0, "2", 1, "1", WORKING),
    wire(1, "2", 2, "1", WORKING),
    wire(2, "2", 3, "1", WORKING),
    wire(3, "2", 4, "1", WORKING),
];
const ELECTRICAL_CONTROL_WIRES: [Wire; 5] = [
    wire(0, "2", 1, "1", WORKING),
    wire(1, "2", 2, "1", WORKING),
    wire(2, "2", 3, "1", WORKING),
    wire(4, "L", 5, "1", ELECTRICAL),
    wire(5, "2", 2, "Y1", ELECTRICAL),
];

type Composition = (Vec<Part>, &'static [Wire]);

fn worked_example(area: Rect, catalog: &Catalog) -> Composition {
    // The OR circuit of the owner: the supply feeds the service unit, the unit feeds two
    // 3/2 valves, the valves feed the two inlets of the shuttle and the shuttle feeds the
    // single-acting cylinder. Each part takes the box its own file declares.
    (
        vec![
            Part::at(
                area,
                catalog,
                punchi_fluid_draw::schematic::SUPPLY_ID,
                0.08,
                0.82,
            ),
            Part::at(
                area,
                catalog,
                punchi_fluid_draw::schematic::PREPARATION_ID,
                0.30,
                0.72,
            ),
            Part::at(
                area,
                catalog,
                punchi_fluid_draw::schematic::VALVE_ID,
                0.54,
                0.56,
            ),
            Part::at(
                area,
                catalog,
                punchi_fluid_draw::schematic::VALVE_ID,
                0.70,
                0.56,
            ),
            Part::at(area, catalog, "valve.shuttle", 0.60, 0.33),
            Part::at(
                area,
                catalog,
                punchi_fluid_draw::schematic::CYLINDER_ID,
                0.84,
                0.20,
            ),
        ],
        &WORKED_EXAMPLE_WIRES,
    )
}

fn double_acting(area: Rect, catalog: &Catalog) -> Composition {
    (
        vec![
            Part::at(area, catalog, "supply.compressor", 0.08, 0.76),
            Part::at(area, catalog, "valve.5_2.spring_return", 0.42, 0.58),
            Part::at(area, catalog, "cylinder.double_acting_32_200", 0.84, 0.24),
        ],
        &DOUBLE_ACTING_WIRES,
    )
}

fn air_preparation(area: Rect, catalog: &Catalog) -> Composition {
    (
        vec![
            Part::at(area, catalog, "supply.compressor", 0.08, 0.76),
            Part::at(area, catalog, "filter.air", 0.26, 0.76),
            Part::at(area, catalog, "preparation.service_unit", 0.50, 0.72),
            Part::at(area, catalog, "valve.3_2.spring_return", 0.75, 0.52),
            Part::at(area, catalog, "cylinder.single_acting_20_100", 0.84, 0.22),
        ],
        &AIR_PREPARATION_WIRES,
    )
}

fn electrical_control(area: Rect, catalog: &Catalog) -> Composition {
    (
        vec![
            Part::at(area, catalog, "supply.compressor", 0.08, 0.77),
            Part::at(area, catalog, "preparation.service_unit", 0.34, 0.72),
            Part::at(area, catalog, "valve.3_2.solenoid", 0.62, 0.77),
            Part::at(area, catalog, "cylinder.single_acting_20_100", 0.86, 0.75),
            Part::at(area, catalog, "electrical.supply", 0.15, 0.20),
            Part::at(area, catalog, "electrical.contact", 0.43, 0.20),
        ],
        &ELECTRICAL_CONTROL_WIRES,
    )
}

fn composition(template: Template, catalog: &Catalog, area: Rect) -> Composition {
    match template {
        Template::WorkedExample => worked_example(area, catalog),
        Template::DoubleActing => double_acting(area, catalog),
        Template::AirPreparation => air_preparation(area, catalog),
        Template::ElectricalControl => electrical_control(area, catalog),
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
    let (parts, wires) = composition(template, catalog, area);

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
        let expected = [(6, 6), (3, 3), (5, 4), (6, 5)];

        for (index, (components, pipes)) in expected.into_iter().enumerate() {
            let index = u32::try_from(index).expect("four template ids fit in u32");
            let document = document(index, &catalog, area).expect("template should build");
            assert_eq!(document.placements().len(), components);
            assert_eq!(document.connections().len(), pipes);
            for (position, part) in document.placements().iter().enumerate() {
                assert!(
                    part.rect().min.x >= area.min.x
                        && part.rect().max.x <= area.max.x
                        && part.rect().min.y >= area.min.y
                        && part.rect().max.y <= area.max.y
                );
                for other in document.placements().iter().skip(position + 1) {
                    let (a, b) = (part.rect(), other.rect());
                    let overlap = a.min.x < b.max.x
                        && b.min.x < a.max.x
                        && a.min.y < b.max.y
                        && b.min.y < a.max.y;
                    assert!(
                        !overlap,
                        "template {}: {} and {} overlap: {a:?} {b:?}",
                        index, part.catalog_id, other.catalog_id
                    );
                }
                // A box that is not the one its file declares stretches the symbol: a 3/2
                // valve in a 1.2 : 1 box is a drawing that is not the symbol. The declared
                // box is the datum, so a template cannot choose another size.
                let definition = catalog
                    .get(&part.catalog_id)
                    .expect("the template's catalog entry exists");
                let (expected_width, expected_height) =
                    punchi_fluid_draw::symbol_box_mm(definition);
                assert!(
                    (part.width - expected_width).abs() < 0.01
                        && (part.height - expected_height).abs() < 0.01,
                    "{} is {} x {}, not the declared {expected_width} x {expected_height}",
                    part.catalog_id,
                    part.width,
                    part.height
                );
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
