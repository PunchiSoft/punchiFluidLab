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
    PF_ABI_VERSION, PF_EDIT_BORE, PF_EDIT_ROD, PF_EDIT_STROKE, PF_ERR_MODEL, PF_HAS_STROKE,
    PF_KIND_TEXT, PF_OK, PfAbi, PfCall, PfCatalog, PfEditSet, PfFrame, PfInk, PfPoint, PfSession,
    PfTelemetry, pf_abi, pf_session_advance, pf_session_build_frame, pf_session_can_undo,
    pf_session_catalog, pf_session_check, pf_session_component_count, pf_session_component_edits,
    pf_session_component_id, pf_session_component_orientation, pf_session_component_rect,
    pf_session_component_valve_position, pf_session_component_valve_spring_return,
    pf_session_connect, pf_session_connection_count, pf_session_copy_selected,
    pf_session_duplicate_component, pf_session_duplicate_selection, pf_session_finding_at,
    pf_session_free, pf_session_hit_component, pf_session_hit_valve, pf_session_last_error,
    pf_session_mirror_selected, pf_session_move_component, pf_session_move_selection,
    pf_session_new, pf_session_open, pf_session_paste, pf_session_paste_offset, pf_session_pipe_at,
    pf_session_pipe_offset, pf_session_place_component, pf_session_port_point,
    pf_session_press_component_valve, pf_session_press_valve, pf_session_preview, pf_session_redo,
    pf_session_release_component_valve, pf_session_release_valve, pf_session_remove_component,
    pf_session_remove_selected_pipe, pf_session_remove_selection, pf_session_restart,
    pf_session_rotate_selected, pf_session_route_pipe, pf_session_save, pf_session_select,
    pf_session_select_all, pf_session_select_pipe, pf_session_select_rect, pf_session_selected,
    pf_session_selected_pipe, pf_session_selection_at, pf_session_selection_count,
    pf_session_set_component_edits, pf_session_telemetry, pf_session_toggle_select,
    pf_session_toggle_valve, pf_session_undo, pf_session_valve_spring_return,
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

    // The valve sits at 47 % of the width and 72 % of the height of the fixed layout.
    // SAFETY: live session.
    assert_eq!(
        unsafe { pf_session_hit_valve(session, 200.0 * 0.47, 130.0 * 0.72) },
        1
    );
    // The supply box is elsewhere, so a click there must not reach the valve.
    // SAFETY: live session.
    assert_eq!(
        unsafe { pf_session_hit_valve(session, 200.0 * 0.17, 130.0 * 0.30) },
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
    assert!(unsafe { CStr::from_ptr(message) }.to_bytes().is_empty());
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
        assert!(!field(item.name_at, item.name_len).is_empty());
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
    assert_eq!(seeded, 3, "the sheet starts with the simulated circuit");

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
    // The starting circuit is seeded at creation — three components and the two hoses that
    // join them — so it exists before any frame is drawn, which is what lets a front end
    // connect before it paints.
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_component_count(session) }, 3);
    // SAFETY: live session.
    let seeded = unsafe { pf_session_connection_count(session) };
    assert_eq!(seeded, 2, "the starting circuit is drawn with its hoses");

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
        unsafe { pf_session_route_pipe(session, 0, 40.0, 70.0) },
        PF_OK
    );
    let mut routed = 0.0_f32;
    // SAFETY: live session and a writable float.
    assert_eq!(
        unsafe { pf_session_pipe_offset(session, 0, &raw mut routed) },
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
    assert_eq!(unsafe { pf_session_connection_count(session) }, 2);
    let mut offset = 0.0_f32;
    // SAFETY: live session and a writable float.
    assert_eq!(
        unsafe { pf_session_pipe_offset(session, 0, &raw mut offset) },
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
    assert_eq!(unsafe { pf_session_component_count(session) }, 3);
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
        unsafe { pf_session_pipe_offset(session, 0, &raw mut offset) },
        PF_OK
    );
    assert!(
        offset.abs() < 1e-6,
        "a new hose runs on the automatic route"
    );

    // Drag the middle segment well away from where it was.
    // SAFETY: live session.
    assert_eq!(
        unsafe { pf_session_route_pipe(session, 0, 40.0, 70.0) },
        PF_OK
    );
    // SAFETY: live session and a writable float.
    assert_eq!(
        unsafe { pf_session_pipe_offset(session, 0, &raw mut offset) },
        PF_OK
    );
    assert!(offset.abs() > 1e-6, "the drag has to move the segment");

    // One drag is one undo entry, and it restores the route exactly.
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_undo(session) }, PF_OK);
    // SAFETY: live session and a writable float.
    assert_eq!(
        unsafe { pf_session_pipe_offset(session, 0, &raw mut offset) },
        PF_OK
    );
    assert!(offset.abs() < 1e-6);
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_redo(session) }, PF_OK);
    // SAFETY: live session and a writable float.
    assert_eq!(
        unsafe { pf_session_pipe_offset(session, 0, &raw mut offset) },
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
    // SAFETY: live session; the starting circuit has supply, valve and cylinder.
    assert_eq!(unsafe { pf_session_component_count(session) }, 3);
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
    // SAFETY: live session; the starting circuit has three components.
    assert_eq!(unsafe { pf_session_component_count(session) }, 3);
    // Ctrl+E marks everything, and the last one marked is the primary the forms open on.
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_select_all(session) }, 3);
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_selection_count(session) }, 3);
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_selected(session) }, 2);
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
        3
    );
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_component_count(session) }, 6);
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_selection_count(session) }, 3);
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_undo(session) }, PF_OK);
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_component_count(session) }, 3);

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
    assert_eq!(unsafe { pf_session_component_count(session) }, 2);
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_undo(session) }, PF_OK);
    // SAFETY: live session.
    assert_eq!(unsafe { pf_session_component_count(session) }, 3);
    // SAFETY: the session is still alive.
    unsafe { pf_session_free(session) };
}
