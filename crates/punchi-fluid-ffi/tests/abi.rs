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

use std::ffi::{CStr, CString};
use std::ptr;

use punchi_fluid::{
    PF_ABI_VERSION, PF_EDIT_BORE, PF_EDIT_ROD, PF_EDIT_STROKE, PF_ERR_MODEL,
    PF_EXPORT_CUSTOM_PAPER, PF_EXPORT_PDF, PF_EXPORT_SVG, PF_HAS_STROKE, PF_KIND_LINE,
    PF_KIND_TEXT, PF_OK, PF_QUANTITY_FLOW, PF_QUANTITY_POSITION, PF_QUANTITY_PRESSURE, PfAbi,
    PfCall, PfCatalog, PfChartSource, PfEditSet, PfExportOptions, PfFrame, PfInk, PfPoint,
    PfSession, PfTelemetry, pf_abi, pf_session_active_circuit, pf_session_add_circuit,
    pf_session_advance, pf_session_align_selection, pf_session_autosave, pf_session_build_frame,
    pf_session_buzzer_active, pf_session_can_undo, pf_session_catalog, pf_session_chart_candidate,
    pf_session_chart_candidate_count, pf_session_chart_source, pf_session_check,
    pf_session_circuit_count, pf_session_circuit_id, pf_session_circuit_name,
    pf_session_clear_chart_source, pf_session_component_count, pf_session_component_designation,
    pf_session_component_edits, pf_session_component_id, pf_session_component_orientation,
    pf_session_component_rect, pf_session_component_valve_position,
    pf_session_component_valve_spring_return, pf_session_connect, pf_session_connection_count,
    pf_session_contact_closed, pf_session_copy_selected, pf_session_describe_component,
    pf_session_duplicate_component, pf_session_duplicate_selection, pf_session_export,
    pf_session_export_with, pf_session_finding_at, pf_session_forget_recent, pf_session_free,
    pf_session_hit_component, pf_session_hit_valve, pf_session_indicator_active,
    pf_session_is_modified, pf_session_last_error, pf_session_mirror_selected,
    pf_session_move_component, pf_session_move_selection, pf_session_new, pf_session_open,
    pf_session_open_recent, pf_session_paste, pf_session_paste_offset, pf_session_pipe_at,
    pf_session_pipe_class, pf_session_pipe_offset, pf_session_place_component,
    pf_session_placement_id, pf_session_port_point, pf_session_press_component_valve,
    pf_session_press_valve, pf_session_preview, pf_session_recent_count, pf_session_recent_path,
    pf_session_recover, pf_session_redo, pf_session_release_component_valve,
    pf_session_release_valve, pf_session_remove_component, pf_session_remove_selected_pipe,
    pf_session_remove_selection, pf_session_rename_circuit, pf_session_restart,
    pf_session_rotate_selected, pf_session_route_pipe, pf_session_save, pf_session_select,
    pf_session_select_all, pf_session_select_circuit, pf_session_select_pipe,
    pf_session_select_rect, pf_session_selected, pf_session_selected_pipe, pf_session_selection_at,
    pf_session_selection_count, pf_session_set_chart_source, pf_session_set_component_designation,
    pf_session_set_component_edits, pf_session_set_pipe_class, pf_session_set_recent_file,
    pf_session_space_selection, pf_session_telemetry, pf_session_toggle_contact,
    pf_session_toggle_select, pf_session_toggle_valve, pf_session_undo,
    pf_session_valve_spring_return,
};

