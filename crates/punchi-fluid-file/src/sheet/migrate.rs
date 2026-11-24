// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! Reading an older sheet as the current format, one version at a time.

use super::{SHEET_MIN, SHEET_VERSION, Sheet, SheetError, StoredBlock, StoredCircuit, StoredSheet};

impl Sheet {
    /// Reads an older file as the current format, one version at a time.
    ///
    /// Each step only touches what that version lacked. A format 4 project already has its
    /// circuits, and a migration that reset them would silently merge every sheet into the
    /// first one.
    pub(super) fn migrate(&mut self) -> Result<(), SheetError> {
        if self.format >= SHEET_VERSION {
            return Ok(());
        }
        if self.format < SHEET_MIN {
            return Err(SheetError::UnsupportedVersion {
                found: self.format,
                supported: SHEET_VERSION,
            });
        }
        // 1–3 → 4: one circuit, which is what a single-sheet file meant.
        if self.format < 4 {
            if self.circuits.is_empty() {
                self.circuits = vec![StoredCircuit {
                    id: 0,
                    title: String::from("Circuito 1"),
                    block: StoredBlock {
                        title: String::from("Circuito 1"),
                        ..StoredBlock::default()
                    },
                }];
            }
            for placement in &mut self.placement {
                placement.circuit = 0;
            }
            for pipe in &mut self.pipe {
                pipe.circuit = 0;
            }
            self.active = 0;
        }
        // 4 → 5: an unbound recorder, which is the `None` the field already defaults to.
        // 6 → 7: the sheet. An older file was drawn on a bare 200 × 130 mm rectangle, so it
        // reads as an unframed A4: the rectangle sits inside the paper and no component
        // becomes "outside the frame" just by opening the file in a newer build.
        if self.format < 7 {
            self.sheet = StoredSheet::default();
        }
        // 7 → 8: the control block. An older circuit had no block, so its title field is
        // the circuit title and the revision the one this build starts at.
        if self.format < 8 {
            for circuit in &mut self.circuits {
                if circuit.block.title.is_empty() {
                    circuit.block.title.clone_from(&circuit.title);
                }
            }
        }
        // 8 → 9: the project, equipment, scale and norms of the block. Serde has already
        // given an absent field its default, so a format 8 block reads as the same block
        // with an empty project and equipment, a `S/E` scale and no norms.
        if self.format < 9 {
            for circuit in &mut self.circuits {
                if circuit.block.scale.is_empty() {
                    circuit.block.scale = String::from("S/E");
                }
            }
        }
        // 9 → 10: the content model and per-field visibility. Serde gives an absent model
        // `industrial` and an absent visibility list that model's full field set, so an
        // older block keeps printing every field it used to print.
        // 10 → 11: the new positions and the free note. `right` and the retired `left` read
        // as bottom right; `full` reads as the bottom band; the note starts empty.
        if self.format < 11 {
            for circuit in &mut self.circuits {
                punchi_fluid_core::BlockPosition::from_name(&circuit.block.position)
                    .as_str()
                    .clone_into(&mut circuit.block.position);
            }
        }
        // 11 → 12: the content-model names. Format 11 wrote the Spanish short names, so a
        // file from that release is read as the model it meant and saved in English.
        if self.format < 12 {
            for circuit in &mut self.circuits {
                let model = match circuit.block.model.as_str() {
                    "escolar" => "school",
                    "tecnico" => "technical",
                    _ => continue,
                };
                model.clone_into(&mut circuit.block.model);
            }
        }
        // 12 → 13: the tube diameter of a pneumatic pipe. The serde default already gives an
        // absent field `None`, which is "unspecified" and keeps the old 0.42 mm width; the
        // writer never adds the key for an unspecified pipe, so an old sheet re-saved by
        // this build still looks as it did.
        // 14 → 15: the compressor gained its intake. Format 14 stored the compressor's only
        // port as `1`, and that port was the delivery; the intake took `1` and the delivery
        // moved to `2`. A pipe the old sheet stored on the compressor's port `1` therefore
        // moves with the delivery, and no other port changes.
        if self.format < 15 {
            let placements = &self.placement;
            let is_compressor = |id: u64| {
                placements.iter().any(|placement| {
                    placement.id == id
                        && matches!(
                            placement.catalog.as_str(),
                            "supply.compressor" | "supply.compressor_adjustable"
                        )
                })
            };
            for pipe in &mut self.pipe {
                if pipe.from_port == "1" && is_compressor(pipe.from) {
                    pipe.from_port = String::from("2");
                }
                if pipe.to_port == "1" && is_compressor(pipe.to) {
                    pipe.to_port = String::from("2");
                }
            }
        }
        self.format = SHEET_VERSION;
        Ok(())
    }
}
