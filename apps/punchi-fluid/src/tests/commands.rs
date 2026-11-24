// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The window tests for the commands the ribbon dispatches: the sheet, the title block, the
//! pipe, the selection, the clock and the start page's recents.

use punchi_fluid_doc::{Document, Endpoint, Placement};

use super::{Pt, Window, theme};

#[test]
fn the_margin_gallery_changes_the_sheet() {
    let mut window = Window::new(theme::Theme::fallback(), Document::new(), None, None);
    let before = window.document.sheet().margins;
    window.dispatch(crate::ribbon::Command::PageMargin(
        crate::ribbon::MarginPreset::Narrow,
    ));
    assert_ne!(
        window.document.sheet().margins,
        before,
        "the margins changed"
    );
    assert_eq!(
        window.page.margins,
        window.document.sheet().margins,
        "the page the window draws follows the document"
    );
    assert!(window.document.can_undo());
}

#[test]
fn the_orientation_menu_changes_the_sheet() {
    let mut window = Window::new(theme::Theme::fallback(), Document::new(), None, None);
    window.dispatch(crate::ribbon::Command::PageOrientation(
        punchi_fluid_doc::paper::Orientation::Portrait,
    ));
    assert_eq!(
        window.document.sheet().orientation,
        punchi_fluid_doc::paper::Orientation::Portrait
    );
}

#[test]
fn the_paper_menu_changes_the_sheet() {
    let mut window = Window::new(theme::Theme::fallback(), Document::new(), None, None);
    window.dispatch(crate::ribbon::Command::PagePaper(
        punchi_fluid_doc::paper::PaperSize::A3,
    ));
    assert_eq!(
        window.document.sheet().paper,
        punchi_fluid_doc::paper::PaperSize::A3
    );
}

#[test]
fn the_sheet_change_is_one_undo() {
    let mut window = Window::new(theme::Theme::fallback(), Document::new(), None, None);
    let before = window.document.sheet();
    window.dispatch(crate::ribbon::Command::PagePaper(
        punchi_fluid_doc::paper::PaperSize::A3,
    ));
    assert!(window.document.can_undo(), "the change is one undo");
    window.undo();
    assert_eq!(window.document.sheet(), before, "undo restores the sheet");
}

#[test]
fn the_block_content_menu_changes_the_block() {
    let mut window = Window::new(theme::Theme::fallback(), Document::new(), None, None);
    let id = window.document.active_circuit();
    window.dispatch(crate::ribbon::Command::BlockModel(
        punchi_fluid_doc::title_block::ContentModel::School,
    ));
    let block = window.document.block(id).expect("the circuit has a block");
    assert_eq!(
        block.model,
        punchi_fluid_doc::title_block::ContentModel::School
    );
    assert_eq!(
        block.visible,
        punchi_fluid_doc::title_block::ContentModel::School.defaults(),
        "the model sets its default fields"
    );
    assert!(window.document.can_undo(), "the change is one undo");
}

#[test]
fn the_block_position_menu_changes_the_block() {
    let mut window = Window::new(theme::Theme::fallback(), Document::new(), None, None);
    let id = window.document.active_circuit();
    window.dispatch(crate::ribbon::Command::BlockPosition(
        punchi_fluid_doc::title_block::BlockPosition::BottomFull,
    ));
    let block = window.document.block(id).expect("the circuit has a block");
    assert_eq!(
        block.position,
        punchi_fluid_doc::title_block::BlockPosition::BottomFull
    );
}

#[test]
fn the_block_reset_restores_the_default() {
    let mut window = Window::new(theme::Theme::fallback(), Document::new(), None, None);
    let id = window.document.active_circuit();
    assert!(
        !window.block_is_customized(),
        "a new circuit is its default"
    );
    window.dispatch(crate::ribbon::Command::BlockModel(
        punchi_fluid_doc::title_block::ContentModel::Technical,
    ));
    assert!(window.block_is_customized(), "the block changed");
    window.dispatch(crate::ribbon::Command::BlockReset);
    assert!(!window.block_is_customized(), "the reset is the default");
    let title = window
        .document
        .circuits()
        .iter()
        .find(|circuit| circuit.id == id)
        .map(|circuit| circuit.title.clone())
        .expect("the active circuit exists");
    assert_eq!(
        window.document.block(id).cloned(),
        Some(punchi_fluid_export::TitleBlock::new(title, "")),
        "the reset is the block a circuit is born with"
    );
}

