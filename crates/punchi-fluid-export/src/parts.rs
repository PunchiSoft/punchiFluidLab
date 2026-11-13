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

use punchi_fluid_catalog::Catalog;

/// Most rows a printed list holds before it stops being a list and becomes a column.
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

/// The list a sheet prints.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PartsList {
    /// Rows, in the order the sheet prints them.
    pub rows: Vec<PartRow>,
    /// Placements left out because the list was full.
    pub omitted: usize,
}

impl PartsList {
    /// `true` when there is nothing to print.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    /// Groups placements into rows, in catalog-name order.
    ///
    /// The input is what the document holds: the catalog id of each placement and the name
    /// the user gave it. A definition the catalog does not know keeps its id as the name,
    /// so a sheet with a user file that failed to load still lists what is drawn.
    #[must_use]
    pub fn group<'a>(
        catalog: &Catalog,
        placements: impl IntoIterator<Item = (&'a str, Option<&'a str>)>,
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
        let omitted = rows.len().saturating_sub(MAX_ROWS);
        rows.truncate(MAX_ROWS);
        Self { rows, omitted }
    }
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
}
