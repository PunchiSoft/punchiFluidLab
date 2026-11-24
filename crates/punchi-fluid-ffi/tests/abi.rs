// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! Drives the C ABI the way a native front end does.
//!
//! These are not unit tests of the model — the model has its own. They prove the
//! boundary: that a session is created from the built-in catalog, that it steps, that
//! the frame carries the primitives the drawing produced, and that the layout the
//! caller reads back is the same one the hit test uses.

#![allow(
    unsafe_code,
    reason = "the point of the test is to call the unsafe C entry points"
)]

use std::ffi::{CStr, CString, c_char};
use std::ptr;

use punchi_fluid::{
    PF_ABI_VERSION, PF_BLOCK_APPROVED, PF_BLOCK_BOTTOM_FULL, PF_BLOCK_BOTTOM_RIGHT,
    PF_BLOCK_CHECKED, PF_BLOCK_DATE, PF_BLOCK_EQUIPMENT, PF_BLOCK_FIELD_SHEET,
    PF_BLOCK_MODEL_INDUSTRIAL, PF_BLOCK_MODEL_SCHOOL, PF_BLOCK_NONE, PF_BLOCK_NORMS,
    PF_BLOCK_NUMBER, PF_BLOCK_OWNER, PF_BLOCK_PREPARED, PF_BLOCK_PROJECT, PF_BLOCK_REVISION,
    PF_BLOCK_SCALE, PF_BLOCK_TITLE, PF_BLOCK_TOP_RIGHT, PF_EDIT_BORE, PF_EDIT_PRESSURE,
    PF_EDIT_ROD, PF_EDIT_STROKE, PF_ERR_LIMIT, PF_ERR_MODEL, PF_ERR_NULL, PF_EXPORT_CUSTOM_PAPER,
    PF_EXPORT_PDF, PF_EXPORT_SVG, PF_HAS_STROKE, PF_KIND_LINE, PF_KIND_TEXT, PF_OK,
    PF_QUANTITY_FLOW, PF_QUANTITY_POSITION, PF_QUANTITY_PRESSURE, PF_SHEET_CUSTOM_PAPER,
    PF_TEMPLATE_COUNT, PfAbi, PfBlockLayout, PfCall, PfCatalog, PfChartSource, PfEditSet,
    PfExportOptions, PfFrame, PfInk, PfPoint, PfSession, PfSheet, PfTelemetry, pf_abi,
    pf_closure_count, pf_closure_id, pf_closure_is_plug, pf_closure_name, pf_export_paper_bounds,
    pf_export_paper_count, pf_export_paper_name, pf_export_paper_size_mm,
    pf_session_active_circuit, pf_session_add_circuit, pf_session_advance,
    pf_session_align_selection, pf_session_autosave, pf_session_block, pf_session_block_customized,
    pf_session_block_layout, pf_session_block_model, pf_session_block_preset,
    pf_session_block_remark, pf_session_block_visible, pf_session_build_frame,
    pf_session_buzzer_active, pf_session_can_undo, pf_session_catalog, pf_session_catalog_problems,
    pf_session_chart_candidate, pf_session_chart_candidate_count, pf_session_chart_source,
    pf_session_check, pf_session_circuit_count, pf_session_circuit_id, pf_session_circuit_name,
    pf_session_clear_chart_source, pf_session_component_circuit, pf_session_component_count,
    pf_session_component_designation, pf_session_component_edits, pf_session_component_id,
    pf_session_component_orientation, pf_session_component_rect,
    pf_session_component_valve_position, pf_session_component_valve_spring_return,
    pf_session_connect, pf_session_connection_count, pf_session_contact_closed,
    pf_session_copy_selected, pf_session_describe_component, pf_session_drag_component,
    pf_session_duplicate_component, pf_session_duplicate_selection, pf_session_export,
    pf_session_export_with, pf_session_finding_at, pf_session_forget_recent,
    pf_session_forget_recent_at, pf_session_free, pf_session_grid_step, pf_session_hit_component,
    pf_session_hit_valve, pf_session_indicator_active, pf_session_is_modified,
    pf_session_last_error, pf_session_load_example, pf_session_load_template,
    pf_session_mirror_selected, pf_session_move_component, pf_session_move_selection,
    pf_session_new, pf_session_new_document, pf_session_open, pf_session_open_recent,
    pf_session_paste, pf_session_paste_offset, pf_session_pipe_at, pf_session_pipe_class,
    pf_session_pipe_offset, pf_session_pipe_tube, pf_session_place_component,
    pf_session_placement_id, pf_session_port_closure, pf_session_port_count,
    pf_session_port_has_pipe, pf_session_port_id, pf_session_port_point, pf_session_port_role,
    pf_session_press_component_valve, pf_session_press_valve, pf_session_preview,
    pf_session_preview_template, pf_session_recent_available, pf_session_recent_count,
    pf_session_recent_path, pf_session_recover, pf_session_redo,
    pf_session_release_component_valve, pf_session_release_valve, pf_session_remove_circuit,
    pf_session_remove_component, pf_session_remove_selected_pipe, pf_session_remove_selection,
    pf_session_rename_circuit, pf_session_reset_block_visibility, pf_session_restart,
    pf_session_rotate_selected, pf_session_route_pipe, pf_session_save, pf_session_select,
    pf_session_select_all, pf_session_select_circuit, pf_session_select_pipe,
    pf_session_select_rect, pf_session_selected, pf_session_selected_pipe, pf_session_selection_at,
    pf_session_selection_count, pf_session_set_block, pf_session_set_block_model,
    pf_session_set_block_preset, pf_session_set_block_remark, pf_session_set_block_visible,
    pf_session_set_chart_source, pf_session_set_component_designation,
    pf_session_set_component_edits, pf_session_set_grid_step, pf_session_set_pipe_class,
    pf_session_set_pipe_tube, pf_session_set_port_closure, pf_session_set_recent_file,
    pf_session_set_sheet, pf_session_sheet, pf_session_space_selection, pf_session_straighten_pipe,
    pf_session_telemetry, pf_session_toggle_contact, pf_session_toggle_select,
    pf_session_toggle_valve, pf_session_undo, pf_session_valve_rect,
    pf_session_valve_spring_return, pf_tube_size_count, pf_tube_size_label, pf_tube_size_nominal,
    pf_tube_size_od,
};

use punchi_fluid_draw::placed::tube_width_mm;
use punchi_fluid_draw::{CYLINDER_RATIO, SYMBOL_BOX_MM};

fn session() -> *mut PfSession {
    // SAFETY: a null directory asks for the built-in catalog, which is what a front end
    // that has no user directory passes.
    let session = unsafe { pf_session_new(ptr::null()) };
    assert!(!session.is_null(), "a session must always be created");
    // Most ABI tests exercise the worked example; it is now an explicit fixture rather
    // than the product's implicit start document (ADR 0031).
    assert_eq!(unsafe { pf_session_load_example(session) }, PF_OK);
    session
}

/// Index in the sidebar list of the entry with this id.
fn catalog_index(session: *mut PfSession, wanted: &str) -> u32 {
    let mut catalog = PfCatalog::default();
    // SAFETY: live session, writable output.
    assert_eq!(
        unsafe { pf_session_catalog(session, &raw mut catalog) },
        PF_OK
    );
    // SAFETY: the slices were produced by the call above and the session is alive.
    let (items, text) = unsafe {
        (
            std::slice::from_raw_parts(catalog.items, catalog.items_len as usize),
            std::slice::from_raw_parts(catalog.text, catalog.text_len as usize),
        )
    };
    let mut found = None;
    for (index, item) in items.iter().enumerate() {
        let start = item.id_at as usize;
        let end = start + item.id_len as usize;
        if text.get(start..end) == Some(wanted.as_bytes()) {
            #[allow(clippy::cast_possible_truncation)]
            {
                found = Some(index as u32);
            }
        }
    }
    assert!(found.is_some(), "the catalog has no `{wanted}`");
    found.unwrap_or_default()
}

/// Index of the first placed component whose catalog id starts with `prefix`.
fn component_with(session: *mut PfSession, prefix: &str) -> u32 {
    let count = unsafe { pf_session_component_count(session) };
    for index in 0..count {
        let mut id = [0 as c_char; 128];
        // SAFETY: live session and a writable buffer.
        if unsafe { pf_session_component_id(session, index, id.as_mut_ptr(), 128) } < 0 {
            continue;
        }
        // SAFETY: the call above wrote a NUL-terminated id.
        let text = unsafe { CStr::from_ptr(id.as_ptr()) }.to_string_lossy();
        if text.starts_with(prefix) {
            return index;
        }
    }
    u32::MAX
}

/// The box of a placement as `x0 y0 x1 y1`, in millimetres.
fn component_box(session: *mut PfSession, index: u32) -> [f32; 4] {
    let mut out = [0.0_f32; 4];
    // SAFETY: live session and four writable floats.
    assert_eq!(
        unsafe { pf_session_component_rect(session, index, out.as_mut_ptr()) },
        PF_OK
    );
    out
}

/// The point of one port of a placement, in millimetres.
fn port_of(session: *mut PfSession, index: u32, port: &str) -> (f32, f32) {
    let id = CString::new(port).unwrap_or_default();
    let mut out = [0.0_f32; 2];
    // SAFETY: live session and two writable floats.
    assert_eq!(
        unsafe { pf_session_port_point(session, index, id.as_ptr(), out.as_mut_ptr()) },
        PF_OK
    );
    (out[0], out[1])
}

/// A string the ABI wrote into a fixed buffer.
fn written(buffer: &[c_char]) -> String {
    // SAFETY: the call that filled the buffer wrote a NUL-terminated string into it.
    unsafe { CStr::from_ptr(buffer.as_ptr()) }
        .to_string_lossy()
        .into_owned()
}

/// Places a supply and a contact and joins them with one wire; returns the wire's index.
///
/// The two ports are `PortRole::Electrical`, which is what makes the pipe a wire: the file
/// format cannot store the electrical class, and the roles are the only authority.
fn electrical_wire(session: *mut PfSession, at: (f32, f32)) -> u32 {
    let rail = catalog_index(session, "electrical.supply");
    let button = catalog_index(session, "electrical.contact");
    // SAFETY: live session and finite coordinates.
    let rail = unsafe { pf_session_place_component(session, rail, at.0, at.1) };
    let button = unsafe { pf_session_place_component(session, button, at.0 + 50.0, at.1) };
    assert!(rail >= 0 && button >= 0, "the control circuit is placed");
    let rail = u32::try_from(rail).unwrap_or_default();
    let button = u32::try_from(button).unwrap_or_default();
    let l = CString::new("L").unwrap_or_default();
    let one = CString::new("1").unwrap_or_default();
    let pipe = unsafe { pf_session_connection_count(session) };
    // SAFETY: live session and NUL-terminated ports.
    assert_eq!(
        unsafe { pf_session_connect(session, rail, l.as_ptr(), button, one.as_ptr()) },
        PF_OK
    );
    pipe
}

/// Writes the active circuit as an SVG page with its parts list and returns the text.
///
/// `None` means the fixture could not write the page; the callers assert on it so a broken
/// export still fails the test with a message instead of a silent empty page.
fn export_svg(session: *mut PfSession, name: &str) -> Option<String> {
    let directory = tempfile::tempdir().ok()?;
    let path = directory.path().join(name);
    let encoded = CString::new(path.to_str().unwrap_or_default()).ok()?;
    let date = CString::new("2026-11-20").ok()?;
    // SAFETY: live session, NUL-terminated path and date, A4 is index 1.
    let code = unsafe {
        pf_session_export(
            session,
            encoded.as_ptr(),
            PF_EXPORT_SVG,
            1,
            0,
            300.0,
            date.as_ptr(),
        )
    };
    assert_eq!(
        code,
        PF_OK,
        "{}",
        // SAFETY: live session.
        unsafe { CStr::from_ptr(pf_session_last_error(session)) }.to_string_lossy()
    );
    std::fs::read_to_string(path.to_str().unwrap_or_default()).ok()
}

/// The one tube row of `size` in an SVG page, if the parts list printed it.
fn tube_row(svg: &str, size: &str) -> Option<String> {
    let marker = format!("Hose {size} · ");
    let start = svg.find(&marker)?;
    let tail = &svg[start..];
    let end = tail.find('<').unwrap_or(tail.len());
    Some(tail[..end].trim().to_owned())
}

#[test]
fn abi_sizes_match_the_types_they_describe() {
    let mut abi = PfAbi::default();
    // SAFETY: a writable structure with the layout the header declares.
    let code = unsafe { pf_abi(&raw mut abi) };
    assert_eq!(code, PF_OK);
    assert_eq!(abi.version, PF_ABI_VERSION);
    assert_eq!(abi.call_size as usize, size_of::<PfCall>());
    assert_eq!(abi.telemetry_size as usize, size_of::<PfTelemetry>());
    assert_eq!(abi.frame_size as usize, size_of::<PfFrame>());
    assert_eq!(abi.ink_size as usize, size_of::<PfInk>());
    assert_eq!(abi.point_size as usize, size_of::<PfPoint>());
    assert_eq!(abi.sheet_size as usize, size_of::<PfSheet>());
}

#[test]
fn a_new_session_has_the_built_in_model() {
    let session = session();
    let mut telemetry = PfTelemetry::default();
    // SAFETY: live session, writable output.
    let code = unsafe { pf_session_telemetry(session, &raw mut telemetry) };
    assert_eq!(code, PF_OK);
    assert_eq!(
        telemetry.has_model, 1,
        "the built-in catalog must yield a model"
    );
    assert_eq!(telemetry.valve_position, 0);
    // SAFETY: the session is still alive.
    unsafe { pf_session_free(session) };
}

#[test]
fn stepping_the_model_moves_the_piston_once_the_valve_opens() {
    let session = session();
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_toggle_valve(session) }, PF_OK);
    for _ in 0..200 {
        // SAFETY: live session.
        let code = unsafe { pf_session_advance(session, 0.002) };
        assert_eq!(code, PF_OK);
    }
    let mut telemetry = PfTelemetry::default();
    // SAFETY: live session, writable output.
    assert_eq!(
        unsafe { pf_session_telemetry(session, &raw mut telemetry) },
        PF_OK
    );
    assert_eq!(telemetry.has_model, 1);
    assert!(telemetry.position_m > 0.0, "the piston must have moved");
    assert!(
        telemetry.pressure_pa > 100_000.0,
        "the cylinder must be pressurised"
    );
    // SAFETY: the session is still alive.
    unsafe { pf_session_free(session) };
}

#[test]
fn the_frame_carries_the_drawing_and_a_text_run() {
    let session = session();
    let mut frame = PfFrame::default();
    // SAFETY: live session, writable output.
    let code = unsafe { pf_session_build_frame(session, 4.0, 0.0, 0, &raw mut frame) };
    assert_eq!(code, PF_OK);
    assert!(
        frame.calls_len > 10,
        "a three-component circuit is not two primitives"
    );
    assert!(frame.points_len > 10);
    assert!(frame.text_len > 0, "the readouts are text");

    // SAFETY: the pointers were produced by the call above and the session is alive.
    let calls = unsafe { std::slice::from_raw_parts(frame.calls, frame.calls_len as usize) };
    assert!(
        calls.iter().any(|call| call.kind == PF_KIND_TEXT),
        "the drawing must label its ports"
    );
    // Everything stays inside the area the caller asked for, with room for a stroke.
    let slack = 20.0;
    for call in calls {
        assert!(
            call.rect[0] >= -slack && call.rect[1] >= -slack,
            "a primitive escaped the top-left corner: {call:?}"
        );
        assert!(
            call.rect[0] <= frame.width_mm + slack && call.rect[1] <= frame.height_mm + slack,
            "a primitive escaped the bottom-right corner: {call:?}"
        );
    }
    // SAFETY: the session is still alive.
    unsafe { pf_session_free(session) };
}

#[test]
fn the_hit_test_uses_the_layout_of_the_last_frame() {
    let session = session();
    let mut frame = PfFrame::default();
    // SAFETY: live session, writable output.
    unsafe { pf_session_build_frame(session, 4.0, 0.0, 0, &raw mut frame) };

    // The valve's own box is where the hit test must answer, whatever paper the document
    // uses; the front end asks for that box to draw its hover ring.
    let mut rect = [0.0_f32; 4];
    // SAFETY: live session and a writable rectangle.
    assert_eq!(
        unsafe { pf_session_valve_rect(session, rect.as_mut_ptr()) },
        PF_OK
    );
    let centre_x = f32::midpoint(rect[0], rect[2]);
    let centre_y = f32::midpoint(rect[1], rect[3]);
    // SAFETY: live session.
    assert_eq!(
        unsafe { pf_session_hit_valve(session, centre_x, centre_y) },
        1
    );
    // The paper's top-left corner is nowhere near the valve.
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_hit_valve(session, 1.0, 1.0) }, 0);
    // SAFETY: the session is still alive.
    unsafe { pf_session_free(session) };
}

#[test]
fn every_paper_reports_the_size_the_drawing_list_needs() {
    assert_eq!(pf_export_paper_count(), 9);

    let mut width = 0.0;
    let mut height = 0.0;
    // A4 is index 1, the order `PaperSize::ALL` and the interface share.
    assert_eq!(
        unsafe { pf_export_paper_size_mm(1, &raw mut width, &raw mut height) },
        PF_OK
    );
    assert_eq!((width, height), (210.0, 297.0));
    // Letter, the paper the interface calls Carta: 215.9 × 279.4 mm.
    assert_eq!(
        unsafe { pf_export_paper_size_mm(6, &raw mut width, &raw mut height) },
        PF_OK
    );
    assert_eq!((width, height), (215.9, 279.4));

    let mut min = 0.0;
    let mut max = 0.0;
    assert_eq!(
        unsafe { pf_export_paper_bounds(&raw mut min, &raw mut max) },
        PF_OK
    );
    assert_eq!((min, max), (50.0, 2000.0));

    // Every named paper has a positive size and a name in the same order.
    for index in 0..pf_export_paper_count() {
        assert_eq!(
            unsafe { pf_export_paper_size_mm(index, &raw mut width, &raw mut height) },
            PF_OK
        );
        assert!(width > 0.0 && height > 0.0);
        let mut label = [0 as c_char; 32];
        assert_eq!(
            unsafe {
                pf_export_paper_name(
                    index,
                    label.as_mut_ptr(),
                    u32::try_from(label.len()).unwrap_or(0),
                )
            },
            PF_OK
        );
        // SAFETY: the call above wrote a NUL-terminated string inside the array.
        assert!(
            !unsafe { CStr::from_ptr(label.as_ptr()) }
                .to_bytes()
                .is_empty()
        );
    }
    assert_eq!(
        unsafe { pf_export_paper_size_mm(99, &raw mut width, &raw mut height) },
        PF_ERR_MODEL
    );
}

#[test]
fn the_grid_step_is_a_view_setting_and_the_snap_follows_it() {
    let session = session();
    let mut step = 0.0;
    assert_eq!(
        unsafe { pf_session_grid_step(session, &raw mut step) },
        PF_OK
    );
    assert!(
        (step - 5.0).abs() < 0.001,
        "the grid starts at the documented step"
    );

    let modified = unsafe { pf_session_is_modified(session) };
    assert_eq!(unsafe { pf_session_set_grid_step(session, 5.0) }, PF_OK);
    assert_eq!(
        unsafe { pf_session_grid_step(session, &raw mut step) },
        PF_OK
    );
    assert!((step - 5.0).abs() < 0.001);
    assert_eq!(
        unsafe { pf_session_is_modified(session) },
        modified,
        "the grid is view state, not a document edit"
    );
    assert_eq!(
        unsafe { pf_session_set_grid_step(session, 0.0) },
        PF_ERR_MODEL,
        "a zero step would divide by zero"
    );

    let index = catalog_index(session, "supply.compressor");
    let mut rect = [0.0_f32; 4];
    let placed = unsafe { pf_session_place_component(session, index, 67.0, 67.0) };
    assert!(placed >= 0);
    assert_eq!(
        unsafe {
            pf_session_component_rect(
                session,
                u32::try_from(placed).expect("a non-negative placement index"),
                rect.as_mut_ptr(),
            )
        },
        PF_OK
    );
    assert!((f32::midpoint(rect[0], rect[2]) - 65.0).abs() < 0.01);
    assert!((f32::midpoint(rect[1], rect[3]) - 65.0).abs() < 0.01);

    assert_eq!(unsafe { pf_session_set_grid_step(session, 20.0) }, PF_OK);
    let placed = unsafe { pf_session_place_component(session, index, 67.0, 67.0) };
    assert!(placed >= 0);
    assert_eq!(
        unsafe {
            pf_session_component_rect(
                session,
                u32::try_from(placed).expect("a non-negative placement index"),
                rect.as_mut_ptr(),
            )
        },
        PF_OK
    );
    assert!((f32::midpoint(rect[0], rect[2]) - 60.0).abs() < 0.01);
    assert!((f32::midpoint(rect[1], rect[3]) - 60.0).abs() < 0.01);
    // SAFETY: the session is still alive.
    unsafe { pf_session_free(session) };
}

#[test]
fn a_new_document_is_an_empty_sheet_and_the_example_loads_on_request() {
    // SAFETY: a null directory asks for the built-in catalog.
    let session = unsafe { pf_session_new(ptr::null()) };
    assert!(!session.is_null());
    // The constructor and every new document are empty; building the first frame does not
    // seed the fixture either. The example is explicit.
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_component_count(session) }, 0);
    let mut frame = PfFrame::default();
    // SAFETY: live session and a writable frame.
    assert_eq!(
        unsafe { pf_session_build_frame(session, 1.0, 0.0, 0, &raw mut frame) },
        PF_OK
    );
    assert_eq!(unsafe { pf_session_component_count(session) }, 0);
    // SAFETY: live session, an empty document.
    assert_eq!(unsafe { pf_session_load_example(session) }, PF_OK);
    assert!(unsafe { pf_session_component_count(session) } >= 4);
    // The worked example is the shared catalog's supply, so it now runs at 4 bar.
    let mut edits = PfEditSet::default();
    // SAFETY: live session and a writable structure.
    assert_eq!(
        unsafe { pf_session_component_edits(session, 0, &raw mut edits) },
        PF_OK
    );
    assert!(
        (edits.pressure_bar - 4.0).abs() < 1e-9,
        "the example's supply reads {} bar, not 4",
        edits.pressure_bar
    );
    // A second call is refused: the example cannot be duplicated behind the reader's back.
    // SAFETY: live session.
    assert_ne!(unsafe { pf_session_load_example(session) }, PF_OK);
    // Starting another document empties the fixture and leaves it clean.
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_new_document(session) }, PF_OK);
    assert_eq!(unsafe { pf_session_component_count(session) }, 0);
    assert_eq!(unsafe { pf_session_is_modified(session) }, 0);
    // SAFETY: the session is still alive.
    unsafe { pf_session_free(session) };
}

#[test]
fn the_four_start_templates_load_clean_and_cannot_replace_a_working_document() {
    let session = unsafe { pf_session_new(ptr::null()) };
    assert!(!session.is_null());
    let expected_components = [4, 3, 5, 6];
    let expected_pipes = [3, 3, 4, 5];

    for index in 0..PF_TEMPLATE_COUNT {
        assert_eq!(unsafe { pf_session_new_document(session) }, PF_OK);
        assert_eq!(unsafe { pf_session_load_template(session, index) }, PF_OK);
        assert_eq!(
            unsafe { pf_session_component_count(session) },
            expected_components[index as usize]
        );
        assert_eq!(
            unsafe { pf_session_connection_count(session) },
            expected_pipes[index as usize]
        );
        assert_eq!(unsafe { pf_session_is_modified(session) }, 0);
        // Every template places a supply, and a new placement reads the 4 bar default.
        let supply = component_with(session, "supply.");
        assert_ne!(supply, u32::MAX, "template {index} places no supply");
        let mut edits = PfEditSet::default();
        // SAFETY: live session and a writable structure.
        assert_eq!(
            unsafe { pf_session_component_edits(session, supply, &raw mut edits) },
            PF_OK
        );
        assert!(
            (edits.pressure_bar - 4.0).abs() < 1e-9,
            "template {index} places its supply at {} bar, not 4",
            edits.pressure_bar
        );
    }

    let before = unsafe { pf_session_component_count(session) };
    assert_eq!(
        unsafe { pf_session_load_template(session, PF_TEMPLATE_COUNT) },
        PF_ERR_MODEL
    );
    assert_eq!(unsafe { pf_session_component_count(session) }, before);
    assert_eq!(
        unsafe { pf_session_load_template(session, 0) },
        PF_ERR_MODEL
    );
    assert_eq!(unsafe { pf_session_component_count(session) }, before);

    unsafe { pf_session_free(session) };
}

#[test]
fn template_previews_use_the_renderer_and_refuse_unbounded_dimensions() {
    let session = unsafe { pf_session_new(ptr::null()) };
    assert!(!session.is_null());

    for index in 0..PF_TEMPLATE_COUNT {
        let mut frame = PfFrame::default();
        assert_eq!(
            unsafe {
                pf_session_preview_template(session, index, 180.0, 100.0, 2.0, &raw mut frame)
            },
            PF_OK
        );
        assert!(
            frame.calls_len > 0,
            "template {index} draws components and pipes"
        );
        assert!(
            frame.points_len > 0,
            "template {index} uses the shared renderer"
        );
        assert_eq!((frame.width_mm, frame.height_mm), (180.0, 100.0));
    }

    let mut frame = PfFrame::default();
    assert_eq!(
        unsafe { pf_session_preview_template(session, 0, 59.0, 100.0, 2.0, &raw mut frame) },
        PF_ERR_MODEL
    );
    assert_eq!(frame.calls_len, 0);
    assert_eq!(
        unsafe {
            pf_session_preview_template(session, 0, 180.0, 100.0, f32::INFINITY, &raw mut frame)
        },
        PF_ERR_MODEL
    );

    unsafe { pf_session_free(session) };
}

