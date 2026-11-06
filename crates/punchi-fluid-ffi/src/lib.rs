// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The C ABI a native front end speaks.
//!
//! The Rust core owns the model and the drawing; this crate turns both into a flat
//! `repr(C)` description that a C++ or QML window replays. Two decisions shape it:
//!
//! - **The drawing crosses as a command buffer, not as callbacks.** One call per frame
//!   fills a [`PfFrame`] with [`PfCall`] records that the front end replays onto a
//!   `QPainter`. The screen then consumes the same primitive list the export tests
//!   already compare, so the window and the paper cannot drift apart.
//! - **Colour comes in from outside.** [`PfInk`] is filled by the front end from the
//!   desktop theme. Nothing here picks a colour, which is the rule the UI already
//!   lives by.
//!
//! Dash patterns are expanded here, in millimetres, by the same function the PDF and
//! PNG backends use. A backend that re-invented them would be a second drawing.
//!
//! Nothing may unwind across the boundary: every entry point catches unwinding and
//! reports a code. Pointers handed out by [`pf_session_build_frame`] point into the
//! session and stay valid until the next rebuild or [`pf_session_free`].

#![allow(
    unsafe_code,
    reason = "a C ABI dereferences caller pointers and is entered with no Rust frame above it; the workspace forbids unsafe everywhere else, and this is the only crate that opts out"
)]
#![deny(clippy::undocumented_unsafe_blocks)]
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    reason = "lengths and counts are bounded by one frame of drawing, far below u32::MAX"
)]

use std::ffi::{CStr, CString, c_char};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::{Path, PathBuf};

use punchi_fluid_catalog::Catalog;
use punchi_fluid_draw::surface::{Anchor, Colour, LineClass, Pt, Rect, Stroke, Surface};
use punchi_fluid_draw::symbol::Ink;
use punchi_fluid_draw::{Layout, dashes};
use punchi_fluid_solver::pneumatic::{PneumaticSystem, Telemetry};

/// The call succeeded.
pub const PF_OK: i32 = 0;
/// A required pointer was null.
pub const PF_ERR_NULL: i32 = -1;
/// The model refused the request; see [`pf_session_last_error`].
pub const PF_ERR_MODEL: i32 = -2;
/// A panic was caught at the boundary.
pub const PF_ERR_PANIC: i32 = -3;

/// Primitive kinds carried by [`PfCall::kind`].
pub const PF_KIND_LINE: u32 = 0;
/// Closed polygon.
pub const PF_KIND_POLYGON: u32 = 1;
/// Rectangle, optionally rounded.
pub const PF_KIND_RECT: u32 = 2;
/// Circle.
pub const PF_KIND_CIRCLE: u32 = 3;
/// Arc.
pub const PF_KIND_ARC: u32 = 4;
/// Text run.
pub const PF_KIND_TEXT: u32 = 5;

/// [`PfCall::flags`] bit: the call carries a fill colour.
pub const PF_HAS_FILL: u32 = 1;
/// [`PfCall::flags`] bit: the call carries a stroke.
pub const PF_HAS_STROKE: u32 = 2;

/// RGB triplet, eight bits per channel, sRGB.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PfRgb {
    /// Red.
    pub r: u8,
    /// Green.
    pub g: u8,
    /// Blue.
    pub b: u8,
}

impl PfRgb {
    const BLACK: Self = Self { r: 0, g: 0, b: 0 };

    const fn from_colour(colour: Colour) -> Self {
        Self {
            r: colour.r,
            g: colour.g,
            b: colour.b,
        }
    }

    const fn to_colour(self) -> Colour {
        Colour {
            r: self.r,
            g: self.g,
            b: self.b,
        }
    }
}

/// Every colour the drawing needs, in the order the front end fills them.
///
/// The front end reads these from the desktop theme; the Rust side never derives one,
/// so a light desktop and a dark desktop produce different ink from the same drawing.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PfInk {
    /// Lines of a symbol.
    pub line: PfRgb,
    /// Lines of an inactive position.
    pub dim: PfRgb,
    /// Secondary text.
    pub muted: PfRgb,
    /// Primary text.
    pub text: PfRgb,
    /// Supply line.
    pub supply: PfRgb,
    /// Working line.
    pub work: PfRgb,
    /// Active square.
    pub accent: PfRgb,
    /// Paper behind the drawing.
    pub background: PfRgb,
}