/// A window whose document holds one hose between a supply and a valve.
fn window_with_a_pipe() -> Window {
    let mut document = Document::new();
    let supply = document.place(Placement::new(
        "supply.compressor",
        Pt::new(50.0, 50.0),
        24.0,
        24.0,
    ));
    let valve = document.place(Placement::new(
        "valve.3_2.spring_return",
        Pt::new(140.0, 50.0),
        24.0,
        24.0,
    ));
    let from = document.placements()[supply].id();
    let to = document.placements()[valve].id();
    assert!(document.connect(Endpoint::new(from, "1"), Endpoint::new(to, "1"), Some(1000)));
    Window::new(theme::Theme::fallback(), document, None, None)
}

/// A window whose document builds the piston model `start` can run.
fn window_with_a_model() -> Window {
    let mut document = Document::new();
    let supply = document.place(Placement::new(
        "supply.compressor",
        Pt::new(60.0, 100.0),
        20.0,
        20.0,
    ));
    let valve = document.place(Placement::new(
        "valve.5_2.spring_return",
        Pt::new(120.0, 100.0),
        20.0,
        20.0,
    ));
    let cylinder = document.place(Placement::new(
        "cylinder.double_acting_32_200",
        Pt::new(200.0, 100.0),
        44.0,
        20.0,
    ));
    let supply_id = document.placements()[supply].id();
    let valve_id = document.placements()[valve].id();
    let cylinder_id = document.placements()[cylinder].id();
    document.connect(
        Endpoint::new(supply_id, "1"),
        Endpoint::new(valve_id, "1"),
        None,
    );
    document.connect(
        Endpoint::new(valve_id, "2"),
        Endpoint::new(cylinder_id, "1"),
        None,
    );
    document.connect(
        Endpoint::new(valve_id, "4"),
        Endpoint::new(cylinder_id, "2"),
        None,
    );
    Window::new(theme::Theme::fallback(), document, None, None)
}

#[test]
fn the_pipe_class_gallery_changes_the_pipe() {
    let mut window = window_with_a_pipe();
    window.selected_pipe = Some(0);
    window.dispatch(crate::ribbon::Command::PipeClass(
        punchi_fluid_draw::LineClass::Drain,
    ));
    assert_eq!(
        window.document.connections()[0].class(),
        punchi_fluid_draw::LineClass::Drain
    );
    assert!(window.document.can_undo(), "the class edit is one undo");
}

#[test]
fn the_tube_control_changes_the_tube() {
    let mut window = window_with_a_pipe();
    window.selected_pipe = Some(0);
    window.dispatch(crate::ribbon::Command::Tube);
    window.apply_inspector(crate::inspector::Action::PipeTube {
        index: 0,
        tube: Some(600),
    });
    assert_eq!(
        window.document.connections()[0].tube_od_hundredths(),
        Some(600)
    );
    assert!(window.document.can_undo(), "the tube edit is one undo");
}

#[test]
fn distributing_the_selection_spaces_it() {
    let mut document = Document::new();
    for x in [40.0, 120.0, 70.0] {
        document.place(Placement::new(
            "valve.3_2.spring_return",
            Pt::new(x, 60.0),
            20.0,
            20.0,
        ));
    }
    let mut window = Window::new(theme::Theme::fallback(), document, None, None);
    window.selected = vec![0, 1, 2];
    let before = window.document.placements()[2].box_rect().center();
    window.dispatch(crate::ribbon::Command::Distribute(
        punchi_fluid_doc::Axis::Horizontal,
    ));
    let after = window.document.placements()[2].box_rect().center();
    assert_ne!(before, after, "the middle placement moved");
    window.undo();
    assert_eq!(
        window.document.placements()[2].box_rect().center(),
        before,
        "one undo restores the spacing"
    );
}

#[test]
fn the_select_all_command_takes_the_circuit() {
    let mut document = Document::new();
    document.place(Placement::new(
        "valve.3_2.spring_return",
        Pt::new(60.0, 60.0),
        20.0,
        20.0,
    ));
    document.place(Placement::new(
        "valve.3_2.spring_return",
        Pt::new(120.0, 60.0),
        20.0,
        20.0,
    ));
    let mut window = Window::new(theme::Theme::fallback(), document, None, None);
    window.dispatch(crate::ribbon::Command::SelectAll);
    assert_eq!(window.selected, vec![0, 1]);
}