#[test]
fn the_sheet_is_a4_landscape_and_a_paper_change_is_one_undo() {
    let session = session();
    let mut sheet = PfSheet::default();
    // SAFETY: live session and a writable record.
    assert_eq!(unsafe { pf_session_sheet(session, &raw mut sheet) }, PF_OK);
    assert_eq!(sheet.paper, 1, "A4 is the default paper");
    assert_eq!(sheet.landscape, 1, "the document opens landscape");
    // The ABI reports the paper's own sides and the orientation flag; the front end turns
    // them into 297 × 210 mm for the canvas.
    assert_eq!((sheet.width_mm, sheet.height_mm), (210.0, 297.0));
    // ISO 5457: 20 mm on the filing edge and 10 mm on the other three.
    assert_eq!((sheet.margin_left, sheet.margin_top), (20.0, 10.0));
    assert_eq!((sheet.margin_right, sheet.margin_bottom), (10.0, 10.0));

    let a3_landscape = PfSheet {
        paper: 2,
        landscape: 1,
        ..sheet
    };
    // SAFETY: live session and a readable record.
    assert_eq!(
        unsafe { pf_session_set_sheet(session, &raw const a3_landscape) },
        PF_OK
    );
    assert_eq!(unsafe { pf_session_is_modified(session) }, 1);
    let mut frame = PfFrame::default();
    // SAFETY: live session and a writable frame.
    assert_eq!(
        unsafe { pf_session_build_frame(session, 4.0, 0.0, 0, &raw mut frame) },
        PF_OK
    );
    assert_eq!(
        (frame.width_mm, frame.height_mm),
        (420.0, 297.0),
        "the drawing area is the paper, turned"
    );
    // One undo returns the sheet the document started on.
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_undo(session) }, PF_OK);
    // SAFETY: live session and a writable record.
    assert_eq!(unsafe { pf_session_sheet(session, &raw mut sheet) }, PF_OK);
    assert_eq!((sheet.paper, sheet.landscape), (1, 1));
    // SAFETY: the session is still alive.
    unsafe { pf_session_free(session) };
}

#[test]
fn the_seeded_example_stays_clear_of_the_title_block_on_a4() {
    let session = session();
    let mut sheet = PfSheet::default();
    // SAFETY: live session and a writable record.
    assert_eq!(unsafe { pf_session_sheet(session, &raw mut sheet) }, PF_OK);
    let page = punchi_fluid_export::Page {
        paper: punchi_fluid_export::PaperSize::A4,
        orientation: punchi_fluid_export::Orientation::Landscape,
        margins: punchi_fluid_export::Margins {
            top: sheet.margin_top,
            right: sheet.margin_right,
            bottom: sheet.margin_bottom,
            left: sheet.margin_left,
        },
        title_block: Some(punchi_fluid_export::TitleBlock::default()),
        parts: None,
        labels: punchi_fluid_export::Labels::english(),
    };
    let block = page.title_rect();
    assert!(!block.is_empty(), "A4 has a title block");
    let count = unsafe { pf_session_component_count(session) };
    assert!(count >= 4, "the seeded example has its components");
    for index in 0..count {
        let mut rect = [0.0_f32; 4];
        // SAFETY: live session, a component index below the count and a writable box.
        assert_eq!(
            unsafe { pf_session_component_rect(session, index, rect.as_mut_ptr()) },
            PF_OK
        );
        assert!(
            rect[3] <= block.min.y + 0.01,
            "component {index} enters the block: {rect:?} against {block:?}"
        );
    }
    // SAFETY: the session is still alive.
    unsafe { pf_session_free(session) };
}

/// The block's own rectangle on the session's sheet, as the ABI protects it.
fn block_box(session: *mut PfSession) -> [f32; 4] {
    let mut sheet = PfSheet::default();
    // SAFETY: live session and a writable record.
    assert_eq!(unsafe { pf_session_sheet(session, &raw mut sheet) }, PF_OK);
    let mut layout = PfBlockLayout::default();
    // SAFETY: live session and a writable layout.
    assert_eq!(
        unsafe { pf_session_block_layout(session, &raw mut layout) },
        PF_OK
    );
    let preset =
        u32::try_from(unsafe { pf_session_block_preset(session) }).unwrap_or(PF_BLOCK_NONE);
    if preset == PF_BLOCK_NONE {
        return [0.0; 4];
    }
    // The ABI reports the paper's own sides and the orientation flag; the sheet the canvas
    // sees is the turned one.
    let (width, height) = if sheet.landscape == 1 {
        (sheet.height_mm, sheet.width_mm)
    } else {
        (sheet.width_mm, sheet.height_mm)
    };
    let left = sheet.margin_left;
    let right = width - sheet.margin_right;
    let top = sheet.margin_top;
    let bottom = height - sheet.margin_bottom;
    let frame_width = right - left;
    let frame_height = bottom - top;
    let width = if preset == PF_BLOCK_BOTTOM_FULL {
        frame_width
    } else {
        frame_width.min(layout.width_mm)
    };
    let height = frame_height.min(layout.height_mm);
    let x = if preset == PF_BLOCK_BOTTOM_FULL {
        left
    } else {
        right - width
    };
    let y = if preset == PF_BLOCK_TOP_RIGHT {
        top
    } else {
        bottom - height
    };
    [x, y, x + width, y + height]
}

/// The centre of a placement, in millimetres.
fn centre_of(boxed: [f32; 4]) -> (f32, f32) {
    (
        f32::midpoint(boxed[0], boxed[2]),
        f32::midpoint(boxed[1], boxed[3]),
    )
}

#[test]
fn the_block_protects_its_own_box_and_the_strip_beside_it_is_drawing_space() {
    let session = session();
    let compressor = catalog_index(session, "supply.compressor");
    let block = block_box(session);
    assert!(block[0] > 0.0, "A4 landscape carries a block");

    // A point below the band the old rule reserved, but to the left of the block, stands
    // where the user asked: no printed thing covers that column.
    // SAFETY: live session.
    let left = unsafe { pf_session_place_component(session, compressor, 60.0, 175.0) };
    assert!(left >= 0, "the drop is accepted");
    let boxed = component_box(session, u32::try_from(left).unwrap_or(0));
    assert!(
        boxed[3] > block[1],
        "the box is below the block's top: {boxed:?}"
    );
    assert!(
        boxed[2] < block[0],
        "and beside it, not under it: {boxed:?}"
    );
    let (_, centre_y) = centre_of(boxed);
    assert!(
        (centre_y - 175.0).abs() < 0.02,
        "the vertical choice is the user's: {centre_y}"
    );
    // SAFETY: the session is still alive.
    unsafe { pf_session_free(session) };
}

#[test]
fn a_drop_inside_the_block_moves_to_its_edge() {
    let session = session();
    let compressor = catalog_index(session, "supply.compressor");
    let block = block_box(session);
    // SAFETY: live session.
    let inside = unsafe { pf_session_place_component(session, compressor, 200.0, 175.0) };
    assert!(inside >= 0);
    let boxed = component_box(session, u32::try_from(inside).unwrap_or(0));
    assert!(
        (boxed[3] - block[1]).abs() < 0.02,
        "the box sits on the block's edge: {boxed:?} against {block:?}"
    );
    let (centre_x, _) = centre_of(boxed);
    assert!(
        (centre_x - 200.0).abs() < 0.02,
        "the horizontal choice is untouched: {centre_x}"
    );
    // SAFETY: the session is still alive.
    unsafe { pf_session_free(session) };
}

#[test]
fn a_shorter_block_reserves_less() {
    let session = session();
    let compressor = catalog_index(session, "supply.compressor");
    let industrial = block_box(session);
    // SAFETY: live session and a known model.
    assert_eq!(
        unsafe { pf_session_set_block_model(session, PF_BLOCK_MODEL_SCHOOL) },
        PF_OK
    );
    let school = block_box(session);
    assert!(
        school[1] > industrial[1],
        "the school block is shorter: {school:?} against {industrial:?}"
    );

    // Inside the industrial block, above the school one: the same drop is pushed by the
    // first and accepted by the second. The probe gives the height the drop will take.
    // SAFETY: live session.
    let probe = unsafe { pf_session_place_component(session, compressor, 100.0, 100.0) };
    let probe_box = component_box(session, u32::try_from(probe).unwrap_or(0));
    let half = (probe_box[3] - probe_box[1]) * 0.5;
    let y = school[1] - half - 4.0;
    assert!(y + half < school[1], "the point clears the school block");
    assert!(
        y + half > industrial[1],
        "and reaches the industrial one: {y} against {industrial:?}"
    );
    // SAFETY: live session.
    let schooled = unsafe { pf_session_place_component(session, compressor, 200.0, y) };
    let (_, school_y) = centre_of(component_box(session, u32::try_from(schooled).unwrap_or(0)));
    assert!(
        (school_y - y).abs() <= 2.5,
        "the shorter block only snaps the point: {school_y} against {y}"
    );

    // SAFETY: live session and a known model.
    assert_eq!(
        unsafe { pf_session_set_block_model(session, PF_BLOCK_MODEL_INDUSTRIAL) },
        PF_OK
    );
    // SAFETY: live session.
    let industrialised = unsafe { pf_session_place_component(session, compressor, 200.0, y) };
    let (_, pushed) = centre_of(component_box(
        session,
        u32::try_from(industrialised).unwrap_or(0),
    ));
    assert!(
        pushed < y - 0.5,
        "the industrial block pushes the same drop up: {pushed} against {y}"
    );
    // SAFETY: the session is still alive.
    unsafe { pf_session_free(session) };
}

#[test]
fn the_full_bottom_position_still_reserves_the_whole_strip() {
    let session = session();
    let compressor = catalog_index(session, "supply.compressor");
    // SAFETY: live session and a known preset.
    assert_eq!(
        unsafe { pf_session_set_block_preset(session, PF_BLOCK_BOTTOM_FULL) },
        PF_OK
    );
    let block = block_box(session);
    // SAFETY: live session.
    let placed = unsafe { pf_session_place_component(session, compressor, 60.0, 175.0) };
    let boxed = component_box(session, u32::try_from(placed).unwrap_or(0));
    assert!(
        (boxed[3] - block[1]).abs() < 0.02,
        "edge to edge covers the whole strip: {boxed:?} against {block:?}"
    );
    // SAFETY: the session is still alive.
    unsafe { pf_session_free(session) };
}

#[test]
fn without_a_block_the_frame_is_free_and_enabling_one_reports_the_collision() {
    let session = session();
    let compressor = catalog_index(session, "supply.compressor");
    // SAFETY: live session and the "no block" preset.
    assert_eq!(
        unsafe { pf_session_set_block_preset(session, PF_BLOCK_NONE) },
        PF_OK
    );
    // SAFETY: live session.
    let placed = unsafe { pf_session_place_component(session, compressor, 200.0, 175.0) };
    let boxed = component_box(session, u32::try_from(placed).unwrap_or(0));
    let (_, free_y) = centre_of(boxed);
    assert!(
        (free_y - 175.0).abs() < 0.02,
        "nothing is printed there, so nothing is reserved: {free_y}"
    );

    // SAFETY: live session and the standard preset.
    assert_eq!(
        unsafe { pf_session_set_block_preset(session, PF_BLOCK_BOTTOM_RIGHT) },
        PF_OK
    );
    let mut text = [0 as c_char; 8192];
    // SAFETY: live session and a writable buffer.
    let code = unsafe {
        pf_session_check(
            session,
            text.as_mut_ptr(),
            u32::try_from(text.len()).unwrap_or(u32::MAX),
        )
    };
    assert!(code > 0, "the check reports the collision");
    // SAFETY: the call above wrote a NUL-terminated report.
    let report = unsafe { CStr::from_ptr(text.as_ptr()) }
        .to_string_lossy()
        .into_owned();
    assert!(
        report.contains("under_title_block"),
        "the component is under the block: {report}"
    );
    // SAFETY: the session is still alive.
    unsafe { pf_session_free(session) };
}

#[test]
fn a_content_model_and_visibility_change_what_prints_not_the_values() {
    let session = session();
    let id = u32::try_from(unsafe { pf_session_active_circuit(session) })
        .expect("the identity of the active circuit");
    // SAFETY: live session.
    assert_eq!(
        unsafe { pf_session_block_model(session) },
        i32::try_from(PF_BLOCK_MODEL_INDUSTRIAL).expect("the model")
    );
    // SAFETY: live session and a known model.
    assert_eq!(
        unsafe { pf_session_set_block_model(session, PF_BLOCK_MODEL_SCHOOL) },
        PF_OK
    );
    assert_eq!(
        unsafe { pf_session_block_visible(session, id, PF_BLOCK_OWNER) },
        1
    );
    assert_eq!(
        unsafe { pf_session_block_visible(session, id, PF_BLOCK_NUMBER) },
        0
    );
    assert_eq!(unsafe { pf_session_block_customized(session) }, 0);

    let number = CString::new("PF-2026-07").expect("a number");
    // SAFETY: live session, the circuit identity and a NUL-terminated value.
    assert_eq!(
        unsafe { pf_session_set_block(session, id, PF_BLOCK_NUMBER, number.as_ptr()) },
        PF_OK
    );
    // SAFETY: live session and a known field.
    assert_eq!(
        unsafe { pf_session_set_block_visible(session, id, PF_BLOCK_TITLE, 0) },
        PF_OK
    );
    assert_eq!(unsafe { pf_session_block_customized(session) }, 1);
    assert_eq!(block_field(session, id, PF_BLOCK_TITLE), "Circuito 1");
    assert_eq!(block_field(session, id, PF_BLOCK_NUMBER), "PF-2026-07");

    let mut layout = PfBlockLayout::default();
    // SAFETY: live session and a writable layout.
    assert_eq!(
        unsafe { pf_session_block_layout(session, &raw mut layout) },
        PF_OK
    );
    assert!((layout.width_mm - 170.0).abs() < 0.01);
    assert!((layout.height_mm - 35.0).abs() < 0.01, "school on A4");
    assert_eq!(layout.rows_len, 2, "the school block has two rows");
    let fields: Vec<u32> = (0..layout.cells_len)
        // SAFETY: the pointers borrow a live session and the lengths come from it.
        .map(|index| unsafe { (*layout.cells.add(index as usize)).field })
        .collect();
    assert!(fields.contains(&PF_BLOCK_FIELD_SHEET));
    assert!(!fields.contains(&PF_BLOCK_NUMBER), "number is hidden");

    // A3 gives the school block its 180 × 40 mm size.
    let mut sheet = PfSheet::default();
    // SAFETY: live session and a writable record.
    assert_eq!(unsafe { pf_session_sheet(session, &raw mut sheet) }, PF_OK);
    sheet.paper = 2;
    sheet.landscape = 1;
    // SAFETY: live session and a readable sheet.
    assert_eq!(
        unsafe { pf_session_set_sheet(session, &raw const sheet) },
        PF_OK
    );
    // SAFETY: live session and a writable layout.
    assert_eq!(
        unsafe { pf_session_block_layout(session, &raw mut layout) },
        PF_OK
    );
    assert!((layout.width_mm - 180.0).abs() < 0.01);
    assert!((layout.height_mm - 40.0).abs() < 0.01, "school on A3");

    // Restablecer restores the model's set; the typed number is still there.
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_reset_block_visibility(session) }, PF_OK);
    assert_eq!(unsafe { pf_session_block_customized(session) }, 0);
    assert_eq!(
        unsafe { pf_session_block_visible(session, id, PF_BLOCK_TITLE) },
        1
    );
    assert_eq!(block_field(session, id, PF_BLOCK_NUMBER), "PF-2026-07");
    // SAFETY: the session is still alive.
    unsafe { pf_session_free(session) };
}

#[test]
fn the_block_remark_round_trips_and_the_position_is_one_undo() {
    let session = session();
    let id = u32::try_from(unsafe { pf_session_active_circuit(session) })
        .expect("the identity of the active circuit");
    let remark = CString::new("Revisar la línea 2.\nPedir cotización.").expect("a remark");
    // SAFETY: live session, the circuit identity and a NUL-terminated text.
    assert_eq!(
        unsafe { pf_session_set_block_remark(session, id, remark.as_ptr()) },
        PF_OK
    );
    let mut buffer = [0 as c_char; 256];
    // SAFETY: live session, a writable buffer and its capacity.
    assert!(
        unsafe {
            pf_session_block_remark(
                session,
                id,
                buffer.as_mut_ptr(),
                u32::try_from(buffer.len()).expect("the buffer capacity"),
            )
        } > 0
    );
    // SAFETY: the call wrote a NUL-terminated string.
    assert_eq!(
        unsafe { CStr::from_ptr(buffer.as_ptr()) }.to_str(),
        Ok("Revisar la línea 2.\nPedir cotización.")
    );

    // SAFETY: live session and a known top-right preset.
    assert_eq!(
        unsafe { pf_session_set_block_preset(session, PF_BLOCK_TOP_RIGHT) },
        PF_OK
    );
    assert_eq!(
        unsafe { pf_session_block_preset(session) },
        i32::try_from(PF_BLOCK_TOP_RIGHT).expect("a preset")
    );
    // SAFETY: live session with the position change to undo.
    assert_eq!(unsafe { pf_session_undo(session) }, PF_OK);
    assert_eq!(
        unsafe { pf_session_block_preset(session) },
        i32::try_from(PF_BLOCK_BOTTOM_RIGHT).expect("a preset")
    );
    // SAFETY: the session is still alive.
    unsafe { pf_session_free(session) };
}

#[test]
fn the_check_measures_the_document_sheet_and_not_a_fixed_area() {
    let session = session();
    // The supply box is placed so it straddles the ISO frame: inside a bare A4, outside
    // the 20/10 drawing space.
    // SAFETY: live session.
    assert!(unsafe { pf_session_place_component(session, 0, 20.0, 20.0) } >= 0);
    let mut text = [0 as c_char; 8192];
    // SAFETY: live session and a writable buffer.
    let code = unsafe {
        pf_session_check(
            session,
            text.as_mut_ptr(),
            u32::try_from(text.len()).unwrap_or(u32::MAX),
        )
    };
    assert!(code > 0, "the check always returns its findings");
    // SAFETY: the call above wrote a NUL-terminated report.
    let report = unsafe { CStr::from_ptr(text.as_ptr()) }
        .to_string_lossy()
        .into_owned();
    assert!(
        report.contains("outside_sheet"),
        "the frame is the drawing space: {report}"
    );

    // A larger sheet with no frame contains the same box, so the finding goes away. The
    // check follows the document, not a constant somewhere else.
    let mut sheet = PfSheet::default();
    // SAFETY: live session and a writable record.
    assert_eq!(unsafe { pf_session_sheet(session, &raw mut sheet) }, PF_OK);
    let unframed = PfSheet {
        paper: PF_SHEET_CUSTOM_PAPER,
        width_mm: 500.0,
        height_mm: 500.0,
        margin_top: 0.0,
        margin_right: 0.0,
        margin_bottom: 0.0,
        margin_left: 0.0,
        ..sheet
    };
    // SAFETY: live session and a readable record.
    assert_eq!(
        unsafe { pf_session_set_sheet(session, &raw const unframed) },
        PF_OK
    );
    // SAFETY: live session and a writable buffer.
    let code = unsafe {
        pf_session_check(
            session,
            text.as_mut_ptr(),
            u32::try_from(text.len()).unwrap_or(u32::MAX),
        )
    };
    assert!(code >= 0, "the check ran");
    // SAFETY: the call above wrote a NUL-terminated report.
    let report = unsafe { CStr::from_ptr(text.as_ptr()) }
        .to_string_lossy()
        .into_owned();
    assert!(
        !report.contains("outside_sheet"),
        "the box is inside the new sheet: {report}"
    );
    // SAFETY: the session is still alive.
    unsafe { pf_session_free(session) };
}

#[test]
fn the_sheet_survives_a_save_and_an_open() {
    let directory = tempfile::tempdir().expect("a temporary folder");
    let session = session();
    let mut wanted = PfSheet::default();
    // SAFETY: live session and a writable record.
    assert_eq!(unsafe { pf_session_sheet(session, &raw mut wanted) }, PF_OK);
    wanted.paper = 2;
    wanted.landscape = 1;
    wanted.width_mm = 297.0;
    wanted.height_mm = 420.0;
    wanted.margin_top = 5.0;
    wanted.margin_right = 6.0;
    wanted.margin_bottom = 7.0;
    wanted.margin_left = 8.0;
    // SAFETY: live session and a readable record.
    assert_eq!(
        unsafe { pf_session_set_sheet(session, &raw const wanted) },
        PF_OK
    );

    let path = directory.path().join("hoja.pfluid");
    let encoded = CString::new(path.to_string_lossy().as_bytes()).expect("no NUL in a path");
    // SAFETY: live session and a NUL-terminated path.
    assert_eq!(unsafe { pf_session_save(session, encoded.as_ptr()) }, PF_OK);
    // SAFETY: live session and a NUL-terminated path.
    assert_eq!(unsafe { pf_session_open(session, encoded.as_ptr()) }, PF_OK);

    let mut read = PfSheet::default();
    // SAFETY: live session and a writable record.
    assert_eq!(unsafe { pf_session_sheet(session, &raw mut read) }, PF_OK);
    assert_eq!(read, wanted, "the file carries the sheet");
    // SAFETY: the session is still alive.
    unsafe { pf_session_free(session) };
}

#[test]
fn an_error_message_survives_until_the_next_call() {
    let session = session();
    // SAFETY: live session.
    let message = unsafe { pf_session_last_error(session) };
    assert!(!message.is_null(), "the error slot is never a null pointer");
    // SAFETY: the pointer is a NUL-terminated string owned by the session.
    assert_eq!(
        unsafe { CStr::from_ptr(message) }.to_bytes(),
        b"",
        "a fresh session has no error to report"
    );
    // SAFETY: the session is still alive.
    unsafe { pf_session_free(session) };
}

#[test]
fn a_null_session_is_refused_instead_of_crashing() {
    let mut telemetry = PfTelemetry::default();
    // SAFETY: the contract allows null; the point of the test is that it is handled.
    assert_eq!(
        unsafe { pf_session_telemetry(ptr::null(), &raw mut telemetry) },
        -1
    );
    // SAFETY: freeing null is documented as a no-op.
    unsafe { pf_session_free(ptr::null_mut()) };
}

#[test]
fn the_catalog_lists_every_component_in_a_stable_order() {
    let session = session();
    let mut catalog = PfCatalog::default();
    // SAFETY: live session, writable output.
    assert_eq!(
        unsafe { pf_session_catalog(session, &raw mut catalog) },
        PF_OK
    );
    assert!(
        catalog.items_len >= 5,
        "the built-in catalog has five definitions"
    );

    // SAFETY: both slices were produced by the call above and the session is alive.
    let (items, text) = unsafe {
        (
            std::slice::from_raw_parts(catalog.items, catalog.items_len as usize),
            std::slice::from_raw_parts(catalog.text, catalog.text_len as usize),
        )
    };
    let field = |at: u32, len: u32| -> String {
        let start = at as usize;
        let end = start + len as usize;
        text.get(start..end).map_or_else(String::new, |bytes| {
            String::from_utf8_lossy(bytes).into_owned()
        })
    };

    let mut previous: Option<(u32, String)> = None;
    let mut found_valve = false;
    for item in items {
        let id = field(item.id_at, item.id_len);
        assert!(!id.is_empty(), "a catalog entry without an id");
        assert_ne!(
            field(item.name_at, item.name_len),
            "",
            "a catalog entry without a name"
        );
        if id == "valve.3_2.spring_return" {
            found_valve = true;
        }
        if let Some((category, previous_id)) = previous {
            assert!(
                (item.category, id.as_str()) >= (category, previous_id.as_str()),
                "the sidebar order must be stable: {id} came after {previous_id}"
            );
        }
        previous = Some((item.category, id));
    }
    assert!(found_valve, "the 3/2 valve must reach the sidebar");
    // SAFETY: the session is still alive.
    unsafe { pf_session_free(session) };
}

#[test]
fn a_broken_user_catalog_file_is_counted_and_the_built_ins_still_load() {
    let directory = tempfile::tempdir().expect("a temporary folder");
    std::fs::write(directory.path().join("broken.toml"), "not a catalog file")
        .expect("writes the broken file");
    let path = CString::new(directory.path().to_string_lossy().as_bytes()).expect("a UTF-8 path");
    // SAFETY: the pointer is a live NUL-terminated path.
    let session = unsafe { pf_session_new(path.as_ptr()) };
    assert!(!session.is_null());
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_catalog_problems(session) }, 1);
    let mut catalog = PfCatalog::default();
    // SAFETY: live session and a writable output.
    assert_eq!(
        unsafe { pf_session_catalog(session, &raw mut catalog) },
        PF_OK
    );
    assert!(
        catalog.items_len >= 5,
        "a broken user file must not take the built-ins away"
    );
    // SAFETY: the session is still alive.
    unsafe { pf_session_free(session) };
}

#[test]
fn a_sidebar_entry_is_drawn_with_its_own_symbol() {
    let session = session();
    let mut frame = PfFrame::default();
    // SAFETY: live session, writable output.
    let code = unsafe { pf_session_preview(session, 0, 40.0, 40.0, 3.0, &raw mut frame) };
    assert_eq!(code, PF_OK);
    assert!(frame.calls_len > 2, "a symbol is more than one primitive");
    // SAFETY: the pointers come from the call above.
    let calls = unsafe { std::slice::from_raw_parts(frame.calls, frame.calls_len as usize) };
    assert!(
        calls.iter().any(|call| (call.flags & PF_HAS_STROKE) != 0),
        "a symbol has at least one stroked primitive"
    );
    // An index that does not exist is refused, not indexed.
    // SAFETY: live session.
    assert_eq!(
        unsafe { pf_session_preview(session, 99, 40.0, 40.0, 3.0, &raw mut frame) },
        PF_ERR_MODEL
    );
    // SAFETY: the session is still alive.
    unsafe { pf_session_free(session) };
}

