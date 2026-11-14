// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The data fields of the control-diagram title block.
//!
//! A pneumatic or electrical schematic is a control document, so its block follows the
//! data fields of `NCh 14.Of93` (`ISO 7200`) on a sheet of `NCh 13.Of93` (`ISO 5457`): the
//! company or owner, the project or installation, the machine or system, the title, the
//! document number, the revision, the date, the sheet number, the three signatures, the
//! scale and the standards the drawing follows (see `docs/reference/title-block.md`).
//!
//! The scale is present because the Chilean guide lists it, but a schematic is not drawn
//! to a ratio: it defaults to `S/E` ("sin escala") and stays editable for a company that
//! wants another note. Material, unit and projection symbol are still absent.
//!
//! The sheet number is not stored: it is the circuit's position in the project, which the
//! document already knows.

/// Longest accepted value of a field, in characters.
///
/// The standard bounds each field, but its exact lengths depend on the company; one
/// generous bound keeps a malicious or pasted text from growing the file, and the block
/// elides what does not fit on the sheet.
pub const MAX_FIELD: usize = 120;

/// Longest accepted note, in characters. It is multi-line, so it gets more room.
pub const MAX_REMARK: usize = 500;

/// Where the block sits on the sheet.
///
/// `NCh 14.Of93` (`ISO 7200:1984`) places it at the bottom right and bounds the
/// identification zone at 170 mm; the recommended template extends the block to 180 mm.
/// The top-right and full-width bottom variants are the other standard placements;
/// `None` removes the block from the sheet without losing its fields.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BlockPosition {
    /// Top right of the drawing space.
    TopRight,
    /// Bottom right: the standard position.
    #[default]
    BottomRight,
    /// Bottom edge, centred and edge to edge.
    BottomFull,
    /// No block on the sheet.
    None,
}

impl BlockPosition {
    /// Every position, in the order the editor offers them on a portrait sheet.
    pub const ALL: [Self; 4] = [
        Self::TopRight,
        Self::BottomRight,
        Self::BottomFull,
        Self::None,
    ];

    /// Stable name used by the file format and the ABI.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::TopRight => "top-right",
            Self::BottomRight => "bottom-right",
            Self::BottomFull => "bottom-full",
            Self::None => "none",
        }
    }

    /// Position named `name`, or `BottomRight` when the name is unknown.
    ///
    /// The old `right`, `full` and `left` names are read as their nearest new position so a
    /// format 10 block never fails to open.
    #[must_use]
    pub fn from_name(name: &str) -> Self {
        match name {
            "top-right" => Self::TopRight,
            "bottom-full" | "full" => Self::BottomFull,
            "none" => Self::None,
            // "bottom-right", "right" and the retired "left" all fall here.
            _ => Self::BottomRight,
        }
    }
}

/// The content a block is for: a school sheet, a technical drawing or a full industrial
/// document.
///
/// The model chooses the default field set; the user may hide fields individually without
/// losing their values, which is the "Personalizado" state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ContentModel {
    /// Title, author, course, date and sheet: the compact school block.
    School,
    /// The school fields plus number, revision, checker, approver and the standards.
    Technical,
    /// Every field, including project, equipment and the `S/E` scale.
    #[default]
    Industrial,
}

impl ContentModel {
    /// Every model, in the order the editor offers them.
    pub const ALL: [Self; 3] = [Self::School, Self::Technical, Self::Industrial];

    /// Stable name used by the file format and the ABI.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::School => "school",
            Self::Technical => "technical",
            Self::Industrial => "industrial",
        }
    }

    /// Model named `name`, or `Industrial` when the name is unknown.
    ///
    /// The retired Spanish names of format 11 are mapped by the reader in
    /// `punchi-fluid-file`, so this function and the format only ever name a model in
    /// English.
    #[must_use]
    pub fn from_name(name: &str) -> Self {
        match name {
            "school" => Self::School,
            "technical" => Self::Technical,
            _ => Self::Industrial,
        }
    }

    /// The stored fields this model shows by default; the sheet `n/N` is always derived.
    #[must_use]
    pub fn defaults(self) -> FieldSet {
        let fields: &[TitleField] = match self {
            Self::School => &[
                TitleField::Owner,
                TitleField::Title,
                TitleField::Prepared,
                TitleField::Date,
            ],
            Self::Technical => &[
                TitleField::Owner,
                TitleField::Title,
                TitleField::Number,
                TitleField::Revision,
                TitleField::Date,
                TitleField::Prepared,
                TitleField::Checked,
                TitleField::Approved,
                TitleField::Norms,
            ],
            Self::Industrial => &TitleField::ALL,
        };
        FieldSet::from_fields(fields)
    }
}

