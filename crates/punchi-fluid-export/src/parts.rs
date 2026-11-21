// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The parts list a printed sheet carries.
//!
//! A schematic without a list of what it contains is a drawing a technician cannot build
//! from: the symbols say *what kind* of valve it is, the list says how many, with which
//! designation, and under the name the catalog gives it.
//!
//! The list is **grouped by definition**: two identical cylinders are one row of quantity
//! two, not two rows, which is how a parts list is read and what makes a change in count
//! visible at a glance. The designations of the group travel in the row, so a sheet with
//! `1A1` and `1A2` still says which is which.
//!
//! Pneumatic tubing is the second half of the list: a row per nominal size and line class,
//! with the length of the route the sheet draws. It is a cut list read with the drawing, not
//! a geometric measurement: no bend allowance, fittings or waste.

use punchi_fluid_catalog::{Catalog, PortRole};
use punchi_fluid_core::tube::tube_label;
use punchi_fluid_draw::surface::LineClass;

/// Most rows a printed list holds before it stops being a list and becomes a column.
///
/// The components, the closures and the tubes share it, in that order: a sheet missing a
/// valve is worse than a sheet missing a closure, and a closure row is more useful than a
/// tube row the drawing still shows.
pub const MAX_ROWS: usize = 24;

/// One row of the parts list: a definition, how many of it, and their names.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PartRow {
    /// How many placements this definition has on the sheet.
    pub quantity: usize,
    /// Name the catalog gives it, in the interface's language.
    pub name: String,
    /// Designations of the group, in placement order, joined by a comma.
    ///
    /// Empty when none of them has a name; the drawing then leaves the column blank
    /// rather than writing a dash that could be read as a name.
    pub designations: String,
}

/// One pipe as the parts list receives it: size, class and drawn length.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TubeRowInput {
    /// Nominal outside diameter in hundredths of a millimetre; `None` is unspecified.
    pub od: Option<u16>,
    /// Line class of the pipe.
    pub class: LineClass,
    /// Length of the drawn route, in whole millimetres.
    pub length_mm: u32,
}

/// One tube row of the parts list: a size and class, with the summed drawn length.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TubeRow {
    /// Nominal outside diameter in hundredths of a millimetre; `None` is unspecified.
    pub od: Option<u16>,
    /// Line class of the grouped pipes.
    pub class: LineClass,
    /// Summed length of their drawn routes, in whole millimetres.
    pub length_mm: u32,
}

impl TubeRow {
    /// The line the sheet prints: size, class and length, with the decimal comma.
    #[must_use]
    pub fn text(&self) -> String {
        let size = self
            .od
            .map_or_else(|| "sin especificar".to_owned(), tube_label);
        format!(
            "Tubo {size} · {} · {}",
            class_label(self.class),
            length_label(self.length_mm)
        )
    }
}

/// Spanish name of a line class, as the printed list reads it.
fn class_label(class: LineClass) -> &'static str {
    match class {
        LineClass::Working => "trabajo",
        LineClass::Pilot => "pilotaje",
        LineClass::Drain => "drenaje",
        LineClass::Electrical => "eléctrico",
    }
}

/// The port display label a closure row prints: the role in Spanish and the port id.
///
/// The document stores the bare port id; the label belongs to the catalog's definition and
/// is resolved here, so the row reads `escape 3` without the drawing having to know it. A
/// parent or port the catalog does not resolve keeps the bare id, which is still a truthful
/// row for a file the machine cannot fully read.
fn port_label(catalog: &Catalog, parent_catalog: &str, port: &str) -> String {
    let role = catalog
        .get(parent_catalog)
        .and_then(|definition| {
            definition
                .port
                .iter()
                .find(|candidate| candidate.id == port)
        })
        .map(|candidate| candidate.role);
    match role {
        Some(PortRole::Supply) => format!("suministro {port}"),
        Some(PortRole::Working) => format!("trabajo {port}"),
        Some(PortRole::Exhaust) => format!("escape {port}"),
        Some(PortRole::Mechanical) => format!("mecánico {port}"),
        Some(PortRole::Electrical) => format!("eléctrico {port}"),
        None => port.to_owned(),
    }
}

/// `true` when the class sorts before the other in the list: working first.
fn class_rank(class: LineClass) -> u8 {
    match class {
        LineClass::Working => 0,
        LineClass::Pilot => 1,
        LineClass::Drain => 2,
        LineClass::Electrical => 3,
    }
}

