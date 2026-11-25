// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! A draft editor for a placed directional valve.

use eframe::egui;
use punchi_fluid_catalog::{
    ActuatorKind, Behaviour, ComponentDefinition, ReturnKind, Side, ValvePosition,
};
use punchi_fluid_doc::{Placement, valid_designation};
use punchi_fluid_draw::{
    Ink, Orientation, Pt, Rect, Turned, draw_component_preview, symbol_box_mm,
};
use punchi_fluid_i18n::Language;

use crate::Window;
use crate::dialog::{Action, ButtonRole, Dialog, DialogResult, DialogStyle};
use crate::surface::{EguiSurface, ScreenInk};
use crate::theme::Tokens;

#[derive(Debug)]
pub(crate) struct ValveEditor {
    target: u64,
    designation: String,
    initial: usize,
    reflect_horizontal: bool,
    reflect_vertical: bool,
    focus_name: bool,
    return_focus: egui::Id,
}

fn editable(definition: &ComponentDefinition) -> bool {
    matches!(&definition.behaviour, Behaviour::Valve { positions, .. }
        if positions.len() == 2 && definition.body.is_some() && matches!(definition.port.len(), 3 | 5))
}

impl Window {
    /// Opens the shared editor on the placement selected from the sheet.
    pub(crate) fn open_valve_editor(&mut self, index: usize, return_focus: egui::Id) -> bool {
        if self.running {
            return false;
        }
        let Some(placement) = self.document.placements().get(index) else {
            return false;
        };
        let Some(definition) = self
            .catalog
            .get(&placement.catalog_id)
            .filter(|def| editable(def))
        else {
            return false;
        };
        let Behaviour::Valve {
            default_position, ..
        } = definition.behaviour
        else {
            return false;
        };
        self.valve_editor = Some(ValveEditor {
            target: placement.id(),
            designation: placement.designation.clone().unwrap_or_default(),
            initial: placement
                .edits
                .valve_initial_position
                .map_or(default_position, usize::from),
            reflect_horizontal: false,
            reflect_vertical: false,
            focus_name: true,
            return_focus,
        });
        true
    }

    pub(crate) fn show_valve_editor(&mut self, context: &egui::Context) {
        if self.close_question_open() {
            return;
        }
        let Some(mut editor) = self.valve_editor.take() else {
            return;
        };
        let Some(index) = self.document.index_of(editor.target) else {
            return;
        };
        let Some(placement) = self.document.placements().get(index) else {
            return;
        };
        if placement.circuit != self.document.active_circuit() {
            return;
        }
        let Some(definition) = self.catalog.get(&placement.catalog_id) else {
            return;
        };
        let Behaviour::Valve {
            positions,
            default_position,
            ..
        } = &definition.behaviour
        else {
            return;
        };
        let form = ValveForm {
            definition,
            placement,
            positions,
            default: *default_position,
            language: self.language,
            tokens: &self.theme.tokens,
            style: DialogStyle::new(&self.theme.tokens),
        };
        let name = match self.language {
            Language::English => definition.name.as_str(),
            Language::Spanish => definition.display_name(),
        };
        let title = self
            .language
            .format("valve_editor.title", &[("name", name)]);
        let dialog = Dialog {
            id: egui::Id::new("valve-editor"),
            title: &title,
            close_label: self.language.text("dialog.close"),
            preferred_width: (self.theme.tokens.unit * 57.0).max(930.0),
            style: form.style,
        };
        let response = dialog.show(
            context,
            &mut editor,
            |ui, editor| form.show(ui, editor),
            |editor| form.actions(editor),
        );
        if matches!(response.inner, Some(DialogResult::Action(true))) {
            let initial = if editor.initial == *default_position {
                None
            } else {
                u8::try_from(editor.initial).ok()
            };
            if self.document.configure_valve(
                index,
                &editor.designation,
                initial,
                editor.orientation(placement.orientation),
            ) {
                self.sync_selection_notice();
            }
        } else if response.inner.is_none() && !response.should_close() {
            self.valve_editor = Some(editor);
            return;
        }
        context.memory_mut(|memory| memory.request_focus(editor.return_focus));
    }
}

impl ValveEditor {
    fn orientation(&self, original: Orientation) -> Orientation {
        let horizontal = if self.reflect_horizontal {
            original.reflected()
        } else {
            original
        };
        if self.reflect_vertical {
            horizontal.reflected_vertically()
        } else {
            horizontal
        }
    }
}

struct ValveForm<'a> {
    definition: &'a ComponentDefinition,
    placement: &'a Placement,
    positions: &'a [ValvePosition],
    default: usize,
    language: Language,
    tokens: &'a Tokens,
    style: DialogStyle,
}