#[test]
fn restart_returns_the_model_to_zero() {
    let session = session();
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_toggle_valve(session) }, PF_OK);
    for _ in 0..100 {
        // SAFETY: live session.
        assert_eq!(unsafe { pf_session_advance(session, 0.002) }, PF_OK);
    }
    let mut moved = PfTelemetry::default();
    // SAFETY: live session, writable output.
    assert_eq!(
        unsafe { pf_session_telemetry(session, &raw mut moved) },
        PF_OK
    );
    assert!(moved.time_s > 0.0);

    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_restart(session) }, PF_OK);
    let mut fresh = PfTelemetry::default();
    // SAFETY: live session, writable output.
    assert_eq!(
        unsafe { pf_session_telemetry(session, &raw mut fresh) },
        PF_OK
    );
    assert!(fresh.time_s.abs() < 1.0e-12, "time must be back at zero");
    assert!(
        fresh.position_m.abs() < 1.0e-12,
        "the piston must be back home"
    );
    assert_eq!(fresh.valve_position, 0);
    // SAFETY: the session is still alive.
    unsafe { pf_session_free(session) };
}

#[test]
fn a_spring_valve_is_only_actuated_while_it_is_held() {
    let session = session();
    // SAFETY: live session.
    assert_eq!(
        unsafe { pf_session_valve_spring_return(session) },
        1,
        "the built-in valve has a spring"
    );

    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_press_valve(session) }, PF_OK);
    let mut held = PfTelemetry::default();
    // SAFETY: live session, writable output.
    assert_eq!(
        unsafe { pf_session_telemetry(session, &raw mut held) },
        PF_OK
    );
    assert_eq!(
        held.valve_position, 1,
        "holding the lever selects the other position"
    );

    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_release_valve(session) }, PF_OK);
    let mut released = PfTelemetry::default();
    // SAFETY: live session, writable output.
    assert_eq!(
        unsafe { pf_session_telemetry(session, &raw mut released) },
        PF_OK
    );
    assert_eq!(
        released.valve_position, 0,
        "the spring returns the spool on release"
    );
    // SAFETY: the session is still alive.
    unsafe { pf_session_free(session) };
}

#[test]
fn a_dropped_valve_and_cylinder_take_the_template_box() {
    let session = self::session();
    let valve = catalog_index(session, "valve.3_2.spring_return");
    let cylinder = catalog_index(session, "cylinder.single_acting_20_100");
    // SAFETY: live session and finite coordinates.
    let valve = unsafe { pf_session_place_component(session, valve, 80.0, 30.0) };
    // SAFETY: live session and finite coordinates.
    let cylinder = unsafe { pf_session_place_component(session, cylinder, 140.0, 60.0) };
    assert!(valve >= 0 && cylinder >= 0, "both components are placed");

    let valve = component_box(session, u32::try_from(valve).unwrap_or_default());
    let cylinder = component_box(session, u32::try_from(cylinder).unwrap_or_default());
    let box_of = |rect: [f32; 4]| (rect[2] - rect[0], rect[3] - rect[1]);

    let (width, height) = box_of(valve);
    assert!(
        (width - SYMBOL_BOX_MM).abs() < 0.01 && (height - SYMBOL_BOX_MM).abs() < 0.01,
        "a dropped valve is the template's {SYMBOL_BOX_MM} mm square, not {width} x {height}"
    );
    let (width, height) = box_of(cylinder);
    let cylinder_width = SYMBOL_BOX_MM * CYLINDER_RATIO;
    assert!(
        (width - cylinder_width).abs() < 0.01 && (height - SYMBOL_BOX_MM).abs() < 0.01,
        "a dropped cylinder is the template's {cylinder_width} x {SYMBOL_BOX_MM} mm box, not {width} x {height}"
    );

    // SAFETY: the session is still alive.
    unsafe { pf_session_free(session) };
}

#[test]
fn a_component_dropped_on_the_sheet_can_be_moved_and_undone() {
    let session = session();
    let mut frame = PfFrame::default();
    // SAFETY: live session, writable output. Seeding happens on the first frame.
    assert_eq!(
        unsafe { pf_session_build_frame(session, 4.0, 0.0, 0, &raw mut frame) },
        PF_OK
    );
    // SAFETY: live session.
    let seeded = unsafe { pf_session_component_count(session) };
    assert_eq!(seeded, 4, "the sheet starts with the simulated circuit");

    // The fifth catalog entry is a component the user can drop; the index is the one the
    // sidebar shows, so this is the same call the drop handler makes.
    // SAFETY: live session.
    let placed = unsafe { pf_session_place_component(session, 0, 63.0, 47.0) };
    let index = u32::try_from(placed).expect("a placed index is not negative");
    assert!(placed >= 0, "placing must succeed");
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_component_count(session) }, seeded + 1);
    // SAFETY: live session.
    assert_eq!(
        unsafe { pf_session_selected(session) },
        placed,
        "a new component is selected"
    );

    let mut bounds = [0.0_f32; 4];
    // SAFETY: live session, four writable floats.
    assert_eq!(
        unsafe { pf_session_component_rect(session, index, bounds.as_mut_ptr()) },
        PF_OK
    );
    assert!(
        (bounds[0] < bounds[2]) && (bounds[1] < bounds[3]),
        "the box has an area"
    );
    assert!(
        (f32::midpoint(bounds[0], bounds[2]) - 60.0).abs() < 6.0,
        "the drop snaps to the ten millimetre grid: {bounds:?}"
    );

    // SAFETY: live session.
    assert_eq!(
        unsafe {
            pf_session_hit_component(
                session,
                f32::midpoint(bounds[0], bounds[2]),
                f32::midpoint(bounds[1], bounds[3]),
            )
        },
        placed,
        "the component answers to a click inside its box"
    );

    // SAFETY: live session.
    assert_eq!(
        unsafe { pf_session_move_component(session, index, 102.0, 76.0) },
        PF_OK
    );
    // SAFETY: live session, four writable floats.
    unsafe { pf_session_component_rect(session, index, bounds.as_mut_ptr()) };
    assert!(
        f32::midpoint(bounds[0], bounds[2]) > 90.0,
        "the move landed on the right"
    );

    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_can_undo(session) }, 1);
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_undo(session) }, PF_OK);
    // SAFETY: live session, four writable floats.
    unsafe { pf_session_component_rect(session, index, bounds.as_mut_ptr()) };
    assert!(
        f32::midpoint(bounds[0], bounds[2]) < 90.0,
        "undo put it back"
    );
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_redo(session) }, PF_OK);

    // SAFETY: live session.
    assert_eq!(
        unsafe { pf_session_remove_component(session, index) },
        PF_OK
    );
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_component_count(session) }, seeded);
    // SAFETY: the session is still alive.
    unsafe { pf_session_free(session) };
}

#[test]
fn two_ports_can_be_joined_and_the_join_is_refused_when_it_makes_no_sense() {
    let session = session();
    // The starting circuit is seeded at creation — compressor, service unit, valve and
    // cylinder, and the three hoses that join them — so it exists before any frame is drawn,
    // which is what lets a front end connect before it paints.
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_component_count(session) }, 4);
    // SAFETY: live session.
    let seeded = unsafe { pf_session_connection_count(session) };
    assert_eq!(seeded, 3, "the starting circuit is drawn with its hoses");

    // A pair the starting circuit does not already use, so this joins rather than repeats.
    let from = CString::new("3").unwrap_or_default();
    let to = CString::new("1").unwrap_or_default();
    // SAFETY: live session and two NUL-terminated strings.
    assert_eq!(
        unsafe { pf_session_connect(session, 1, from.as_ptr(), 2, to.as_ptr()) },
        PF_OK
    );
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_connection_count(session) }, seeded + 1);

    // The same pipe from the other end is the same pipe.
    // SAFETY: live session and two NUL-terminated strings.
    assert_ne!(
        unsafe { pf_session_connect(session, 2, to.as_ptr(), 1, from.as_ptr()) },
        PF_OK
    );
    // A port joined to itself is not a pipe.
    // SAFETY: live session and two NUL-terminated strings.
    assert_ne!(
        unsafe { pf_session_connect(session, 1, from.as_ptr(), 1, from.as_ptr()) },
        PF_OK
    );
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_connection_count(session) }, seeded + 1);

    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_undo(session) }, PF_OK);
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_connection_count(session) }, seeded);
    // SAFETY: the session is still alive.
    unsafe { pf_session_free(session) };
}

#[test]
fn every_port_can_be_enumerated_with_its_role() {
    let session = session();
    let valve = component_with(session, "valve.");
    assert_ne!(valve, u32::MAX, "the worked example has a valve");
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_port_count(session, valve) }, 3);
    // SAFETY: live session.
    assert!(unsafe { pf_session_port_count(session, 999) } < 0);

    let mut ids = Vec::new();
    for slot in 0..3 {
        let mut buffer = [0 as c_char; 32];
        // SAFETY: live session and a writable buffer.
        let length = unsafe { pf_session_port_id(session, valve, slot, buffer.as_mut_ptr(), 32) };
        assert!(length > 0, "slot {slot} must name a port");
        ids.push(written(&buffer));
    }
    assert_eq!(ids, ["1", "2", "3"]);

    let role = |port: &str| {
        let id = CString::new(port).unwrap_or_default();
        // SAFETY: live session and a NUL-terminated port.
        unsafe { pf_session_port_role(session, valve, id.as_ptr()) }
    };
    assert_eq!(role("1"), 0, "supply");
    assert_eq!(role("2"), 1, "working");
    assert_eq!(role("3"), 2, "exhaust");
    assert!(role("9") < 0, "a port the definition does not have");

    // A slot past the end and a buffer too small are errors, not silent zeros.
    let mut one = [0 as c_char; 1];
    // SAFETY: live session and a one-byte buffer.
    assert_eq!(
        unsafe { pf_session_port_id(session, valve, 99, one.as_mut_ptr(), 1) },
        PF_ERR_MODEL
    );
    // SAFETY: live session and a one-byte buffer.
    assert_eq!(
        unsafe { pf_session_port_id(session, valve, 0, one.as_mut_ptr(), 1) },
        PF_ERR_MODEL
    );
    // SAFETY: the session is still alive.
    unsafe { pf_session_free(session) };
}

#[test]
fn a_port_closure_is_fitted_cleared_and_undone_as_one_step() {
    let session = session();
    let valve = component_with(session, "valve.");
    let exhaust = CString::new("3").unwrap_or_default();
    let supply = CString::new("1").unwrap_or_default();
    let plug = CString::new("plug.blind").unwrap_or_default();
    let silencer = CString::new("silencer.pneumatic").unwrap_or_default();
    let clear = CString::new("").unwrap_or_default();

    let read = |buffer: &mut [c_char; 64]| {
        // SAFETY: live session, NUL-terminated port and a writable buffer.
        unsafe {
            pf_session_port_closure(session, valve, exhaust.as_ptr(), buffer.as_mut_ptr(), 64)
        }
    };
    let mut buffer = [0 as c_char; 64];
    assert_eq!(read(&mut buffer), 0, "a port starts with no closure");
    assert_eq!(written(&buffer), "");

    // A plug on the exhaust, then back and forth: one undo per change.
    // SAFETY: live session and NUL-terminated strings.
    assert_eq!(
        unsafe { pf_session_set_port_closure(session, valve, exhaust.as_ptr(), plug.as_ptr()) },
        PF_OK
    );
    assert!(read(&mut buffer) > 0);
    assert_eq!(written(&buffer), "plug.blind");
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_can_undo(session) }, 1);
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_undo(session) }, PF_OK);
    assert_eq!(read(&mut buffer), 0);
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_redo(session) }, PF_OK);
    assert!(read(&mut buffer) > 0);
    assert_eq!(written(&buffer), "plug.blind");

    // A silencer replaces it, and an empty id clears it.
    // SAFETY: live session and NUL-terminated strings.
    assert_eq!(
        unsafe { pf_session_set_port_closure(session, valve, exhaust.as_ptr(), silencer.as_ptr()) },
        PF_OK
    );
    assert!(read(&mut buffer) > 0);
    assert_eq!(written(&buffer), "silencer.pneumatic");
    // SAFETY: live session and NUL-terminated strings.
    assert_eq!(
        unsafe { pf_session_set_port_closure(session, valve, exhaust.as_ptr(), clear.as_ptr()) },
        PF_OK
    );
    assert_eq!(read(&mut buffer), 0);

    // A silencer cannot vent a supply port: the role decides, not the caller.
    // SAFETY: live session and NUL-terminated strings.
    assert_ne!(
        unsafe { pf_session_set_port_closure(session, valve, supply.as_ptr(), silencer.as_ptr()) },
        PF_OK
    );
    // SAFETY: live session.
    assert!(
        !unsafe { CStr::from_ptr(pf_session_last_error(session)) }
            .to_bytes()
            .is_empty()
    );

    // An unknown id and an unknown port are refused, and none of it changed the document.
    let unknown = CString::new("not.a.closure").unwrap_or_default();
    let nine = CString::new("9").unwrap_or_default();
    // SAFETY: live session and NUL-terminated strings.
    assert_ne!(
        unsafe { pf_session_set_port_closure(session, valve, exhaust.as_ptr(), unknown.as_ptr()) },
        PF_OK
    );
    // SAFETY: live session and NUL-terminated strings.
    assert_ne!(
        unsafe { pf_session_set_port_closure(session, valve, nine.as_ptr(), plug.as_ptr()) },
        PF_OK
    );
    assert_eq!(read(&mut buffer), 0);

    // The getter's own bound: an empty closure fits one byte, a real id does not.
    let mut one = [0 as c_char; 1];
    // SAFETY: live session, NUL-terminated port and a one-byte buffer.
    assert_eq!(
        unsafe { pf_session_port_closure(session, valve, exhaust.as_ptr(), one.as_mut_ptr(), 1) },
        0
    );
    // SAFETY: live session and NUL-terminated strings.
    assert_eq!(
        unsafe { pf_session_set_port_closure(session, valve, exhaust.as_ptr(), plug.as_ptr()) },
        PF_OK
    );
    // SAFETY: live session, NUL-terminated port and a one-byte buffer.
    assert_eq!(
        unsafe { pf_session_port_closure(session, valve, exhaust.as_ptr(), one.as_mut_ptr(), 1) },
        PF_ERR_MODEL
    );
    // SAFETY: the session is still alive.
    unsafe { pf_session_free(session) };
}

#[test]
fn an_electrical_port_takes_no_closure() {
    let session = session();
    let rail = catalog_index(session, "electrical.supply");
    // SAFETY: live session and finite coordinates.
    let rail = unsafe { pf_session_place_component(session, rail, 40.0, 20.0) };
    assert!(rail >= 0, "the rail is placed");
    let rail = u32::try_from(rail).unwrap_or_default();

    let live = CString::new("L").unwrap_or_default();
    // SAFETY: live session and a NUL-terminated port.
    assert_eq!(
        unsafe { pf_session_port_role(session, rail, live.as_ptr()) },
        4
    );

    let plug = CString::new("plug.blind").unwrap_or_default();
    let clear = CString::new("").unwrap_or_default();
    // SAFETY: live session and NUL-terminated strings.
    assert_ne!(
        unsafe { pf_session_set_port_closure(session, rail, live.as_ptr(), plug.as_ptr()) },
        PF_OK
    );
    // SAFETY: live session.
    assert!(
        !unsafe { CStr::from_ptr(pf_session_last_error(session)) }
            .to_bytes()
            .is_empty()
    );
    // Even clearing is refused: the port has no closure to manage at all.
    // SAFETY: live session and NUL-terminated strings.
    assert_ne!(
        unsafe { pf_session_set_port_closure(session, rail, live.as_ptr(), clear.as_ptr()) },
        PF_OK
    );

    let mut buffer = [0 as c_char; 32];
    // SAFETY: live session, NUL-terminated port and a writable buffer.
    assert_eq!(
        unsafe { pf_session_port_closure(session, rail, live.as_ptr(), buffer.as_mut_ptr(), 32) },
        0
    );
    assert_eq!(written(&buffer), "");
    // SAFETY: the session is still alive.
    unsafe { pf_session_free(session) };
}

#[test]
fn a_hose_blocks_a_closure_until_it_is_removed() {
    let session = session();
    let valve = component_with(session, "valve.");
    let working = CString::new("2").unwrap_or_default();
    let exhaust = CString::new("3").unwrap_or_default();
    let nine = CString::new("9").unwrap_or_default();

    // SAFETY: live session and NUL-terminated ports.
    assert_eq!(
        unsafe { pf_session_port_has_pipe(session, valve, working.as_ptr()) },
        1
    );
    // SAFETY: live session and a NUL-terminated port.
    assert_eq!(
        unsafe { pf_session_port_has_pipe(session, valve, exhaust.as_ptr()) },
        0
    );
    // SAFETY: live session and a NUL-terminated port.
    assert!(unsafe { pf_session_port_has_pipe(session, valve, nine.as_ptr()) } < 0);

    // A working port accepts a plug, but not while the hose is there.
    let plug = CString::new("plug.blind").unwrap_or_default();
    // SAFETY: live session and NUL-terminated strings.
    assert_ne!(
        unsafe { pf_session_set_port_closure(session, valve, working.as_ptr(), plug.as_ptr()) },
        PF_OK
    );
    // SAFETY: live session.
    let message = unsafe { CStr::from_ptr(pf_session_last_error(session)) }
        .to_string_lossy()
        .into_owned();
    assert!(
        message.contains("hose"),
        "the refusal explains the hose: {message}"
    );

    // The worked example's third pipe joins the valve's working port `2` to the cylinder.
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_select_pipe(session, 2) }, PF_OK);
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_remove_selected_pipe(session) }, PF_OK);
    // SAFETY: live session and a NUL-terminated port.
    assert_eq!(
        unsafe { pf_session_port_has_pipe(session, valve, working.as_ptr()) },
        0
    );
    // SAFETY: live session and NUL-terminated strings.
    assert_eq!(
        unsafe { pf_session_set_port_closure(session, valve, working.as_ptr(), plug.as_ptr()) },
        PF_OK
    );
    // SAFETY: the session is still alive.
    unsafe { pf_session_free(session) };
}

#[test]
fn the_closure_definitions_come_from_the_session_catalog() {
    let session = session();
    // SAFETY: live session.
    assert_eq!(unsafe { pf_closure_count(session) }, 2);

    let mut id = [0 as c_char; 64];
    let mut name = [0 as c_char; 64];
    // The catalog orders definitions by id, so the plug comes first.
    // SAFETY: live session and writable buffers.
    assert!(unsafe { pf_closure_id(session, 0, id.as_mut_ptr(), 64) } > 0);
    assert_eq!(written(&id), "plug.blind");
    // SAFETY: live session and writable buffers.
    assert!(unsafe { pf_closure_name(session, 0, name.as_mut_ptr(), 64) } > 0);
    assert_eq!(written(&name), "Tapón ciego");
    // SAFETY: live session.
    assert_eq!(unsafe { pf_closure_is_plug(session, 0) }, 1);

    // SAFETY: live session and writable buffers.
    assert!(unsafe { pf_closure_id(session, 1, id.as_mut_ptr(), 64) } > 0);
    assert_eq!(written(&id), "silencer.pneumatic");
    // SAFETY: live session and writable buffers.
    assert!(unsafe { pf_closure_name(session, 1, name.as_mut_ptr(), 64) } > 0);
    assert_eq!(written(&name), "Silenciador");
    // SAFETY: live session.
    assert_eq!(unsafe { pf_closure_is_plug(session, 1) }, 0);

    // Past the end and a short buffer are errors.
    let mut one = [0 as c_char; 1];
    // SAFETY: live session and a one-byte buffer.
    assert_eq!(
        unsafe { pf_closure_id(session, 9, one.as_mut_ptr(), 1) },
        PF_ERR_MODEL
    );
    // SAFETY: live session and a one-byte buffer.
    assert_eq!(
        unsafe { pf_closure_id(session, 0, one.as_mut_ptr(), 1) },
        PF_ERR_MODEL
    );
    // SAFETY: live session.
    assert!(unsafe { pf_closure_is_plug(session, 9) } < 0);
    // SAFETY: the session is still alive.
    unsafe { pf_session_free(session) };
}

#[test]
fn the_abi_version_is_the_one_the_header_declares() {
    assert_eq!(PF_ABI_VERSION, 13);
}

#[test]
fn the_frame_accepts_a_phase_and_a_motion_flag_and_refuses_any_other_flag() {
    let session = session();
    let mut frame = PfFrame::default();
    // SAFETY: live session and a writable frame.
    assert_eq!(
        unsafe { pf_session_build_frame(session, 4.0, 0.0, 0, &raw mut frame) },
        PF_OK,
        "still motion is a valid flag"
    );
    // SAFETY: live session and a writable frame.
    assert_eq!(
        unsafe { pf_session_build_frame(session, 4.0, 1.25, 1, &raw mut frame) },
        PF_OK,
        "a running pulse is a valid flag"
    );
    // SAFETY: live session and a writable frame.
    assert_eq!(
        unsafe { pf_session_build_frame(session, 4.0, 1.25, 2, &raw mut frame) },
        PF_ERR_MODEL,
        "a flag that is neither 0 nor 1 is refused, not treated as true"
    );
    // SAFETY: live session and a writable frame.
    assert_eq!(
        unsafe { pf_session_build_frame(session, 4.0, 1.25, -1, &raw mut frame) },
        PF_ERR_MODEL
    );
    // SAFETY: the session is still alive.
    unsafe { pf_session_free(session) };
}

#[test]
fn a_five_port_valve_and_a_double_acting_cylinder_are_a_circuit_of_files() {
    let session = session();
    let valve = catalog_index(session, "valve.5_2.spring_return");
    let cylinder = catalog_index(session, "cylinder.double_acting_32_200");

    // Clear the worked example so the new cylinder is the one telemetry reports.
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_remove_component(session, 2) }, PF_OK);
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_remove_component(session, 1) }, PF_OK);

    // SAFETY: live session.
    let valve_index = unsafe { pf_session_place_component(session, valve, 80.0, 30.0) };
    assert!(valve_index >= 0);
    // SAFETY: live session.
    let cylinder_index = unsafe { pf_session_place_component(session, cylinder, 140.0, 60.0) };
    assert!(cylinder_index >= 0);
    let valve_index = u32::try_from(valve_index).unwrap_or_default();
    let cylinder_index = u32::try_from(cylinder_index).unwrap_or_default();

    let supply = CString::new("1").unwrap_or_default();
    let one = CString::new("1").unwrap_or_default();
    let two = CString::new("2").unwrap_or_default();
    let four = CString::new("4").unwrap_or_default();
    // supply:1 -> valve:1; while actuated the valve feeds 4, so 4 -> cylinder:1
    // (cap end, extends) and 2 -> cylinder:2 (rod end, vents through 3 at rest).
    // SAFETY: live session and NUL-terminated strings.
    assert_eq!(
        unsafe { pf_session_connect(session, 0, supply.as_ptr(), valve_index, one.as_ptr()) },
        PF_OK
    );
    // SAFETY: live session and NUL-terminated strings.
    assert_eq!(
        unsafe {
            pf_session_connect(
                session,
                valve_index,
                four.as_ptr(),
                cylinder_index,
                one.as_ptr(),
            )
        },
        PF_OK
    );
    // SAFETY: live session and NUL-terminated strings.
    assert_eq!(
        unsafe {
            pf_session_connect(
                session,
                valve_index,
                two.as_ptr(),
                cylinder_index,
                two.as_ptr(),
            )
        },
        PF_OK
    );

    // The five-port valve is a valve to the ABI, and it starts at rest.
    // SAFETY: live session.
    assert_eq!(
        unsafe { pf_session_component_valve_position(session, valve_index) },
        0
    );
    // SAFETY: live session.
    assert_eq!(
        unsafe { pf_session_component_valve_spring_return(session, valve_index) },
        1
    );

    // Pressed, the spool moves and the rod goes out.
    // SAFETY: live session.
    assert_eq!(
        unsafe { pf_session_press_component_valve(session, valve_index) },
        PF_OK
    );
    // SAFETY: live session.
    assert_eq!(
        unsafe { pf_session_component_valve_position(session, valve_index) },
        1
    );
    for _ in 0..500 {
        assert_eq!(unsafe { pf_session_advance(session, 0.016) }, PF_OK);
    }
    let mut out = PfTelemetry::default();
    // SAFETY: live session, writable output.
    assert_eq!(
        unsafe { pf_session_telemetry(session, &raw mut out) },
        PF_OK
    );
    assert_eq!(out.has_model, 1, "the document is the model");
    assert!(
        out.position_m > out.stroke_m * 0.99,
        "the rod reached {:.1} of {:.1} mm",
        out.position_m * 1000.0,
        out.stroke_m * 1000.0
    );

    // Released, the spring returns the spool and the other working line drives it back.
    // SAFETY: live session.
    assert_eq!(
        unsafe { pf_session_release_component_valve(session, valve_index) },
        PF_OK
    );
    for _ in 0..500 {
        assert_eq!(unsafe { pf_session_advance(session, 0.016) }, PF_OK);
    }
    // SAFETY: live session, writable output.
    assert_eq!(
        unsafe { pf_session_telemetry(session, &raw mut out) },
        PF_OK
    );
    assert!(
        out.position_m < 0.002,
        "the rod returned to {:.1} mm",
        out.position_m * 1000.0
    );
    // SAFETY: the session is still alive.
    unsafe { pf_session_free(session) };
}