/// A drawn length as metres with one decimal, or `< 0,1 m` below a decimetre.
fn length_label(length_mm: u32) -> String {
    if length_mm < 100 {
        return String::from("< 0,1 m");
    }
    let tenths = (u64::from(length_mm) + 50) / 100;
    let whole = tenths / 10;
    let fraction = tenths % 10;
    format!("{whole},{fraction} m")
}

/// The line a closure prints: the closure, the component that carries it and the port.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClosureRow {
    /// How many ports carry the same closure on the same parent and port.
    pub quantity: usize,
    /// Name the catalog gives the closure, in the interface's language.
    pub closure: String,
    /// Designation of the parent, or its catalog id when it has none.
    pub parent: String,
    /// Port display label, `<role> <id>` in Spanish (`escape 3`).
    pub port: String,
}

impl ClosureRow {
    /// The line the sheet prints: closure, parent and port.
    #[must_use]
    pub fn text(&self) -> String {
        format!("{} · {} · {}", self.closure, self.parent, self.port)
    }
}

/// The list a sheet prints.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PartsList {
    /// Component rows, in the order the sheet prints them.
    pub rows: Vec<PartRow>,
    /// Components left out because the list was full.
    pub omitted: usize,
    /// Closure rows, after the components and before the tubes.
    pub closures: Vec<ClosureRow>,
    /// Closures left out because the components and the earlier closures took the room.
    pub omitted_closures: usize,
    /// Tube rows, after the closures, sorted by class and diameter.
    pub tubes: Vec<TubeRow>,
    /// Tubes left out because the components and closures took the room.
    pub omitted_tubes: usize,
}

impl PartsList {
    /// `true` when there is nothing to print.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty() && self.closures.is_empty() && self.tubes.is_empty()
    }

    /// Groups placements into component rows, port closures into closure rows and pipes
    /// into tube rows.
    ///
    /// The input is what the document holds: the catalog id of each placement and the name
    /// the user gave it, one tuple per port closure `(parent catalog, parent designation,
    /// port id, closure catalog)`, and one [`TubeRowInput`] per drawn pneumatic pipe. A
    /// definition the catalog does not know keeps its id as the name, so a sheet with a
    /// user file that failed to load still lists what is drawn.
    #[must_use]
    pub fn group<'a>(
        catalog: &Catalog,
        placements: impl IntoIterator<Item = (&'a str, Option<&'a str>)>,
        closures: impl IntoIterator<Item = (&'a str, Option<&'a str>, &'a str, &'a str)>,
        tubes: impl IntoIterator<Item = TubeRowInput>,
    ) -> Self {
        let mut groups: Vec<(String, Vec<Option<&'a str>>)> = Vec::new();
        for (id, designation) in placements {
            if let Some((_, designations)) = groups.iter_mut().find(|(key, _)| key == id) {
                designations.push(designation);
            } else {
                groups.push((id.to_owned(), vec![designation]));
            }
        }
        let mut rows: Vec<PartRow> = groups
            .into_iter()
            .map(|(id, designations)| {
                let name = catalog.get(&id).map_or_else(
                    || id.clone(),
                    |definition| definition.display_name().to_owned(),
                );
                let named: Vec<&str> = designations.iter().flatten().copied().collect();
                PartRow {
                    quantity: designations.len(),
                    name,
                    designations: named.join(", "),
                }
            })
            .collect();
        // A list is read by name, and the count is the first column.
        rows.sort_by(|left, right| {
            left.name
                .to_lowercase()
                .cmp(&right.name.to_lowercase())
                .then_with(|| left.name.cmp(&right.name))
        });

        let mut closure_rows = group_closures(catalog, closures);

        let mut tube_rows: Vec<TubeRow> = Vec::new();
        for input in tubes {
            if let Some(row) = tube_rows
                .iter_mut()
                .find(|row| row.od == input.od && row.class == input.class)
            {
                row.length_mm = row.length_mm.saturating_add(input.length_mm);
            } else {
                tube_rows.push(TubeRow {
                    od: input.od,
                    class: input.class,
                    length_mm: input.length_mm,
                });
            }
        }
        // The same order the drawing reads: class first, then diameter, with an
        // unspecified size last so the named tubing stays together.
        tube_rows.sort_by(|left, right| {
            class_rank(left.class)
                .cmp(&class_rank(right.class))
                .then_with(|| {
                    left.od
                        .unwrap_or(u16::MAX)
                        .cmp(&right.od.unwrap_or(u16::MAX))
                })
        });

        // The components take the room first, then the closures; the tubes get what is
        // left, and each kind reports its own cut so a full list never hides one silently.
        let omitted = rows.len().saturating_sub(MAX_ROWS);
        rows.truncate(MAX_ROWS);
        let room = MAX_ROWS - rows.len();
        let omitted_closures = closure_rows.len().saturating_sub(room);
        closure_rows.truncate(room);
        let room = MAX_ROWS - rows.len() - closure_rows.len();
        let omitted_tubes = tube_rows.len().saturating_sub(room);
        tube_rows.truncate(room);
        Self {
            rows,
            omitted,
            closures: closure_rows,
            omitted_closures,
            tubes: tube_rows,
            omitted_tubes,
        }
    }
}