impl PfInk {
    fn to_ink(self) -> Ink {
        Ink {
            line: self.line.to_colour(),
            dim: self.dim.to_colour(),
            muted: self.muted.to_colour(),
            text: self.text.to_colour(),
            supply: self.supply.to_colour(),
            work: self.work.to_colour(),
            accent: self.accent.to_colour(),
            background: self.background.to_colour(),
        }
    }
}

/// The state the front end reads every frame, in SI plus the valve position.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct PfTelemetry {
    /// Simulated time, `s`.
    pub time_s: f64,
    /// Absolute cylinder pressure, `Pa`.
    pub pressure_pa: f64,
    /// Piston position, `m`.
    pub position_m: f64,
    /// Piston velocity, `m/s`.
    pub velocity_m_s: f64,
    /// Gas mass in, `kg`.
    pub mass_in_kg: f64,
    /// Gas mass out, `kg`.
    pub mass_out_kg: f64,
    /// Active valve position.
    pub valve_position: u32,
    /// `1` when a model is loaded, `0` when the catalog had no usable definition.
    pub has_model: u32,
}

/// One primitive of one frame.
///
/// Fields that the `kind` does not use are zero. `points`/`text` are ranges into the
/// frame's arenas, not pointers, so the whole frame is one allocation to copy.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct PfCall {
    /// One of the `PF_KIND_*` values.
    pub kind: u32,
    /// `PF_HAS_FILL`, `PF_HAS_STROKE`, or both.
    pub flags: u32,
    /// Line class: 0 solid, 1 pilot, 2 drain, 3 electrical. Informational.
    pub line_class: u32,
    /// Text anchor: 0 start, 1 middle, 2 end.
    pub anchor: u32,
    /// First index into `PfFrame::points`.
    pub points_at: u32,
    /// Number of points.
    pub points_len: u32,
    /// Byte offset into `PfFrame::text`.
    pub text_at: u32,
    /// Byte length of the UTF-8 run.
    pub text_len: u32,
    /// Rectangle `x0 y0 x1 y1`; for a circle or an arc, `cx cy r 0`.
    pub rect: [f32; 4],
    /// Arc start and end angle, degrees clockwise.
    pub angles: [f32; 2],
    /// Corner radius of a rectangle, millimetres.
    pub corner: f32,
    /// Cap height of a text run, millimetres.
    pub size: f32,
    /// Stroke width, millimetres.
    pub width: f32,
    /// Stroke colour.
    pub stroke: PfRgb,
    /// Fill colour, and the colour of a text run.
    pub fill: PfRgb,
}

/// A point in millimetres of the drawing.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct PfPoint {
    /// Horizontal.
    pub x: f32,
    /// Vertical, downwards.
    pub y: f32,
}

/// One frame of drawing, borrowed from the session.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct PfFrame {
    /// Primitive list.
    pub calls: *const PfCall,
    /// Number of primitives.
    pub calls_len: u32,
    /// Point arena.
    pub points: *const PfPoint,
    /// Number of points.
    pub points_len: u32,
    /// UTF-8 text arena.
    pub text: *const u8,
    /// Bytes of text.
    pub text_len: u32,
    /// Drawing width in millimetres, as the model laid it out.
    pub width_mm: f32,
    /// Drawing height in millimetres.
    pub height_mm: f32,
}

/// Sizes of the structures below, so the C side can prove it agrees with Rust.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PfAbi {
    /// Bumped whenever a structure changes shape.
    pub version: u32,
    /// `sizeof(PfCall)`.
    pub call_size: u32,
    /// `sizeof(PfTelemetry)`.
    pub telemetry_size: u32,
    /// `sizeof(PfFrame)`.
    pub frame_size: u32,
    /// `sizeof(PfInk)`.
    pub ink_size: u32,
    /// `sizeof(PfPoint)`.
    pub point_size: u32,
}

/// The ABI layout this build produces.
pub const PF_ABI_VERSION: u32 = 1;

fn rgb(colour: Option<Colour>) -> PfRgb {
    colour.map_or(PfRgb::BLACK, PfRgb::from_colour)
}

fn class_code(class: LineClass) -> u32 {
    match class {
        LineClass::Working => 0,
        LineClass::Pilot => 1,
        LineClass::Drain => 2,
        LineClass::Electrical => 3,
    }
}

