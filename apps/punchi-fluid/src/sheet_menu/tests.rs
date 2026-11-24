// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The sheet menu, the armed move and the two mockup windows, drawn headless.

use eframe::egui;
use egui_kittest::kittest::Queryable;
use punchi_fluid_doc::{Document, Endpoint, Placement};
use punchi_fluid_draw::{Orientation, Pt};

use punchi_fluid_i18n::Language;

use crate::{Window, theme};

use super::MenuAction;

/// What the frame left behind, so a test can read the window while the harness borrows it.
#[expect(
    clippy::struct_excessive_bools,
    reason = "the probe records independent facts of one frame, not a state machine"
)]
#[derive(Debug, Default)]
struct Probe {
    revision: u64,
    menu: bool,
    moving: bool,
    mockup: bool,
    inspector: bool,
    selected: Vec<usize>,
    count: usize,
    centre: Option<Pt>,
    orientation: Option<Orientation>,
    can_undo: bool,
    pressed_valve: bool,
    valve_position: usize,
}

impl Probe {
    fn read(&mut self, window: &Window) {
        self.revision = window.document.revision();
        self.menu = window.menu.is_open();
        self.moving = window.moving.is_some();
        self.mockup = window.mockups.is_open();
        self.inspector = window.inspector.is_open();
        self.selected.clone_from(&window.selected);
        self.count = window.document.placements().len();
        self.centre = window
            .document
            .placements()
            .first()
            .map(|placement| placement.centre);
        self.orientation = window
            .document
            .placements()
            .first()
            .map(|placement| placement.orientation);
        self.can_undo = window.document.can_undo();
        self.pressed_valve = window.pressed_valve.is_some();
        self.valve_position = window
            .model
            .as_ref()
            .map_or(0, |model| model.valve_position(1));
    }
}

fn window_with_component() -> Window {
    let mut window = Window::new(theme::Theme::fallback(), Document::new(), None, None);
    let centre = window.page.rect().center();
    window.document.place(Placement::new(
        "valve.3_2.spring_return",
        centre,
        20.0,
        20.0,
    ));
    window
}

/// A supply, a 5/2 valve at the centre of the page and a cylinder, so the valve has a model.
fn window_with_single_acting_cylinder() -> Window {
    let mut window = Window::new(theme::Theme::fallback(), Document::new(), None, None);
    let centre = window.page.rect().center();
    window.document.place(Placement::new(
        "cylinder.single_acting_20_100",
        centre,
        44.0,
        20.0,
    ));
    window
}

fn window_with_a_model() -> Window {
    let mut window = Window::new(theme::Theme::fallback(), Document::new(), None, None);
    let centre = window.page.rect().center();
    let supply = window.document.place(Placement::new(
        "supply.compressor",
        Pt::new(centre.x - 80.0, centre.y),
        20.0,
        20.0,
    ));
    let valve = window.document.place(Placement::new(
        "valve.5_2.spring_return",
        centre,
        20.0,
        20.0,
    ));
    let cylinder = window.document.place(Placement::new(
        "cylinder.double_acting_32_200",
        Pt::new(centre.x + 80.0, centre.y),
        44.0,
        20.0,
    ));
    let supply_id = window.document.placements()[supply].id();
    let valve_id = window.document.placements()[valve].id();
    let cylinder_id = window.document.placements()[cylinder].id();
    window.document.connect(
        Endpoint::new(supply_id, "1"),
        Endpoint::new(valve_id, "1"),
        None,
    );
    window.document.connect(
        Endpoint::new(valve_id, "2"),
        Endpoint::new(cylinder_id, "1"),
        None,
    );
    window.document.connect(
        Endpoint::new(valve_id, "4"),
        Endpoint::new(cylinder_id, "2"),
        None,
    );
    window.rebuild_if_stale();
    assert!(window.model.is_some(), "the test circuit has a model");
    window
}