#[test]
fn a_cylinder_edit_changes_the_model_and_is_one_undo() {
    let session = session();
    let mut current = PfEditSet::default();
    // SAFETY: live session and a writable structure.
    assert_eq!(
        unsafe { pf_session_component_edits(session, 2, &raw mut current) },
        PF_OK
    );
    assert_ne!(current.mask & PF_EDIT_STROKE, 0);
    assert!((current.stroke_mm - 100.0).abs() < 1e-9);
    assert!((current.bore_mm - 20.0).abs() < 1e-9);

    let edit = PfEditSet {
        mask: PF_EDIT_STROKE | PF_EDIT_BORE,
        stroke_mm: 50.0,
        bore_mm: 25.0,
        ..PfEditSet::default()
    };
    // SAFETY: live session and a readable structure.
    assert_eq!(
        unsafe { pf_session_set_component_edits(session, 2, &raw const edit) },
        PF_OK
    );
    let mut telemetry = PfTelemetry::default();
    // SAFETY: live session, writable output.
    assert_eq!(
        unsafe { pf_session_telemetry(session, &raw mut telemetry) },
        PF_OK
    );
    assert!(
        (telemetry.stroke_m - 0.050).abs() < 1e-9,
        "the model uses the edited stroke, not {} mm",
        telemetry.stroke_m * 1000.0
    );

    // The effective values are the edited ones, and one undo brings the file's back.
    // SAFETY: live session and a writable structure.
    assert_eq!(
        unsafe { pf_session_component_edits(session, 2, &raw mut current) },
        PF_OK
    );
    assert!((current.stroke_mm - 50.0).abs() < 1e-9);
    assert!((current.bore_mm - 25.0).abs() < 1e-9);
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_undo(session) }, PF_OK);
    // SAFETY: live session, writable output.
    assert_eq!(
        unsafe { pf_session_telemetry(session, &raw mut telemetry) },
        PF_OK
    );
    assert!((telemetry.stroke_m - 0.100).abs() < 1e-9);

    // A rod as large as the bore is refused, and nothing changes.
    let bad = PfEditSet {
        mask: PF_EDIT_ROD,
        rod_mm: 25.0,
        ..PfEditSet::default()
    };
    // SAFETY: live session and a readable structure.
    assert_eq!(
        unsafe { pf_session_set_component_edits(session, 2, &raw const bad) },
        PF_ERR_MODEL
    );
    // SAFETY: live session and a writable structure.
    assert_eq!(
        unsafe { pf_session_component_edits(session, 2, &raw mut current) },
        PF_OK
    );
    assert!((current.rod_mm - 0.0).abs() < 1e-9);
    // SAFETY: the session is still alive.
    unsafe { pf_session_free(session) };
}

#[test]
fn a_supply_pressure_edit_reaches_the_gas() {
    let session = session();
    let edit = PfEditSet {
        mask: 1,
        pressure_bar: 3.0,
        ..PfEditSet::default()
    };
    // SAFETY: live session and a readable structure.
    assert_eq!(
        unsafe { pf_session_set_component_edits(session, 0, &raw const edit) },
        PF_OK
    );
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_press_valve(session) }, PF_OK);
    for _ in 0..60 {
        assert_eq!(unsafe { pf_session_advance(session, 0.016) }, PF_OK);
    }
    let mut telemetry = PfTelemetry::default();
    // SAFETY: live session, writable output.
    assert_eq!(
        unsafe { pf_session_telemetry(session, &raw mut telemetry) },
        PF_OK
    );
    let gauge = (telemetry.pressure_pa - 101_325.0) / 1e5;
    assert!(
        (gauge - 3.0).abs() < 0.05,
        "the chamber settles at {gauge:.2} bar, not at the edited 3 bar"
    );
    // SAFETY: the session is still alive.
    unsafe { pf_session_free(session) };
}

#[test]
fn a_new_supply_is_four_bar() {
    // The owner chose 4 bar gauge as the industry default on 2026-11-20. A new placement
    // reads the catalog value, so all three built-in supplies have to start there.
    let session = unsafe { pf_session_new(ptr::null()) };
    assert!(!session.is_null(), "a session must always be created");

    for id in [
        "supply.compressor",
        "supply.compressor_adjustable",
        "supply.network",
    ] {
        // SAFETY: live session, a catalog index and sheet coordinates.
        let index =
            unsafe { pf_session_place_component(session, catalog_index(session, id), 40.0, 40.0) };
        assert!(index >= 0, "`{id}` could not be placed");
        let index = u32::try_from(index).unwrap_or_default();
        let mut edits = PfEditSet::default();
        // SAFETY: live session and a writable structure.
        assert_eq!(
            unsafe { pf_session_component_edits(session, index, &raw mut edits) },
            PF_OK
        );
        assert_ne!(edits.mask & PF_EDIT_PRESSURE, 0, "`{id}` has no pressure");
        assert!(
            (edits.pressure_bar - 4.0).abs() < 1e-9,
            "`{id}` starts at {} bar, not at the 4 bar default",
            edits.pressure_bar
        );
    }
    // SAFETY: the session is still alive.
    unsafe { pf_session_free(session) };
}

#[test]
fn an_explicit_six_bar_edit_is_kept_when_the_file_is_reopened() {
    let directory = tempfile::tempdir().expect("a temporary folder");
    let path = directory.path().join("seis-bar.pfluid");
    let encoded = CString::new(path.to_string_lossy().as_bytes()).unwrap_or_default();

    let session = session();
    let edit = PfEditSet {
        mask: PF_EDIT_PRESSURE,
        pressure_bar: 6.0,
        ..PfEditSet::default()
    };
    // SAFETY: live session and a readable structure.
    assert_eq!(
        unsafe { pf_session_set_component_edits(session, 0, &raw const edit) },
        PF_OK
    );
    // SAFETY: live session and a NUL-terminated path.
    assert_eq!(unsafe { pf_session_save(session, encoded.as_ptr()) }, PF_OK);
    // SAFETY: the session is still alive.
    unsafe { pf_session_free(session) };

    // The explicit value is the document's, not a copy of the catalog default.
    let session = self::session();
    // SAFETY: live session and a NUL-terminated path.
    assert_eq!(unsafe { pf_session_open(session, encoded.as_ptr()) }, PF_OK);
    let mut edits = PfEditSet::default();
    // SAFETY: live session and a writable structure.
    assert_eq!(
        unsafe { pf_session_component_edits(session, 0, &raw mut edits) },
        PF_OK
    );
    assert!(
        (edits.pressure_bar - 6.0).abs() < 1e-9,
        "the explicit 6 bar edit was not kept: {} bar",
        edits.pressure_bar
    );

    // The gas reads the reopened edit: 6 bar gauge is 6e5 Pa above atmosphere.
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_press_valve(session) }, PF_OK);
    for _ in 0..60 {
        // SAFETY: live session.
        assert_eq!(unsafe { pf_session_advance(session, 0.016) }, PF_OK);
    }
    let mut telemetry = PfTelemetry::default();
    // SAFETY: live session, writable output.
    assert_eq!(
        unsafe { pf_session_telemetry(session, &raw mut telemetry) },
        PF_OK
    );
    let gauge = (telemetry.pressure_pa - 101_325.0) / 1e5;
    assert!(
        (gauge - 6.0).abs() < 0.05,
        "the chamber settles at {gauge:.2} bar, not at the reopened 6 bar"
    );
    // SAFETY: the session is still alive.
    unsafe { pf_session_free(session) };
}

#[test]
fn an_old_supply_without_an_edit_reads_the_new_default() {
    // A sheet written before this change carries a supply but no per-placement pressure.
    // The value was never stored, so opening it reads the catalog default; that deliberately
    // changes the circuit to 4 bar, and a document that needs 6 bar must carry the edit.
    let directory = tempfile::tempdir().expect("a temporary folder");
    let path = directory.path().join("sin-edicion.pfluid");
    std::fs::write(
        &path,
        r#"{
  "format": 3,
  "title": "Un documento anterior",
  "placement": [
    { "id": 1, "catalog": "supply.compressor", "x": 40.0, "y": 40.0, "width": 20.0, "height": 20.0 },
    { "id": 2, "catalog": "valve.3_2.spring_return", "x": 100.0, "y": 80.0, "width": 20.0, "height": 20.0 },
    { "id": 3, "catalog": "cylinder.single_acting_20_100", "x": 160.0, "y": 80.0, "width": 44.0, "height": 20.0 }
  ],
  "pipe": [
    { "from": 1, "from_port": "1", "to": 2, "to_port": "1" },
    { "from": 2, "from_port": "2", "to": 3, "to_port": "1" }
  ]
}"#,
    )
    .expect("the old sheet is written");
    let encoded = CString::new(path.to_string_lossy().as_bytes()).unwrap_or_default();

    let session = unsafe { pf_session_new(ptr::null()) };
    assert!(!session.is_null(), "a session must always be created");
    // SAFETY: live session and a NUL-terminated path.
    assert_eq!(unsafe { pf_session_open(session, encoded.as_ptr()) }, PF_OK);
    let mut edits = PfEditSet::default();
    // SAFETY: live session and a writable structure.
    assert_eq!(
        unsafe { pf_session_component_edits(session, 0, &raw mut edits) },
        PF_OK
    );
    assert!(
        (edits.pressure_bar - 4.0).abs() < 1e-9,
        "the unedited supply reads {} bar, not the new 4 bar default",
        edits.pressure_bar
    );

    // It runs at 4 bar: 4e5 Pa above atmosphere.
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_press_valve(session) }, PF_OK);
    for _ in 0..60 {
        // SAFETY: live session.
        assert_eq!(unsafe { pf_session_advance(session, 0.016) }, PF_OK);
    }
    let mut telemetry = PfTelemetry::default();
    // SAFETY: live session, writable output.
    assert_eq!(
        unsafe { pf_session_telemetry(session, &raw mut telemetry) },
        PF_OK
    );
    let gauge = (telemetry.pressure_pa - 101_325.0) / 1e5;
    assert!(
        (gauge - 4.0).abs() < 0.05,
        "the old sheet settles at {gauge:.2} bar, not at the new 4 bar default"
    );
    // SAFETY: the session is still alive.
    unsafe { pf_session_free(session) };
}

#[test]
fn a_pressure_gauge_reads_the_line_it_is_connected_to() {
    let session = session();
    let gauge = catalog_index(session, "gauge.pressure");
    // SAFETY: live session.
    let gauge_index = unsafe { pf_session_place_component(session, gauge, 160.0, 100.0) };
    assert!(gauge_index >= 0);
    let gauge_index = u32::try_from(gauge_index).unwrap_or_default();
    let working = CString::new("2").unwrap_or_default();
    let one = CString::new("1").unwrap_or_default();
    // The gauge hangs from the valve's working line, beside the cylinder.
    // SAFETY: live session and NUL-terminated strings.
    assert_eq!(
        unsafe { pf_session_connect(session, 1, working.as_ptr(), gauge_index, one.as_ptr()) },
        PF_OK
    );
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_press_valve(session) }, PF_OK);
    for _ in 0..60 {
        assert_eq!(unsafe { pf_session_advance(session, 0.016) }, PF_OK);
    }

    let mut frame = PfFrame::default();
    // SAFETY: live session, writable output.
    assert_eq!(
        unsafe { pf_session_build_frame(session, 4.0, 0.0, 0, &raw mut frame) },
        PF_OK
    );
    // SAFETY: the slices were produced by the call above and the session is alive.
    let (calls, text) = unsafe {
        (
            std::slice::from_raw_parts(frame.calls, frame.calls_len as usize),
            std::slice::from_raw_parts(frame.text, frame.text_len as usize),
        )
    };
    let field = |at: u32, len: u32| -> &[u8] {
        let start = at as usize;
        let end = start + len as usize;
        text.get(start..end).unwrap_or_default()
    };
    // The gauge is 22 × 22 mm at (160, 100): its reading sits inside that box.
    let inside = |call: &PfCall| {
        call.rect[0] > 149.0 && call.rect[0] < 171.0 && call.rect[1] > 89.0 && call.rect[1] < 111.0
    };
    let mut reading = String::new();
    for call in calls
        .iter()
        .filter(|call| call.kind == PF_KIND_TEXT && inside(call))
    {
        let text = String::from_utf8_lossy(field(call.text_at, call.text_len)).into_owned();
        if text.ends_with(" bar") {
            reading = text;
            break;
        }
    }
    assert!(
        !reading.is_empty(),
        "the gauge has no reading among the {} text calls in the frame",
        calls
            .iter()
            .filter(|call| call.kind == PF_KIND_TEXT)
            .count()
    );
    let value: f32 = reading.trim_end_matches(" bar").parse().unwrap_or_default();
    assert!(
        (3.5..=4.5).contains(&value),
        "the gauge reads `{reading}`, not the 4 bar line it is connected to"
    );
    // SAFETY: the session is still alive.
    unsafe { pf_session_free(session) };
}

#[test]
fn copy_paste_and_duplicate_are_one_change_each() {
    let session = session();
    let before = unsafe { pf_session_component_count(session) };
    // Copying without a selection is refused, not answered with the last clipboard.
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_copy_selected(session) }, PF_ERR_MODEL);

    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_select(session, 1) }, PF_OK);
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_copy_selected(session) }, PF_OK);
    // SAFETY: live session.
    let pasted = unsafe { pf_session_paste(session, 50.0, 50.0) };
    assert!(pasted >= 0);
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_component_count(session) }, before + 1);
    // The copy is the same catalog entry and is selected.
    let mut id = [0_i8; 128];
    // SAFETY: live session and a writable buffer.
    assert!(
        unsafe {
            pf_session_component_id(
                session,
                u32::try_from(pasted).unwrap_or_default(),
                id.as_mut_ptr().cast(),
                128,
            )
        } >= 0
    );
    let copied = unsafe { CStr::from_ptr(id.as_ptr()) }
        .to_str()
        .unwrap_or_default();
    assert_eq!(copied, "valve.3_2.spring_return");
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_selected(session) }, pasted);
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_undo(session) }, PF_OK);
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_component_count(session) }, before);

    // Paste offset and duplicate keep the per-instance values and are undoable.
    let edit = PfEditSet {
        mask: PF_EDIT_STROKE,
        stroke_mm: 50.0,
        ..PfEditSet::default()
    };
    // SAFETY: live session and a readable structure.
    assert_eq!(
        unsafe { pf_session_set_component_edits(session, 2, &raw const edit) },
        PF_OK
    );
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_select(session, 2) }, PF_OK);
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_copy_selected(session) }, PF_OK);
    // SAFETY: live session.
    let offset = unsafe { pf_session_paste_offset(session, 10.0, 10.0) };
    assert!(offset >= 0);
    let mut current = PfEditSet::default();
    // SAFETY: live session and a writable structure.
    assert_eq!(
        unsafe {
            pf_session_component_edits(
                session,
                u32::try_from(offset).unwrap_or_default(),
                &raw mut current,
            )
        },
        PF_OK
    );
    assert!((current.stroke_mm - 50.0).abs() < 1e-9);
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_undo(session) }, PF_OK);

    // SAFETY: live session.
    let copy = unsafe { pf_session_duplicate_component(session, 1, 10.0, 10.0) };
    assert!(copy >= 0);
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_component_count(session) }, before + 1);
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_undo(session) }, PF_OK);
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_component_count(session) }, before);
    // SAFETY: live session.
    assert_eq!(
        unsafe { pf_session_duplicate_component(session, 99, 0.0, 0.0) },
        PF_ERR_MODEL
    );
    // SAFETY: the session is still alive.
    unsafe { pf_session_free(session) };
}

#[test]
fn a_sheet_saves_and_opens_with_its_edits_and_routes() {
    let directory = tempfile::tempdir().expect("a temporary folder");
    let path = directory.path().join("circuito.pfluid");
    let encoded = CString::new(path.to_string_lossy().as_bytes()).unwrap_or_default();

    let session = session();
    // An edit and a route have to survive the round trip, not just the placement list.
    let edit = PfEditSet {
        mask: PF_EDIT_STROKE,
        stroke_mm: 50.0,
        ..PfEditSet::default()
    };
    // SAFETY: live session and a readable structure.
    assert_eq!(
        unsafe { pf_session_set_component_edits(session, 2, &raw const edit) },
        PF_OK
    );
    // SAFETY: live session.
    assert_eq!(
        unsafe { pf_session_route_pipe(session, 2, 40.0, 20.0) },
        PF_OK
    );
    let mut routed = 0.0_f32;
    // SAFETY: live session and a writable float.
    assert_eq!(
        unsafe { pf_session_pipe_offset(session, 2, &raw mut routed) },
        PF_OK
    );
    // SAFETY: live session and a NUL-terminated path.
    assert_eq!(unsafe { pf_session_save(session, encoded.as_ptr()) }, PF_OK);
    assert!(path.exists());
    // SAFETY: the session is still alive.
    unsafe { pf_session_free(session) };

    // A fresh session opens the file and reports the edited stroke again.
    let session = self::session();
    // SAFETY: live session and a NUL-terminated path.
    assert_eq!(unsafe { pf_session_open(session, encoded.as_ptr()) }, PF_OK);
    let mut telemetry = PfTelemetry::default();
    // SAFETY: live session, writable output.
    assert_eq!(
        unsafe { pf_session_telemetry(session, &raw mut telemetry) },
        PF_OK
    );
    assert_eq!(telemetry.has_model, 1);
    assert!((telemetry.stroke_m - 0.050).abs() < 1e-9);
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_connection_count(session) }, 3);
    let mut offset = 0.0_f32;
    // SAFETY: live session and a writable float.
    assert_eq!(
        unsafe { pf_session_pipe_offset(session, 2, &raw mut offset) },
        PF_OK
    );
    assert!(
        (offset - routed).abs() < 1e-4,
        "the route is part of the sheet"
    );

    // A malformed file is refused and the open document is untouched.
    let bad = directory.path().join("futuro.pfluid");
    std::fs::write(&bad, r#"{"format":99,"placement":[],"pipe":[]}"#).expect("writes the bad file");
    let bad = CString::new(bad.to_string_lossy().as_bytes()).unwrap_or_default();
    // SAFETY: live session and a NUL-terminated path.
    assert_eq!(
        unsafe { pf_session_open(session, bad.as_ptr()) },
        PF_ERR_MODEL
    );
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_component_count(session) }, 4);
    // SAFETY: the session is still alive.
    unsafe { pf_session_free(session) };
}

#[test]
fn a_pipe_segment_can_be_dragged_and_undo_puts_it_back() {
    let session = session();
    // SAFETY: live session; the starting circuit always has two hoses.
    assert!(unsafe { pf_session_connection_count(session) } >= 1);
    let mut offset = f32::NAN;
    // SAFETY: live session and a writable float.
    assert_eq!(
        unsafe { pf_session_pipe_offset(session, 2, &raw mut offset) },
        PF_OK
    );
    assert!(
        offset.abs() < 1e-6,
        "a new hose runs on the automatic route"
    );

    // Drag the middle segment well away from where it was.
    // SAFETY: live session.
    assert_eq!(
        unsafe { pf_session_route_pipe(session, 2, 40.0, 20.0) },
        PF_OK
    );
    // SAFETY: live session and a writable float.
    assert_eq!(
        unsafe { pf_session_pipe_offset(session, 2, &raw mut offset) },
        PF_OK
    );
    assert!(offset.abs() > 1e-6, "the drag has to move the segment");

    // One drag is one undo entry, and it restores the route exactly.
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_undo(session) }, PF_OK);
    // SAFETY: live session and a writable float.
    assert_eq!(
        unsafe { pf_session_pipe_offset(session, 2, &raw mut offset) },
        PF_OK
    );
    assert!(offset.abs() < 1e-6);
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_redo(session) }, PF_OK);
    // SAFETY: live session and a writable float.
    assert_eq!(
        unsafe { pf_session_pipe_offset(session, 2, &raw mut offset) },
        PF_OK
    );
    assert!(offset.abs() > 1e-6);

    // A pipe that does not exist is refused, not answered with junk.
    // SAFETY: live session and a writable float.
    assert_eq!(
        unsafe { pf_session_pipe_offset(session, 99, &raw mut offset) },
        PF_ERR_MODEL
    );
    // SAFETY: live session.
    assert_eq!(
        unsafe { pf_session_route_pipe(session, 99, 10.0, 10.0) },
        PF_ERR_MODEL
    );
    // SAFETY: the session is still alive.
    unsafe { pf_session_free(session) };
}

#[test]
fn dragging_a_component_pulls_its_port_onto_the_other_end() {
    let session = session();
    let valve = component_with(session, "valve.");
    let cylinder = component_with(session, "cylinder.");
    assert!(
        valve < u32::MAX && cylinder < u32::MAX,
        "the example has both"
    );
    let valve_port = port_of(session, valve, "2");
    let cylinder_port = port_of(session, cylinder, "1");
    let bounds = component_box(session, cylinder);
    let centre = (
        f32::midpoint(bounds[0], bounds[2]),
        f32::midpoint(bounds[1], bounds[3]),
    );
    // The port sits at an offset from the centre, so the move is aimed at the valve's axis.
    // The magnet only reaches half a grid step plus one millimetre, so a move far from the
    // axis must leave the hose bent and a move next to it must straighten it exactly. The
    // offset is measured, never assumed: it depends on the box the symbol is drawn in.
    let offset_x = cylinder_port.0 - centre.0;

    // The far move is the control: the grid alone leaves the hose visibly bent.
    // SAFETY: live session.
    assert_eq!(
        unsafe {
            pf_session_move_component(session, cylinder, valve_port.0 - offset_x + 15.0, centre.1)
        },
        PF_OK
    );
    let far_port = port_of(session, cylinder, "1");
    assert!(
        (far_port.0 - valve_port.0).abs() > 5.0,
        "a move away from the axis does not straighten the hose"
    );
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_undo(session) }, PF_OK);
    let target_x = valve_port.0 - offset_x + 1.5;

    // The drag snaps the port onto the other end's axis, off the grid.
    // SAFETY: live session.
    assert_eq!(
        unsafe { pf_session_drag_component(session, cylinder, target_x, centre.1) },
        PF_OK
    );
    let moved = port_of(session, cylinder, "1");
    assert!(
        (moved.0 - valve_port.0).abs() < 0.01,
        "the port is {:.2}, the other end is {:.2}",
        moved.0,
        valve_port.0
    );
    // One undo returns the component to where it was.
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_undo(session) }, PF_OK);
    let restored = component_box(session, cylinder);
    assert!(
        (f32::midpoint(restored[0], restored[2]) - centre.0).abs() < 0.01,
        "the drag is one undo"
    );
    // SAFETY: the session is still alive.
    unsafe { pf_session_free(session) };
}

#[test]
fn a_hose_between_parallel_ports_straightens_and_a_bent_one_is_refused() {
    let session = session();
    let valve = component_with(session, "valve.");
    let cylinder = component_with(session, "cylinder.");
    let valve_port = port_of(session, valve, "2");
    let cylinder_port = port_of(session, cylinder, "1");
    // The hose starts at the valve's top port; asking there finds it whatever bends the
    // route takes around the symbols.
    // SAFETY: live session.
    let pipe = unsafe { pf_session_pipe_at(session, valve_port.0, valve_port.1, 2.0) };
    assert!(pipe >= 0, "the hose is under the point");
    // SAFETY: live session.
    assert_eq!(
        unsafe { pf_session_straighten_pipe(session, u32::try_from(pipe).unwrap_or(0)) },
        PF_OK
    );
    let after = port_of(session, cylinder, "1");
    assert!(
        (after.0 - valve_port.0).abs() < 0.01,
        "the hose is straight: {:.2} against {:.2}",
        after.0,
        valve_port.0
    );
    // One undo takes the component back.
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_undo(session) }, PF_OK);
    let restored = port_of(session, cylinder, "1");
    assert!((restored.0 - cylinder_port.0).abs() < 0.01);

    // A hose between a top port and a side port has a bend by construction; moving one
    // component cannot make it straight, so the command refuses instead of guessing.
    let supply = component_with(session, "supply.");
    let supply_port = port_of(session, supply, "1");
    // SAFETY: live session.
    let bent = unsafe { pf_session_pipe_at(session, supply_port.0, supply_port.1, 2.0) };
    assert!(bent >= 0, "the one-bend hose is under the point");
    // SAFETY: live session.
    assert_eq!(
        unsafe { pf_session_straighten_pipe(session, u32::try_from(bent).unwrap_or(0)) },
        PF_ERR_MODEL
    );
    // SAFETY: the session is still alive.
    unsafe { pf_session_free(session) };
}

#[test]
fn a_pipe_can_be_found_by_its_line_and_deleted() {
    let session = session();
    let exhaust = CString::new("3").unwrap_or_default();
    let working = CString::new("1").unwrap_or_default();
    // SAFETY: live session and two NUL-terminated strings.
    assert_eq!(
        unsafe { pf_session_connect(session, 1, exhaust.as_ptr(), 2, working.as_ptr()) },
        PF_OK
    );
    // The starting circuit already draws two hoses, so the counts below are relative.
    // SAFETY: live session.
    let before = unsafe { pf_session_connection_count(session) };

    // The routed pipe is an elbow, so the middle of its bounding box is never far from it.
    // Walk the whole sheet and accept the first hit: the starting circuit is laid out for
    // the paper, and the default paper is landscape now.
    let mut sheet = PfSheet::default();
    // SAFETY: live session and a writable record.
    assert_eq!(unsafe { pf_session_sheet(session, &raw mut sheet) }, PF_OK);
    let (width_mm, height_mm) = if sheet.landscape == 0 {
        (sheet.width_mm, sheet.height_mm)
    } else {
        (sheet.height_mm, sheet.width_mm)
    };
    let mut found = -1;
    let mut x = 0.0;
    while x <= width_mm && found < 0 {
        let mut y = 0.0;
        while y <= height_mm {
            // SAFETY: live session.
            let hit = unsafe { pf_session_pipe_at(session, x, y, 2.5) };
            if hit >= 0 {
                found = hit;
                break;
            }
            y += 3.0;
        }
        x += 3.0;
    }
    assert!(found >= 0, "the pipe must answer to a point on its line");

    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_select_pipe(session, found) }, PF_OK);
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_selected_pipe(session) }, found);
    // SAFETY: live session.
    assert_eq!(
        unsafe { pf_session_selected(session) },
        -1,
        "one kind at a time"
    );

    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_remove_selected_pipe(session) }, PF_OK);
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_connection_count(session) }, before - 1);
    // SAFETY: live session.
    assert_eq!(
        unsafe { pf_session_undo(session) },
        PF_OK,
        "deleting a pipe is undoable"
    );
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_connection_count(session) }, before);
    // SAFETY: the session is still alive.
    unsafe { pf_session_free(session) };
}