fn anchor_code(anchor: Anchor) -> u32 {
    match anchor {
        Anchor::Start => 0,
        Anchor::Middle => 1,
        Anchor::End => 2,
    }
}

/// Writes the drawing into the flat frame the front end replays.
struct FrameBuffer<'a> {
    calls: &'a mut Vec<PfCall>,
    points: &'a mut Vec<PfPoint>,
    text: &'a mut Vec<u8>,
    scale: f32,
}

impl FrameBuffer<'_> {
    fn push_points(&mut self, source: &[Pt]) -> (u32, u32) {
        let at = self.points.len() as u32;
        self.points.extend(source.iter().map(|point| PfPoint {
            x: point.x,
            y: point.y,
        }));
        (at, source.len() as u32)
    }
}

impl Surface for FrameBuffer<'_> {
    fn line(&mut self, points: &[Pt], stroke: Stroke) {
        // The pattern is expanded here so the window, the PNG and the PDF consume the
        // same segments instead of three dash implementations that can disagree.
        for run in dashes(points, stroke.class.pattern()) {
            let (points_at, points_len) = self.push_points(&run);
            self.calls.push(PfCall {
                kind: PF_KIND_LINE,
                flags: PF_HAS_STROKE,
                line_class: class_code(stroke.class),
                points_at,
                points_len,
                width: stroke.width,
                stroke: PfRgb::from_colour(stroke.colour),
                ..PfCall::default()
            });
        }
    }

    fn polygon(&mut self, points: &[Pt], fill: Option<Colour>, stroke: Option<Stroke>) {
        let (points_at, points_len) = self.push_points(points);
        let mut flags = 0;
        if fill.is_some() {
            flags |= PF_HAS_FILL;
        }
        if stroke.is_some() {
            flags |= PF_HAS_STROKE;
        }
        self.calls.push(PfCall {
            kind: PF_KIND_POLYGON,
            flags,
            points_at,
            points_len,
            width: stroke.map_or(0.0, |stroke| stroke.width),
            line_class: stroke.map_or(0, |stroke| class_code(stroke.class)),
            stroke: rgb(stroke.map(|stroke| stroke.colour)),
            fill: rgb(fill),
            ..PfCall::default()
        });
    }

    fn rect(&mut self, rect: Rect, corner: f32, fill: Option<Colour>, stroke: Option<Stroke>) {
        let mut flags = 0;
        if fill.is_some() {
            flags |= PF_HAS_FILL;
        }
        if stroke.is_some() {
            flags |= PF_HAS_STROKE;
        }
        self.calls.push(PfCall {
            kind: PF_KIND_RECT,
            flags,
            rect: [rect.min.x, rect.min.y, rect.max.x, rect.max.y],
            corner,
            width: stroke.map_or(0.0, |stroke| stroke.width),
            line_class: stroke.map_or(0, |stroke| class_code(stroke.class)),
            stroke: rgb(stroke.map(|stroke| stroke.colour)),
            fill: rgb(fill),
            ..PfCall::default()
        });
    }

    fn circle(&mut self, centre: Pt, radius: f32, fill: Option<Colour>, stroke: Option<Stroke>) {
        let mut flags = 0;
        if fill.is_some() {
            flags |= PF_HAS_FILL;
        }
        if stroke.is_some() {
            flags |= PF_HAS_STROKE;
        }
        self.calls.push(PfCall {
            kind: PF_KIND_CIRCLE,
            flags,
            rect: [centre.x, centre.y, radius, 0.0],
            width: stroke.map_or(0.0, |stroke| stroke.width),
            line_class: stroke.map_or(0, |stroke| class_code(stroke.class)),
            stroke: rgb(stroke.map(|stroke| stroke.colour)),
            fill: rgb(fill),
            ..PfCall::default()
        });
    }

    fn arc(&mut self, centre: Pt, radius: f32, start: f32, end: f32, stroke: Stroke) {
        self.calls.push(PfCall {
            kind: PF_KIND_ARC,
            flags: PF_HAS_STROKE,
            rect: [centre.x, centre.y, radius, 0.0],
            angles: [start, end],
            width: stroke.width,
            line_class: class_code(stroke.class),
            stroke: PfRgb::from_colour(stroke.colour),
            ..PfCall::default()
        });
    }

    fn text(&mut self, at: Pt, text: &str, size: f32, colour: Colour, anchor: Anchor) {
        let text_at = self.text.len() as u32;
        self.text.extend_from_slice(text.as_bytes());
        self.calls.push(PfCall {
            kind: PF_KIND_TEXT,
            flags: PF_HAS_FILL,
            anchor: anchor_code(anchor),
            text_at,
            text_len: text.len() as u32,
            rect: [at.x, at.y, 0.0, 0.0],
            size,
            fill: PfRgb::from_colour(colour),
            ..PfCall::default()
        });
    }

    fn device_scale(&self) -> f32 {
        self.scale
    }
}