fn harness(window: &mut Window) -> egui_kittest::Harness<'_, Probe> {
    let harness = egui_kittest::Harness::builder()
        .with_size(egui::vec2(1200.0, 800.0))
        .with_step_dt(1.0 / 60.0)
        .with_max_steps(120)
        .build_state(
            move |context, probe: &mut Probe| {
                window.show_sheet(context);
                window.show_mockups(context);
                probe.read(window);
            },
            Probe::default(),
        );
    if crate::theme::install_font(&harness.ctx, None).is_err() {
        eprintln!("no system font: the menu tests need one");
    }
    harness
}

fn right_click_at(harness: &mut egui_kittest::Harness<'_, Probe>, position: egui::Pos2) {
    harness
        .input_mut()
        .events
        .push(egui::Event::PointerMoved(position));
    for pressed in [true, false] {
        harness.input_mut().events.push(egui::Event::PointerButton {
            pos: position,
            button: egui::PointerButton::Secondary,
            pressed,
            modifiers: egui::Modifiers::default(),
        });
    }
    harness.run();
}

fn primary_click_at(harness: &mut egui_kittest::Harness<'_, Probe>, position: egui::Pos2) {
    harness
        .input_mut()
        .events
        .push(egui::Event::PointerMoved(position));
    for pressed in [true, false] {
        harness.input_mut().events.push(egui::Event::PointerButton {
            pos: position,
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: egui::Modifiers::default(),
        });
    }
    harness.run();
}

#[test]
fn right_click_selects_the_symbol_and_opens_its_menu() {
    let mut window = window_with_component();
    let mut harness = harness(&mut window);
    let revision = harness.state().revision;
    let sheet = harness.get_by_label("drawing sheet").rect();
    right_click_at(&mut harness, sheet.center());
    assert!(harness.state().menu, "the menu is open");
    assert_eq!(harness.state().selected, vec![0]);
    assert_eq!(
        harness.state().revision,
        revision,
        "opening the menu is not a document change"
    );
    assert!(harness.query_by_label("Configure").is_some());
    assert!(harness.query_by_label("Mirror vertically").is_some());
}

#[test]
fn right_click_on_the_empty_sheet_opens_nothing() {
    let mut window = Window::new(theme::Theme::fallback(), Document::new(), None, None);
    let mut harness = harness(&mut window);
    let sheet = harness.get_by_label("drawing sheet").rect();
    right_click_at(&mut harness, sheet.center());
    assert!(!harness.state().menu);
    assert!(harness.state().selected.is_empty());
}

#[test]
fn right_click_on_a_pipe_opens_nothing_and_keeps_the_selection() {
    let mut document = Document::new();
    let supply = document.place(Placement::new(
        "supply.compressor",
        Pt::new(50.0, 50.0),
        20.0,
        20.0,
    ));
    let valve = document.place(Placement::new(
        "valve.3_2.spring_return",
        Pt::new(140.0, 50.0),
        20.0,
        20.0,
    ));
    let supply_id = document.placements()[supply].id();
    let valve_id = document.placements()[valve].id();
    document.connect(
        Endpoint::new(supply_id, "1"),
        Endpoint::new(valve_id, "1"),
        None,
    );
    let mut window = Window::new(theme::Theme::fallback(), document, None, None);
    let route = crate::selection::route_of(&window.document, &window.catalog, 0)
        .expect("the pipe has a route");
    let point = route
        .windows(2)
        .filter_map(|segment| match segment {
            [start, end] => Some(Pt::new(start.x.midpoint(end.x), start.y.midpoint(end.y))),
            _ => None,
        })
        .find(|point| {
            window.hit_placement(*point).is_none()
                && crate::selection::pipe_at(&window.document, &window.catalog, *point) == Some(0)
        })
        .expect("the route has a point clear of the components");
    window.selected_pipe = Some(0);
    window.open_menu_at_point(point, egui::pos2(10.0, 10.0));
    assert!(!window.menu.is_open(), "a hose has no menu yet");
    assert_eq!(
        window.selected_pipe,
        Some(0),
        "the hose keeps its selection"
    );
    assert!(window.selected.is_empty());
}