#[test]
fn every_reported_fault_has_a_place_to_point_at() {
    let session = session();
    let mut report = vec![0 as std::os::raw::c_char; 2048];
    let check = |session: *mut PfSession, report: &mut [std::os::raw::c_char]| {
        // SAFETY: a live session and a buffer of the length given.
        unsafe {
            pf_session_check(
                session,
                report.as_mut_ptr(),
                u32::try_from(report.len()).unwrap_or(0),
            )
        }
    };

    // The seeded circuit is drawn with its hoses, so it has nothing to report.
    assert_eq!(
        check(session, &mut report),
        0,
        "the starting circuit is clean"
    );

    // A second pneumatic supply dropped on the sheet hangs in the air: its port has no
    // pipe. It is looked up by id, not by palette position: an electrical supply has no
    // pneumatic port to leave open.
    let supply = catalog_index(session, "supply.compressor");
    // SAFETY: live session.
    assert!(unsafe { pf_session_place_component(session, supply, 30.0, 110.0) } >= 0);
    let faults = check(session, &mut report);
    assert!(faults > 0, "an unconnected port is a fault");
    let faults = u32::try_from(faults).expect("a fault count is not negative");

    // Every fault indexes into the same list the text came from, so the drawing can point
    // at it instead of making the user map a sentence back onto the sheet.
    let mut at = [0.0_f32; 2];
    for index in 0..faults {
        // SAFETY: live session and two writable floats.
        assert_eq!(
            unsafe { pf_session_finding_at(session, index, at.as_mut_ptr()) },
            PF_OK,
            "fault {index} has no place"
        );
    }
    // One past the end is refused rather than answered with nonsense.
    // SAFETY: live session and two writable floats.
    assert_ne!(
        unsafe { pf_session_finding_at(session, faults, at.as_mut_ptr()) },
        PF_OK
    );
    // SAFETY: the session is still alive.
    unsafe { pf_session_free(session) };
}

#[test]
fn the_piston_does_not_leave_the_stop_it_has_reached() {
    let session = session();
    let press = || {
        // SAFETY: live session.
        unsafe { pf_session_press_valve(session) }
    };
    let advance = |dt: f64| {
        // SAFETY: live session.
        unsafe { pf_session_advance(session, dt) }
    };
    let telemetry = || {
        let mut out = PfTelemetry::default();
        // SAFETY: live session and a writable structure.
        let code = unsafe { pf_session_telemetry(session, &raw mut out) };
        assert_eq!(code, PF_OK);
        out
    };

    // The sequence the window performs: a full stroke, a release, then a restart, then the
    // valve held again and stepped. Reproduced exactly, because a restart from a disturbed
    // state is not the same thing as a restart from a fresh one.
    assert_eq!(press(), PF_OK);
    for _ in 0..40 {
        assert_eq!(advance(0.016), PF_OK);
    }
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_release_valve(session) }, PF_OK);
    for _ in 0..80 {
        assert_eq!(advance(0.016), PF_OK);
    }
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_restart(session) }, PF_OK);
    assert_eq!(press(), PF_OK);
    let stroke = telemetry().stroke_m;
    assert!(stroke > 0.0, "the stroke has to be known");

    let mut arrived = false;
    let mut lowest_after_arrival = stroke;
    let mut worst_velocity = 0.0_f64;
    for _ in 0..40 {
        assert_eq!(advance(0.016), PF_OK);
        let state = telemetry();
        if state.position_m >= stroke - 1e-9 {
            arrived = true;
        } else if arrived {
            lowest_after_arrival = lowest_after_arrival.min(state.position_m);
            worst_velocity = worst_velocity.max(state.velocity_m_s.abs());
        }
    }
    assert!(arrived, "the piston never reached the stop");
    assert!(
        stroke - lowest_after_arrival < 1e-9,
        "the piston left the stop it had reached: back to {:.2} mm of {:.2} mm, at {:.1} mm/s",
        lowest_after_arrival * 1000.0,
        stroke * 1000.0,
        worst_velocity * 1000.0
    );
    // SAFETY: the session is still alive.
    unsafe { pf_session_free(session) };
}

#[test]
fn releasing_the_valve_vents_the_chamber_and_the_spring_brings_the_rod_back() {
    let session = session();
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_press_valve(session) }, PF_OK);
    for _ in 0..90 {
        assert_eq!(unsafe { pf_session_advance(session, 0.016) }, PF_OK);
    }
    let mut extended = PfTelemetry::default();
    // SAFETY: live session, writable output.
    assert_eq!(
        unsafe { pf_session_telemetry(session, &raw mut extended) },
        PF_OK
    );
    assert!(extended.position_m > 0.09, "the rod is out");

    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_release_valve(session) }, PF_OK);
    for _ in 0..200 {
        assert_eq!(unsafe { pf_session_advance(session, 0.016) }, PF_OK);
    }
    let mut retracted = PfTelemetry::default();
    // SAFETY: live session, writable output.
    assert_eq!(
        unsafe { pf_session_telemetry(session, &raw mut retracted) },
        PF_OK
    );
    assert_eq!(retracted.valve_position, 0);
    assert!(
        retracted.position_m < 0.01,
        "the spring returned the rod to {:.1} mm of a {:.1} mm stroke",
        retracted.position_m * 1000.0,
        retracted.stroke_m * 1000.0
    );
    // SAFETY: the session is still alive.
    unsafe { pf_session_free(session) };
}

#[test]
fn a_turned_component_puts_its_ports_where_the_symbol_is_drawn() {
    let session = session();
    // SAFETY: live session; the starting circuit has a compressor, a service unit, a valve
    // and a cylinder.
    assert_eq!(unsafe { pf_session_component_count(session) }, 4);
    // The valve waits for a port to be dragged to: it is the one a turn is visible on.
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_select(session, 1) }, PF_OK);
    let point = |session: *mut PfSession| {
        let mut out = [0.0_f32; 2];
        // SAFETY: live session, a NUL-terminated port id and two writable floats.
        let status = unsafe { pf_session_port_point(session, 1, c"2".as_ptr(), out.as_mut_ptr()) };
        assert_eq!(status, PF_OK, "the valve has port 2");
        (out[0], out[1])
    };
    let before = point(session);
    let mut boxed = [0.0_f32; 4];
    // SAFETY: live session and four writable floats.
    assert_eq!(
        unsafe { pf_session_component_rect(session, 1, boxed.as_mut_ptr()) },
        PF_OK
    );
    let centre = (
        f32::midpoint(boxed[0], boxed[2]),
        f32::midpoint(boxed[1], boxed[3]),
    );
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_rotate_selected(session, 0) }, PF_OK);
    let mut orientation = [0_u32; 2];
    // SAFETY: live session and two writable integers.
    assert_eq!(
        unsafe { pf_session_component_orientation(session, 1, orientation.as_mut_ptr()) },
        PF_OK
    );
    assert_eq!(orientation, [1, 0], "one clockwise quarter, not mirrored");
    let after = point(session);
    assert!(
        (before.0 - after.0).abs() > 0.5 || (before.1 - after.1).abs() > 0.5,
        "a quarter turn moves the port: {before:?} -> {after:?}"
    );
    // The symbol turns about its own centre, so the box keeps the same middle.
    let mut turned_box = [0.0_f32; 4];
    // SAFETY: live session and four writable floats.
    assert_eq!(
        unsafe { pf_session_component_rect(session, 1, turned_box.as_mut_ptr()) },
        PF_OK
    );
    let turned_centre = (
        f32::midpoint(turned_box[0], turned_box[2]),
        f32::midpoint(turned_box[1], turned_box[3]),
    );
    assert!(
        (turned_centre.0 - centre.0).abs() < 1e-3 && (turned_centre.1 - centre.1).abs() < 1e-3,
        "{turned_centre:?} is not {centre:?}"
    );

    // One undo puts the port back where it was drawn.
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_undo(session) }, PF_OK);
    let undone = point(session);
    assert!(
        (undone.0 - before.0).abs() < 1e-3 && (undone.1 - before.1).abs() < 1e-3,
        "{undone:?} is not {before:?}"
    );

    // A mirror is the other command, and it survives a save and an open.
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_mirror_selected(session) }, PF_OK);
    let directory = tempfile::tempdir().expect("a temporary folder");
    let path = directory.path().join("girado.pfluid");
    let encoded = CString::new(path.to_string_lossy().as_bytes()).unwrap_or_default();
    // SAFETY: live session and a NUL-terminated path.
    assert_eq!(unsafe { pf_session_save(session, encoded.as_ptr()) }, PF_OK);
    // SAFETY: the session is still alive.
    unsafe { pf_session_free(session) };

    let session = self::session();
    // SAFETY: live session and a NUL-terminated path.
    assert_eq!(unsafe { pf_session_open(session, encoded.as_ptr()) }, PF_OK);
    let mut reopened = [0_u32; 2];
    // SAFETY: live session and two writable integers.
    assert_eq!(
        unsafe { pf_session_component_orientation(session, 1, reopened.as_mut_ptr()) },
        PF_OK
    );
    assert_eq!(reopened, [0, 1], "the mirror is part of the sheet");
    // SAFETY: the session is still alive.
    unsafe { pf_session_free(session) };
}

#[test]
fn a_selection_of_several_components_moves_copies_and_goes_away_as_one_change() {
    let session = session();
    // SAFETY: live session; the starting circuit has four components.
    assert_eq!(unsafe { pf_session_component_count(session) }, 4);
    // Ctrl+E marks everything, and the last one marked is the primary the forms open on.
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_select_all(session) }, 4);
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_selection_count(session) }, 4);
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_selected(session) }, 3);
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_selection_at(session, 0) }, 0);

    let rect_of = |session: *mut PfSession, index: u32| {
        let mut out = [0.0_f32; 4];
        // SAFETY: live session and four writable floats.
        assert_eq!(
            unsafe { pf_session_component_rect(session, index, out.as_mut_ptr()) },
            PF_OK
        );
        out
    };
    let before = rect_of(session, 0);
    // The whole selection moves by one step: the group keeps its shape.
    // SAFETY: live session.
    assert_eq!(
        unsafe { pf_session_move_selection(session, 10.0, 0.0) },
        PF_OK
    );
    let moved = rect_of(session, 0);
    assert!(
        (moved[0] - before[0] - 10.0).abs() < 1e-3,
        "{before:?} -> {moved:?}"
    );
    // One undo puts all three back, because the drag of a group is one change.
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_undo(session) }, PF_OK);
    let back = rect_of(session, 0);
    assert!(
        (back[0] - before[0]).abs() < 1e-3,
        "{back:?} is not {before:?}"
    );

    // Copying the selection copies all of it, once, and selects the copies.
    // SAFETY: live session.
    assert_eq!(
        unsafe { pf_session_duplicate_selection(session, 20.0, 20.0) },
        4
    );
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_component_count(session) }, 8);
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_selection_count(session) }, 4);
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_undo(session) }, PF_OK);
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_component_count(session) }, 4);

    // A rubber band marks what it covers; one that covers nothing marks nothing.
    let band = rect_of(session, 2);
    // SAFETY: live session.
    assert_eq!(
        unsafe {
            pf_session_select_rect(
                session,
                band[0] - 1.0,
                band[1] - 1.0,
                band[2] + 1.0,
                band[3] + 1.0,
            )
        },
        1
    );
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_selection_at(session, 0) }, 2);
    // Mayús and a click takes it out of the selection and puts it back.
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_toggle_select(session, 2) }, PF_OK);
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_selection_count(session) }, 0);
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_toggle_select(session, 1) }, PF_OK);
    assert_eq!(unsafe { pf_session_selection_count(session) }, 1);

    // Deleting the selection is one change, and it takes the pipes with it.
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_remove_selection(session) }, 1);
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_component_count(session) }, 3);
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_undo(session) }, PF_OK);
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_component_count(session) }, 4);
    // SAFETY: the session is still alive.
    unsafe { pf_session_free(session) };
}

#[test]
fn lining_up_the_selection_moves_it_onto_the_component_furthest_out() {
    let session = session();
    // Two components of different sizes, so an edge is not the centre.
    // SAFETY: live session.
    assert_eq!(
        unsafe { pf_session_place_component(session, 1, 40.0, 100.0) },
        4
    );
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_select_all(session) }, 5);
    let lefts = |session: *mut PfSession| {
        let mut out = Vec::new();
        for index in 0..5_u32 {
            let mut bounds = [0.0_f32; 4];
            // SAFETY: live session and four writable floats.
            assert_eq!(
                unsafe { pf_session_component_rect(session, index, bounds.as_mut_ptr()) },
                PF_OK
            );
            out.push(bounds[0]);
        }
        out
    };
    let before = lefts(session);
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_align_selection(session, 0) }, PF_OK);
    let after = lefts(session);
    let leftmost = before.iter().copied().fold(f32::MAX, f32::min);
    assert!(
        after.iter().all(|left| (left - leftmost).abs() < 1e-3),
        "{before:?} -> {after:?}"
    );
    // One undo puts every component back where it was.
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_undo(session) }, PF_OK);
    let restored = lefts(session);
    for (was, now) in before.iter().zip(&restored) {
        assert!((was - now).abs() < 1e-3, "{before:?} is not {restored:?}");
    }
    // An edge that does not exist is refused instead of guessed.
    // SAFETY: live session.
    assert_eq!(
        unsafe { pf_session_align_selection(session, 9) },
        PF_ERR_MODEL
    );
    // SAFETY: the session is still alive.
    unsafe { pf_session_free(session) };
}

#[test]
fn unsaved_work_is_known_and_survives_a_crash_in_a_safety_copy() {
    let directory = tempfile::tempdir().expect("a temporary folder");
    let path = directory.path().join("circuito.pfluid");
    let encoded = CString::new(path.to_string_lossy().as_bytes()).unwrap_or_default();
    let safety = directory.path().join("autosave.pfluid");
    let safety_encoded = CString::new(safety.to_string_lossy().as_bytes()).unwrap_or_default();

    let session = session();
    // The worked example the window opens with is not unsaved work: nothing was typed.
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_is_modified(session) }, 0);
    // Marking a component is not an edit either, so the title has nothing to mark.
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_select(session, 1) }, PF_OK);
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_is_modified(session) }, 0);

    // A real edit is unsaved work, and an undo does not pretend otherwise: the circuit was
    // changed, even if it was changed back.
    // SAFETY: live session.
    assert_eq!(
        unsafe { pf_session_move_component(session, 1, 70.0, 90.0) },
        PF_OK
    );
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_is_modified(session) }, 1);
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_undo(session) }, PF_OK);
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_is_modified(session) }, 1);

    // Saving clears it: the file is the document now.
    // SAFETY: live session and a NUL-terminated path.
    assert_eq!(unsafe { pf_session_save(session, encoded.as_ptr()) }, PF_OK);
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_is_modified(session) }, 0);

    // A safety copy is a copy: it exists, it holds the work, and the document is still
    // unsaved because the user never chose that path.
    // SAFETY: live session.
    assert_eq!(
        unsafe { pf_session_move_component(session, 1, 80.0, 90.0) },
        PF_OK
    );
    // SAFETY: live session and a NUL-terminated path.
    assert_eq!(
        unsafe { pf_session_autosave(session, safety_encoded.as_ptr()) },
        PF_OK
    );
    assert!(safety.exists(), "the copy is on disk");
    // SAFETY: live session.
    assert_eq!(
        unsafe { pf_session_is_modified(session) },
        1,
        "an autosave is not a save"
    );

    // Recovering the copy brings the work back and leaves it unsaved, so the next save asks
    // where it goes instead of overwriting the safety copy.
    // SAFETY: the session is still alive.
    unsafe { pf_session_free(session) };
    let session = self::session();
    // SAFETY: live session and a NUL-terminated path.
    assert_eq!(
        unsafe { pf_session_recover(session, safety_encoded.as_ptr()) },
        PF_OK
    );
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_is_modified(session) }, 1);
    let mut bounds = [0.0_f32; 4];
    // SAFETY: live session and four writable floats.
    assert_eq!(
        unsafe { pf_session_component_rect(session, 1, bounds.as_mut_ptr()) },
        PF_OK
    );
    assert!(
        (f32::midpoint(bounds[0], bounds[2]) - 80.0).abs() < 1e-3,
        "the recovered copy has the moved valve: {bounds:?}"
    );
    // A file that is not a sheet is refused and the open document survives.
    let broken = directory.path().join("roto.pfluid");
    std::fs::write(&broken, "not a sheet").expect("writes");
    let broken = CString::new(broken.to_string_lossy().as_bytes()).unwrap_or_default();
    // SAFETY: live session and a NUL-terminated path.
    assert_eq!(
        unsafe { pf_session_recover(session, broken.as_ptr()) },
        PF_ERR_MODEL
    );
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_component_count(session) }, 4);
    // SAFETY: the session is still alive.
    unsafe { pf_session_free(session) };
}

#[test]
fn the_documents_that_were_saved_or_opened_are_remembered_and_can_be_reopened() {
    let directory = tempfile::tempdir().expect("a temporary folder");
    let list = CString::new(
        directory
            .path()
            .join("recent.json")
            .to_string_lossy()
            .as_bytes(),
    )
    .unwrap_or_default();
    let first = directory.path().join("uno.pfluid");
    let second = directory.path().join("dos.pfluid");
    let first_encoded = CString::new(first.to_string_lossy().as_bytes()).unwrap_or_default();
    let second_encoded = CString::new(second.to_string_lossy().as_bytes()).unwrap_or_default();

    let session = session();
    // SAFETY: live session and a NUL-terminated path.
    assert_eq!(
        unsafe { pf_session_set_recent_file(session, list.as_ptr()) },
        PF_OK
    );
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_recent_count(session) }, 0);

    // Saving and opening both remember the document, newest first.
    // SAFETY: live session and NUL-terminated paths.
    assert_eq!(
        unsafe { pf_session_save(session, first_encoded.as_ptr()) },
        PF_OK
    );
    // SAFETY: live session and NUL-terminated paths.
    assert_eq!(
        unsafe { pf_session_save(session, second_encoded.as_ptr()) },
        PF_OK
    );
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_recent_count(session) }, 2);
    let path_of = |session: *mut PfSession, slot: u32| {
        let mut buffer = vec![0_u8; 4096];
        // SAFETY: live session and a writable buffer of the declared size.
        let written = unsafe {
            pf_session_recent_path(
                session,
                slot,
                buffer.as_mut_ptr().cast::<std::ffi::c_char>(),
                u32::try_from(buffer.len()).unwrap_or(0),
            )
        };
        assert!(written > 0, "slot {slot} has a path");
        let text = &buffer[..usize::try_from(written).unwrap_or(0)];
        String::from_utf8_lossy(text).into_owned()
    };
    assert!(
        path_of(session, 0).ends_with("dos.pfluid"),
        "the newest is first"
    );
    assert!(path_of(session, 1).ends_with("uno.pfluid"));

    // A document that vanished is reported and kept: the start view greys it until the user
    // removes it, so the row never disappears behind the reader's back.
    std::fs::remove_file(&second).expect("removes");
    // The document on the sheet is not modified by this, so opening the first is allowed.
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_open_recent(session, 0) }, PF_ERR_MODEL);
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_recent_count(session) }, 2);
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_recent_available(session, 0) }, 0);
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_recent_available(session, 1) }, 1);
    // The other one still opens, and it does not take the unavailable row with it.
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_open_recent(session, 1) }, PF_OK);
    // The list survives a session: a new one starts with what was written.
    // SAFETY: the session is still alive.
    unsafe { pf_session_free(session) };
    let session = self::session();
    // SAFETY: live session and a NUL-terminated path.
    assert_eq!(
        unsafe { pf_session_set_recent_file(session, list.as_ptr()) },
        PF_OK
    );
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_recent_count(session) }, 2);
    assert!(path_of(session, 0).ends_with("uno.pfluid"));
    assert!(path_of(session, 1).ends_with("dos.pfluid"));
    // Forgetting empties the list, in memory and on disk.
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_forget_recent(session) }, PF_OK);
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_recent_count(session) }, 0);
    // SAFETY: the session is still alive.
    unsafe { pf_session_free(session) };
    let session = self::session();
    // SAFETY: live session and a NUL-terminated path.
    assert_eq!(
        unsafe { pf_session_set_recent_file(session, list.as_ptr()) },
        PF_OK
    );
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_recent_count(session) }, 0);
    // SAFETY: the session is still alive.
    unsafe { pf_session_free(session) };
}

#[test]
fn a_missing_document_stays_in_the_list_until_it_is_removed() {
    let directory = tempfile::tempdir().expect("a temporary folder");
    let list_path = directory.path().join("recent.json");
    let list = CString::new(list_path.to_string_lossy().as_bytes()).unwrap_or_default();
    let present = directory.path().join("presente.pfluid");
    let gone = directory.path().join("perdido.pfluid");
    let present_encoded = CString::new(present.to_string_lossy().as_bytes()).unwrap_or_default();

    let session = session();
    // SAFETY: live session and a NUL-terminated path.
    assert_eq!(
        unsafe { pf_session_set_recent_file(session, list.as_ptr()) },
        PF_OK
    );
    // SAFETY: live session and a NUL-terminated path.
    assert_eq!(
        unsafe { pf_session_save(session, present_encoded.as_ptr()) },
        PF_OK
    );
    // A path that never existed reaches the list the way a document that moved does: it is
    // remembered, it is reported as unavailable, and it waits for the user.
    std::fs::write(
        &list_path,
        format!(
            r#"{{"version":1,"path":["{}","{}"]}}"#,
            gone.to_string_lossy().replace('\\', "\\\\"),
            present.to_string_lossy().replace('\\', "\\\\")
        ),
    )
    .expect("writes the list");
    // SAFETY: the session is still alive.
    unsafe { pf_session_free(session) };
    let session = self::session();
    // SAFETY: live session and a NUL-terminated path.
    assert_eq!(
        unsafe { pf_session_set_recent_file(session, list.as_ptr()) },
        PF_OK
    );
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_recent_count(session) }, 2);
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_recent_available(session, 0) }, 0);
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_recent_available(session, 1) }, 1);
    // An unknown slot is refused instead of guessed, and the list is untouched.
    // SAFETY: live session.
    assert_eq!(
        unsafe { pf_session_forget_recent_at(session, 99) },
        PF_ERR_MODEL
    );
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_recent_count(session) }, 2);
    // Removing the missing row leaves the available one where it was.
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_forget_recent_at(session, 0) }, PF_OK);
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_recent_count(session) }, 1);
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_recent_available(session, 0) }, 1);
    // The removal reached the file: a new session reads one entry.
    // SAFETY: the session is still alive.
    unsafe { pf_session_free(session) };
    let session = self::session();
    // SAFETY: live session and a NUL-terminated path.
    assert_eq!(
        unsafe { pf_session_set_recent_file(session, list.as_ptr()) },
        PF_OK
    );
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_recent_count(session) }, 1);
    // SAFETY: the session is still alive.
    unsafe { pf_session_free(session) };
}

#[test]
fn a_component_can_be_named_and_the_name_survives_a_save_and_an_undo() {
    let directory = tempfile::tempdir().expect("a temporary folder");
    let path = directory.path().join("nombres.pfluid");
    let encoded = CString::new(path.to_string_lossy().as_bytes()).unwrap_or_default();

    let session = session();
    let name_of = |session: *mut PfSession, index: u32| {
        let mut buffer = [0_u8; 128];
        // SAFETY: live session and a writable buffer of the declared size.
        let written = unsafe {
            pf_session_component_designation(
                session,
                index,
                buffer.as_mut_ptr().cast::<std::ffi::c_char>(),
                u32::try_from(buffer.len()).unwrap_or(0),
            )
        };
        assert!(written >= 0, "the component exists");
        String::from_utf8_lossy(&buffer[..usize::try_from(written).unwrap_or(0)]).into_owned()
    };
    // The worked example starts with three unnamed components.
    // SAFETY: live session.
    assert_eq!(name_of(session, 2), "");
    let text = CString::new(" 1A1 ").unwrap_or_default();
    // SAFETY: live session and a NUL-terminated string.
    assert_eq!(
        unsafe { pf_session_set_component_designation(session, 2, text.as_ptr()) },
        PF_OK
    );
    // SAFETY: live session.
    assert_eq!(name_of(session, 2), "1A1", "trimmed, as the sheet draws it");
    // Naming it again with the same name is not an error and not a change.
    // SAFETY: live session and a NUL-terminated string.
    assert_eq!(
        unsafe { pf_session_set_component_designation(session, 2, text.as_ptr()) },
        PF_OK
    );
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_is_modified(session) }, 1);

    // The name is part of the document: it is saved, reopened and undone like anything
    // else in it.
    // SAFETY: live session and a NUL-terminated path.
    assert_eq!(unsafe { pf_session_save(session, encoded.as_ptr()) }, PF_OK);
    // SAFETY: the session is still alive.
    unsafe { pf_session_free(session) };
    let session = self::session();
    // SAFETY: live session and a NUL-terminated path.
    assert_eq!(unsafe { pf_session_open(session, encoded.as_ptr()) }, PF_OK);
    // SAFETY: live session.
    assert_eq!(name_of(session, 2), "1A1");

    let empty = CString::new("").unwrap_or_default();
    // SAFETY: live session and a NUL-terminated string.
    assert_eq!(
        unsafe { pf_session_set_component_designation(session, 2, empty.as_ptr()) },
        PF_OK
    );
    // SAFETY: live session.
    assert_eq!(name_of(session, 2), "", "an empty text is no name");
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_undo(session) }, PF_OK);
    // SAFETY: live session.
    assert_eq!(name_of(session, 2), "1A1", "one undo brings the name back");
    // A name nothing could draw is refused instead of stored.
    let long = CString::new("x".repeat(41)).unwrap_or_default();
    // SAFETY: live session and a NUL-terminated string.
    assert_eq!(
        unsafe { pf_session_set_component_designation(session, 2, long.as_ptr()) },
        PF_ERR_MODEL
    );
    // SAFETY: live session.
    assert_eq!(name_of(session, 2), "1A1");
    // SAFETY: the session is still alive.
    unsafe { pf_session_free(session) };
}

