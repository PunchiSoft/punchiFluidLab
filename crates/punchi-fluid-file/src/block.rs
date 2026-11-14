// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The control-diagram block as a file stores it.
//!
//! The twelve text fields and the position of the block; the sheet number is derived from
//! the project and the warning note belongs to the print, not to the file.

use serde::{Deserialize, Serialize};

/// The twelve stored fields of a control-diagram block.
///
/// The sheet number is not here: it is the circuit's position in the project and the
/// document derives it when the block is printed. The scale is stored because the Chilean
/// guide lists it, but it starts at `S/E` ("sin escala") because a schematic is not drawn
/// to a ratio. Material, unit and projection are still deliberately absent.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StoredBlock {
    /// Company or owner.
    #[serde(default)]
    pub owner: String,
    /// Project, plant, line or area.
    #[serde(default)]
    pub project: String,
    /// Machine, power unit or pneumatic circuit.
    #[serde(default)]
    pub equipment: String,
    /// Document title; empty means the circuit title.
    #[serde(default)]
    pub title: String,
    /// Document number; empty means the file stem when it is printed.
    #[serde(default)]
    pub number: String,
    /// Revision index.
    #[serde(default = "default_revision")]
    pub revision: String,
    /// Date of issue.
    #[serde(default)]
    pub date: String,
    /// Prepared by.
    #[serde(default)]
    pub prepared: String,
    /// Checked by.
    #[serde(default)]
    pub checked: String,
    /// Approved by.
    #[serde(default)]
    pub approved: String,
    /// Scale, `S/E` when the schematic is not to scale.
    #[serde(default = "default_scale")]
    pub scale: String,
    /// Representation and symbology standards actually used.
    #[serde(default)]
    pub norms: String,
    /// Free note for the editor; not printed yet.
    #[serde(default)]
    pub remark: String,
    /// Content model: `school`, `technical` or `industrial`; format 11 wrote the first two
    /// in Spanish and the reader renames them.
    #[serde(default = "default_model")]
    pub model: String,
    /// Hidden fields, when the user customized the model; absent means the model's set.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub visible: Option<Vec<String>>,
    /// Where the block sits: `right` (default), `full`, `left` or `none`.
    #[serde(default = "default_position")]
    pub position: String,
}

fn default_model() -> String {
    String::from("industrial")
}

fn default_position() -> String {
    String::from("bottom-right")
}

fn default_revision() -> String {
    String::from("A")
}

fn default_scale() -> String {
    String::from("S/E")
}

impl Default for StoredBlock {
    fn default() -> Self {
        Self {
            owner: String::new(),
            project: String::new(),
            equipment: String::new(),
            title: String::new(),
            number: String::new(),
            revision: default_revision(),
            date: String::new(),
            prepared: String::new(),
            checked: String::new(),
            approved: String::new(),
            scale: default_scale(),
            norms: String::new(),
            remark: String::new(),
            model: default_model(),
            visible: None,
            position: default_position(),
        }
    }
}

impl StoredBlock {
    /// The twelve fields, by name, for validation and for the conversions.
    #[must_use]
    pub fn fields(&self) -> [(&'static str, &str); 12] {
        [
            ("owner", &self.owner),
            ("project", &self.project),
            ("equipment", &self.equipment),
            ("title", &self.title),
            ("number", &self.number),
            ("revision", &self.revision),
            ("date", &self.date),
            ("prepared", &self.prepared),
            ("checked", &self.checked),
            ("approved", &self.approved),
            ("scale", &self.scale),
            ("norms", &self.norms),
        ]
    }

    /// The stored form of a domain block.
    #[must_use]
    pub fn from_block(block: &punchi_fluid_core::TitleBlock) -> Self {
        Self {
            owner: block.owner.clone(),
            project: block.project.clone(),
            equipment: block.equipment.clone(),
            title: block.title.clone(),
            number: block.number.clone(),
            revision: block.revision.clone(),
            date: block.date.clone(),
            prepared: block.prepared.clone(),
            checked: block.checked.clone(),
            approved: block.approved.clone(),
            scale: block.scale.clone(),
            norms: block.norms.clone(),
            remark: block.remark.clone(),
            model: block.model.as_str().to_owned(),
            visible: if block.is_customized() {
                Some(
                    punchi_fluid_core::TitleField::ALL
                        .into_iter()
                        .filter(|field| block.visible.contains(*field))
                        .map(|field| field.as_str().to_owned())
                        .collect(),
                )
            } else {
                None
            },
            position: block.position.as_str().to_owned(),
        }
    }

    /// The domain block, with the circuit title as the default of the title field.
    #[must_use]
    pub fn to_block(&self, circuit_title: &str) -> punchi_fluid_core::TitleBlock {
        let model = punchi_fluid_core::ContentModel::from_name(&self.model);
        let visible = self.visible.as_ref().map_or_else(
            || model.defaults(),
            |names| {
                let fields: Vec<punchi_fluid_core::TitleField> = names
                    .iter()
                    .filter_map(|name| punchi_fluid_core::TitleField::from_name(name))
                    .collect();
                punchi_fluid_core::FieldSet::from_fields(&fields)
            },
        );
        let mut block = punchi_fluid_core::TitleBlock {
            owner: self.owner.clone(),
            project: self.project.clone(),
            equipment: self.equipment.clone(),
            title: self.title.clone(),
            number: self.number.clone(),
            revision: self.revision.clone(),
            date: self.date.clone(),
            prepared: self.prepared.clone(),
            checked: self.checked.clone(),
            approved: self.approved.clone(),
            scale: self.scale.clone(),
            norms: self.norms.clone(),
            remark: self.remark.clone(),
            sheet: String::from("1/1"),
            model,
            visible,
            position: punchi_fluid_core::BlockPosition::from_name(&self.position),
            note: None,
        };
        if block.title.is_empty() {
            circuit_title.clone_into(&mut block.title);
        }
        block.sanitized()
    }
}

/// Longest accepted value of a block field, in characters.
///
/// It matches `punchi_fluid_core::title_block::MAX_FIELD`; the format crate does not depend
/// on the core crate, so the number is stated in both and tested here.
pub const MAX_BLOCK_FIELD: usize = 120;