#[test]
fn escape_closes_the_menu() {
    let mut window = window_with_component();
    let mut harness = harness(&mut window);
    let sheet = harness.get_by_label("drawing sheet").rect();
    right_click_at(&mut harness, sheet.center());
    assert!(harness.state().menu);
    harness.key_press(egui::Key::Escape);
    harness.run();
    assert!(!harness.state().menu);
}

#[test]
fn shift_f10_opens_the_menu_of_the_selection() {
    let mut window = window_with_component();
    window.selected = vec![0];
    let mut harness = harness(&mut window);
    harness.key_press_modifiers(egui::Modifiers::SHIFT, egui::Key::F10);
    harness.run();
    assert!(harness.state().menu, "Shift+F10 opens the menu");
    assert!(harness.query_by_label("Rotate").is_some());
}

#[test]
fn the_first_menu_entry_takes_the_keyboard() {
    let mut window = window_with_component();
    window.selected = vec![0];
    let mut harness = harness(&mut window);
    harness.key_press_modifiers(egui::Modifiers::SHIFT, egui::Key::F10);
    harness.run();
    assert!(harness.state().menu);
    harness.key_press(egui::Key::Enter);
    harness.run();
    assert!(!harness.state().menu, "Enter chose the focused entry");
    assert!(
        harness.state().moving,
        "the first entry is Mover and it armed the gesture"
    );
}

#[test]
fn the_menu_follows_the_language() {
    let mut window = window_with_component();
    window.language = Language::Spanish;
    let mut harness = harness(&mut window);
    let sheet = harness.get_by_label("hoja de dibujo").rect();
    right_click_at(&mut harness, sheet.center());
    assert!(harness.query_by_label("Girar").is_some());
    assert!(harness.query_by_label("Rotate").is_none());
}

#[test]
fn shift_f10_without_a_selection_opens_nothing() {
    let mut window = Window::new(theme::Theme::fallback(), Document::new(), None, None);
    let mut harness = harness(&mut window);
    harness.key_press_modifiers(egui::Modifiers::SHIFT, egui::Key::F10);
    harness.run();
    assert!(!harness.state().menu);
}

#[test]
fn the_menu_rotates_the_selection_and_closes() {
    let mut window = window_with_component();
    let mut harness = harness(&mut window);
    let sheet = harness.get_by_label("drawing sheet").rect();
    right_click_at(&mut harness, sheet.center());
    harness.get_by_label("Rotate").click();
    harness.run();
    assert!(!harness.state().menu, "choosing a command closes the menu");
    assert_ne!(
        harness.state().orientation,
        Some(Orientation::default()),
        "the symbol turned"
    );
}

#[test]
fn the_menu_mirrors_the_selection_horizontally() {
    let mut window = window_with_component();
    let mut harness = harness(&mut window);
    let sheet = harness.get_by_label("drawing sheet").rect();
    right_click_at(&mut harness, sheet.center());
    harness.get_by_label("Mirror horizontally").click();
    harness.run();
    assert!(
        harness
            .state()
            .orientation
            .is_some_and(Orientation::mirrored),
        "the symbol is mirrored"
    );
}

#[test]
fn the_menu_removes_the_selection_as_one_undo() {
    let mut window = window_with_component();
    let mut harness = harness(&mut window);
    let sheet = harness.get_by_label("drawing sheet").rect();
    right_click_at(&mut harness, sheet.center());
    harness.get_by_label("Delete").click();
    harness.run();
    assert_eq!(harness.state().count, 0);
    assert!(harness.state().can_undo, "the removal is one undo");
}

#[test]
fn the_menu_opens_the_properties_even_when_the_dock_was_open() {
    let mut window = window_with_component();
    window.inspector.close();
    let mut harness = harness(&mut window);
    let sheet = harness.get_by_label("drawing sheet").rect();
    right_click_at(&mut harness, sheet.center());
    harness.get_by_label("Properties").click();
    harness.run();
    assert!(harness.state().inspector, "Propiedades opens the dock");
}