/// `true` when the PDF carries `needle` as a hexadecimal string, which is how the writer
/// encodes text.
fn pdf_shows(pdf: &[u8], needle: &str) -> bool {
    let hex: String = needle.bytes().fold(String::new(), |mut out, byte| {
        let _ = std::fmt::Write::write_fmt(&mut out, format_args!("{byte:02X}"));
        out
    });
    String::from_utf8_lossy(pdf).contains(&hex)
}

#[test]
fn an_export_carries_the_caller_date_and_fits_a_large_circuit() {
    let session = session();
    let cylinder = catalog_index(session, "cylinder.single_acting_20_100");
    // Far from the seeded circuit, so the drawing is larger than the page and must shrink.
    // SAFETY: live session, a catalog index the call above found, and finite coordinates.
    let placed = unsafe { pf_session_place_component(session, cylinder, 600.0, 400.0) };
    assert!(placed >= 0, "the far component is placed");

    let directory = tempfile::tempdir().expect("a temporary folder");
    let path = directory.path().join("plano.pdf");
    let path = CString::new(path.to_str().unwrap_or_default()).expect("a path");
    let date = CString::new("2026-11-12").expect("a date");
    // SAFETY: live session, NUL-terminated path and date, A4 is index 1.
    let code = unsafe {
        pf_session_export(
            session,
            path.as_ptr(),
            PF_EXPORT_PDF,
            1,
            0,
            300.0,
            date.as_ptr(),
        )
    };
    assert_eq!(
        code,
        PF_OK,
        "{}",
        // SAFETY: live session.
        unsafe { CStr::from_ptr(pf_session_last_error(session)) }.to_string_lossy()
    );
    let pdf = std::fs::read(path.to_str().unwrap_or_default()).expect("the PDF is written");
    assert!(pdf_shows(&pdf, "2026-11-12"), "the date reaches the page");
    assert!(
        !pdf_shows(&pdf, "1:"),
        "a schematic is not to scale and the block must not claim one"
    );
    assert!(pdf_shows(&pdf, "Circuito 1"), "the circuit names the page");
    assert!(
        pdf_shows(&pdf, "parts list"),
        "the sheet carries its parts list"
    );

    // SAFETY: the session is still alive.
    unsafe { pf_session_free(session) };
}

#[test]
fn circuits_can_be_removed_undone_and_the_last_one_is_protected() {
    let session = session();
    let first =
        u32::try_from(unsafe { pf_session_active_circuit(session) }).expect("first circuit id");
    let first_components = unsafe { pf_session_component_count(session) };
    let title = CString::new("Mando").expect("a title");
    let second = u32::try_from(unsafe { pf_session_add_circuit(session, title.as_ptr()) })
        .expect("second circuit id");
    assert!(second > 0);
    assert_eq!(unsafe { pf_session_circuit_count(session) }, 2);
    assert_eq!(
        unsafe { pf_session_active_circuit(session) },
        i32::try_from(second).expect("id fits")
    );

    assert_eq!(unsafe { pf_session_remove_circuit(session, second) }, PF_OK);
    assert_eq!(unsafe { pf_session_circuit_count(session) }, 1);
    assert_eq!(
        unsafe { pf_session_active_circuit(session) },
        i32::try_from(first).expect("id fits")
    );
    assert_eq!(unsafe { pf_session_undo(session) }, PF_OK);
    assert_eq!(unsafe { pf_session_circuit_count(session) }, 2);
    assert_eq!(
        unsafe { pf_session_circuit_id(session, 1) },
        i32::try_from(second).expect("id fits")
    );
    assert_eq!(
        unsafe { pf_session_active_circuit(session) },
        i32::try_from(second).expect("id fits")
    );

    // Removing a circuit with live contents uses the same ABI path; undo restores both.
    assert!(unsafe { pf_session_place_component(session, 0, 40.0, 40.0) } >= 0);
    let before = unsafe { pf_session_component_count(session) };
    assert_eq!(unsafe { pf_session_remove_circuit(session, second) }, PF_OK);
    assert_eq!(
        unsafe { pf_session_component_count(session) },
        first_components
    );
    assert_eq!(unsafe { pf_session_undo(session) }, PF_OK);
    assert_eq!(unsafe { pf_session_component_count(session) }, before);
    assert!(
        before > first_components,
        "undo restored the added component"
    );
    assert_eq!(
        unsafe { pf_session_active_circuit(session) },
        i32::try_from(second).expect("id fits")
    );

    assert_eq!(unsafe { pf_session_remove_circuit(session, second) }, PF_OK);
    assert_eq!(
        unsafe { pf_session_remove_circuit(session, first) },
        PF_ERR_MODEL
    );
    assert_eq!(unsafe { pf_session_circuit_count(session) }, 1);
    unsafe { pf_session_free(session) };
}

#[test]
fn a_placement_reports_the_circuit_it_belongs_to() {
    let session = session();
    let first = unsafe { pf_session_active_circuit(session) };
    let count = unsafe { pf_session_component_count(session) };
    assert!(count > 0, "the worked example fills the first circuit");
    for index in 0..count {
        assert_eq!(
            unsafe { pf_session_component_circuit(session, index) },
            first,
            "placement {index} belongs to the first circuit"
        );
    }

    let title = CString::new("Circuito 2").expect("a title");
    // SAFETY: live session and a NUL-terminated title.
    let second = unsafe { pf_session_add_circuit(session, title.as_ptr()) };
    assert!(second > 0, "a second circuit gets its own identity");
    // SAFETY: live session and a valid catalog slot.
    let placed = unsafe { pf_session_place_component(session, 0, 20.0, 20.0) };
    assert!(placed >= 0, "the second circuit takes a placement");
    let placed = u32::try_from(placed).expect("a placement index");
    assert_eq!(
        unsafe { pf_session_component_circuit(session, placed) },
        second
    );
    for index in 0..count {
        assert_eq!(
            unsafe { pf_session_component_circuit(session, index) },
            first,
            "the first circuit keeps its own placements"
        );
    }
    assert_eq!(
        unsafe { pf_session_component_circuit(session, placed + 1) },
        PF_ERR_MODEL
    );
    // SAFETY: the session is still alive.
    unsafe { pf_session_free(session) };
}

#[test]
fn circuit_limit_is_enforced_at_the_abi() {
    let session = session();
    let title = CString::new("Circuito").expect("a title");
    for _ in 1..punchi_fluid::PF_MAX_CIRCUITS {
        assert!(unsafe { pf_session_add_circuit(session, title.as_ptr()) } >= 0);
    }
    assert_eq!(
        unsafe { pf_session_circuit_count(session) } as usize,
        punchi_fluid::PF_MAX_CIRCUITS
    );
    assert_eq!(
        unsafe { pf_session_add_circuit(session, title.as_ptr()) },
        PF_ERR_LIMIT
    );
    assert_eq!(
        unsafe { pf_session_circuit_count(session) } as usize,
        punchi_fluid::PF_MAX_CIRCUITS
    );
    unsafe { pf_session_free(session) };
}

#[test]
fn circuits_can_be_added_renamed_and_survive_a_save_and_open() {
    let original = session();
    assert_eq!(unsafe { pf_session_circuit_count(original) }, 1);
    let first = unsafe { pf_session_active_circuit(original) };
    assert_eq!(first, 0);
    assert_eq!(unsafe { pf_session_circuit_id(original, 0) }, 0);

    let title = CString::new("Control").expect("a title");
    // SAFETY: live session and a NUL-terminated title.
    let second = unsafe { pf_session_add_circuit(original, title.as_ptr()) };
    assert!(second > 0, "a new circuit gets a new identity");
    assert_eq!(unsafe { pf_session_circuit_count(original) }, 2);
    assert_eq!(unsafe { pf_session_active_circuit(original) }, second);

    let mut name = [0 as std::os::raw::c_char; 128];
    // SAFETY: live session, a writable buffer and its capacity.
    assert!(
        unsafe {
            pf_session_circuit_name(
                original,
                1,
                name.as_mut_ptr(),
                u32::try_from(name.len()).expect("the buffer capacity"),
            )
        } >= 0
    );
    // SAFETY: the call above wrote a NUL-terminated string.
    assert_eq!(
        unsafe { CStr::from_ptr(name.as_ptr()) }.to_str(),
        Ok("Control")
    );

    let renamed = CString::new("Power").expect("a title");
    // SAFETY: live session, the identity just returned and a NUL-terminated title.
    assert_eq!(
        unsafe {
            pf_session_rename_circuit(
                original,
                u32::try_from(second).expect("a circuit identity"),
                renamed.as_ptr(),
            )
        },
        PF_OK
    );

    let directory = tempfile::tempdir().expect("a temporary folder");
    let path = directory.path().join("proyecto.pfluid");
    let path = CString::new(path.to_str().unwrap_or_default()).expect("a path");
    // SAFETY: live session and a NUL-terminated path.
    assert_eq!(unsafe { pf_session_save(original, path.as_ptr()) }, PF_OK);

    let reopened = session();
    // SAFETY: live session and a NUL-terminated path.
    assert_eq!(unsafe { pf_session_open(reopened, path.as_ptr()) }, PF_OK);
    assert_eq!(unsafe { pf_session_circuit_count(reopened) }, 2);
    assert_eq!(unsafe { pf_session_active_circuit(reopened) }, second);
    let mut name = [0 as std::os::raw::c_char; 128];
    // SAFETY: live session, a writable buffer and its capacity.
    assert!(
        unsafe {
            pf_session_circuit_name(
                reopened,
                1,
                name.as_mut_ptr(),
                u32::try_from(name.len()).expect("the buffer capacity"),
            )
        } >= 0
    );
    // SAFETY: the call above wrote a NUL-terminated string.
    assert_eq!(
        unsafe { CStr::from_ptr(name.as_ptr()) }.to_str(),
        Ok("Power")
    );

    // SAFETY: live session and an identity from the file.
    assert_eq!(
        unsafe {
            pf_session_select_circuit(reopened, u32::try_from(first).expect("a circuit identity"))
        },
        PF_OK
    );
    assert_eq!(unsafe { pf_session_active_circuit(reopened) }, first);

    // SAFETY: the sessions are still alive.
    unsafe {
        pf_session_free(reopened);
        pf_session_free(original);
    }
}

/// Reads one field of a circuit's title block, for the block tests.
fn block_field(session: *const PfSession, id: u32, field: u32) -> String {
    let mut buffer = [0 as c_char; 160];
    // SAFETY: the caller passes a live session and the buffer is writable.
    let length = unsafe {
        pf_session_block(
            session,
            id,
            field,
            buffer.as_mut_ptr(),
            u32::try_from(buffer.len()).unwrap_or(0),
        )
    };
    assert!(length >= 0, "reading a field must succeed");
    // SAFETY: the call wrote a NUL-terminated string.
    let text = unsafe { CStr::from_ptr(buffer.as_ptr()) }.to_str();
    assert!(text.is_ok(), "the field is UTF-8");
    text.unwrap_or_default().to_owned()
}

/// The twelve field indices the header names.
const BLOCK_FIELDS: [u32; 12] = [
    PF_BLOCK_OWNER,
    PF_BLOCK_PROJECT,
    PF_BLOCK_EQUIPMENT,
    PF_BLOCK_TITLE,
    PF_BLOCK_NUMBER,
    PF_BLOCK_REVISION,
    PF_BLOCK_DATE,
    PF_BLOCK_PREPARED,
    PF_BLOCK_CHECKED,
    PF_BLOCK_APPROVED,
    PF_BLOCK_SCALE,
    PF_BLOCK_NORMS,
];

#[test]
fn the_title_block_defaults_are_the_guide_ones_and_one_edit_is_one_undo() {
    let original = session();
    let id = u32::try_from(unsafe { pf_session_active_circuit(original) })
        .expect("the identity of the active circuit");

    // A new circuit starts with its own title, revision A and the guide's S/E scale; the
    // project, equipment and norms start empty.
    assert_eq!(block_field(original, id, PF_BLOCK_TITLE), "Circuito 1");
    assert_eq!(block_field(original, id, PF_BLOCK_REVISION), "A");
    assert_eq!(block_field(original, id, PF_BLOCK_SCALE), "S/E");
    assert!(block_field(original, id, PF_BLOCK_OWNER).is_empty());
    assert!(block_field(original, id, PF_BLOCK_PROJECT).is_empty());
    assert!(block_field(original, id, PF_BLOCK_EQUIPMENT).is_empty());
    assert!(block_field(original, id, PF_BLOCK_NORMS).is_empty());

    let owner = CString::new("Curso de Neumática").expect("a name");
    // SAFETY: live session, the circuit identity and a NUL-terminated value.
    assert_eq!(
        unsafe { pf_session_set_block(original, id, PF_BLOCK_OWNER, owner.as_ptr()) },
        PF_OK
    );
    let number = CString::new("PF-2026-07").expect("a number");
    // SAFETY: the same contract.
    assert_eq!(
        unsafe { pf_session_set_block(original, id, PF_BLOCK_NUMBER, number.as_ptr()) },
        PF_OK
    );
    assert_eq!(
        block_field(original, id, PF_BLOCK_OWNER),
        "Curso de Neumática"
    );
    assert_eq!(block_field(original, id, PF_BLOCK_NUMBER), "PF-2026-07");

    // One edit was one entry in the history: undo takes the last field back.
    // SAFETY: live session with a change to undo.
    assert_eq!(unsafe { pf_session_undo(original) }, PF_OK);
    assert!(block_field(original, id, PF_BLOCK_NUMBER).is_empty());
    assert_eq!(
        block_field(original, id, PF_BLOCK_OWNER),
        "Curso de Neumática"
    );

    // The scale is editable: the default is S/E, and a company may write a ratio.
    let scale = CString::new("1:50").expect("a scale");
    // SAFETY: live session, the circuit identity and a NUL-terminated value.
    assert_eq!(
        unsafe { pf_session_set_block(original, id, PF_BLOCK_SCALE, scale.as_ptr()) },
        PF_OK
    );
    assert_eq!(block_field(original, id, PF_BLOCK_SCALE), "1:50");

    // A field that does not exist is refused instead of reading past the list.
    let mut buffer = [0 as c_char; 8];
    // SAFETY: live session, a writable buffer and an out-of-range field index.
    assert_eq!(
        unsafe { pf_session_block(original, id, 12, buffer.as_mut_ptr(), 8) },
        PF_ERR_MODEL
    );
    // Every field index the header names is readable, so the front end can paint the block.
    for field in BLOCK_FIELDS {
        assert!(block_field(original, id, field).len() < 160);
    }

    // SAFETY: the session is still alive.
    unsafe { pf_session_free(original) };
}

#[test]
fn the_title_block_survives_a_save_and_an_open() {
    let original = session();
    let id = u32::try_from(unsafe { pf_session_active_circuit(original) })
        .expect("the identity of the active circuit");
    for (field, value) in [
        (PF_BLOCK_OWNER, "Curso de Neumática"),
        (PF_BLOCK_PROJECT, "Línea 2"),
        (PF_BLOCK_EQUIPMENT, "Prensa 4"),
        (PF_BLOCK_NUMBER, "PF-2026-07"),
        (PF_BLOCK_NORMS, "NCh 14 · NCh 1768/1"),
        (PF_BLOCK_SCALE, "1:50"),
    ] {
        let value = CString::new(value).expect("a NUL-free value");
        // SAFETY: live session, the circuit identity and a NUL-terminated value.
        assert_eq!(
            unsafe { pf_session_set_block(original, id, field, value.as_ptr()) },
            PF_OK
        );
    }

    let directory = tempfile::tempdir().expect("a temporary folder");
    let path = directory.path().join("cajetin.pfluid");
    let path = CString::new(path.to_str().unwrap_or_default()).expect("a path");
    // SAFETY: live session and a NUL-terminated path.
    assert_eq!(unsafe { pf_session_save(original, path.as_ptr()) }, PF_OK);
    let reopened = session();
    // SAFETY: live session and a NUL-terminated path.
    assert_eq!(unsafe { pf_session_open(reopened, path.as_ptr()) }, PF_OK);
    assert_eq!(
        block_field(reopened, id, PF_BLOCK_OWNER),
        "Curso de Neumática"
    );
    assert_eq!(block_field(reopened, id, PF_BLOCK_PROJECT), "Línea 2");
    assert_eq!(block_field(reopened, id, PF_BLOCK_EQUIPMENT), "Prensa 4");
    assert_eq!(block_field(reopened, id, PF_BLOCK_NUMBER), "PF-2026-07");
    assert_eq!(block_field(reopened, id, PF_BLOCK_SCALE), "1:50");
    assert_eq!(
        block_field(reopened, id, PF_BLOCK_NORMS),
        "NCh 14 · NCh 1768/1"
    );

    // SAFETY: both sessions are still alive.
    unsafe {
        pf_session_free(reopened);
        pf_session_free(original);
    }
}

#[test]
fn the_block_can_be_moved_or_removed_and_one_undo_returns_it() {
    let original = session();
    // SAFETY: live session.
    assert_eq!(
        unsafe { pf_session_block_preset(original) },
        i32::try_from(PF_BLOCK_BOTTOM_RIGHT).expect("a preset")
    );
    // SAFETY: live session and a known preset.
    assert_eq!(
        unsafe { pf_session_set_block_preset(original, PF_BLOCK_TOP_RIGHT) },
        PF_OK
    );
    assert_eq!(
        unsafe { pf_session_block_preset(original) },
        i32::try_from(PF_BLOCK_TOP_RIGHT).expect("a preset")
    );
    // SAFETY: live session and the "no block" preset.
    assert_eq!(
        unsafe { pf_session_set_block_preset(original, PF_BLOCK_NONE) },
        PF_OK
    );
    assert_eq!(
        unsafe { pf_session_block_preset(original) },
        i32::try_from(PF_BLOCK_NONE).expect("a preset")
    );
    // SAFETY: live session with two changes to undo.
    assert_eq!(unsafe { pf_session_undo(original) }, PF_OK);
    assert_eq!(
        unsafe { pf_session_block_preset(original) },
        i32::try_from(PF_BLOCK_TOP_RIGHT).expect("a preset")
    );
    // SAFETY: live session and an unknown preset.
    assert_eq!(
        unsafe { pf_session_set_block_preset(original, 9) },
        punchi_fluid::PF_ERR_MODEL
    );
    // SAFETY: the session is still alive.
    unsafe { pf_session_free(original) };
}

/// The text of the first text call inside `area` — `x0 y0 x1 y1` — that ends with
/// `suffix`, or an empty string when the frame carries none.
fn text_in(calls: &[PfCall], bytes: &[u8], area: [f32; 4], suffix: &str) -> String {
    let field = |at: u32, len: u32| -> &[u8] {
        bytes
            .get(at as usize..at as usize + len as usize)
            .unwrap_or_default()
    };
    for call in calls.iter().filter(|call| call.kind == PF_KIND_TEXT) {
        let inside = call.rect[0] > area[0]
            && call.rect[0] < area[2]
            && call.rect[1] > area[1]
            && call.rect[1] < area[3];
        if !inside {
            continue;
        }
        let value = String::from_utf8_lossy(field(call.text_at, call.text_len)).into_owned();
        if value.ends_with(suffix) {
            return value;
        }
    }
    String::new()
}

#[test]
fn a_flow_meter_reads_the_line_it_is_connected_to() {
    let session = session();
    // The reading only means something with the meter in the line, so the hose from the
    // valve to the cylinder comes out and the meter goes in its place.
    let mut from = [0.0_f32; 2];
    let valve_port = CString::new("2").unwrap_or_default();
    let one = CString::new("1").unwrap_or_default();
    // SAFETY: live session, a NUL-terminated port and a writable pair.
    assert_eq!(
        unsafe { pf_session_port_point(session, 1, valve_port.as_ptr(), from.as_mut_ptr()) },
        PF_OK
    );
    // The hose starts at the valve's port; asking there finds it whatever bends the route
    // takes around the symbols.
    let hose = unsafe { pf_session_pipe_at(session, from[0], from[1], 2.0) };
    assert!(hose >= 0, "the valve-to-cylinder hose was not found");
    // SAFETY: live session and an index the hit test just returned.
    assert_eq!(unsafe { pf_session_select_pipe(session, hose) }, PF_OK);
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_remove_selected_pipe(session) }, PF_OK);

    let meter = catalog_index(session, "flow.meter");
    // SAFETY: live session.
    let meter_index = unsafe { pf_session_place_component(session, meter, 100.0, 60.0) };
    assert!(meter_index >= 0);
    let meter_index = u32::try_from(meter_index).unwrap_or_default();
    let two = CString::new("2").unwrap_or_default();
    // SAFETY: live session and NUL-terminated ports.
    assert_eq!(
        unsafe { pf_session_connect(session, 1, valve_port.as_ptr(), meter_index, one.as_ptr()) },
        PF_OK
    );
    // SAFETY: live session and NUL-terminated ports.
    assert_eq!(
        unsafe { pf_session_connect(session, meter_index, two.as_ptr(), 2, one.as_ptr()) },
        PF_OK
    );

    // The file says what the instrument is and what it is scaled to, before any air moves.
    let mut described = [0 as std::os::raw::c_char; 512];
    // SAFETY: live session, a writable buffer and its capacity.
    let written = unsafe {
        pf_session_describe_component(
            session,
            meter_index,
            described.as_mut_ptr(),
            u32::try_from(described.len()).unwrap_or_default(),
        )
    };
    assert!(written >= 0, "the meter has no description");
    // SAFETY: the call above wrote a NUL-terminated string.
    let text = unsafe { CStr::from_ptr(described.as_ptr()) }
        .to_str()
        .unwrap_or_default()
        .to_owned();
    assert!(text.contains("100 l/min"), "the scale is not in `{text}`");
    assert!(
        text.contains("Lee el caudal"),
        "the meter does not say what it reads: `{text}`"
    );

    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_press_valve(session) }, PF_OK);
    for _ in 0..5 {
        // SAFETY: live session.
        assert_eq!(unsafe { pf_session_advance(session, 0.016) }, PF_OK);
    }

    let mut frame = PfFrame::default();
    // SAFETY: live session, writable output.
    assert_eq!(
        unsafe { pf_session_build_frame(session, 4.0, 0.0, 0, &raw mut frame) },
        PF_OK
    );
    // SAFETY: the slices were produced by the call above and the session is alive.
    let (calls, text) = unsafe {
        (
            std::slice::from_raw_parts(frame.calls, frame.calls_len as usize),
            std::slice::from_raw_parts(frame.text, frame.text_len as usize),
        )
    };
    // The meter is 22 × 22 mm at (100, 60): its reading sits inside that box.
    let reading = text_in(calls, text, [89.0, 49.0, 111.0, 71.0], " l/min");
    assert!(
        !reading.is_empty(),
        "the meter has no reading among the {} text calls in the frame",
        calls
            .iter()
            .filter(|call| call.kind == PF_KIND_TEXT)
            .count()
    );
    let value: f32 = reading
        .trim_end_matches(" l/min")
        .parse()
        .unwrap_or_default();
    assert!(
        value > 0.0,
        "the meter reads `{reading}` while the valve feeds it, not a flow"
    );
    // SAFETY: the session is still alive.
    unsafe { pf_session_free(session) };
}

/// A session with a recorder and a gauge on the working line, and the sources the
/// recorder can chart. The caller frees the session.
fn recorder_fixture() -> (*mut PfSession, u32, Vec<PfChartSource>) {
    let session = session();
    let chart = catalog_index(session, "chart.recorder");
    // SAFETY: live session.
    let chart_index = unsafe { pf_session_place_component(session, chart, 60.0, 60.0) };
    assert!(chart_index >= 0);
    let chart_index = u32::try_from(chart_index).unwrap_or_default();

    let gauge = catalog_index(session, "gauge.pressure");
    // SAFETY: live session.
    let gauge_index = unsafe { pf_session_place_component(session, gauge, 160.0, 100.0) };
    assert!(gauge_index >= 0);
    let gauge_index = u32::try_from(gauge_index).unwrap_or_default();
    let working = CString::new("2").unwrap_or_default();
    let one = CString::new("1").unwrap_or_default();
    // SAFETY: live session and NUL-terminated ports.
    assert_eq!(
        unsafe { pf_session_connect(session, 1, working.as_ptr(), gauge_index, one.as_ptr()) },
        PF_OK
    );
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_press_valve(session) }, PF_OK);
    for _ in 0..40 {
        // SAFETY: live session.
        assert_eq!(unsafe { pf_session_advance(session, 0.016) }, PF_OK);
    }

    // SAFETY: live session.
    let count = unsafe { pf_session_chart_candidate_count(session, chart_index) };
    assert!(count > 0, "the circuit has sources to chart");
    let mut candidates = Vec::new();
    for slot in 0..count {
        let mut source = PfChartSource::default();
        // SAFETY: live session and a writable record.
        assert_eq!(
            unsafe { pf_session_chart_candidate(session, chart_index, slot, &raw mut source) },
            PF_OK
        );
        candidates.push(source);
    }
    (session, chart_index, candidates)
}