impl ValveForm<'_> {
    fn show(&self, ui: &mut egui::Ui, editor: &mut ValveEditor) {
        let unit = self.tokens.unit;
        let label = self.section_heading(ui, "valve_editor.designation");
        let response = ui
            .add_sized(
                egui::vec2(ui.available_width(), (unit * 2.0).max(34.0)),
                egui::TextEdit::singleline(&mut editor.designation)
                    .desired_width(ui.available_width()),
            )
            .labelled_by(label.id);
        if std::mem::take(&mut editor.focus_name) {
            response.request_focus();
        }
        if !valid_designation(&editor.designation) {
            ui.colored_label(
                self.style.error,
                self.language.text("valve_editor.invalid_designation"),
            );
        }
        ui.add_space(4.0);
        if ui.available_width() >= 680.0 {
            ui.columns(3, |columns| {
                self.side(&mut columns[0], Side::Left);
                self.body(&mut columns[1], editor);
                self.side(&mut columns[2], Side::Right);
            });
        } else {
            self.body(ui, editor);
            if ui.available_width() >= 460.0 {
                ui.columns(2, |columns| {
                    self.side(&mut columns[0], Side::Left);
                    self.side(&mut columns[1], Side::Right);
                });
            } else {
                self.side(ui, Side::Left);
                self.side(ui, Side::Right);
            }
        }
        ui.add_space(4.0);
        self.preview(ui, editor);
    }

    fn actions(&self, editor: &ValveEditor) -> Vec<Action<'static, bool>> {
        let source = self
            .placement
            .edits
            .valve_initial_position
            .map_or(self.default, usize::from);
        let changed = editor.designation.trim()
            != self.placement.designation.as_deref().unwrap_or("")
            || editor.initial != source
            || editor.reflect_horizontal
            || editor.reflect_vertical;
        vec![
            Action {
                label: self.language.text("valve_editor.accept"),
                value: true,
                role: ButtonRole::Primary,
                enabled: valid_designation(&editor.designation) && changed,
                focus: false,
            },
            Action {
                label: self.language.text("valve_editor.cancel"),
                value: false,
                role: ButtonRole::Secondary,
                enabled: true,
                focus: false,
            },
        ]
    }

    fn preview(&self, ui: &mut egui::Ui, editor: &mut ValveEditor) {
        let unit = self.tokens.unit;
        self.style.card(self.style.band).show(ui, |ui| {
            ui.set_width(ui.available_width());
            self.section_heading(ui, "valve_editor.preview");
            let orientation = editor.orientation(self.placement.orientation);
            let (area, _) = ui.allocate_exact_size(
                egui::vec2(ui.available_width(), unit * 8.0),
                egui::Sense::hover(),
            );
            paint_valve(
                ui,
                area,
                self.definition,
                editor.initial,
                orientation,
                self.style,
            );
            if let Some(position) = self.positions.get(editor.initial) {
                ui.colored_label(self.style.muted, position_summary(position, self.language));
            }
            ui.horizontal_wrapped(|ui| {
                ui.checkbox(
                    &mut editor.reflect_horizontal,
                    self.language.text("menu.mirror_horizontal"),
                );
                ui.checkbox(
                    &mut editor.reflect_vertical,
                    self.language.text("menu.mirror_vertical"),
                );
            });
        });
    }

    fn section_heading(&self, ui: &mut egui::Ui, key: &'static str) -> egui::Response {
        ui.label(
            egui::RichText::new(self.language.text(key))
                .size(self.style.text_size)
                .color(self.style.text)
                .strong(),
        )
    }

    fn side(&self, ui: &mut egui::Ui, side: Side) {
        let language = self.language;
        self.style.card(self.style.surface).show(ui, |ui| {
            ui.set_width(ui.available_width());
            self.section_heading(
                ui,
                if side == Side::Left {
                    "valve_editor.left"
                } else {
                    "valve_editor.right"
                },
            );
            let actuator = self.definition.actuator.filter(|part| part.side == side);
            let return_part = self
                .definition
                .resolved_return()
                .filter(|part| part.side == side);
            if let Some(part) = actuator {
                ui.label(language.text(actuator_key(part.kind)));
            }
            if let Some(part) = return_part {
                ui.label(language.text(return_key(part.kind)));
            }
            if actuator.is_none() && return_part.is_none() {
                ui.label(language.text("valve_editor.no_actuation"));
            }
            ui.label(
                egui::RichText::new(language.text("valve_editor.from_catalog"))
                    .small()
                    .color(self.style.muted),
            );
        });
    }

    fn body(&self, ui: &mut egui::Ui, editor: &mut ValveEditor) {
        self.style.card(self.style.surface).show(ui, |ui| {
            ui.set_width(ui.available_width());
            self.section_heading(ui, "valve_editor.body");
            ui.label(self.language.text("valve_editor.initial"));
            ui.columns(2, |columns| {
                for (index, (column, position)) in
                    columns.iter_mut().zip(self.positions).enumerate()
                {
                    let size = column.available_width().min(self.tokens.unit * 5.0);
                    let (area, symbol) =
                        column.allocate_exact_size(egui::vec2(size, size), egui::Sense::CLICK);
                    if symbol.clicked() {
                        editor.initial = index;
                    }
                    paint_position(
                        column,
                        area,
                        self.definition,
                        index,
                        editor.initial == index,
                        self.style,
                    );
                    let label = match self.language {
                        Language::English => &position.name,
                        Language::Spanish => position.display_name(),
                    };
                    column
                        .radio_value(&mut editor.initial, index, label)
                        .on_hover_text(position_summary(position, self.language));
                }
            });
        });
    }
}