/// Groups the closures into one row per distinct fit.
///
/// A closure is grouped by its catalog id, the parent's label and the port's display
/// label: two valves named `1V1` with a silencer on `3` share one row. The label comes
/// from the host catalog definition, because only it knows the role.
fn group_closures<'a>(
    catalog: &Catalog,
    closures: impl IntoIterator<Item = (&'a str, Option<&'a str>, &'a str, &'a str)>,
) -> Vec<ClosureRow> {
    let mut closure_groups: Vec<((String, String, String), usize)> = Vec::new();
    for (parent_catalog, parent_designation, port, closure_catalog) in closures {
        let parent = parent_designation
            .filter(|designation| !designation.is_empty())
            .unwrap_or(parent_catalog)
            .to_owned();
        let port = port_label(catalog, parent_catalog, port);
        let key = (closure_catalog.to_owned(), parent, port);
        if let Some((_, quantity)) = closure_groups
            .iter_mut()
            .find(|(candidate, _)| *candidate == key)
        {
            *quantity += 1;
        } else {
            closure_groups.push((key, 1));
        }
    }
    let mut closure_rows: Vec<ClosureRow> = closure_groups
        .into_iter()
        .map(|((closure, parent, port), quantity)| ClosureRow {
            quantity,
            closure: catalog
                .get(&closure)
                .map_or(closure, |definition| definition.display_name().to_owned()),
            parent,
            port,
        })
        .collect();
    // A closure row is read by what is fitted, then by where: the same order the
    // component rows use keeps the list predictable.
    closure_rows.sort_by(|left, right| {
        left.closure
            .to_lowercase()
            .cmp(&right.closure.to_lowercase())
            .then_with(|| left.closure.cmp(&right.closure))
            .then_with(|| left.parent.cmp(&right.parent))
            .then_with(|| left.port.cmp(&right.port))
    });
    closure_rows
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identical_components_are_one_row_of_two() {
        let catalog = Catalog::builtin();
        let list = PartsList::group(
            &catalog,
            [
                ("cylinder.single_acting_20_100", Some("1A1")),
                ("cylinder.single_acting_20_100", Some("1A2")),
                ("supply.compressor", None),
            ],
            [],
            [],
        );
        assert_eq!(list.rows.len(), 2);
        let cylinders = list
            .rows
            .iter()
            .find(|row| row.name.contains("Cilindro"))
            .expect("the cylinder row exists");
        assert_eq!(cylinders.quantity, 2);
        assert_eq!(cylinders.designations, "1A1, 1A2");
        assert!(list.rows.iter().any(|row| row.designations.is_empty()));
    }

    #[test]
    fn the_parts_list_has_a_tube_row_per_size_and_class_with_the_drawn_length() {
        let list = PartsList::group(
            &Catalog::builtin(),
            [],
            [],
            [
                TubeRowInput {
                    od: Some(1000),
                    class: LineClass::Working,
                    length_mm: 1_200,
                },
                TubeRowInput {
                    od: Some(1000),
                    class: LineClass::Working,
                    length_mm: 2_200,
                },
                TubeRowInput {
                    od: Some(400),
                    class: LineClass::Working,
                    length_mm: 300,
                },
                TubeRowInput {
                    od: Some(1000),
                    class: LineClass::Pilot,
                    length_mm: 50,
                },
            ],
        );
        assert_eq!(list.tubes.len(), 3);
        assert_eq!(list.tubes[0].od, Some(400));
        assert_eq!(list.tubes[0].text(), "Tubo 4 mm · trabajo · 0,3 m");
        assert_eq!(list.tubes[1].od, Some(1000));
        assert_eq!(list.tubes[1].length_mm, 3_400);
        assert_eq!(list.tubes[1].text(), "Tubo 10 mm · trabajo · 3,4 m");
        assert_eq!(list.tubes[2].class, LineClass::Pilot);
        assert_eq!(list.tubes[2].text(), "Tubo 10 mm · pilotaje · < 0,1 m");
    }

    #[test]
    fn an_unspecified_pipe_has_its_own_tube_row() {
        let list = PartsList::group(
            &Catalog::builtin(),
            [],
            [],
            [TubeRowInput {
                od: None,
                class: LineClass::Drain,
                length_mm: 4_000,
            }],
        );
        assert_eq!(
            list.tubes[0].text(),
            "Tubo sin especificar · drenaje · 4,0 m"
        );
    }

    #[test]
    fn the_parts_list_names_the_parent_and_the_port() {
        let catalog = Catalog::builtin();
        let list = PartsList::group(
            &catalog,
            [("valve.3_2.spring_return", Some("1A1"))],
            [
                (
                    "valve.3_2.spring_return",
                    Some("1A1"),
                    "3",
                    "silencer.pneumatic",
                ),
                (
                    "valve.3_2.spring_return",
                    Some("1A1"),
                    "3",
                    "silencer.pneumatic",
                ),
            ],
            [],
        );
        assert_eq!(list.closures.len(), 1, "one port, one grouped row");
        assert_eq!(list.closures[0].quantity, 2);
        assert_eq!(list.closures[0].text(), "Silenciador · 1A1 · escape 3");
        assert!(!list.is_empty());

        // A closure with no designation names its catalog id, as the component does.
        let only = PartsList::group(
            &catalog,
            [],
            [("valve.3_2.spring_return", None, "3", "plug.blind")],
            [],
        );
        assert_eq!(
            only.closures[0].text(),
            "Tapón ciego · valve.3_2.spring_return · escape 3"
        );
        assert!(!only.is_empty(), "a list with only a closure is not empty");
    }

    #[test]
    fn closures_take_the_rows_after_the_components_and_before_the_tubes() {
        let catalogue = Catalog::builtin();
        let placements: Vec<(&str, Option<&str>)> = (0..MAX_ROWS - 1)
            .map(|_| ("supply.compressor", None))
            .collect();
        // The placements above group into one row, so the closures must fill the rest of
        // the list on their own; their ports are unknown to the parent so each keeps its
        // bare id and stays a separate row.
        let ports: Vec<String> = (0..=MAX_ROWS).map(|index| format!("p{index}")).collect();
        let closures: Vec<(&str, Option<&str>, &str, &str)> = ports
            .iter()
            .map(|port| ("unknown.parent", Some("1V1"), port.as_str(), "plug.blind"))
            .collect();
        let list = PartsList::group(
            &catalogue,
            placements,
            closures,
            (0..2).map(|index| TubeRowInput {
                od: Some(u16::try_from(400 + index).unwrap_or_default()),
                class: LineClass::Working,
                length_mm: 100,
            }),
        );
        assert_eq!(list.rows.len(), 1);
        // The component row leaves room for `MAX_ROWS - 1` closures; the rest are cut and
        // counted, and the tubes get no room at all because the closures took it.
        assert_eq!(list.closures.len(), MAX_ROWS - 1);
        assert_eq!(list.omitted_closures, 2);
        assert_eq!(list.tubes.len(), 0);
        assert_eq!(list.omitted_tubes, 2);
    }

    #[test]
    fn the_components_take_the_rows_first_and_the_tubes_report_their_cut() {
        let catalogue = Catalog::builtin();
        let placements: Vec<(&str, Option<&str>)> =
            (0..MAX_ROWS).map(|_| ("supply.compressor", None)).collect();
        // One row, because they are one definition; the room left is for tubes.
        let list = PartsList::group(
            &catalogue,
            placements,
            [],
            (0..MAX_ROWS + 2).map(|index| TubeRowInput {
                od: Some(u16::try_from(400 + index).unwrap_or_default()),
                class: LineClass::Working,
                length_mm: 100,
            }),
        );
        assert_eq!(list.rows.len(), 1);
        assert_eq!(list.tubes.len(), MAX_ROWS - 1);
        assert_eq!(list.omitted_tubes, 3);
        assert_eq!(list.omitted, 0);
    }
}