#[test]
fn the_configuration_entry_opens_a_blank_window_named_configuration() {
    let mut window = window_with_component();
    let mut harness = harness(&mut window);
    let revision = harness.state().revision;
    let sheet = harness.get_by_label("drawing sheet").rect();
    right_click_at(&mut harness, sheet.center());
    harness.get_by_label("Configure").click();
    harness.run();
    assert!(harness.state().mockup, "the mockup is open");
    assert!(harness.query_by_label("Configuration").is_some());
    assert!(harness.query_by_label_contains("Mockup").is_some());
    assert_eq!(
        harness.state().revision,
        revision,
        "the mockup does not touch the document"
    );
    harness.get_by_label("Close").click();
    harness.run();
    assert!(!harness.state().mockup);
}

#[test]
fn the_simple_cylinder_description_shows_its_card() {
    let mut window = window_with_single_acting_cylinder();
    let mut harness = harness(&mut window);
    let revision = harness.state().revision;
    let sheet = harness.get_by_label("drawing sheet").rect();
    right_click_at(&mut harness, sheet.center());
    harness.get_by_label("Component description").click();
    harness.run();
    assert!(harness.state().mockup, "the description window is open");
    assert!(harness.query_by_label("Description").is_some());
    assert!(
        harness
            .query_by_role_and_label(
                egui::accesskit::Role::Label,
                "Single-acting cylinder, 20/100",
            )
            .is_some(),
        "the card names the definition"
    );
    assert!(
        harness
            .query_by_role_and_label(
                egui::accesskit::Role::Label,
                "20 mm bore, 100 mm stroke, return spring.",
            )
            .is_some(),
        "the card carries the definition's summary"
    );
    assert_eq!(
        harness.state().revision,
        revision,
        "the card does not touch the document"
    );
    harness.get_by_label("Close").click();
    harness.run();
    assert!(!harness.state().mockup);
}

/// The card is a PNG compiled into the binary: `egui_extras` needs its `image` feature, or the
/// window paints the loader's «?» placeholder and the picture never appears. The texture is
/// the 800 × 450 PNG, so a frame that uploads it proves the loader decoded it.
#[test]
fn the_description_card_image_is_loaded() {
    let mut window = window_with_single_acting_cylinder();
    let mut harness = harness(&mut window);
    egui_extras::install_image_loaders(&harness.ctx);
    let sheet = harness.get_by_label("drawing sheet").rect();
    right_click_at(&mut harness, sheet.center());
    harness.get_by_label("Component description").click();
    let mut loaded = false;
    for _ in 0..120 {
        harness.step();
        loaded = harness
            .output()
            .textures_delta
            .set
            .iter()
            .any(|(_, delta)| {
                matches!(&delta.image, egui::ImageData::Color(_))
                    && delta.image.size() == [800, 450]
            });
        if loaded {
            break;
        }
        // The image loader decodes the PNG on its own thread: the harness step does not wait
        // for it, so the loop has to give it wall-clock time or a fast machine runs the 120
        // steps before the texture exists.
        std::thread::sleep(std::time::Duration::from_millis(2));
    }
    assert!(
        loaded,
        "the card image is decoded and uploaded; without the image loader the window shows a «?»"
    );
}

#[test]
fn the_card_follows_the_language() {
    let mut window = window_with_single_acting_cylinder();
    window.language = Language::Spanish;
    let mut harness = harness(&mut window);
    let sheet = harness.get_by_label("hoja de dibujo").rect();
    right_click_at(&mut harness, sheet.center());
    harness.get_by_label("Descripción del componente").click();
    harness.run();
    assert!(
        harness
            .query_by_role_and_label(
                egui::accesskit::Role::Label,
                "Cilindro de simple efecto, 20/100",
            )
            .is_some(),
        "the card names the definition in Spanish"
    );
    assert!(
        harness
            .query_by_role_and_label(
                egui::accesskit::Role::Label,
                "Diámetro 20 mm, carrera 100 mm, muelle de retorno.",
            )
            .is_some(),
        "the card carries the Spanish summary when the definition has one"
    );
    harness.get_by_label("Cerrar").click();
    harness.run();
}