/// One simulation and one drawing, owned by the front end through an opaque pointer.
pub struct PfSession {
    catalog: Catalog,
    system: Option<PneumaticSystem>,
    ink: Ink,
    calls: Vec<PfCall>,
    points: Vec<PfPoint>,
    text: Vec<u8>,
    layout: Option<Layout>,
    error: CString,
    width_mm: f32,
    height_mm: f32,
    scale: f32,
}

impl std::fmt::Debug for PfSession {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("PfSession")
            .field("has_model", &self.system.is_some())
            .field("calls", &self.calls.len())
            .finish_non_exhaustive()
    }
}

impl PfSession {
    fn new(user_dir: Option<&Path>) -> Self {
        let catalog = match user_dir {
            Some(dir) => Catalog::builtin().with_user_dir(dir),
            None => Catalog::builtin(),
        };
        let (system, error) = match PneumaticSystem::from_catalog(&catalog) {
            Ok(system) => (Some(system), CString::default()),
            Err(cause) => (None, cstring(&cause.to_string())),
        };
        Self {
            catalog,
            system,
            ink: Ink::print(),
            calls: Vec::new(),
            points: Vec::new(),
            text: Vec::new(),
            layout: None,
            error,
            width_mm: 200.0,
            height_mm: 130.0,
            scale: 1.0,
        }
    }

    fn fail(&mut self, message: &str) -> i32 {
        self.error = cstring(message);
        PF_ERR_MODEL
    }

    fn telemetry(&self) -> PfTelemetry {
        let Some(system) = self.system.as_ref() else {
            return PfTelemetry::default();
        };
        let Telemetry {
            time,
            pressure_pa,
            position,
            velocity,
            valve_position,
            mass_in,
            mass_out,
        } = system.telemetry();
        PfTelemetry {
            time_s: time,
            pressure_pa,
            position_m: position,
            velocity_m_s: velocity,
            mass_in_kg: mass_in,
            mass_out_kg: mass_out,
            valve_position: valve_position as u32,
            has_model: 1,
        }
    }

    fn advance(&mut self, dt: f64) -> i32 {
        let Some(system) = self.system.as_mut() else {
            return self.fail("no model is loaded");
        };
        match system.advance(dt) {
            Ok(_) => PF_OK,
            Err(cause) => self.fail(&cause.to_string()),
        }
    }

    fn build_frame(&mut self, width_mm: f32, height_mm: f32, scale: f32) {
        self.width_mm = width_mm.max(1.0);
        self.height_mm = height_mm.max(1.0);
        self.scale = scale.max(0.01);
        self.calls.clear();
        self.points.clear();
        self.text.clear();
        let area = Rect::from_min_size(Pt::new(0.0, 0.0), self.width_mm, self.height_mm);
        let Self {
            catalog,
            system,
            ink,
            calls,
            points,
            text,
            layout,
            scale,
            ..
        } = self;
        let Some(system) = system.as_ref() else {
            *layout = None;
            return;
        };
        let mut buffer = FrameBuffer {
            calls,
            points,
            text,
            scale: *scale,
        };
        *layout = punchi_fluid_draw::draw(&mut buffer, area, system, catalog, ink);
    }

    /// Hit test against the layout of the last built frame, so the click target is
    /// exactly what the user sees rather than a second recomputation.
    fn hit_valve(&self, x: f32, y: f32) -> bool {
        self.layout
            .is_some_and(|layout| layout.valve.expand(2.0).contains(Pt::new(x, y)))
    }
}

fn cstring(message: &str) -> CString {
    CString::new(message.replace('\0', " ")).unwrap_or_default()
}