/// Which stored fields are visible in the block.
///
/// A bit per [`TitleField`]. The derived sheet is not part of the set: it is always shown.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FieldSet(u16);

impl FieldSet {
    /// No stored field is shown.
    pub const NONE: Self = Self(0);

    /// Every stored field is shown.
    #[must_use]
    pub const fn all() -> Self {
        Self((1 << TitleField::ALL.len()) - 1)
    }

    /// A set with exactly `fields`.
    #[must_use]
    pub fn from_fields(fields: &[TitleField]) -> Self {
        let mut set = Self::NONE;
        for field in fields {
            set.insert(*field);
        }
        set
    }

    /// `true` when `field` is shown.
    #[must_use]
    pub const fn contains(self, field: TitleField) -> bool {
        self.0 & (1 << field.index()) != 0
    }

    /// Shows `field`.
    pub fn insert(&mut self, field: TitleField) {
        self.0 |= 1 << field.index();
    }

    /// Hides `field`.
    pub fn remove(&mut self, field: TitleField) {
        self.0 &= !(1 << field.index());
    }

    /// Shows or hides `field`.
    pub fn set(&mut self, field: TitleField, visible: bool) {
        if visible {
            self.insert(field);
        } else {
            self.remove(field);
        }
    }

    /// The raw bits, for the file and the ABI.
    #[must_use]
    pub const fn bits(self) -> u16 {
        self.0
    }

    /// A set from raw bits; fields this build does not know are ignored.
    #[must_use]
    pub const fn from_bits(bits: u16) -> Self {
        Self(bits & FieldSet::all().0)
    }
}

/// One editable field of the block.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum TitleField {
    /// Company or owner responsible for the document.
    Owner,
    /// Project, plant, line or area.
    Project,
    /// Machine, hydraulic power unit or pneumatic circuit.
    Equipment,
    /// Title of the document.
    Title,
    /// Document identification number.
    Number,
    /// Revision index.
    Revision,
    /// Date of issue, as the user wrote it.
    Date,
    /// Person who prepared the drawing.
    Prepared,
    /// Person who checked it.
    Checked,
    /// Person who approved it.
    Approved,
    /// Scale of the drawing, `S/E` when it is not to scale.
    Scale,
    /// Representation and symbology standards actually used.
    Norms,
}

impl TitleField {
    /// Every editable field, in the order the block reads.
    pub const ALL: [Self; 12] = [
        Self::Owner,
        Self::Project,
        Self::Equipment,
        Self::Title,
        Self::Number,
        Self::Revision,
        Self::Date,
        Self::Prepared,
        Self::Checked,
        Self::Approved,
        Self::Scale,
        Self::Norms,
    ];

    /// Position in [`Self::ALL`], used as the bit of a [`FieldSet`].
    #[must_use]
    pub const fn index(self) -> u16 {
        self as u16
    }

    /// Stable name used by the file format and the ABI.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Owner => "owner",
            Self::Project => "project",
            Self::Equipment => "equipment",
            Self::Title => "title",
            Self::Number => "number",
            Self::Revision => "revision",
            Self::Date => "date",
            Self::Prepared => "prepared",
            Self::Checked => "checked",
            Self::Approved => "approved",
            Self::Scale => "scale",
            Self::Norms => "norms",
        }
    }

    /// Field named `name`, or `None` when the name is not one of the twelve.
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|field| field.as_str() == name)
    }
}

/// The identification block of one circuit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TitleBlock {
    /// Company or owner responsible for the document.
    pub owner: String,
    /// Project, plant, line or area.
    pub project: String,
    /// Machine or system the diagram belongs to.
    pub equipment: String,
    /// Document title; the circuit title is the default.
    pub title: String,
    /// Document number; the file stem is the default when the document is printed.
    pub number: String,
    /// Revision index.
    pub revision: String,
    /// Date of issue; the caller supplies it when the field is empty.
    pub date: String,
    /// Prepared by.
    pub prepared: String,
    /// Checked by.
    pub checked: String,
    /// Approved by.
    pub approved: String,
    /// Scale, `S/E` when the schematic is not to scale.
    pub scale: String,
    /// Representation and symbology standards actually used.
    pub norms: String,
    /// Free note for the editor; not printed on the sheet yet.
    pub remark: String,
    /// Sheet number as `n/N`: the circuit's position and the number of circuits.
    ///
    /// Derived when the document is printed, so it is not one of the twelve stored fields.
    pub sheet: String,
    /// Content model that chooses the default field set.
    pub model: ContentModel,
    /// Stored fields shown in the block; hiding one keeps its value.
    pub visible: FieldSet,
    /// Where the block sits on the sheet.
    pub position: BlockPosition,
    /// Extra warning, for example that the symbols are not verified yet.
    pub note: Option<String>,
}