#[test]
fn a_recorder_charts_the_quantity_it_is_bound_to() {
    let (session, chart_index, candidates) = recorder_fixture();

    // Unbound, the recorder reads nothing.
    let mut bound = PfChartSource::default();
    // SAFETY: live session and a writable record.
    assert_eq!(
        unsafe { pf_session_chart_source(session, chart_index, &raw mut bound) },
        0
    );

    // The gauge was placed last, so it is the last candidate and reports pressure alone.
    let gauge_source = *candidates.last().expect("the gauge is a candidate");
    assert_eq!(gauge_source.quantity, PF_QUANTITY_PRESSURE);
    // SAFETY: live session.
    assert_eq!(
        unsafe {
            pf_session_set_chart_source(
                session,
                chart_index,
                gauge_source.placement,
                PF_QUANTITY_PRESSURE,
            )
        },
        PF_OK
    );
    // SAFETY: live session and a writable record.
    assert_eq!(
        unsafe { pf_session_chart_source(session, chart_index, &raw mut bound) },
        1
    );
    assert_eq!(bound, gauge_source);

    // The recorder samples on every advance; a few frames give it a curve.
    for _ in 0..20 {
        // SAFETY: live session.
        assert_eq!(unsafe { pf_session_advance(session, 0.016) }, PF_OK);
    }
    let mut frame = PfFrame::default();
    // SAFETY: live session, writable output.
    assert_eq!(
        unsafe { pf_session_build_frame(session, 4.0, 0.0, 0, &raw mut frame) },
        PF_OK
    );
    // SAFETY: the slices were produced by the call above and the session is alive.
    let (calls, text) = unsafe {
        (
            std::slice::from_raw_parts(frame.calls, frame.calls_len as usize),
            std::slice::from_raw_parts(frame.text, frame.text_len as usize),
        )
    };
    // The recorder is 23.8 × 23.8 mm at (60, 60).
    let reading = text_in(calls, text, [48.0, 48.0, 72.0, 72.0], " bar");
    assert!(
        !reading.is_empty(),
        "the recorder does not print the value it charts"
    );
    // SAFETY: the point arena belongs to the frame the call above produced.
    let points = unsafe { std::slice::from_raw_parts(frame.points, frame.points_len as usize) };
    let curved = calls.iter().any(|call| {
        if call.kind != PF_KIND_LINE || call.points_len <= 2 {
            return false;
        }
        let start = call.points_at as usize;
        let Some(slice) = points.get(start..start + call.points_len as usize) else {
            return false;
        };
        slice
            .iter()
            .all(|point| (40.0..80.0).contains(&point.x) && (40.0..80.0).contains(&point.y))
    });
    assert!(curved, "the recorder draws a curve, not a single segment");

    // SAFETY: the session is still alive.
    unsafe { pf_session_free(session) };
}

#[test]
fn a_recorder_refuses_a_quantity_it_cannot_read_and_undoes_the_binding() {
    let (session, chart_index, candidates) = recorder_fixture();
    let gauge_source = *candidates.last().expect("the gauge is a candidate");
    // SAFETY: live session.
    assert_eq!(
        unsafe {
            pf_session_set_chart_source(
                session,
                chart_index,
                gauge_source.placement,
                PF_QUANTITY_PRESSURE,
            )
        },
        PF_OK
    );
    // A gauge reports no flow, so asking for one is refused and the binding stays.
    // SAFETY: live session.
    assert_eq!(
        unsafe {
            pf_session_set_chart_source(
                session,
                chart_index,
                gauge_source.placement,
                PF_QUANTITY_FLOW,
            )
        },
        PF_ERR_MODEL
    );

    // A cylinder reports its position, so that binding is accepted and undoing it returns
    // the gauge.
    let cylinder = candidates
        .iter()
        .find(|candidate| candidate.quantity == PF_QUANTITY_POSITION)
        .copied()
        .expect("the cylinder reports its position");
    // SAFETY: live session.
    assert_eq!(
        unsafe {
            pf_session_set_chart_source(
                session,
                chart_index,
                cylinder.placement,
                PF_QUANTITY_POSITION,
            )
        },
        PF_OK
    );
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_undo(session) }, PF_OK);
    let mut bound = PfChartSource::default();
    // SAFETY: live session and a writable record.
    assert_eq!(
        unsafe { pf_session_chart_source(session, chart_index, &raw mut bound) },
        1
    );
    assert_eq!(bound, gauge_source, "one undo returns the previous source");

    // SAFETY: live session.
    assert_eq!(
        unsafe { pf_session_clear_chart_source(session, chart_index) },
        PF_OK
    );
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_undo(session) }, PF_OK);
    // SAFETY: live session and a writable record.
    assert_eq!(
        unsafe { pf_session_chart_source(session, chart_index, &raw mut bound) },
        1,
        "undoing the clear returns the binding"
    );
    // SAFETY: the session is still alive.
    unsafe { pf_session_free(session) };
}

#[test]
fn even_spacing_keeps_the_ends_and_is_one_undo() {
    let session = session();
    let count = unsafe { pf_session_component_count(session) };
    assert!(count >= 3, "the worked example has components to spread");
    // SAFETY: live session.
    assert!(
        unsafe { pf_session_select_all(session) } >= 3,
        "nothing to spread"
    );

    let mut before = Vec::new();
    for index in 0..count {
        let mut rect = [0.0_f32; 4];
        // SAFETY: live session and a writable rectangle.
        assert_eq!(
            unsafe { pf_session_component_rect(session, index, rect.as_mut_ptr()) },
            PF_OK
        );
        before.push(rect);
    }

    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_space_selection(session, 0) }, PF_OK);

    let mut after = Vec::new();
    for index in 0..count {
        let mut rect = [0.0_f32; 4];
        // SAFETY: live session and a writable rectangle.
        assert_eq!(
            unsafe { pf_session_component_rect(session, index, rect.as_mut_ptr()) },
            PF_OK
        );
        after.push(rect);
    }
    // Ordered left to right, the gaps between boxes are equal.
    after.sort_by(|left, right| left[0].total_cmp(&right[0]));
    let gaps: Vec<f32> = after
        .windows(2)
        .map(|pair| pair[1][0] - pair[0][2])
        .collect();
    let first = gaps.first().copied().unwrap_or_default();
    assert!(first > 0.0, "the spread left the symbols overlapping");
    for gap in &gaps {
        assert!(
            (gap - first).abs() < 0.05,
            "the gaps are {gaps:?}, not equal"
        );
    }
    // The outermost kept their places: the spread changes the inside, not the extent.
    let leftmost = after.first().copied().unwrap_or_default();
    let rightmost = after.last().copied().unwrap_or_default();
    let original_left = before
        .iter()
        .map(|rect| rect[0])
        .fold(f32::INFINITY, f32::min);
    let original_right = before
        .iter()
        .map(|rect| rect[2])
        .fold(f32::NEG_INFINITY, f32::max);
    assert!((leftmost[0] - original_left).abs() < 0.05);
    assert!((rightmost[2] - original_right).abs() < 0.05);

    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_undo(session) }, PF_OK);
    for (index, rect) in before.iter().enumerate() {
        let mut restored = [0.0_f32; 4];
        // SAFETY: live session and a writable rectangle.
        assert_eq!(
            unsafe {
                pf_session_component_rect(
                    session,
                    u32::try_from(index).unwrap_or_default(),
                    restored.as_mut_ptr(),
                )
            },
            PF_OK
        );
        assert!(
            restored
                .iter()
                .zip(rect.iter())
                .all(|(left, right)| (left - right).abs() < 1.0e-6),
            "one undo puts component {index} back: {restored:?} against {rect:?}"
        );
    }
    // SAFETY: the session is still alive.
    unsafe { pf_session_free(session) };
}

#[test]
fn a_pipe_takes_another_line_class_and_keeps_it_in_the_file() {
    let original = session();
    // The worked example draws its hoses as working lines.
    let pipes = unsafe { pf_session_connection_count(original) };
    assert!(pipes > 0, "the example has hoses");
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_pipe_class(original, 0) }, 0);

    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_set_pipe_class(original, 0, 1) }, PF_OK);
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_pipe_class(original, 0) }, 1);
    // The same class twice is not an edit, so it is refused rather than recorded.
    // SAFETY: live session.
    assert_eq!(
        unsafe { pf_session_set_pipe_class(original, 0, 1) },
        PF_ERR_MODEL
    );
    // A class this build does not know is refused.
    // SAFETY: live session.
    assert_eq!(
        unsafe { pf_session_set_pipe_class(original, 0, 9) },
        PF_ERR_MODEL
    );
    // A pipe that is not there is refused, not answered with another pipe.
    // SAFETY: live session.
    assert_eq!(
        unsafe { pf_session_pipe_class(original, 9_999) },
        PF_ERR_MODEL
    );

    let directory = tempfile::tempdir().expect("a temporary folder");
    let path = directory.path().join("clases.pfluid");
    let path = CString::new(path.to_str().unwrap_or_default()).expect("a path");
    // SAFETY: live session and a NUL-terminated path.
    assert_eq!(unsafe { pf_session_save(original, path.as_ptr()) }, PF_OK);
    let reopened = session();
    // SAFETY: live session and a NUL-terminated path.
    assert_eq!(unsafe { pf_session_open(reopened, path.as_ptr()) }, PF_OK);
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_pipe_class(reopened, 0) }, 1);

    // One undo returns the working line, and redo takes the pilot one back.
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_undo(original) }, PF_OK);
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_pipe_class(original, 0) }, 0);
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_redo(original) }, PF_OK);
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_pipe_class(original, 0) }, 1);
    // SAFETY: both sessions are still alive.
    unsafe {
        pf_session_free(reopened);
        pf_session_free(original);
    }
}

#[test]
fn a_new_pneumatic_connection_is_ten_millimetres_and_an_electrical_wire_has_no_tube() {
    let session = session();
    let pipes = unsafe { pf_session_connection_count(session) };
    assert!(pipes >= 3, "the example draws its pneumatic circuit");
    for index in 0..pipes {
        // SAFETY: live session.
        assert_eq!(
            unsafe { pf_session_pipe_tube(session, index) },
            1000,
            "the seeded pipe {index} is not 10 mm"
        );
    }

    // A pipe the user makes starts at the same documented 10 mm.
    let supply = catalog_index(session, "supply.compressor");
    let valve = catalog_index(session, "valve.3_2.spring_return");
    // SAFETY: live session and finite coordinates.
    let supply = unsafe { pf_session_place_component(session, supply, 120.0, 40.0) };
    let valve = unsafe { pf_session_place_component(session, valve, 160.0, 40.0) };
    assert!(supply >= 0 && valve >= 0);
    let supply = u32::try_from(supply).unwrap_or_default();
    let valve = u32::try_from(valve).unwrap_or_default();
    let one = CString::new("1").unwrap_or_default();
    let before = unsafe { pf_session_connection_count(session) };
    // SAFETY: live session and a NUL-terminated port.
    assert_eq!(
        unsafe { pf_session_connect(session, supply, one.as_ptr(), valve, one.as_ptr()) },
        PF_OK
    );
    let pipe = before;
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_pipe_tube(session, pipe) }, 1000);

    // The bound: a pipe that does not exist has no size at all.
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_pipe_tube(session, 9_999) }, -1);

    // A valid change is one undo.
    // SAFETY: live session.
    assert_eq!(
        unsafe { pf_session_set_pipe_tube(session, pipe, 1600) },
        PF_OK
    );
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_pipe_tube(session, pipe) }, 1600);
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_undo(session) }, PF_OK);
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_pipe_tube(session, pipe) }, 1000);
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_redo(session) }, PF_OK);
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_pipe_tube(session, pipe) }, 1600);

    // Every value outside `100..=5000` is refused, and the pipe keeps its size.
    for value in [99, 5001, 65000, -1, -1600] {
        // SAFETY: live session.
        assert_eq!(
            unsafe { pf_session_set_pipe_tube(session, pipe, value) },
            1,
            "{value} must be refused"
        );
    }
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_pipe_tube(session, pipe) }, 1600);
    // `0` takes the size away, and both ends of the range are accepted.
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_set_pipe_tube(session, pipe, 0) }, PF_OK);
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_pipe_tube(session, pipe) }, 0);
    for value in [100, 5000] {
        // SAFETY: live session.
        assert_eq!(
            unsafe { pf_session_set_pipe_tube(session, pipe, value) },
            PF_OK
        );
        // SAFETY: live session.
        assert_eq!(unsafe { pf_session_pipe_tube(session, pipe) }, value);
    }

    // The wire: no tube, and the setter refuses it with a reason.
    let wire = electrical_wire(session, (40.0, 80.0));
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_pipe_tube(session, wire) }, 0);
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_set_pipe_tube(session, wire, 1600) }, 1);
    // SAFETY: live session.
    let error = unsafe { CStr::from_ptr(pf_session_last_error(session)) }.to_string_lossy();
    assert!(
        error.contains("electrical"),
        "the refusal says why: {error}"
    );
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_pipe_tube(session, wire) }, 0);

    // SAFETY: the session is still alive.
    unsafe { pf_session_free(session) };
}

#[test]
fn the_tube_series_abi_enumerates_metric_and_inch_entries() {
    let count = pf_tube_size_count();
    assert_eq!(count, 23, "13 metric entries and 10 inch ones");
    for index in 0..count {
        let index_u32 = u32::try_from(index).unwrap_or_default();
        let nominal = pf_tube_size_nominal(index_u32);
        assert_eq!(
            nominal,
            i32::from(index >= 13),
            "entry {index} is in the wrong group"
        );
        assert!(pf_tube_size_od(index_u32) >= 100);
    }
    // 10 mm and 3/8" are separate entries with separate stored values.
    assert_eq!(pf_tube_size_od(4), 1000);
    assert_eq!(pf_tube_size_nominal(4), 0);
    assert_eq!(pf_tube_size_od(13), 318);
    assert_eq!(pf_tube_size_nominal(13), 1);
    let missing = u32::try_from(count).unwrap_or_default();
    assert_eq!(pf_tube_size_od(missing), -1);
    assert_eq!(pf_tube_size_nominal(missing), -1);

    let mut buffer = [0 as c_char; 16];
    // SAFETY: a writable buffer of the stated capacity.
    assert_eq!(unsafe { pf_tube_size_label(4, buffer.as_mut_ptr(), 16) }, 5);
    // SAFETY: the call above wrote a NUL-terminated label.
    assert_eq!(
        unsafe { CStr::from_ptr(buffer.as_ptr()) }.to_string_lossy(),
        "10 mm"
    );
    // SAFETY: a writable buffer of the stated capacity.
    assert_eq!(
        unsafe { pf_tube_size_label(13, buffer.as_mut_ptr(), 16) },
        4
    );
    // SAFETY: the call above wrote a NUL-terminated label.
    assert_eq!(
        unsafe { CStr::from_ptr(buffer.as_ptr()) }.to_string_lossy(),
        "1/8\""
    );
    // A short buffer and a missing entry are refused, not truncated.
    // SAFETY: a writable buffer of the stated capacity.
    assert_eq!(
        unsafe { pf_tube_size_label(4, buffer.as_mut_ptr(), 3) },
        PF_ERR_MODEL
    );
    // SAFETY: a writable buffer of the stated capacity.
    assert_eq!(
        unsafe { pf_tube_size_label(missing, buffer.as_mut_ptr(), 16) },
        PF_ERR_MODEL
    );
    // SAFETY: a null output is refused before anything is written.
    assert_eq!(
        unsafe { pf_tube_size_label(4, ptr::null_mut(), 16) },
        PF_ERR_NULL
    );
}

#[test]
fn an_electrical_connection_has_no_tube_and_no_tube_row() {
    let session = unsafe { pf_session_new(ptr::null()) };
    assert!(!session.is_null());
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_new_document(session) }, PF_OK);
    let wire = electrical_wire(session, (40.0, 40.0));
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_pipe_tube(session, wire) }, 0);

    let exported = export_svg(session, "control.svg");
    assert!(exported.is_some(), "the SVG page is written");
    let svg = exported.unwrap_or_default();
    assert!(svg.contains("parts list"), "the list is printed");
    assert!(
        !svg.contains("Hose"),
        "an electrical wire is not tubing and must not print a tube row"
    );
    assert!(
        !svg.contains("unspecified"),
        "the wire must not become an unspecified tube either"
    );

    // SAFETY: the session is still alive.
    unsafe { pf_session_free(session) };
}

#[test]
fn the_parts_list_has_a_tube_row_with_the_drawn_length() {
    let session = unsafe { pf_session_new(ptr::null()) };
    assert!(!session.is_null());
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_new_document(session) }, PF_OK);
    let supply = catalog_index(session, "supply.compressor");
    let cylinder = catalog_index(session, "cylinder.single_acting_20_100");
    // Far apart, so the drawn route is longer than a decimetre and the row cannot be
    // mistaken for a rounding of a short stub.
    // SAFETY: live session and finite coordinates.
    let supply = unsafe { pf_session_place_component(session, supply, 30.0, 30.0) };
    let cylinder = unsafe { pf_session_place_component(session, cylinder, 150.0, 220.0) };
    assert!(supply >= 0 && cylinder >= 0);
    let supply = u32::try_from(supply).unwrap_or_default();
    let cylinder = u32::try_from(cylinder).unwrap_or_default();
    let one = CString::new("1").unwrap_or_default();
    // SAFETY: live session and a NUL-terminated port.
    assert_eq!(
        unsafe { pf_session_connect(session, supply, one.as_ptr(), cylinder, one.as_ptr()) },
        PF_OK
    );

    let exported = export_svg(session, "tubing.svg");
    assert!(exported.is_some(), "the SVG page is written");
    let svg = exported.unwrap_or_default();
    let row = tube_row(&svg, "10 mm").expect("the 10 mm pipe has a row");
    assert!(
        row.contains("working"),
        "the row names the line class: {row}"
    );
    assert!(row.ends_with(" m"), "the row carries a length: {row}");
    assert!(
        !row.contains("< 0.1 m"),
        "a route across the sheet is longer than a decimetre: {row}"
    );
    assert!(
        svg.contains("drawn route"),
        "the page says the length is the drawn route"
    );

    // SAFETY: the session is still alive.
    unsafe { pf_session_free(session) };
}

#[test]
fn an_old_pipe_reads_as_unspecified_and_keeps_its_width() {
    let original = session();
    let pipes = unsafe { pf_session_connection_count(original) };
    assert!(pipes > 0);
    for index in 0..pipes {
        // SAFETY: live session.
        assert_eq!(
            unsafe { pf_session_set_pipe_tube(original, index, 0) },
            PF_OK
        );
    }
    let directory = tempfile::tempdir().expect("a temporary folder");
    let current = directory.path().join("current.pfluid");
    let current = CString::new(current.to_str().unwrap_or_default()).expect("a path");
    // SAFETY: live session and a NUL-terminated path.
    assert_eq!(
        unsafe { pf_session_save(original, current.as_ptr()) },
        PF_OK
    );
    let text = std::fs::read_to_string(
        // SAFETY: the path is a valid NUL-terminated C string.
        unsafe { CStr::from_ptr(current.as_ptr()) }
            .to_str()
            .unwrap_or_default(),
    )
    .expect("the sheet is written");
    assert!(text.contains("\"format\": 14"), "this build writes 14");
    assert!(
        !text.contains("tube_od_hundredths"),
        "an unspecified pipe writes no key"
    );

    // What a format-12 release wrote: the same sheet without the key.
    let older = text.replace("\"format\": 14", "\"format\": 12");
    let older_path = directory.path().join("older.pfluid");
    std::fs::write(&older_path, older).expect("the older sheet is written");
    let older_path = CString::new(older_path.to_str().unwrap_or_default()).expect("a path");
    let reopened = unsafe { pf_session_new(ptr::null()) };
    assert!(!reopened.is_null());
    // SAFETY: live session and a NUL-terminated path.
    assert_eq!(
        unsafe { pf_session_open(reopened, older_path.as_ptr()) },
        PF_OK
    );
    for index in 0..pipes {
        // SAFETY: live session.
        assert_eq!(
            unsafe { pf_session_pipe_tube(reopened, index) },
            0,
            "an old pipe {index} reads as unspecified, not as 10 mm"
        );
    }
    // `None` keeps the stroke the drawing has always used, which is below any named width.
    assert!((tube_width_mm(None) - 0.42).abs() < 1e-6);

    // SAFETY: both sessions are still alive.
    unsafe {
        pf_session_free(reopened);
        pf_session_free(original);
    }
}

#[test]
fn an_export_can_be_a_vector_drawing() {
    let session = session();
    let directory = tempfile::tempdir().expect("a temporary folder");
    let path = directory.path().join("plano.svg");
    let path = CString::new(path.to_str().unwrap_or_default()).expect("a path");
    let date = CString::new("2026-11-12").expect("a date");
    // SAFETY: live session, NUL-terminated path and date, A4 is index 1.
    assert_eq!(
        unsafe {
            pf_session_export(
                session,
                path.as_ptr(),
                PF_EXPORT_SVG,
                1,
                0,
                300.0,
                date.as_ptr(),
            )
        },
        PF_OK
    );
    let svg =
        std::fs::read_to_string(path.to_str().unwrap_or_default()).expect("the SVG is written");
    assert!(svg.starts_with("<?xml"), "an XML document");
    assert!(svg.contains("<svg"), "with the document element");
    assert!(
        svg.contains("viewBox=\"0 0 210.0 297.0\""),
        "A4 in millimetres"
    );
    for text in [
        "2026-11-12",
        "Circuito 1",
        "parts list",
        "Single-acting cylinder",
    ] {
        assert!(svg.contains(text), "the sheet is missing {text}");
    }
    // SAFETY: the session is still alive.
    unsafe { pf_session_free(session) };
}

#[test]
fn a_custom_print_paper_reaches_the_page() {
    let session = session();
    let directory = tempfile::tempdir().expect("a temporary folder");
    let path = directory.path().join("print.svg");
    let path_c = CString::new(path.to_str().unwrap_or_default()).expect("a path");
    let date = CString::new("2026-11-12").expect("a date");

    // A custom sheet with 5 mm margins and no parts list, as a vector page.
    let options = PfExportOptions {
        kind: PF_EXPORT_SVG,
        paper: PF_EXPORT_CUSTOM_PAPER,
        flags: 0,
        margin_top: 5.0,
        margin_right: 5.0,
        margin_bottom: 5.0,
        margin_left: 5.0,
        width_mm: 100.0,
        height_mm: 80.0,
        ..PfExportOptions::default()
    };
    // SAFETY: live session, NUL-terminated strings and a readable structure.
    assert_eq!(
        unsafe {
            pf_session_export_with(session, path_c.as_ptr(), date.as_ptr(), &raw const options)
        },
        PF_OK
    );
    let svg = std::fs::read_to_string(path.to_str().unwrap_or_default()).expect("the SVG exists");
    assert!(
        svg.contains("viewBox=\"0 0 100.0 80.0\""),
        "the custom paper reached the page"
    );
    assert!(!svg.contains("parts list"), "the list can be left off");

    // A page whose margins leave no room is refused before anything is written.
    let impossible = PfExportOptions {
        margin_top: 60.0,
        margin_bottom: 60.0,
        ..options
    };
    // SAFETY: the same contracts as above.
    assert_eq!(
        unsafe {
            pf_session_export_with(
                session,
                path_c.as_ptr(),
                date.as_ptr(),
                &raw const impossible,
            )
        },
        PF_ERR_MODEL
    );
    // A custom size outside the bounds is refused, not clamped.
    let too_small = PfExportOptions {
        width_mm: 10.0,
        height_mm: 10.0,
        ..options
    };
    // SAFETY: the same contracts as above.
    assert_eq!(
        unsafe {
            pf_session_export_with(
                session,
                path_c.as_ptr(),
                date.as_ptr(),
                &raw const too_small,
            )
        },
        PF_ERR_MODEL
    );
    // SAFETY: the session is still alive.
    unsafe { pf_session_free(session) };
}

#[test]
fn a_contact_energises_the_solenoid_it_is_wired_to() {
    let session = session();
    // The control circuit: rail, button and the coil of a solenoid valve.
    let rail = catalog_index(session, "electrical.supply");
    let button = catalog_index(session, "electrical.contact");
    let valve = catalog_index(session, "valve.3_2.solenoid");
    // SAFETY: live session.
    let rail = unsafe { pf_session_place_component(session, rail, 40.0, 20.0) };
    let button = unsafe { pf_session_place_component(session, button, 90.0, 20.0) };
    let valve = unsafe { pf_session_place_component(session, valve, 140.0, 20.0) };
    assert!(rail >= 0 && button >= 0 && valve >= 0);
    let rail = u32::try_from(rail).unwrap_or_default();
    let button = u32::try_from(button).unwrap_or_default();
    let valve = u32::try_from(valve).unwrap_or_default();
    let l = CString::new("L").unwrap_or_default();
    let one = CString::new("1").unwrap_or_default();
    let two = CString::new("2").unwrap_or_default();
    let coil = CString::new("Y1").unwrap_or_default();

    // SAFETY: live session and NUL-terminated ports.
    assert_eq!(
        unsafe { pf_session_connect(session, rail, l.as_ptr(), button, one.as_ptr()) },
        PF_OK
    );
    // SAFETY: live session and NUL-terminated ports.
    assert_eq!(
        unsafe { pf_session_connect(session, button, two.as_ptr(), valve, coil.as_ptr()) },
        PF_OK
    );

    // The button starts open, so the coil is dead and the valve rests.
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_contact_closed(session, button) }, 0);
    // SAFETY: live session.
    assert_eq!(
        unsafe { pf_session_component_valve_position(session, valve) },
        0
    );

    // Pressed, the wire is live and the valve follows its coil.
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_toggle_contact(session, button) }, PF_OK);
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_contact_closed(session, button) }, 1);
    // SAFETY: live session.
    assert_eq!(
        unsafe { pf_session_component_valve_position(session, valve) },
        1
    );

    // Released, the spring returns the spool.
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_toggle_contact(session, button) }, PF_OK);
    // SAFETY: live session.
    assert_eq!(
        unsafe { pf_session_component_valve_position(session, valve) },
        0
    );

    // A pneumatic component is not a contact and says so.
    // SAFETY: live session.
    assert_eq!(
        unsafe { pf_session_toggle_contact(session, 0) },
        PF_ERR_MODEL
    );
    // SAFETY: the session is still alive.
    unsafe { pf_session_free(session) };
}