fn guard(body: impl FnOnce() -> i32) -> i32 {
    catch_unwind(AssertUnwindSafe(body)).unwrap_or(PF_ERR_PANIC)
}

/// # Safety
///
/// `session` must be null or a pointer returned by [`pf_session_new`] that has not been
/// freed. The returned reference may not outlive that session.
unsafe fn borrow<'a>(session: *mut PfSession) -> Option<&'a mut PfSession> {
    // SAFETY: the caller upholds the contract above.
    unsafe { session.as_mut() }
}

/// # Safety
///
/// `session` must be null or a pointer returned by [`pf_session_new`] that has not been
/// freed. The returned reference may not outlive that session.
unsafe fn borrow_shared<'a>(session: *const PfSession) -> Option<&'a PfSession> {
    // SAFETY: the caller upholds the contract above.
    unsafe { session.as_ref() }
}

/// Library version as a NUL-terminated string. Never null, never freed.
#[unsafe(no_mangle)]
pub extern "C" fn pf_version() -> *const c_char {
    concat!(env!("CARGO_PKG_VERSION"), "\0").as_ptr().cast()
}

/// Fills `out` with the layout of the structures this build produces.
///
/// # Safety
///
/// `out` must be null or point to a writable [`PfAbi`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_abi(out: *mut PfAbi) -> i32 {
    guard(|| {
        // SAFETY: the caller promises a writable structure.
        let Some(out) = (unsafe { out.as_mut() }) else {
            return PF_ERR_NULL;
        };
        *out = PfAbi {
            version: PF_ABI_VERSION,
            call_size: std::mem::size_of::<PfCall>() as u32,
            telemetry_size: std::mem::size_of::<PfTelemetry>() as u32,
            frame_size: std::mem::size_of::<PfFrame>() as u32,
            ink_size: std::mem::size_of::<PfInk>() as u32,
            point_size: std::mem::size_of::<PfPoint>() as u32,
        };
        PF_OK
    })
}

/// Creates a session, optionally over a directory of user catalog files.
///
/// Returns null on failure. Never null for a valid call: a catalog that yields no model
/// still produces a session whose `has_model` is `0` and whose error can be read.
///
/// # Safety
///
/// `user_catalog_dir` must be null or a valid NUL-terminated UTF-8 string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_session_new(user_catalog_dir: *const c_char) -> *mut PfSession {
    catch_unwind(AssertUnwindSafe(|| {
        let dir = if user_catalog_dir.is_null() {
            None
        } else {
            // SAFETY: the caller promises a NUL-terminated string.
            let text = unsafe { CStr::from_ptr(user_catalog_dir) };
            text.to_str()
                .ok()
                .filter(|text| !text.is_empty())
                .map(PathBuf::from)
        };
        Box::into_raw(Box::new(PfSession::new(dir.as_deref())))
    }))
    .unwrap_or(std::ptr::null_mut())
}

/// Frees a session. A null pointer is ignored.
///
/// # Safety
///
/// `session` must be null or a pointer returned by [`pf_session_new`], and it must not
/// be used again afterwards.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_session_free(session: *mut PfSession) {
    if session.is_null() {
        return;
    }
    // SAFETY: the caller promises ownership of a pointer from `pf_session_new`.
    drop(unsafe { Box::from_raw(session) });
}

/// Last error message of the session, NUL-terminated. Valid until the next failing call.
///
/// # Safety
///
/// `session` must be null or a live session pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_session_last_error(session: *const PfSession) -> *const c_char {
    // SAFETY: the caller promises a live session pointer.
    unsafe { borrow_shared(session) }.map_or(std::ptr::null(), |session| session.error.as_ptr())
}

/// Reads the state of the model.
///
/// # Safety
///
/// `session` must be null or a live session pointer, and `out` must be null or writable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_session_telemetry(
    session: *const PfSession,
    out: *mut PfTelemetry,
) -> i32 {
    guard(|| {
        // SAFETY: the caller upholds both contracts.
        let (Some(session), Some(out)) =
            (unsafe { borrow_shared(session) }, unsafe { out.as_mut() })
        else {
            return PF_ERR_NULL;
        };
        *out = session.telemetry();
        PF_OK
    })
}

