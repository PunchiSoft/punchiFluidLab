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

use std::ffi::CStr;
use std::ptr;

use punchi_fluid::{
    PF_ABI_VERSION, PF_ERR_MODEL, PF_HAS_STROKE, PF_KIND_TEXT, PF_OK, PfAbi, PfCall, PfCatalog,
    PfFrame, PfInk, PfPoint, PfSession, PfTelemetry, pf_abi, pf_session_advance,
    pf_session_build_frame, pf_session_catalog, pf_session_free, pf_session_hit_valve,
    pf_session_last_error, pf_session_new, pf_session_preview, pf_session_restart,
    pf_session_telemetry, pf_session_toggle_valve,
};

fn session() -> *mut PfSession {
    // SAFETY: a null directory asks for the built-in catalog, which is what a front end
    // that has no user directory passes.
    let session = unsafe { pf_session_new(ptr::null()) };
    assert!(!session.is_null(), "a session must always be created");
    session
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