fn position_summary(position: &ValvePosition, language: Language) -> String {
    let paths = position
        .connection
        .iter()
        .map(|path| {
            language.format(
                "valve_editor.path",
                &[("from", &path.from), ("to", &path.to)],
            )
        })
        .collect::<Vec<_>>()
        .join(" · ");
    if position.blocked.is_empty() {
        return paths;
    }
    let ports = position.blocked.join(", ");
    let closed = language.format("valve_editor.blocked", &[("ports", &ports)]);
    if paths.is_empty() {
        closed
    } else {
        format!("{paths} · {closed}")
    }
}

fn ink(style: DialogStyle, selected: bool, background: egui::Color32) -> ScreenInk {
    ScreenInk {
        ink: style.text,
        paper: background,
        muted: style.muted,
        dim: style.muted,
        accent: if selected { style.selected } else { background },
    }
}

fn paint_valve(
    ui: &egui::Ui,
    area: egui::Rect,
    definition: &ComponentDefinition,
    position: usize,
    orientation: Orientation,
    style: DialogStyle,
) {
    let (width, height) = symbol_box_mm(definition);
    let scale = area.width().min(area.height()) * 0.9 / width.max(height);
    let centre = Pt::new(width * 0.5, height * 0.5);
    let origin = area.center() - egui::vec2(centre.x * scale, centre.y * scale);
    let mut surface =
        EguiSurface::new(ui.ctx(), origin, scale).with_screen_ink(ink(style, true, style.band));
    let mut turned = Turned::new(&mut surface, orientation, centre);
    draw_component_preview(
        &mut turned,
        Rect::from_min_size(Pt::new(0.0, 0.0), width, height),
        definition,
        position,
        0.0,
        None,
        &Ink::print(),
    );
    ui.painter().with_clip_rect(area).extend(surface.shapes);
}

#[expect(
    clippy::cast_possible_truncation,
    reason = "the catalog's normalised square coordinates fit drawing points"
)]
fn paint_position(
    ui: &egui::Ui,
    area: egui::Rect,
    definition: &ComponentDefinition,
    position: usize,
    selected: bool,
    style: DialogStyle,
) {
    let mut squares = definition.body_squares();
    if definition
        .resolved_return()
        .is_some_and(|part| part.side == Side::Right)
    {
        squares.reverse();
    }
    let Some([x, y, w, h]) = squares.get(position).copied() else {
        return;
    };
    let area = area.shrink(4.0);
    let (width, height) = symbol_box_mm(definition);
    let span = (w as f32 * width).max(h as f32 * height);
    let scale = area.width().min(area.height()) / span;
    let origin = area.min - egui::vec2(x as f32 * width * scale, y as f32 * height * scale);
    let mut surface = EguiSurface::new(ui.ctx(), origin, scale).with_screen_ink(ink(
        style,
        selected,
        style.surface,
    ));
    draw_component_preview(
        &mut surface,
        Rect::from_min_size(Pt::new(0.0, 0.0), width, height),
        definition,
        position,
        0.0,
        None,
        &Ink::print(),
    );
    ui.painter()
        .with_clip_rect(area.expand(0.5))
        .extend(surface.shapes);
}

const fn actuator_key(kind: ActuatorKind) -> &'static str {
    match kind {
        ActuatorKind::Lever => "valve_actuator.lever",
        ActuatorKind::LeverDetent => "valve_actuator.lever_detent",
        ActuatorKind::PushButton => "valve_actuator.push_button",
        ActuatorKind::MushroomButton => "valve_actuator.mushroom_button",
        ActuatorKind::Pedal => "valve_actuator.pedal",
        ActuatorKind::Plunger => "valve_actuator.plunger",
        ActuatorKind::Roller => "valve_actuator.roller",
        ActuatorKind::RollerOneWay => "valve_actuator.roller_one_way",
        ActuatorKind::Pilot => "valve_actuator.pilot",
        ActuatorKind::PilotHydraulic => "valve_actuator.pilot_hydraulic",
        ActuatorKind::Solenoid => "valve_actuator.solenoid",
        ActuatorKind::SolenoidDouble => "valve_actuator.solenoid_double",
        ActuatorKind::SolenoidServo => "valve_actuator.solenoid_servo",
        ActuatorKind::Motor => "valve_actuator.motor",
    }
}

const fn return_key(kind: ReturnKind) -> &'static str {
    match kind {
        ReturnKind::Spring => "valve_return.spring",
        ReturnKind::Pneumatic => "valve_return.pneumatic",
        ReturnKind::Hydraulic => "valve_return.hydraulic",
        ReturnKind::Detent => "valve_return.detent",
    }
}