/// Replaces the ink. The front end fills it from the desktop theme.
///
/// # Safety
///
/// `session` must be null or a live session pointer, and `ink` must be null or readable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_session_set_ink(session: *mut PfSession, ink: *const PfInk) -> i32 {
    guard(|| {
        // SAFETY: the caller upholds both contracts.
        let (Some(session), Some(ink)) = (unsafe { borrow(session) }, unsafe { ink.as_ref() })
        else {
            return PF_ERR_NULL;
        };
        session.ink = ink.to_ink();
        PF_OK
    })
}

/// Advances the model by `dt` seconds.
///
/// # Safety
///
/// `session` must be null or a live session pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_session_advance(session: *mut PfSession, dt: f64) -> i32 {
    guard(|| {
        // SAFETY: the caller promises a live session pointer.
        let Some(session) = (unsafe { borrow(session) }) else {
            return PF_ERR_NULL;
        };
        session.advance(dt)
    })
}

/// Moves the valve to `position`. Out-of-range positions are refused, not clamped.
///
/// # Safety
///
/// `session` must be null or a live session pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_session_set_valve(session: *mut PfSession, position: u32) -> i32 {
    guard(|| {
        // SAFETY: the caller promises a live session pointer.
        let Some(session) = (unsafe { borrow(session) }) else {
            return PF_ERR_NULL;
        };
        if session.system.is_none() {
            return session.fail("no model is loaded");
        }
        let Some(system) = session.system.as_mut() else {
            return PF_ERR_MODEL;
        };
        if system.set_valve_position(position as usize) {
            PF_OK
        } else {
            session.fail("that valve position does not exist")
        }
    })
}

/// Toggles the valve between its positions.
///
/// # Safety
///
/// `session` must be null or a live session pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_session_toggle_valve(session: *mut PfSession) -> i32 {
    guard(|| {
        // SAFETY: the caller promises a live session pointer.
        let Some(session) = (unsafe { borrow(session) }) else {
            return PF_ERR_NULL;
        };
        if session.system.is_none() {
            return session.fail("no model is loaded");
        }
        let Some(system) = session.system.as_mut() else {
            return PF_ERR_MODEL;
        };
        if system.toggle_valve() {
            PF_OK
        } else {
            session.fail("the valve has a single position")
        }
    })
}

/// Rebuilds the frame for an area of `width_mm` × `height_mm`, at `pixels_per_mm`.
///
/// The pointers in `out` borrow the session: they stay valid until the next call to this
/// function or to [`pf_session_free`]. The function allocates only when the drawing grew.
///
/// # Safety
///
/// `session` must be null or a live session pointer, and `out` must be null or writable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_session_build_frame(
    session: *mut PfSession,
    width_mm: f32,
    height_mm: f32,
    pixels_per_mm: f32,
    out: *mut PfFrame,
) -> i32 {
    guard(|| {
        // SAFETY: the caller upholds both contracts.
        let (Some(session), Some(out)) = (unsafe { borrow(session) }, unsafe { out.as_mut() })
        else {
            return PF_ERR_NULL;
        };
        session.build_frame(width_mm, height_mm, pixels_per_mm);
        *out = PfFrame {
            calls: session.calls.as_ptr(),
            calls_len: session.calls.len() as u32,
            points: session.points.as_ptr(),
            points_len: session.points.len() as u32,
            text: session.text.as_ptr(),
            text_len: session.text.len() as u32,
            width_mm: session.width_mm,
            height_mm: session.height_mm,
        };
        PF_OK
    })
}

/// `1` when the point, in millimetres of the last built frame, falls on the valve.
///
/// # Safety
///
/// `session` must be null or a live session pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_session_hit_valve(
    session: *const PfSession,
    x_mm: f32,
    y_mm: f32,
) -> i32 {
    guard(|| {
        // SAFETY: the caller promises a live session pointer.
        let Some(session) = (unsafe { borrow_shared(session) }) else {
            return 0;
        };
        i32::from(session.hit_valve(x_mm, y_mm))
    })
}

/// `1` when the drawing has a layout, `0` when the area is too small to be readable.
///
/// # Safety
///
/// `session` must be null or a live session pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pf_session_has_layout(session: *const PfSession) -> i32 {
    guard(|| {
        // SAFETY: the caller promises a live session pointer.
        let Some(session) = (unsafe { borrow_shared(session) }) else {
            return 0;
        };
        i32::from(session.layout.is_some())
    })
}