fn session() -> *mut PfSession {
    // SAFETY: a null directory asks for the built-in catalog, which is what a front end
    // that has no user directory passes.
    let session = unsafe { pf_session_new(ptr::null()) };
    assert!(!session.is_null(), "a session must always be created");
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
    let code = unsafe { pf_session_build_frame(session, 200.0, 130.0, 4.0, &raw mut frame) };
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
    unsafe { pf_session_build_frame(session, 200.0, 130.0, 4.0, &raw mut frame) };

    // The valve sits at 62 % of the width and 52 % of the height of the fixed layout.
    // SAFETY: live session.
    assert_eq!(
        unsafe { pf_session_hit_valve(session, 200.0 * 0.62, 130.0 * 0.52) },
        1
    );
    // The supply box is elsewhere, so a click there must not reach the valve.
    // SAFETY: live session.
    assert_eq!(
        unsafe { pf_session_hit_valve(session, 200.0 * 0.13, 130.0 * 0.20) },
        0
    );
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
fn a_component_dropped_on_the_sheet_can_be_moved_and_undone() {
    let session = session();
    let mut frame = PfFrame::default();
    // SAFETY: live session, writable output. Seeding happens on the first frame.
    assert_eq!(
        unsafe { pf_session_build_frame(session, 200.0, 130.0, 4.0, &raw mut frame) },
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
        unsafe { pf_session_build_frame(session, 200.0, 130.0, 4.0, &raw mut frame) },
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
        (1.0..=10.0).contains(&value),
        "the gauge reads `{reading}`, not a pressure on its 0..10 bar scale"
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

    // Halfway along the routed pipe: the run is an elbow, so the middle of the bounding
    // box is never far from it. Walk a small grid and accept the first hit.
    let mut found = -1;
    for step in 0..40 {
        #[allow(clippy::cast_precision_loss)]
        let x = 20.0 + step as f32 * 3.0;
        for row in 0..40 {
            #[allow(clippy::cast_precision_loss)]
            let y = 20.0 + row as f32 * 3.0;
            // SAFETY: live session.
            let hit = unsafe { pf_session_pipe_at(session, x, y, 2.5) };
            if hit >= 0 {
                found = hit;
                break;
            }
        }
        if found >= 0 {
            break;
        }
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

    // A second supply dropped on the sheet hangs in the air: its port has no pipe.
    // SAFETY: live session.
    assert!(unsafe { pf_session_place_component(session, 0, 30.0, 110.0) } >= 0);
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
        for index in 0..4_u32 {
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

    // A document that vanished is forgotten instead of offered, and the list on disk keeps
    // only what can still be opened.
    std::fs::remove_file(&second).expect("removes");
    // The document on the sheet is not modified by this, so opening the first is allowed.
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_open_recent(session, 0) }, PF_ERR_MODEL);
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_recent_count(session) }, 1);
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_open_recent(session, 0) }, PF_OK);
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
    assert_eq!(unsafe { pf_session_recent_count(session) }, 1);
    assert!(path_of(session, 0).ends_with("uno.pfluid"));
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
    assert!(pdf_shows(&pdf, "1:"), "the title block records the scale");
    assert!(pdf_shows(&pdf, "Circuito 1"), "the circuit names the page");
    assert!(
        pdf_shows(&pdf, "lista de piezas"),
        "the sheet carries its parts list"
    );

    // SAFETY: the session is still alive.
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
    let mut to = [0.0_f32; 2];
    // SAFETY: live session and a writable pair.
    assert_eq!(
        unsafe { pf_session_port_point(session, 2, one.as_ptr(), to.as_mut_ptr()) },
        PF_OK
    );
    // The route of a hose is a Z: its vertical segment runs through the middle of the
    // chord, which is the point `pf_session_pipe_at` is given here.
    let middle = (f32::midpoint(from[0], to[0]), f32::midpoint(from[1], to[1]));
    // SAFETY: live session.
    let hose = unsafe { pf_session_pipe_at(session, middle.0, middle.1, 1.0) };
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
        unsafe { pf_session_build_frame(session, 200.0, 130.0, 4.0, &raw mut frame) },
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
        unsafe { pf_session_build_frame(session, 200.0, 130.0, 4.0, &raw mut frame) },
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
    for text in ["2026-11-12", "Circuito 1", "lista de piezas", "Cilindro"] {
        assert!(svg.contains(text), "the sheet is missing {text}");
    }
    // SAFETY: the session is still alive.
    unsafe { pf_session_free(session) };
}

#[test]
fn the_export_panel_settings_reach_the_page() {
    let session = session();
    let directory = tempfile::tempdir().expect("a temporary folder");
    let path = directory.path().join("panel.svg");
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
    assert!(
        !svg.contains("lista de piezas"),
        "the panel turned the list off"
    );

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