#[test]
fn a_relay_closes_its_contact_while_its_coil_is_live() {
    let session = session();
    let rail = catalog_index(session, "electrical.supply");
    let button = catalog_index(session, "electrical.contact");
    let relay = catalog_index(session, "electrical.relay");
    let valve = catalog_index(session, "valve.3_2.solenoid");
    // SAFETY: live session.
    let rail = unsafe { pf_session_place_component(session, rail, 20.0, 20.0) };
    let button = unsafe { pf_session_place_component(session, button, 70.0, 20.0) };
    let relay = unsafe { pf_session_place_component(session, relay, 120.0, 20.0) };
    let valve = unsafe { pf_session_place_component(session, valve, 170.0, 20.0) };
    assert!(rail >= 0 && button >= 0 && relay >= 0 && valve >= 0);
    let rail = u32::try_from(rail).unwrap_or_default();
    let button = u32::try_from(button).unwrap_or_default();
    let relay = u32::try_from(relay).unwrap_or_default();
    let valve = u32::try_from(valve).unwrap_or_default();
    let l = CString::new("L").unwrap_or_default();
    let n = CString::new("N").unwrap_or_default();
    let one = CString::new("1").unwrap_or_default();
    let two = CString::new("2").unwrap_or_default();
    let a1 = CString::new("A1").unwrap_or_default();
    let a2 = CString::new("A2").unwrap_or_default();
    let eleven = CString::new("11").unwrap_or_default();
    let fourteen = CString::new("14").unwrap_or_default();
    let coil = CString::new("Y1").unwrap_or_default();

    // The rail feeds the button; the button feeds the coil and the relay's own contact;
    // the contact feeds the solenoid. The relay is in the middle of its own net.
    // SAFETY: live session and NUL-terminated ports.
    assert_eq!(
        unsafe { pf_session_connect(session, rail, l.as_ptr(), button, one.as_ptr()) },
        PF_OK
    );
    // SAFETY: live session and NUL-terminated ports.
    assert_eq!(
        unsafe { pf_session_connect(session, button, two.as_ptr(), relay, a1.as_ptr()) },
        PF_OK
    );
    // SAFETY: live session and NUL-terminated ports.
    assert_eq!(
        unsafe { pf_session_connect(session, button, two.as_ptr(), relay, eleven.as_ptr()) },
        PF_OK
    );
    // SAFETY: live session and NUL-terminated ports.
    assert_eq!(
        unsafe { pf_session_connect(session, relay, a2.as_ptr(), rail, n.as_ptr()) },
        PF_OK
    );
    // SAFETY: live session and NUL-terminated ports.
    assert_eq!(
        unsafe { pf_session_connect(session, relay, fourteen.as_ptr(), valve, coil.as_ptr()) },
        PF_OK
    );

    // At rest the button is open, the coil is dead and the contact is a gap.
    // SAFETY: live session.
    assert_eq!(
        unsafe { pf_session_component_valve_position(session, valve) },
        0
    );

    // Pressed, the coil sees one live terminal and the contact closes the solenoid's line.
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_toggle_contact(session, button) }, PF_OK);
    // SAFETY: live session.
    assert_eq!(
        unsafe { pf_session_component_valve_position(session, valve) },
        1
    );

    // Released, the coil rests and the contact opens again: no state is held by the relay.
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_toggle_contact(session, button) }, PF_OK);
    // SAFETY: live session.
    assert_eq!(
        unsafe { pf_session_component_valve_position(session, valve) },
        0
    );

    // SAFETY: the session is still alive.
    unsafe { pf_session_free(session) };
}

/// Places a rail, a button, a timer and a solenoid valve, and wires them as a timer rig.
///
/// The button feeds the coil and the timer's own contact; the contact feeds the valve.
fn timer_rig(session: *mut PfSession, timer: &str) -> (u32, u32) {
    let rail = catalog_index(session, "electrical.supply");
    let button = catalog_index(session, "electrical.contact");
    let timer = catalog_index(session, timer);
    let valve = catalog_index(session, "valve.3_2.solenoid");
    // SAFETY: live session.
    let rail = unsafe { pf_session_place_component(session, rail, 20.0, 20.0) };
    let button = unsafe { pf_session_place_component(session, button, 70.0, 20.0) };
    let timer = unsafe { pf_session_place_component(session, timer, 120.0, 20.0) };
    let valve = unsafe { pf_session_place_component(session, valve, 170.0, 20.0) };
    assert!(rail >= 0 && button >= 0 && timer >= 0 && valve >= 0);
    let rail = u32::try_from(rail).unwrap_or_default();
    let button = u32::try_from(button).unwrap_or_default();
    let timer = u32::try_from(timer).unwrap_or_default();
    let valve = u32::try_from(valve).unwrap_or_default();
    let l = CString::new("L").unwrap_or_default();
    let n = CString::new("N").unwrap_or_default();
    let one = CString::new("1").unwrap_or_default();
    let two = CString::new("2").unwrap_or_default();
    let a1 = CString::new("A1").unwrap_or_default();
    let a2 = CString::new("A2").unwrap_or_default();
    let eleven = CString::new("11").unwrap_or_default();
    let fourteen = CString::new("14").unwrap_or_default();
    let coil = CString::new("Y1").unwrap_or_default();
    for (from, from_port, to, to_port) in [
        (rail, &l, button, &one),
        (button, &two, timer, &a1),
        (rail, &l, timer, &eleven),
        (timer, &a2, rail, &n),
        (timer, &fourteen, valve, &coil),
    ] {
        // SAFETY: live session and NUL-terminated ports.
        assert_eq!(
            unsafe { pf_session_connect(session, from, from_port.as_ptr(), to, to_port.as_ptr()) },
            PF_OK
        );
    }
    (button, valve)
}

#[test]
fn a_ton_contact_closes_after_its_delay() {
    let session = session();
    let (button, valve) = timer_rig(session, "electrical.timer.ton");
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_toggle_contact(session, button) }, PF_OK);
    // One second of steps: the two-second delay has not elapsed and the valve rests.
    for _ in 0..100 {
        // SAFETY: live session.
        assert_eq!(unsafe { pf_session_advance(session, 0.01) }, PF_OK);
    }
    // SAFETY: live session.
    assert_eq!(
        unsafe { pf_session_component_valve_position(session, valve) },
        0
    );
    // Two more tenths of a second cross the delay; the contact closes and the valve moves.
    for _ in 0..110 {
        assert_eq!(unsafe { pf_session_advance(session, 0.01) }, PF_OK);
    }
    // SAFETY: live session.
    assert_eq!(
        unsafe { pf_session_component_valve_position(session, valve) },
        1
    );
    // A dead coil resets the timer at once, without waiting for the delay.
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_toggle_contact(session, button) }, PF_OK);
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_advance(session, 0.01) }, PF_OK);
    // SAFETY: live session.
    assert_eq!(
        unsafe { pf_session_component_valve_position(session, valve) },
        0
    );
    // SAFETY: the session is still alive.
    unsafe { pf_session_free(session) };
}

#[test]
fn a_tof_contact_opens_after_its_delay() {
    let session = session();
    let (button, valve) = timer_rig(session, "electrical.timer.tof");
    // A TOF follows a live coil at once: the first step closes its contact.
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_toggle_contact(session, button) }, PF_OK);
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_advance(session, 0.01) }, PF_OK);
    // SAFETY: live session.
    assert_eq!(
        unsafe { pf_session_component_valve_position(session, valve) },
        1
    );
    // Released, the contact stays closed during the delay.
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_toggle_contact(session, button) }, PF_OK);
    for _ in 0..100 {
        assert_eq!(unsafe { pf_session_advance(session, 0.01) }, PF_OK);
    }
    // SAFETY: live session.
    assert_eq!(
        unsafe { pf_session_component_valve_position(session, valve) },
        1
    );
    // Past the delay the contact opens and the valve springs back.
    for _ in 0..110 {
        assert_eq!(unsafe { pf_session_advance(session, 0.01) }, PF_OK);
    }
    // SAFETY: live session.
    assert_eq!(
        unsafe { pf_session_component_valve_position(session, valve) },
        0
    );
    // SAFETY: the session is still alive.
    unsafe { pf_session_free(session) };
}

#[test]
fn a_lamp_lights_while_its_line_is_live() {
    let session = session();
    let rail = catalog_index(session, "electrical.supply");
    let button = catalog_index(session, "electrical.contact");
    let lamp = catalog_index(session, "electrical.lamp");
    // SAFETY: live session.
    let rail = unsafe { pf_session_place_component(session, rail, 20.0, 20.0) };
    let button = unsafe { pf_session_place_component(session, button, 70.0, 20.0) };
    let lamp = unsafe { pf_session_place_component(session, lamp, 120.0, 20.0) };
    assert!(rail >= 0 && button >= 0 && lamp >= 0);
    let rail = u32::try_from(rail).unwrap_or_default();
    let button = u32::try_from(button).unwrap_or_default();
    let lamp = u32::try_from(lamp).unwrap_or_default();
    let l = CString::new("L").unwrap_or_default();
    let one = CString::new("1").unwrap_or_default();
    let two = CString::new("2").unwrap_or_default();
    // SAFETY: live session and NUL-terminated ports.
    assert_eq!(
        unsafe { pf_session_connect(session, rail, l.as_ptr(), button, one.as_ptr()) },
        PF_OK
    );
    // SAFETY: live session and NUL-terminated ports.
    assert_eq!(
        unsafe { pf_session_connect(session, button, two.as_ptr(), lamp, one.as_ptr()) },
        PF_OK
    );

    // The button is open, so the lamp's line is dead.
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_indicator_active(session, lamp) }, 0);
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_toggle_contact(session, button) }, PF_OK);
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_indicator_active(session, lamp) }, 1);
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_toggle_contact(session, button) }, PF_OK);
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_indicator_active(session, lamp) }, 0);

    // A pneumatic component is not an indicator and says so.
    // SAFETY: live session.
    assert_eq!(
        unsafe { pf_session_indicator_active(session, 0) },
        PF_ERR_MODEL
    );

    // A lamp has no gas to report: it is not a quantity a recorder can chart.
    let chart = catalog_index(session, "chart.recorder");
    // SAFETY: live session.
    let chart = unsafe { pf_session_place_component(session, chart, 20.0, 80.0) };
    assert!(chart >= 0);
    let chart = u32::try_from(chart).unwrap_or_default();
    // SAFETY: live session.
    let count = unsafe { pf_session_chart_candidate_count(session, chart) };
    for slot in 0..count {
        let mut source = PfChartSource::default();
        // SAFETY: live session and a writable record.
        assert_eq!(
            unsafe { pf_session_chart_candidate(session, chart, slot, &raw mut source) },
            PF_OK
        );
        assert_ne!(
            source.placement, lamp,
            "a lamp was offered as a chart source"
        );
    }
    // SAFETY: the session is still alive.
    unsafe { pf_session_free(session) };
}

#[test]
fn a_buzzer_sounds_while_its_line_is_live() {
    let session = session();
    let rail = catalog_index(session, "electrical.supply");
    let button = catalog_index(session, "electrical.contact");
    let buzzer = catalog_index(session, "electrical.buzzer");
    // SAFETY: live session.
    let rail = unsafe { pf_session_place_component(session, rail, 20.0, 20.0) };
    let button = unsafe { pf_session_place_component(session, button, 70.0, 20.0) };
    let buzzer = unsafe { pf_session_place_component(session, buzzer, 120.0, 20.0) };
    assert!(rail >= 0 && button >= 0 && buzzer >= 0);
    let rail = u32::try_from(rail).unwrap_or_default();
    let button = u32::try_from(button).unwrap_or_default();
    let buzzer = u32::try_from(buzzer).unwrap_or_default();
    let l = CString::new("L").unwrap_or_default();
    let one = CString::new("1").unwrap_or_default();
    let two = CString::new("2").unwrap_or_default();
    // SAFETY: live session and NUL-terminated ports.
    assert_eq!(
        unsafe { pf_session_connect(session, rail, l.as_ptr(), button, one.as_ptr()) },
        PF_OK
    );
    // SAFETY: live session and NUL-terminated ports.
    assert_eq!(
        unsafe { pf_session_connect(session, button, two.as_ptr(), buzzer, one.as_ptr()) },
        PF_OK
    );

    // The button is open, so no buzzer is sounding.
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_buzzer_active(session) }, 0);
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_toggle_contact(session, button) }, PF_OK);
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_buzzer_active(session) }, 1);
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_indicator_active(session, buzzer) }, 1);
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_toggle_contact(session, button) }, PF_OK);
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_buzzer_active(session) }, 0);

    // SAFETY: the session is still alive.
    unsafe { pf_session_free(session) };
}

#[test]
fn a_limit_switch_closes_at_the_end_of_its_cylinder() {
    let session = session();
    // The seeded circuit keeps its first three placements in this order: supply, valve and
    // cylinder. The switch watches the cylinder, chosen through the same source binding a
    // recorder uses.
    // SAFETY: live session.
    let cylinder = unsafe { pf_session_placement_id(session, 2) };
    assert!(cylinder > 0, "the seeded cylinder has an identity");
    let rail = catalog_index(session, "electrical.supply");
    let limit = catalog_index(session, "electrical.limit_switch.extended");
    let lamp = catalog_index(session, "electrical.lamp");
    // SAFETY: live session.
    let rail = unsafe { pf_session_place_component(session, rail, 20.0, 20.0) };
    let limit = unsafe { pf_session_place_component(session, limit, 20.0, 80.0) };
    let lamp = unsafe { pf_session_place_component(session, lamp, 70.0, 80.0) };
    assert!(rail >= 0 && limit >= 0 && lamp >= 0);
    let rail = u32::try_from(rail).unwrap_or_default();
    let limit = u32::try_from(limit).unwrap_or_default();
    let lamp = u32::try_from(lamp).unwrap_or_default();
    assert_eq!(
        unsafe {
            pf_session_set_chart_source(
                session,
                limit,
                u32::try_from(cylinder).unwrap_or_default(),
                PF_QUANTITY_POSITION,
            )
        },
        PF_OK
    );
    let mut bound = PfChartSource::default();
    // SAFETY: live session and a writable record.
    assert_eq!(
        unsafe { pf_session_chart_source(session, limit, &raw mut bound) },
        1
    );
    assert_eq!(bound.placement, u32::try_from(cylinder).unwrap_or_default());

    // The rail feeds the switch and the switch feeds the lamp, so the lamp shows the state
    // the student would read on the machine.
    let l = CString::new("L").unwrap_or_default();
    let one = CString::new("1").unwrap_or_default();
    let two = CString::new("2").unwrap_or_default();
    // SAFETY: live session and NUL-terminated ports.
    assert_eq!(
        unsafe { pf_session_connect(session, rail, l.as_ptr(), limit, one.as_ptr()) },
        PF_OK
    );
    // SAFETY: live session and NUL-terminated ports.
    assert_eq!(
        unsafe { pf_session_connect(session, limit, two.as_ptr(), lamp, one.as_ptr()) },
        PF_OK
    );
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_indicator_active(session, lamp) }, 0);

    // Pressed, the rod goes out and the roller reaches the far end.
    // SAFETY: live session.
    assert_eq!(
        unsafe { pf_session_press_component_valve(session, 1) },
        PF_OK
    );
    for _ in 0..500 {
        assert_eq!(unsafe { pf_session_advance(session, 0.016) }, PF_OK);
    }
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_indicator_active(session, lamp) }, 1);

    // Released, the rod returns and the switch opens again.
    // SAFETY: live session.
    assert_eq!(
        unsafe { pf_session_release_component_valve(session, 1) },
        PF_OK
    );
    for _ in 0..500 {
        assert_eq!(unsafe { pf_session_advance(session, 0.016) }, PF_OK);
    }
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_indicator_active(session, lamp) }, 0);

    // SAFETY: the session is still alive.
    unsafe { pf_session_free(session) };
}

#[test]
fn a_pressure_switch_follows_its_line_up_and_down() {
    let session = session();
    // The seeded valve's working line 2 feeds the cylinder; the switch reads the same line,
    // so it sees the pressure the student would measure there.
    let rail = catalog_index(session, "electrical.supply");
    let switch = catalog_index(session, "electrical.pressure_switch");
    let lamp = catalog_index(session, "electrical.lamp");
    // SAFETY: live session.
    let rail = unsafe { pf_session_place_component(session, rail, 20.0, 20.0) };
    let switch = unsafe { pf_session_place_component(session, switch, 70.0, 80.0) };
    let lamp = unsafe { pf_session_place_component(session, lamp, 120.0, 80.0) };
    assert!(rail >= 0 && switch >= 0 && lamp >= 0);
    let rail = u32::try_from(rail).unwrap_or_default();
    let switch = u32::try_from(switch).unwrap_or_default();
    let lamp = u32::try_from(lamp).unwrap_or_default();
    let l = CString::new("L").unwrap_or_default();
    let three = CString::new("3").unwrap_or_default();
    let four = CString::new("4").unwrap_or_default();
    let one = CString::new("1").unwrap_or_default();
    let two = CString::new("2").unwrap_or_default();
    // SAFETY: live session and NUL-terminated ports.
    assert_eq!(
        unsafe { pf_session_connect(session, 1, two.as_ptr(), switch, one.as_ptr()) },
        PF_OK
    );
    // SAFETY: live session and NUL-terminated ports.
    assert_eq!(
        unsafe { pf_session_connect(session, rail, l.as_ptr(), switch, three.as_ptr()) },
        PF_OK
    );
    // SAFETY: live session and NUL-terminated ports.
    assert_eq!(
        unsafe { pf_session_connect(session, switch, four.as_ptr(), lamp, one.as_ptr()) },
        PF_OK
    );

    // At rest the line is at atmosphere, below the three-bar threshold.
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_indicator_active(session, lamp) }, 0);
    // SAFETY: live session.
    assert_eq!(
        unsafe { pf_session_press_component_valve(session, 1) },
        PF_OK
    );
    for _ in 0..500 {
        assert_eq!(unsafe { pf_session_advance(session, 0.016) }, PF_OK);
    }
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_indicator_active(session, lamp) }, 1);

    // Exhausted, the line falls below the threshold and the contact opens again.
    // SAFETY: live session.
    assert_eq!(
        unsafe { pf_session_release_component_valve(session, 1) },
        PF_OK
    );
    for _ in 0..500 {
        assert_eq!(unsafe { pf_session_advance(session, 0.016) }, PF_OK);
    }
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_indicator_active(session, lamp) }, 0);

    // SAFETY: the session is still alive.
    unsafe { pf_session_free(session) };
}

#[test]
fn a_normally_closed_contact_opens_when_clicked() {
    let session = session();
    let rail = catalog_index(session, "electrical.supply");
    let button = catalog_index(session, "electrical.contact.nc");
    let lamp = catalog_index(session, "electrical.lamp");
    // SAFETY: live session.
    let rail = unsafe { pf_session_place_component(session, rail, 20.0, 20.0) };
    let button = unsafe { pf_session_place_component(session, button, 70.0, 20.0) };
    let lamp = unsafe { pf_session_place_component(session, lamp, 120.0, 20.0) };
    assert!(rail >= 0 && button >= 0 && lamp >= 0);
    let rail = u32::try_from(rail).unwrap_or_default();
    let button = u32::try_from(button).unwrap_or_default();
    let lamp = u32::try_from(lamp).unwrap_or_default();
    let l = CString::new("L").unwrap_or_default();
    let one = CString::new("1").unwrap_or_default();
    let two = CString::new("2").unwrap_or_default();
    // SAFETY: live session and NUL-terminated ports.
    assert_eq!(
        unsafe { pf_session_connect(session, rail, l.as_ptr(), button, one.as_ptr()) },
        PF_OK
    );
    // SAFETY: live session and NUL-terminated ports.
    assert_eq!(
        unsafe { pf_session_connect(session, button, two.as_ptr(), lamp, one.as_ptr()) },
        PF_OK
    );

    // Normally closed: the lamp is lit before anyone touches it.
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_indicator_active(session, lamp) }, 1);
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_toggle_contact(session, button) }, PF_OK);
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_indicator_active(session, lamp) }, 0);
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_toggle_contact(session, button) }, PF_OK);
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_indicator_active(session, lamp) }, 1);

    // SAFETY: the session is still alive.
    unsafe { pf_session_free(session) };
}

#[test]
fn a_latching_relay_holds_until_the_reset_coil() {
    let session = session();
    let rail = catalog_index(session, "electrical.supply");
    let button = catalog_index(session, "electrical.contact");
    let relay = catalog_index(session, "electrical.relay.latching");
    let lamp = catalog_index(session, "electrical.lamp");
    // SAFETY: live session.
    let rail = unsafe { pf_session_place_component(session, rail, 10.0, 10.0) };
    let set = unsafe { pf_session_place_component(session, button, 50.0, 10.0) };
    let reset = unsafe { pf_session_place_component(session, button, 50.0, 60.0) };
    let relay = unsafe { pf_session_place_component(session, relay, 100.0, 10.0) };
    let lamp = unsafe { pf_session_place_component(session, lamp, 150.0, 10.0) };
    assert!(rail >= 0 && set >= 0 && reset >= 0 && relay >= 0 && lamp >= 0);
    let rail = u32::try_from(rail).unwrap_or_default();
    let set = u32::try_from(set).unwrap_or_default();
    let reset = u32::try_from(reset).unwrap_or_default();
    let relay = u32::try_from(relay).unwrap_or_default();
    let lamp = u32::try_from(lamp).unwrap_or_default();
    let l = CString::new("L").unwrap_or_default();
    let n = CString::new("N").unwrap_or_default();
    let one = CString::new("1").unwrap_or_default();
    let two = CString::new("2").unwrap_or_default();
    let a1 = CString::new("A1").unwrap_or_default();
    let a2 = CString::new("A2").unwrap_or_default();
    let r1 = CString::new("R1").unwrap_or_default();
    let r2 = CString::new("R2").unwrap_or_default();
    let eleven = CString::new("11").unwrap_or_default();
    let fourteen = CString::new("14").unwrap_or_default();
    for (from, from_port, to, to_port) in [
        (rail, &l, set, &one),
        (set, &two, relay, &a1),
        (relay, &a2, rail, &n),
        (rail, &l, reset, &one),
        (reset, &two, relay, &r1),
        (relay, &r2, rail, &n),
        (rail, &l, relay, &eleven),
        (relay, &fourteen, lamp, &one),
    ] {
        // SAFETY: live session and NUL-terminated ports.
        assert_eq!(
            unsafe { pf_session_connect(session, from, from_port.as_ptr(), to, to_port.as_ptr()) },
            PF_OK
        );
    }

    // At rest the contact is open.
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_indicator_active(session, lamp) }, 0);
    // A pulse on the set coil closes it, and releasing the button does not open it.
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_toggle_contact(session, set) }, PF_OK);
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_indicator_active(session, lamp) }, 1);
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_toggle_contact(session, set) }, PF_OK);
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_indicator_active(session, lamp) }, 1);
    // A pulse on the reset coil opens it, and releasing it stays open.
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_toggle_contact(session, reset) }, PF_OK);
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_indicator_active(session, lamp) }, 0);
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_toggle_contact(session, reset) }, PF_OK);
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_indicator_active(session, lamp) }, 0);

    // SAFETY: the session is still alive.
    unsafe { pf_session_free(session) };
}

#[test]
fn a_new_valve_keeps_the_symbol_square() {
    let session = session();
    let valve = catalog_index(session, "valve.3_2.spring_return");
    // SAFETY: live session.
    let first = unsafe { pf_session_place_component(session, valve, 60.0, 60.0) };
    let second = unsafe { pf_session_place_component(session, valve, 120.0, 60.0) };
    assert!(first >= 0 && second >= 0);
    let first = u32::try_from(first).unwrap_or_default();
    let second = u32::try_from(second).unwrap_or_default();
    let rect = |index: u32| {
        let mut bounds = [0.0_f32; 4];
        // SAFETY: live session and a writable array.
        assert_eq!(
            unsafe { pf_session_component_rect(session, index, bounds.as_mut_ptr()) },
            PF_OK
        );
        bounds
    };
    let first_before = rect(first);
    let second_rect = rect(second);
    let width = first_before[2] - first_before[0];
    let height = first_before[3] - first_before[1];
    assert!(
        (width - height).abs() < 0.01,
        "the first valve is not square: {width:.1} x {height:.1}"
    );
    let second_width = second_rect[2] - second_rect[0];
    let second_height = second_rect[3] - second_rect[1];
    assert!(
        (second_width - second_height).abs() < 0.01,
        "the second valve is not square: {second_width:.1} x {second_height:.1}"
    );
    let first_after = rect(first);
    assert!(
        first_before
            .iter()
            .zip(&first_after)
            .all(|(before, after)| (before - after).abs() < 1.0e-6),
        "placing a second valve changed the first one: {first_before:?} -> {first_after:?}"
    );
    // SAFETY: the session is still alive.
    unsafe { pf_session_free(session) };
}