#[test]
fn the_description_entry_opens_a_blank_window_named_descripcion() {
    let mut window = window_with_component();
    let mut harness = harness(&mut window);
    let sheet = harness.get_by_label("drawing sheet").rect();
    right_click_at(&mut harness, sheet.center());
    harness.get_by_label("Component description").click();
    harness.run();
    assert!(harness.state().mockup, "the mockup is open");
    assert!(harness.query_by_label("Description").is_some());
    assert!(harness.query_by_label_contains("Mockup").is_some());
    harness.key_press(egui::Key::Escape);
    harness.run();
    assert!(!harness.state().mockup);
}

#[test]
fn moving_follows_the_pointer_and_drops_with_one_click() {
    let mut window = window_with_component();
    let mut harness = harness(&mut window);
    let sheet = harness.get_by_label("drawing sheet").rect();
    right_click_at(&mut harness, sheet.center());
    harness.get_by_label("Move").click();
    harness.run();
    assert!(harness.state().moving, "the move is armed");
    let revision = harness.state().revision;
    let before = harness.state().centre.expect("the component has a centre");
    let target = sheet.center() + egui::vec2(120.0, 60.0);
    harness
        .input_mut()
        .events
        .push(egui::Event::PointerMoved(target));
    harness.run();
    assert_eq!(
        harness.state().revision,
        revision,
        "the preview is not a document change"
    );
    assert!(harness.state().moving);
    primary_click_at(&mut harness, target);
    let after = harness.state().centre.expect("the component has a centre");
    assert!(!harness.state().moving, "the click drops the selection");
    assert!(harness.state().can_undo, "the drop is one undo");
    assert!(
        (after.x - before.x).abs() > 1.0 || (after.y - before.y).abs() > 1.0,
        "the component moved: {before:?} -> {after:?}"
    );
}

#[test]
fn escape_cancels_the_armed_move_without_a_document_change() {
    let mut window = window_with_component();
    let mut harness = harness(&mut window);
    let sheet = harness.get_by_label("drawing sheet").rect();
    right_click_at(&mut harness, sheet.center());
    harness.get_by_label("Move").click();
    harness.run();
    let revision = harness.state().revision;
    let before = harness.state().centre.expect("the component has a centre");
    harness.input_mut().events.push(egui::Event::PointerMoved(
        sheet.center() + egui::vec2(60.0, 30.0),
    ));
    harness.run();
    harness.key_press(egui::Key::Escape);
    harness.run();
    assert!(!harness.state().moving);
    assert_eq!(harness.state().revision, revision);
    assert_eq!(harness.state().centre, Some(before));
}

#[test]
fn the_arrow_keys_nudge_the_armed_selection_one_millimetre() {
    let mut window = window_with_component();
    let mut harness = harness(&mut window);
    let sheet = harness.get_by_label("drawing sheet").rect();
    right_click_at(&mut harness, sheet.center());
    harness.get_by_label("Move").click();
    harness.run();
    let before = harness.state().centre.expect("the component has a centre");
    harness.key_press(egui::Key::ArrowRight);
    harness.run();
    harness.key_press(egui::Key::Enter);
    harness.run();
    let after = harness.state().centre.expect("the component has a centre");
    assert!(!harness.state().moving, "Enter drops the selection");
    assert!(
        (after.x - before.x - 1.0).abs() < 0.01,
        "one arrow is one millimetre: {before:?} -> {after:?}"
    );
    assert!((after.y - before.y).abs() < 0.01);
}