#[test]
fn the_speed_changes_the_clock() {
    let mut window = Window::new(theme::Theme::fallback(), Document::new(), None, None);
    window.dispatch(crate::ribbon::Command::Speed(crate::ribbon::Speed::Double));
    assert!(
        (window.speed.factor() - 2.0).abs() < f32::EPSILON,
        "the clock runs twice as fast"
    );
    assert_eq!(
        window.ribbon.active_speed, "ribbon.item.speed_double",
        "the ribbon marks it"
    );
    window.dispatch(crate::ribbon::Command::Speed(crate::ribbon::Speed::Quarter));
    assert!((window.speed.factor() - 0.25).abs() < f32::EPSILON);
}

#[test]
fn the_reset_returns_the_model_to_zero() {
    let mut window = window_with_a_model();
    window.start();
    assert!(window.model.is_some());
    window.advance(0.5);
    let time = window
        .model
        .as_ref()
        .map_or(0.0, |model| model.network().telemetry().time);
    assert!(time > 0.0, "the model advanced");
    window.dispatch(crate::ribbon::Command::SimReset);
    assert!(window.model.is_none(), "the model is thrown away");
    assert!(!window.running);
    window.start();
    let time = window
        .model
        .as_ref()
        .map_or(-1.0, |model| model.network().telemetry().time);
    assert!(time.abs() < 1e-9, "the reset starts at t = 0");
}

#[test]
fn home_opens_the_start_view() {
    let mut window = Window::new(theme::Theme::fallback(), Document::new(), None, None);
    window.start_open = false;
    window.dispatch(crate::ribbon::Command::Home);
    assert!(window.start_open, "Inicio opens the start page");
}

#[test]
fn the_recent_menu_lists_the_real_documents() {
    let path = std::env::temp_dir().join(format!(
        "punchi-window-recent-{}.pfluid",
        std::process::id()
    ));
    punchi_fluid_session::save(&Document::new(), &path).expect("the document writes");
    let mut window = Window::new(theme::Theme::fallback(), Document::new(), None, None);
    // The in-memory list: a test never writes the reader's recent file.
    window.recents = punchi_fluid_session::recents::Recents::at(None);
    window.recents.remember(&path);
    window.dispatch(crate::ribbon::Command::OpenRecent(0));
    assert_eq!(
        window.path.as_deref(),
        Some(path.as_path()),
        "the remembered document opened"
    );
    std::fs::remove_file(&path).ok();
}

#[test]
fn the_notices_follow_the_language() {
    let mut window = Window::new(theme::Theme::fallback(), Document::new(), None, None);
    window.new_document();
    assert_eq!(window.notice.as_deref(), Some("new document"));
    window.language = punchi_fluid_i18n::Language::Spanish;
    window.new_document();
    assert_eq!(window.notice.as_deref(), Some("documento nuevo"));
}

#[test]
fn a_missing_recent_is_marked_and_refused() {
    let mut window = Window::new(theme::Theme::fallback(), Document::new(), None, None);
    window.recents = punchi_fluid_session::recents::Recents::at(None);
    window
        .recents
        .remember(std::path::Path::new("/punchi/no-existe.pfluid"));
    let revision = window.document.revision();
    window.dispatch(crate::ribbon::Command::OpenRecent(0));
    assert!(window.path.is_none(), "nothing opened");
    assert_eq!(
        window.document.revision(),
        revision,
        "the document is untouched"
    );
    assert!(
        window
            .notice
            .as_deref()
            .is_some_and(|notice| notice.contains("Could not open")),
        "the refusal is explained: {:?}",
        window.notice
    );
}

#[test]
fn hiding_the_block_changes_nothing_in_the_document() {
    let mut window = Window::new(theme::Theme::fallback(), Document::new(), None, None);
    let revision = window.document.revision();
    window.dispatch(crate::ribbon::Command::BlockVisible);
    assert!(!window.show_block, "the view toggle hides the block");
    window.dispatch(crate::ribbon::Command::BlockVisible);
    assert!(window.show_block, "the view toggle shows it again");
    assert_eq!(
        window.document.revision(),
        revision,
        "the block's visibility is view state, not an edit"
    );
}

#[test]
fn entering_the_simulation_leaves_the_hand_mode() {
    let mut window = Window::new(theme::Theme::fallback(), Document::new(), None, None);
    window.hand_mode = true;
    window.dispatch(crate::ribbon::Command::Start);
    assert!(!window.hand_mode, "a click actuates while the model runs");
}