impl Default for TitleBlock {
    fn default() -> Self {
        Self {
            owner: String::new(),
            project: String::new(),
            equipment: String::new(),
            title: String::new(),
            number: String::new(),
            revision: String::from("A"),
            date: String::new(),
            prepared: String::new(),
            checked: String::new(),
            approved: String::new(),
            scale: String::from("S/E"),
            norms: String::new(),
            remark: String::new(),
            sheet: String::from("1/1"),
            model: ContentModel::Industrial,
            visible: FieldSet::all(),
            position: BlockPosition::BottomRight,
            note: None,
        }
    }
}

impl TitleBlock {
    /// A block with a title and a date; the revision starts at `A` and the scale at `S/E`.
    #[must_use]
    pub fn new(title: impl Into<String>, date: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            date: date.into(),
            ..Self::default()
        }
    }

    /// Adds the note that the symbols have not been verified against the standard.
    #[must_use]
    pub fn with_unverified_note(mut self) -> Self {
        self.note = Some(String::from("symbols not yet verified against ISO 1219-1"));
        self
    }

    /// Same block with a note only when `unverified`.
    #[must_use]
    pub fn with_note_if(mut self, unverified: bool) -> Self {
        if unverified {
            self.note = Some(String::from("symbols not yet verified against ISO 1219-1"));
        } else {
            self.note = None;
        }
        self
    }

    /// Value of `field`.
    #[must_use]
    pub fn field(&self, field: TitleField) -> &str {
        match field {
            TitleField::Owner => &self.owner,
            TitleField::Project => &self.project,
            TitleField::Equipment => &self.equipment,
            TitleField::Title => &self.title,
            TitleField::Number => &self.number,
            TitleField::Revision => &self.revision,
            TitleField::Date => &self.date,
            TitleField::Prepared => &self.prepared,
            TitleField::Checked => &self.checked,
            TitleField::Approved => &self.approved,
            TitleField::Scale => &self.scale,
            TitleField::Norms => &self.norms,
        }
    }

    /// Sets `field`, trimmed and bounded. Returns `false` when the text is unusable.
    ///
    /// An empty value clears the field: it is not an error, because a course may leave the
    /// checker blank. Control characters are refused rather than stripped, so a pasted
    /// line break cannot break the one-line cell.
    pub fn set(&mut self, field: TitleField, value: impl Into<String>) -> bool {
        let Some(text) = clean(&value.into()) else {
            return false;
        };
        match field {
            TitleField::Owner => self.owner = text,
            TitleField::Project => self.project = text,
            TitleField::Equipment => self.equipment = text,
            TitleField::Title => self.title = text,
            TitleField::Number => self.number = text,
            TitleField::Revision => self.revision = text,
            TitleField::Date => self.date = text,
            TitleField::Prepared => self.prepared = text,
            TitleField::Checked => self.checked = text,
            TitleField::Approved => self.approved = text,
            TitleField::Scale => self.scale = text,
            TitleField::Norms => self.norms = text,
        }
        true
    }

    /// Switches to `model` and restores that model's default field set.
    ///
    /// The values stay: changing the model is a choice of what to print, not a reset of the
    /// document.
    pub fn set_model(&mut self, model: ContentModel) {
        self.model = model;
        self.visible = model.defaults();
    }

    /// Sets the free note, trimmed and bounded. Line breaks are allowed; other control
    /// characters are refused.
    pub fn set_remark(&mut self, text: impl Into<String>) -> bool {
        let Some(text) = clean_multiline(&text.into()) else {
            return false;
        };
        self.remark = text;
        true
    }

    /// Shows or hides one stored field, keeping its value.
    pub fn set_visible(&mut self, field: TitleField, visible: bool) {
        self.visible.set(field, visible);
    }

    /// Restores the field set of the current model.
    pub fn reset_visibility(&mut self) {
        self.visible = self.model.defaults();
    }

    /// `true` when the user hid or showed a field away from the model's default set.
    #[must_use]
    pub fn is_customized(&self) -> bool {
        self.visible != self.model.defaults()
    }

    /// The stored fields in printing order, for the layout and the editor.
    #[must_use]
    pub fn visible_fields(&self) -> Vec<TitleField> {
        TitleField::ALL
            .into_iter()
            .filter(|field| self.visible.contains(*field))
            .collect()
    }

    /// The block with every field trimmed and bounded, for a value read from a file.
    #[must_use]
    pub fn sanitized(mut self) -> Self {
        for field in TitleField::ALL {
            let text = clean(self.field(field)).unwrap_or_default();
            let _ = self.set(field, text);
        }
        let remark = clean_multiline(&self.remark).unwrap_or_default();
        self.remark = remark;
        self
    }
}