#[test]
fn right_click_on_a_valve_does_not_press_it() {
    let mut window = window_with_a_model();
    let mut harness = harness(&mut window);
    let position = harness.state().valve_position;
    let sheet = harness.get_by_label("drawing sheet").rect();
    let centre = sheet.center();
    harness
        .input_mut()
        .events
        .push(egui::Event::PointerMoved(centre));
    harness.input_mut().events.push(egui::Event::PointerButton {
        pos: centre,
        button: egui::PointerButton::Secondary,
        pressed: true,
        modifiers: egui::Modifiers::default(),
    });
    harness.run();
    assert!(
        !harness.state().pressed_valve,
        "the right button does not press the valve"
    );
    assert_eq!(harness.state().valve_position, position);
    harness.input_mut().events.push(egui::Event::PointerButton {
        pos: centre,
        button: egui::PointerButton::Secondary,
        pressed: false,
        modifiers: egui::Modifiers::default(),
    });
    harness.run();
    assert!(harness.state().menu, "the release opens the menu");
}

#[test]
fn a_click_outside_closes_the_menu() {
    let mut window = window_with_component();
    let mut harness = harness(&mut window);
    let sheet = harness.get_by_label("drawing sheet").rect();
    right_click_at(&mut harness, sheet.center());
    assert!(harness.state().menu);
    primary_click_at(&mut harness, sheet.left_top() + egui::vec2(8.0, 8.0));
    assert!(!harness.state().menu);
}

#[test]
fn the_mockup_subject_is_the_selected_component_name() {
    let mut window = window_with_component();
    {
        let mut harness = harness(&mut window);
        let sheet = harness.get_by_label("drawing sheet").rect();
        right_click_at(&mut harness, sheet.center());
        harness.get_by_label("Configure").click();
        harness.run();
        assert!(
            harness.query_by_label_contains("3/2 valve").is_some(),
            "the mockup names the component in English"
        );
        harness.key_press(egui::Key::Escape);
        harness.run();
    }
    window.language = Language::Spanish;
    {
        let mut harness = harness(&mut window);
        let sheet = harness.get_by_label("hoja de dibujo").rect();
        right_click_at(&mut harness, sheet.center());
        harness.get_by_label("Configurar").click();
        harness.run();
        assert!(
            harness.query_by_label_contains("Válvula").is_some(),
            "the mockup names the component in Spanish"
        );
    }
}

#[test]
fn the_menu_mirrors_the_selection_vertically() {
    let mut window = window_with_component();
    let mut harness = harness(&mut window);
    let sheet = harness.get_by_label("drawing sheet").rect();
    right_click_at(&mut harness, sheet.center());
    harness.get_by_label("Mirror vertically").click();
    harness.run();
    assert!(!harness.state().menu, "choosing the entry closes the menu");
    assert_eq!(
        harness.state().orientation,
        Some(Orientation::new(2, true)),
        "the symbol is mirrored top to bottom"
    );
    assert!(harness.state().can_undo, "the vertical mirror is one undo");
}

#[test]
fn the_menu_entries_carry_their_icon() {
    // The menu's icons are looked up from the same family the ribbon uses; a missing drawing
    // would leave a textual entry and a silent regression.
    for action in [
        MenuAction::Move,
        MenuAction::Rotate,
        MenuAction::MirrorHorizontal,
        MenuAction::MirrorVertical,
        MenuAction::Configure,
        MenuAction::Properties,
        MenuAction::Describe,
        MenuAction::Delete,
    ] {
        let id = match action {
            MenuAction::Move => "hand",
            MenuAction::Rotate => "object-rotate-right",
            MenuAction::MirrorHorizontal => "object-flip-horizontal",
            MenuAction::MirrorVertical => "object-flip-vertical",
            MenuAction::Configure => "configure",
            MenuAction::Properties => "document-properties",
            MenuAction::Describe => "describe",
            MenuAction::Delete => "edit-delete",
        };
        assert!(
            crate::ribbon::icon_bytes(id).is_some(),
            "the menu asks for the drawing `{id}`"
        );
    }
}