/// Trims `text` and bounds it, or `None` when it carries a control character.
fn clean(text: &str) -> Option<String> {
    let text = text.trim();
    if text.chars().any(char::is_control) {
        return None;
    }
    let bounded: String = text.chars().take(MAX_FIELD).collect();
    Some(bounded)
}

/// Like [`clean`], but a note may carry line breaks and tabs.
fn clean_multiline(text: &str) -> Option<String> {
    let text = text.trim();
    if text
        .chars()
        .any(|character| character.is_control() && !matches!(character, '\n' | '\r' | '\t'))
    {
        return None;
    }
    let bounded: String = text.chars().take(MAX_REMARK).collect();
    Some(bounded)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_field_is_trimmed_and_bounded() {
        let mut block = TitleBlock::default();
        assert!(block.set(TitleField::Owner, "  Curso de Neumática  "));
        assert_eq!(block.owner, "Curso de Neumática");
        assert!(block.set(TitleField::Title, "x".repeat(MAX_FIELD + 40)));
        assert_eq!(block.title.chars().count(), MAX_FIELD);
    }

    #[test]
    fn a_control_character_is_refused_instead_of_breaking_the_cell() {
        let mut block = TitleBlock::default();
        assert!(!block.set(TitleField::Checked, "Ana\nLuis"));
        assert!(block.checked.is_empty());
    }

    #[test]
    fn the_default_revision_is_a_and_the_scale_is_not_to_scale() {
        let block = TitleBlock::default();
        assert_eq!(block.revision, "A");
        assert_eq!(block.scale, "S/E");
        assert!(block.project.is_empty());
        assert!(block.equipment.is_empty());
        assert!(block.norms.is_empty());
    }

    #[test]
    fn the_guide_fields_round_trip() {
        let mut block = TitleBlock::default();
        for (field, value) in [
            (TitleField::Project, "Línea 2"),
            (TitleField::Equipment, "Prensa 4"),
            (TitleField::Scale, "1:50"),
            (TitleField::Norms, "NCh 14 · NCh 1768/1"),
        ] {
            assert!(block.set(field, value));
            assert_eq!(block.field(field), value);
        }
    }

    #[test]
    fn a_model_chooses_its_fields_and_a_custom_visibility_keeps_the_values() {
        let mut block = TitleBlock::default();
        assert_eq!(block.model, ContentModel::Industrial);
        assert_eq!(block.visible, FieldSet::all());
        assert!(!block.is_customized());

        block.set_model(ContentModel::School);
        assert!(block.visible.contains(TitleField::Owner));
        assert!(block.visible.contains(TitleField::Title));
        assert!(!block.visible.contains(TitleField::Number));
        assert!(!block.is_customized());

        assert!(block.set(TitleField::Number, "PF-1"));
        block.set_visible(TitleField::Number, true);
        assert!(block.is_customized());
        assert_eq!(block.number, "PF-1");
        block.set_visible(TitleField::Number, false);
        assert_eq!(block.number, "PF-1", "hiding keeps the value");
        block.reset_visibility();
        assert_eq!(block.visible, ContentModel::School.defaults());
    }

    #[test]
    fn a_field_set_is_a_bit_per_field() {
        let set = FieldSet::from_fields(&[TitleField::Owner, TitleField::Scale]);
        assert!(set.contains(TitleField::Owner));
        assert!(set.contains(TitleField::Scale));
        assert!(!set.contains(TitleField::Title));
        assert_eq!(FieldSet::from_bits(set.bits()), set);
        assert_eq!(FieldSet::all().bits().count_ones(), 12);
    }

    #[test]
    fn a_note_keeps_line_breaks_and_refuses_other_controls() {
        let mut block = TitleBlock::default();
        assert!(block.set_remark("Primera línea.\nSegunda línea."));
        assert_eq!(block.remark, "Primera línea.\nSegunda línea.");
        assert!(!block.set_remark("Nota\u{0007}"));
        assert_eq!(block.remark, "Primera línea.\nSegunda línea.");
    }

    #[test]
    fn the_old_positions_read_as_the_new_ones() {
        assert_eq!(
            BlockPosition::from_name("right"),
            BlockPosition::BottomRight
        );
        assert_eq!(BlockPosition::from_name("left"), BlockPosition::BottomRight);
        assert_eq!(BlockPosition::from_name("full"), BlockPosition::BottomFull);
        assert_eq!(
            BlockPosition::from_name("top-right"),
            BlockPosition::TopRight
        );
        assert_eq!(BlockPosition::from_name("none"), BlockPosition::None);
    }

    #[test]
    fn every_field_has_a_stable_name() {
        for field in TitleField::ALL {
            assert_eq!(TitleField::from_name(field.as_str()), Some(field));
        }
        assert_eq!(TitleField::from_name("material"), None);
    }
}
