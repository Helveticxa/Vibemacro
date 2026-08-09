#![windows_subsystem = "windows"]

use std::ffi::c_void;
use std::mem::{size_of, zeroed};
use std::path::PathBuf;
use std::ptr::{null, null_mut};
use std::sync::atomic::{AtomicBool, AtomicPtr, Ordering};
use std::thread;
use std::time::{Duration, Instant};

#[cfg(test)]
use std::sync::atomic::{AtomicIsize, AtomicUsize};

use vibe_timer_core::macro_engine::{
    MacroDefinition, MacroEvent, MacroLibrary, MacroMode, MacroTrigger, MouseButton,
    default_data_path, load_library, save_library,
};
use vibe_timer_core::{DurationFields, format_duration};

type Bool = i32;
type Dword = u32;
type Hbrush = isize;
type Hcursor = isize;
type Hdc = isize;
type Hgdiobj = isize;
type Hicon = isize;
type Hinstance = isize;
type Hhook = isize;
type Hmenu = isize;
type Hwnd = isize;
type Lparam = isize;
type Lresult = isize;
type Uint = u32;
type Wparam = usize;

const FALSE: Bool = 0;
const TRUE: Bool = 1;

const WS_OVERLAPPED: Dword = 0x0000_0000;
const WS_CAPTION: Dword = 0x00C0_0000;
const WS_SYSMENU: Dword = 0x0008_0000;
const WS_MINIMIZEBOX: Dword = 0x0002_0000;
const WS_CHILD: Dword = 0x4000_0000;
const WS_VISIBLE: Dword = 0x1000_0000;
const ES_CENTER: Dword = 0x0001;
const ES_AUTOHSCROLL: Dword = 0x0080;
const ES_NUMBER: Dword = 0x2000;

const CW_USEDEFAULT: i32 = i32::MIN;
const SW_HIDE: i32 = 0;
const SW_SHOWNORMAL: i32 = 1;
const SW_MINIMIZE: i32 = 6;
const SW_RESTORE: i32 = 9;

const WM_CREATE: Uint = 0x0001;
const WM_DESTROY: Uint = 0x0002;
const WM_CLOSE: Uint = 0x0010;
const WM_PAINT: Uint = 0x000F;
const WM_ERASEBKGND: Uint = 0x0014;
const WM_SETCURSOR: Uint = 0x0020;
const WM_NCCREATE: Uint = 0x0081;
const WM_NCDESTROY: Uint = 0x0082;
const WM_COMMAND: Uint = 0x0111;
const WM_TIMER: Uint = 0x0113;
const WM_CTLCOLOREDIT: Uint = 0x0133;
const WM_CTLCOLORSTATIC: Uint = 0x0138;
const WM_MOUSEMOVE: Uint = 0x0200;
const WM_LBUTTONUP: Uint = 0x0202;
const WM_MOUSELEAVE: Uint = 0x02A3;
const WM_KEYDOWN: Uint = 0x0100;
const WM_KEYUP: Uint = 0x0101;
const WM_SYSKEYDOWN: Uint = 0x0104;
const WM_SYSKEYUP: Uint = 0x0105;
const WM_MBUTTONDOWN: Uint = 0x0207;
const WM_MBUTTONUP: Uint = 0x0208;
const WM_MOUSEWHEEL: Uint = 0x020A;
const WM_XBUTTONDOWN: Uint = 0x020B;
const WM_XBUTTONUP: Uint = 0x020C;
const WM_SETFONT: Uint = 0x0030;
const WM_SETICON: Uint = 0x0080;
const WM_APP_MACRO_DONE: Uint = 0x8001;

const EM_SETMARGINS: Uint = 0x00D3;
const EM_SETLIMITTEXT: Uint = 0x00C5;
const EC_LEFTMARGIN: Wparam = 0x0001;
const EC_RIGHTMARGIN: Wparam = 0x0002;

const GWLP_USERDATA: i32 = -21;
const TIMER_COUNTDOWN: usize = 1;
const TIMER_CAPTURE: usize = 2;

const MB_OK: Uint = 0x0000;
const MB_OKCANCEL: Uint = 0x0001;
const MB_ICONERROR: Uint = 0x0010;
const MB_ICONINFORMATION: Uint = 0x0040;
const MB_ICONWARNING: Uint = 0x0030;
const MB_YESNO: Uint = 0x0004;
const IDOK: i32 = 1;
const IDYES: i32 = 6;

const DT_LEFT: Uint = 0x0000;
const DT_CENTER: Uint = 0x0001;
const DT_RIGHT: Uint = 0x0002;
const DT_VCENTER: Uint = 0x0004;
const DT_SINGLELINE: Uint = 0x0020;
const DT_NOPREFIX: Uint = 0x0800;
const DT_END_ELLIPSIS: Uint = 0x8000;

const TRANSPARENT: i32 = 1;
const PS_SOLID: i32 = 0;
const SRCCOPY: Dword = 0x00CC_0020;

const TME_LEAVE: Dword = 0x0000_0002;
const INPUT_KEYBOARD: Dword = 1;
const INPUT_MOUSE: Dword = 0;
const KEYEVENTF_KEYUP: Dword = 0x0002;
const KEYEVENTF_UNICODE: Dword = 0x0004;
const MOUSEEVENTF_LEFTDOWN: Dword = 0x0002;
const MOUSEEVENTF_LEFTUP: Dword = 0x0004;
const MOUSEEVENTF_RIGHTDOWN: Dword = 0x0008;
const MOUSEEVENTF_RIGHTUP: Dword = 0x0010;
const MOUSEEVENTF_MIDDLEDOWN: Dword = 0x0020;
const MOUSEEVENTF_MIDDLEUP: Dword = 0x0040;
const MOUSEEVENTF_WHEEL: Dword = 0x0800;
const MOUSEEVENTF_XDOWN: Dword = 0x0080;
const MOUSEEVENTF_XUP: Dword = 0x0100;
const VK_RETURN: u16 = 0x0D;
const VK_ESCAPE: u16 = 0x1B;
const VK_F8: u16 = 0x77;
const VK_F9: u16 = 0x78;
const WH_KEYBOARD_LL: i32 = 13;
const WH_MOUSE_LL: i32 = 14;
const HC_ACTION: i32 = 0;
const LLKHF_INJECTED: Dword = 0x10;
const LLMHF_INJECTED: Dword = 0x01;
const XBUTTON1: u16 = 1;
const XBUTTON2: u16 = 2;
#[cfg(test)]
const ES_MULTILINE: Dword = 0x0004;
#[cfg(test)]
const ES_WANTRETURN: Dword = 0x1000;
#[cfg(test)]
const PM_REMOVE: Uint = 0x0001;
#[cfg(test)]
const WM_CHAR: Uint = 0x0102;
#[cfg(test)]
const DIB_RGB_COLORS: Uint = 0;
#[cfg(test)]
const PW_RENDERFULLCONTENT: Uint = 0x0000_0002;

const IDC_ARROW: usize = 32512;
const IDC_HAND: usize = 32649;
const DWMWA_USE_IMMERSIVE_DARK_MODE: Dword = 20;
const DWMWA_WINDOW_CORNER_PREFERENCE: Dword = 33;
const DWMWCP_ROUND: Dword = 2;
const ICON_SMALL: Wparam = 0;
const ICON_BIG: Wparam = 1;

const CLIENT_WIDTH: i32 = 520;
const MACRO_CLIENT_WIDTH: i32 = 960;
const CLIENT_HEIGHT: i32 = 650;
const MAX_RECORDED_EVENTS: usize = 10_000;
const SWP_NOMOVE: Uint = 0x0002;
const SWP_NOZORDER: Uint = 0x0004;

const COLOR_BG: u32 = rgb(9, 10, 9);
const COLOR_SURFACE: u32 = rgb(19, 21, 18);
const COLOR_SURFACE_2: u32 = rgb(29, 32, 27);
const COLOR_BORDER: u32 = rgb(55, 60, 51);
const COLOR_BORDER_HOT: u32 = rgb(139, 168, 64);
const COLOR_TEXT: u32 = rgb(244, 246, 238);
const COLOR_MUTED: u32 = rgb(157, 163, 149);
const COLOR_DIM: u32 = rgb(98, 105, 91);
const COLOR_ACCENT: u32 = rgb(202, 255, 55);
const COLOR_ACCENT_HOT: u32 = rgb(218, 255, 112);
const COLOR_ACCENT_DARK: u32 = rgb(42, 55, 15);
const COLOR_PANEL: u32 = rgb(17, 20, 16);
const COLOR_PANEL_2: u32 = rgb(28, 32, 25);
const COLOR_PANEL_3: u32 = rgb(37, 42, 33);
const COLOR_PANEL_BORDER: u32 = rgb(48, 55, 43);
const COLOR_INK: u32 = rgb(13, 15, 12);
const COLOR_INK_MUTED: u32 = rgb(82, 88, 77);
const COLOR_SUCCESS: u32 = rgb(48, 210, 137);
const COLOR_WARNING: u32 = rgb(248, 180, 64);
const COLOR_ERROR: u32 = rgb(244, 91, 105);

#[cfg(test)]
static TEST_INPUT_TARGET: AtomicIsize = AtomicIsize::new(0);
#[cfg(test)]
static TEST_MACRO_INPUT_COUNT: AtomicUsize = AtomicUsize::new(0);
static APP_STATE_POINTER: AtomicPtr<AppState> = AtomicPtr::new(null_mut());
static MACRO_PLAYING: AtomicBool = AtomicBool::new(false);
static MACRO_STOP: AtomicBool = AtomicBool::new(false);
static TRIGGER_HELD: AtomicBool = AtomicBool::new(false);

const fn rgb(red: u8, green: u8, blue: u8) -> u32 {
    red as u32 | ((green as u32) << 8) | ((blue as u32) << 16)
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
struct Point {
    x: i32,
    y: i32,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
struct Rect {
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
}

impl Rect {
    const fn new(left: i32, top: i32, right: i32, bottom: i32) -> Self {
        Self {
            left,
            top,
            right,
            bottom,
        }
    }

    fn contains(self, x: i32, y: i32) -> bool {
        x >= self.left && x < self.right && y >= self.top && y < self.bottom
    }
}

#[repr(C)]
struct WndClassW {
    style: Uint,
    wnd_proc: Option<unsafe extern "system" fn(Hwnd, Uint, Wparam, Lparam) -> Lresult>,
    class_extra: i32,
    window_extra: i32,
    instance: Hinstance,
    icon: Hicon,
    cursor: Hcursor,
    background: Hbrush,
    menu_name: *const u16,
    class_name: *const u16,
}

#[repr(C)]
struct CreateStructW {
    create_params: *mut c_void,
    instance: Hinstance,
    menu: Hmenu,
    parent: Hwnd,
    height: i32,
    width: i32,
    y: i32,
    x: i32,
    style: i32,
    name: *const u16,
    class_name: *const u16,
    ex_style: Dword,
}

#[repr(C)]
struct Msg {
    window: Hwnd,
    message: Uint,
    wparam: Wparam,
    lparam: Lparam,
    time: Dword,
    point: Point,
    private: Dword,
}

#[repr(C)]
struct PaintStruct {
    dc: Hdc,
    erase: Bool,
    paint: Rect,
    restore: Bool,
    inc_update: Bool,
    reserved: [u8; 32],
}

#[cfg(test)]
#[repr(C)]
#[derive(Default)]
struct BitmapInfoHeader {
    size: Dword,
    width: i32,
    height: i32,
    planes: u16,
    bit_count: u16,
    compression: Dword,
    size_image: Dword,
    x_pixels_per_meter: i32,
    y_pixels_per_meter: i32,
    colors_used: Dword,
    colors_important: Dword,
}

#[cfg(test)]
#[repr(C)]
#[derive(Default)]
struct BitmapInfo {
    header: BitmapInfoHeader,
    colors: [Dword; 1],
}

#[repr(C)]
struct TrackMouseEvent {
    size: Dword,
    flags: Dword,
    tracked: Hwnd,
    hover_time: Dword,
}

#[repr(C)]
struct KbdLlHookStruct {
    vk_code: Dword,
    scan_code: Dword,
    flags: Dword,
    time: Dword,
    extra_info: usize,
}

#[repr(C)]
struct MsLlHookStruct {
    point: Point,
    mouse_data: Dword,
    flags: Dword,
    time: Dword,
    extra_info: usize,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct KeyboardInput {
    virtual_key: u16,
    scan_code: u16,
    flags: Dword,
    time: Dword,
    extra_info: usize,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct MouseInput {
    dx: i32,
    dy: i32,
    mouse_data: Dword,
    flags: Dword,
    time: Dword,
    extra_info: usize,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct HardwareInput {
    message: Dword,
    param_l: u16,
    param_h: u16,
}

#[repr(C)]
#[derive(Clone, Copy)]
union InputData {
    keyboard: KeyboardInput,
    mouse: MouseInput,
    hardware: HardwareInput,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct Input {
    kind: Dword,
    data: InputData,
}

#[link(name = "user32")]
unsafe extern "system" {
    fn RegisterClassW(class: *const WndClassW) -> u16;
    fn CreateWindowExW(
        ex_style: Dword,
        class_name: *const u16,
        window_name: *const u16,
        style: Dword,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        parent: Hwnd,
        menu: Hmenu,
        instance: Hinstance,
        param: *mut c_void,
    ) -> Hwnd;
    fn DefWindowProcW(window: Hwnd, message: Uint, wparam: Wparam, lparam: Lparam) -> Lresult;
    fn GetMessageW(message: *mut Msg, window: Hwnd, min: Uint, max: Uint) -> Bool;
    fn TranslateMessage(message: *const Msg) -> Bool;
    fn DispatchMessageW(message: *const Msg) -> Lresult;
    fn PostQuitMessage(exit_code: i32);
    fn DestroyWindow(window: Hwnd) -> Bool;
    fn LoadCursorW(instance: Hinstance, name: *const u16) -> Hcursor;
    fn CreateIcon(
        instance: Hinstance,
        width: i32,
        height: i32,
        planes: u8,
        bits_per_pixel: u8,
        and_bits: *const u8,
        xor_bits: *const u8,
    ) -> Hicon;
    fn SetCursor(cursor: Hcursor) -> Hcursor;
    fn ShowWindow(window: Hwnd, command: i32) -> Bool;
    fn UpdateWindow(window: Hwnd) -> Bool;
    fn SetWindowPos(
        window: Hwnd,
        insert_after: Hwnd,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        flags: Uint,
    ) -> Bool;
    fn SetWindowLongPtrW(window: Hwnd, index: i32, value: isize) -> isize;
    fn GetWindowLongPtrW(window: Hwnd, index: i32) -> isize;
    fn GetClientRect(window: Hwnd, rect: *mut Rect) -> Bool;
    #[cfg(test)]
    fn GetWindowRect(window: Hwnd, rect: *mut Rect) -> Bool;
    fn AdjustWindowRectEx(rect: *mut Rect, style: Dword, menu: Bool, ex_style: Dword) -> Bool;
    fn BeginPaint(window: Hwnd, paint: *mut PaintStruct) -> Hdc;
    fn EndPaint(window: Hwnd, paint: *const PaintStruct) -> Bool;
    fn InvalidateRect(window: Hwnd, rect: *const Rect, erase: Bool) -> Bool;
    fn SendMessageW(window: Hwnd, message: Uint, wparam: Wparam, lparam: Lparam) -> Lresult;
    fn EnableWindow(window: Hwnd, enabled: Bool) -> Bool;
    fn SetWindowTextW(window: Hwnd, text: *const u16) -> Bool;
    fn GetWindowTextW(window: Hwnd, text: *mut u16, max_count: i32) -> i32;
    fn GetWindowTextLengthW(window: Hwnd) -> i32;
    fn GetForegroundWindow() -> Hwnd;
    fn SetForegroundWindow(window: Hwnd) -> Bool;
    fn BringWindowToTop(window: Hwnd) -> Bool;
    fn IsIconic(window: Hwnd) -> Bool;
    fn IsWindow(window: Hwnd) -> Bool;
    fn IsWindowVisible(window: Hwnd) -> Bool;
    fn GetWindowThreadProcessId(window: Hwnd, process_id: *mut Dword) -> Dword;
    fn AttachThreadInput(id_attach: Dword, id_attach_to: Dword, attach: Bool) -> Bool;
    fn SetTimer(window: Hwnd, id: usize, interval_ms: Uint, callback: *const c_void) -> usize;
    fn KillTimer(window: Hwnd, id: usize) -> Bool;
    fn MessageBoxW(window: Hwnd, text: *const u16, caption: *const u16, kind: Uint) -> i32;
    fn MessageBeep(kind: Uint) -> Bool;
    fn TrackMouseEvent(event: *mut TrackMouseEvent) -> Bool;
    fn SetProcessDpiAwarenessContext(value: isize) -> Bool;
    fn SendInput(count: Uint, inputs: *const Input, input_size: i32) -> Uint;
    fn PostMessageW(window: Hwnd, message: Uint, wparam: Wparam, lparam: Lparam) -> Bool;
    fn SetWindowsHookExW(
        hook_id: i32,
        callback: Option<unsafe extern "system" fn(i32, Wparam, Lparam) -> Lresult>,
        module: Hinstance,
        thread_id: Dword,
    ) -> Hhook;
    fn UnhookWindowsHookEx(hook: Hhook) -> Bool;
    fn CallNextHookEx(hook: Hhook, code: i32, wparam: Wparam, lparam: Lparam) -> Lresult;
    fn FillRect(dc: Hdc, rect: *const Rect, brush: Hbrush) -> i32;
    #[cfg(test)]
    fn SetFocus(window: Hwnd) -> Hwnd;
    #[cfg(test)]
    fn SetActiveWindow(window: Hwnd) -> Hwnd;
    #[cfg(test)]
    fn PeekMessageW(message: *mut Msg, window: Hwnd, min: Uint, max: Uint, remove: Uint) -> Bool;
    #[cfg(test)]
    fn GetDC(window: Hwnd) -> Hdc;
    #[cfg(test)]
    fn ReleaseDC(window: Hwnd, dc: Hdc) -> i32;
    #[cfg(test)]
    fn PrintWindow(window: Hwnd, dc: Hdc, flags: Uint) -> Bool;
}

#[link(name = "kernel32")]
unsafe extern "system" {
    fn GetModuleHandleW(module_name: *const u16) -> Hinstance;
    fn GetCurrentThreadId() -> Dword;
    #[cfg(test)]
    fn GetCurrentProcessId() -> Dword;
}

#[link(name = "gdi32")]
unsafe extern "system" {
    fn CreateSolidBrush(color: u32) -> Hbrush;
    fn CreatePen(style: i32, width: i32, color: u32) -> Hgdiobj;
    fn CreateFontW(
        height: i32,
        width: i32,
        escapement: i32,
        orientation: i32,
        weight: i32,
        italic: Dword,
        underline: Dword,
        strike_out: Dword,
        charset: Dword,
        output_precision: Dword,
        clip_precision: Dword,
        quality: Dword,
        pitch_and_family: Dword,
        face: *const u16,
    ) -> Hgdiobj;
    fn SelectObject(dc: Hdc, object: Hgdiobj) -> Hgdiobj;
    fn DeleteObject(object: Hgdiobj) -> Bool;
    fn DeleteDC(dc: Hdc) -> Bool;
    fn CreateCompatibleDC(dc: Hdc) -> Hdc;
    fn CreateCompatibleBitmap(dc: Hdc, width: i32, height: i32) -> Hgdiobj;
    fn BitBlt(
        destination: Hdc,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        source: Hdc,
        source_x: i32,
        source_y: i32,
        operation: Dword,
    ) -> Bool;
    fn SetTextColor(dc: Hdc, color: u32) -> u32;
    fn SetBkColor(dc: Hdc, color: u32) -> u32;
    fn SetBkMode(dc: Hdc, mode: i32) -> i32;
    fn RoundRect(
        dc: Hdc,
        left: i32,
        top: i32,
        right: i32,
        bottom: i32,
        width: i32,
        height: i32,
    ) -> Bool;
    fn Ellipse(dc: Hdc, left: i32, top: i32, right: i32, bottom: i32) -> Bool;
    fn MoveToEx(dc: Hdc, x: i32, y: i32, old: *mut Point) -> Bool;
    fn LineTo(dc: Hdc, x: i32, y: i32) -> Bool;
    #[cfg(test)]
    fn GetDIBits(
        dc: Hdc,
        bitmap: Hgdiobj,
        start: Uint,
        lines: Uint,
        bits: *mut c_void,
        info: *mut BitmapInfo,
        usage: Uint,
    ) -> i32;
}

#[link(name = "user32")]
unsafe extern "system" {
    fn DrawTextW(dc: Hdc, text: *mut u16, count: i32, rect: *mut Rect, format: Uint) -> i32;
}

#[link(name = "dwmapi")]
unsafe extern "system" {
    fn DwmSetWindowAttribute(
        window: Hwnd,
        attribute: Dword,
        value: *const c_void,
        size: Dword,
    ) -> i32;
}

#[link(name = "uxtheme")]
unsafe extern "system" {
    fn SetWindowTheme(window: Hwnd, sub_app: *const u16, sub_id: *const u16) -> i32;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ActionMode {
    EnterOnly,
    TextAndEnter,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AppTab {
    Timer,
    Macro,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MacroLane {
    OnPress,
    WhileHolding,
    OnRelease,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum StatusKind {
    Ready,
    Running,
    Sent,
    Warning,
    Error,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum HitTarget {
    None,
    TimerTab,
    MacroTab,
    AddThirtyMinutes,
    AddOneHour,
    AddThreeHours,
    PickTarget,
    EnterOnly,
    TextAndEnter,
    MainAction,
    MacroNew,
    MacroItem(usize),
    MacroMode(MacroMode),
    MacroTrigger(MacroTrigger),
    MacroLane(MacroLane),
    MacroRecord,
    MacroClear,
    MacroSave,
}

#[derive(Clone)]
struct TargetWindow {
    window: Hwnd,
    process_id: Dword,
    title: String,
}

#[derive(Default)]
struct Fonts {
    title: Hgdiobj,
    timer: Hgdiobj,
    body: Hgdiobj,
    semibold: Hgdiobj,
    small: Hgdiobj,
}

struct AppState {
    window: Hwnd,
    tab: AppTab,
    hour_edit: Hwnd,
    minute_edit: Hwnd,
    second_edit: Hwnd,
    prompt_edit: Hwnd,
    macro_name_edit: Hwnd,
    edit_brush: Hbrush,
    panel_edit_brush: Hbrush,
    fonts: Fonts,
    action_mode: ActionMode,
    status_kind: StatusKind,
    status: String,
    target: Option<TargetWindow>,
    running: bool,
    deadline: Option<Instant>,
    capture_deadline: Option<Instant>,
    original_seconds: u64,
    remaining_seconds: u64,
    armed_prompt: String,
    hot: HitTarget,
    tracking_mouse: bool,
    macro_library: MacroLibrary,
    macro_path: PathBuf,
    macro_status_kind: StatusKind,
    macro_status: String,
    macro_dirty: bool,
    macro_lane: MacroLane,
    recording: bool,
    record_last_event: Option<Instant>,
    suppress_escape_until_up: bool,
    trigger_down: bool,
    keyboard_hook: Hhook,
    mouse_hook: Hhook,
}

impl AppState {
    fn new() -> Self {
        let macro_path = default_data_path();
        let (macro_library, macro_status_kind, macro_status) = match load_library(&macro_path) {
            Ok(library) => (
                library,
                StatusKind::Ready,
                "Macro siap. Pilih pemicu lalu rekam langkah.".to_owned(),
            ),
            Err(error) => (
                MacroLibrary::default(),
                StatusKind::Warning,
                format!("File macro tidak dapat dibaca: {error}"),
            ),
        };
        Self {
            window: 0,
            tab: AppTab::Timer,
            hour_edit: 0,
            minute_edit: 0,
            second_edit: 0,
            prompt_edit: 0,
            macro_name_edit: 0,
            edit_brush: 0,
            panel_edit_brush: 0,
            fonts: Fonts::default(),
            action_mode: ActionMode::TextAndEnter,
            status_kind: StatusKind::Ready,
            status: "Pilih jendela AI untuk mulai.".to_owned(),
            target: None,
            running: false,
            deadline: None,
            capture_deadline: None,
            original_seconds: 0,
            remaining_seconds: 0,
            armed_prompt: String::new(),
            hot: HitTarget::None,
            tracking_mouse: false,
            macro_library,
            macro_path,
            macro_status_kind,
            macro_status,
            macro_dirty: false,
            macro_lane: MacroLane::OnPress,
            recording: false,
            record_last_event: None,
            suppress_escape_until_up: false,
            trigger_down: false,
            keyboard_hook: 0,
            mouse_hook: 0,
        }
    }

    unsafe fn set_controls_visible(&self, visible: bool) {
        let timer_command = if visible && self.tab == AppTab::Timer {
            SW_SHOWNORMAL
        } else {
            SW_HIDE
        };
        unsafe {
            ShowWindow(self.hour_edit, timer_command);
            ShowWindow(self.minute_edit, timer_command);
            ShowWindow(self.second_edit, timer_command);
            ShowWindow(self.prompt_edit, timer_command);
            ShowWindow(
                self.macro_name_edit,
                if self.tab == AppTab::Macro && !self.recording {
                    SW_SHOWNORMAL
                } else {
                    SW_HIDE
                },
            );
        }
    }

    unsafe fn set_prompt_enabled(&self) {
        let enabled = self.action_mode == ActionMode::TextAndEnter && !self.running;
        unsafe {
            EnableWindow(self.prompt_edit, if enabled { TRUE } else { FALSE });
            EnableWindow(
                self.macro_name_edit,
                if !self.recording { TRUE } else { FALSE },
            );
        }
    }

    unsafe fn cleanup(&mut self) {
        unsafe {
            for font in [
                self.fonts.title,
                self.fonts.timer,
                self.fonts.body,
                self.fonts.semibold,
                self.fonts.small,
            ] {
                if font != 0 {
                    DeleteObject(font);
                }
            }
            if self.edit_brush != 0 {
                DeleteObject(self.edit_brush);
            }
            if self.panel_edit_brush != 0 {
                DeleteObject(self.panel_edit_brush);
            }
            if self.keyboard_hook != 0 {
                UnhookWindowsHookEx(self.keyboard_hook);
                self.keyboard_hook = 0;
            }
            if self.mouse_hook != 0 {
                UnhookWindowsHookEx(self.mouse_hook);
                self.mouse_hook = 0;
            }
        }
    }
}

const RECT_QUICK_30: Rect = Rect::new(48, 224, 156, 258);
const RECT_QUICK_60: Rect = Rect::new(164, 224, 272, 258);
const RECT_QUICK_180: Rect = Rect::new(280, 224, 388, 258);
const RECT_PICK_TARGET: Rect = Rect::new(358, 309, 474, 351);
const RECT_MODE_ENTER: Rect = Rect::new(42, 424, 249, 462);
const RECT_MODE_TEXT: Rect = Rect::new(251, 424, 458, 462);
const RECT_MAIN_ACTION: Rect = Rect::new(24, 548, 496, 604);
const RECT_TAB_TIMER: Rect = Rect::new(300, 24, 382, 54);
const RECT_TAB_MACRO: Rect = Rect::new(386, 24, 482, 54);
const RECT_MACRO_NEW: Rect = Rect::new(42, 117, 218, 157);
const RECT_MACRO_MODE_NO_REPEAT: Rect = Rect::new(278, 173, 428, 239);
const RECT_MACRO_MODE_HOLD: Rect = Rect::new(438, 173, 588, 239);
const RECT_MACRO_MODE_TOGGLE: Rect = Rect::new(598, 173, 748, 239);
const RECT_MACRO_MODE_SEQUENCE: Rect = Rect::new(758, 173, 908, 239);
const RECT_MACRO_LANE_PRESS: Rect = Rect::new(278, 331, 412, 365);
const RECT_MACRO_LANE_HOLD: Rect = Rect::new(420, 331, 570, 365);
const RECT_MACRO_LANE_RELEASE: Rect = Rect::new(578, 331, 722, 365);
const RECT_MACRO_RECORD: Rect = Rect::new(278, 548, 474, 596);
const RECT_MACRO_CLEAR: Rect = Rect::new(484, 548, 634, 596);
const RECT_MACRO_SAVE: Rect = Rect::new(758, 548, 912, 596);

fn macro_trigger_rect(index: usize) -> Rect {
    let left = 278 + index as i32 * 116;
    Rect::new(left, 272, left + 106, 310)
}

fn macro_item_rect(index: usize) -> Rect {
    let top = 169 + index as i32 * 58;
    Rect::new(42, top, 218, top + 48)
}

fn wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(std::iter::once(0)).collect()
}

fn low_word(value: Lparam) -> i32 {
    (value as u32 & 0xFFFF) as i16 as i32
}

fn high_word(value: Lparam) -> i32 {
    ((value as u32 >> 16) & 0xFFFF) as i16 as i32
}

fn hit_test(x: i32, y: i32, state: &AppState) -> HitTarget {
    if RECT_TAB_TIMER.contains(x, y) {
        return HitTarget::TimerTab;
    }
    if RECT_TAB_MACRO.contains(x, y) {
        return HitTarget::MacroTab;
    }
    if state.tab == AppTab::Macro {
        if RECT_MACRO_NEW.contains(x, y) {
            return HitTarget::MacroNew;
        }
        for (index, _) in state.macro_library.macros.iter().take(7).enumerate() {
            if macro_item_rect(index).contains(x, y) {
                return HitTarget::MacroItem(index);
            }
        }
        for (target, rect) in [
            (MacroMode::NoRepeat, RECT_MACRO_MODE_NO_REPEAT),
            (MacroMode::RepeatWhileHolding, RECT_MACRO_MODE_HOLD),
            (MacroMode::Toggle, RECT_MACRO_MODE_TOGGLE),
            (MacroMode::Sequence, RECT_MACRO_MODE_SEQUENCE),
        ] {
            if rect.contains(x, y) {
                return HitTarget::MacroMode(target);
            }
        }
        for (index, trigger) in MacroTrigger::ALL.into_iter().enumerate() {
            if macro_trigger_rect(index).contains(x, y) {
                return HitTarget::MacroTrigger(trigger);
            }
        }
        for (lane, rect) in [
            (MacroLane::OnPress, RECT_MACRO_LANE_PRESS),
            (MacroLane::WhileHolding, RECT_MACRO_LANE_HOLD),
            (MacroLane::OnRelease, RECT_MACRO_LANE_RELEASE),
        ] {
            if rect.contains(x, y) {
                return HitTarget::MacroLane(lane);
            }
        }
        if RECT_MACRO_RECORD.contains(x, y) {
            return HitTarget::MacroRecord;
        }
        if RECT_MACRO_CLEAR.contains(x, y) {
            return HitTarget::MacroClear;
        }
        if RECT_MACRO_SAVE.contains(x, y) {
            return HitTarget::MacroSave;
        }
        return HitTarget::None;
    }
    if RECT_MAIN_ACTION.contains(x, y) {
        return HitTarget::MainAction;
    }
    if state.running {
        return HitTarget::None;
    }
    if RECT_QUICK_30.contains(x, y) {
        HitTarget::AddThirtyMinutes
    } else if RECT_QUICK_60.contains(x, y) {
        HitTarget::AddOneHour
    } else if RECT_QUICK_180.contains(x, y) {
        HitTarget::AddThreeHours
    } else if RECT_PICK_TARGET.contains(x, y) {
        HitTarget::PickTarget
    } else if RECT_MODE_ENTER.contains(x, y) {
        HitTarget::EnterOnly
    } else if RECT_MODE_TEXT.contains(x, y) {
        HitTarget::TextAndEnter
    } else {
        HitTarget::None
    }
}

unsafe fn make_font(size: i32, weight: i32) -> Hgdiobj {
    let face = wide("Segoe UI Variable Display");
    unsafe {
        CreateFontW(
            -size,
            0,
            0,
            0,
            weight,
            0,
            0,
            0,
            1,
            0,
            0,
            5,
            0,
            face.as_ptr(),
        )
    }
}

unsafe fn fill_rect_color(dc: Hdc, rect: Rect, color: u32) {
    unsafe {
        let brush = CreateSolidBrush(color);
        FillRect(dc, &rect, brush);
        DeleteObject(brush);
    }
}

unsafe fn rounded_box(dc: Hdc, rect: Rect, radius: i32, fill: u32, border: u32) {
    unsafe {
        let brush = CreateSolidBrush(fill);
        let pen = CreatePen(PS_SOLID, 1, border);
        let old_brush = SelectObject(dc, brush);
        let old_pen = SelectObject(dc, pen);
        RoundRect(
            dc,
            rect.left,
            rect.top,
            rect.right,
            rect.bottom,
            radius,
            radius,
        );
        SelectObject(dc, old_brush);
        SelectObject(dc, old_pen);
        DeleteObject(brush);
        DeleteObject(pen);
    }
}

unsafe fn filled_circle(dc: Hdc, rect: Rect, color: u32) {
    unsafe {
        let brush = CreateSolidBrush(color);
        let pen = CreatePen(PS_SOLID, 1, color);
        let old_brush = SelectObject(dc, brush);
        let old_pen = SelectObject(dc, pen);
        Ellipse(dc, rect.left, rect.top, rect.right, rect.bottom);
        SelectObject(dc, old_brush);
        SelectObject(dc, old_pen);
        DeleteObject(brush);
        DeleteObject(pen);
    }
}

unsafe fn create_app_icon(instance: Hinstance, size: i32) -> Hicon {
    let side = size.max(16) as usize;
    let mask_stride = side.div_ceil(32) * 4;
    let mut and_mask = vec![0xFFu8; mask_stride * side];
    let mut pixels = vec![0u8; side * side * 4];
    let center = (side as f32 - 1.0) / 2.0;
    let outer_radius = side as f32 * 0.45;

    for y in 0..side {
        for x in 0..side {
            let dx = x as f32 - center;
            let dy = y as f32 - center;
            let distance = (dx * dx + dy * dy).sqrt();
            if distance > outer_radius {
                continue;
            }

            let dib_y = side - 1 - y;
            let pixel_index = (dib_y * side + x) * 4;
            let is_outline = distance > outer_radius - (side as f32 * 0.075).max(1.0);
            let vertical_hand = (x as i32 - center.round() as i32).abs() <= 1
                && y as f32 >= center - side as f32 * 0.22
                && y as f32 <= center + 1.0;
            let diagonal_y = center + (x as f32 - center) * 0.48;
            let diagonal_hand = x as f32 >= center
                && x as f32 <= center + side as f32 * 0.22
                && (y as f32 - diagonal_y).abs() <= 1.2;
            let (red, green, blue) = if is_outline || vertical_hand || diagonal_hand {
                (244u8, 240u8, 255u8)
            } else {
                (139u8, 92u8, 246u8)
            };
            pixels[pixel_index] = blue;
            pixels[pixel_index + 1] = green;
            pixels[pixel_index + 2] = red;
            pixels[pixel_index + 3] = 255;

            let mask_index = dib_y * mask_stride + x / 8;
            and_mask[mask_index] &= !(0x80 >> (x % 8));
        }
    }

    unsafe {
        CreateIcon(
            instance,
            side as i32,
            side as i32,
            1,
            32,
            and_mask.as_ptr(),
            pixels.as_ptr(),
        )
    }
}

unsafe fn draw_label(dc: Hdc, text: &str, mut rect: Rect, color: u32, font: Hgdiobj, format: Uint) {
    let mut utf16 = wide(text);
    unsafe {
        let old_font = SelectObject(dc, font);
        SetTextColor(dc, color);
        SetBkMode(dc, TRANSPARENT);
        DrawTextW(
            dc,
            utf16.as_mut_ptr(),
            (utf16.len() - 1) as i32,
            &mut rect,
            format | DT_NOPREFIX,
        );
        SelectObject(dc, old_font);
    }
}

unsafe fn draw_button(dc: Hdc, rect: Rect, label: &str, selected: bool, hot: bool, font: Hgdiobj) {
    let (fill, border, text) = if selected {
        (
            if hot { COLOR_ACCENT_HOT } else { COLOR_ACCENT },
            if hot { COLOR_ACCENT_HOT } else { COLOR_ACCENT },
            COLOR_INK,
        )
    } else {
        (
            if hot {
                rgb(30, 40, 54)
            } else {
                COLOR_SURFACE_2
            },
            if hot { COLOR_BORDER_HOT } else { COLOR_BORDER },
            if hot { COLOR_TEXT } else { COLOR_MUTED },
        )
    };
    unsafe {
        rounded_box(dc, rect, 12, fill, border);
        draw_label(
            dc,
            label,
            rect,
            text,
            font,
            DT_CENTER | DT_VCENTER | DT_SINGLELINE,
        );
    }
}

unsafe fn draw_flat_button(
    dc: Hdc,
    rect: Rect,
    label: &str,
    fill: u32,
    text: u32,
    hot: bool,
    font: Hgdiobj,
) {
    let fill = if hot {
        if fill == COLOR_ACCENT {
            COLOR_ACCENT_HOT
        } else if fill == COLOR_INK {
            rgb(35, 39, 32)
        } else {
            COLOR_PANEL_3
        }
    } else {
        fill
    };
    unsafe {
        rounded_box(dc, rect, 14, fill, fill);
        draw_label(
            dc,
            label,
            rect,
            text,
            font,
            DT_CENTER | DT_VCENTER | DT_SINGLELINE | DT_END_ELLIPSIS,
        );
    }
}

unsafe fn draw_hairline(dc: Hdc, left: i32, top: i32, right: i32, color: u32) {
    unsafe {
        let pen = CreatePen(PS_SOLID, 1, color);
        let old_pen = SelectObject(dc, pen);
        MoveToEx(dc, left, top, null_mut());
        LineTo(dc, right, top);
        SelectObject(dc, old_pen);
        DeleteObject(pen);
    }
}

unsafe fn draw_clock_mark(dc: Hdc) {
    unsafe {
        let brush = CreateSolidBrush(COLOR_ACCENT);
        let pen = CreatePen(PS_SOLID, 2, COLOR_INK);
        let old_brush = SelectObject(dc, brush);
        let old_pen = SelectObject(dc, pen);
        RoundRect(dc, 24, 20, 54, 50, 12, 12);
        Ellipse(dc, 31, 27, 47, 43);
        MoveToEx(dc, 39, 30, null_mut());
        LineTo(dc, 39, 36);
        LineTo(dc, 44, 39);
        SelectObject(dc, old_brush);
        SelectObject(dc, old_pen);
        DeleteObject(brush);
        DeleteObject(pen);
    }
}

unsafe fn draw_status_pill(dc: Hdc, state: &AppState) {
    let kind = if state.tab == AppTab::Macro {
        state.macro_status_kind
    } else {
        state.status_kind
    };
    let (label, dot_color, width) = match kind {
        StatusKind::Ready => ("Siap", COLOR_MUTED, 58),
        StatusKind::Running => ("Aktif", COLOR_ACCENT, 62),
        StatusKind::Sent => ("Selesai", COLOR_SUCCESS, 76),
        StatusKind::Warning => ("Periksa", COLOR_WARNING, 76),
        StatusKind::Error => ("Gagal", COLOR_ERROR, 64),
    };
    let right = if state.tab == AppTab::Macro { 936 } else { 496 };
    let rect = Rect::new(right - width, 26, right, 50);
    unsafe {
        let dot = Rect::new(rect.left + 5, 35, rect.left + 11, 41);
        filled_circle(dc, dot, dot_color);
        draw_label(
            dc,
            label,
            Rect::new(rect.left + 16, rect.top, rect.right, rect.bottom),
            COLOR_MUTED,
            state.fonts.small,
            DT_LEFT | DT_VCENTER | DT_SINGLELINE,
        );
    }
}

unsafe fn draw_tabs(dc: Hdc, state: &AppState) {
    unsafe {
        for (rect, label, target, active) in [
            (
                RECT_TAB_TIMER,
                "Timer",
                HitTarget::TimerTab,
                state.tab == AppTab::Timer,
            ),
            (
                RECT_TAB_MACRO,
                "Macro",
                HitTarget::MacroTab,
                state.tab == AppTab::Macro,
            ),
        ] {
            draw_label(
                dc,
                label,
                Rect::new(rect.left, rect.top - 2, rect.right, rect.bottom - 4),
                if active || state.hot == target {
                    COLOR_TEXT
                } else {
                    COLOR_MUTED
                },
                state.fonts.small,
                DT_CENTER | DT_VCENTER | DT_SINGLELINE,
            );
            if active {
                rounded_box(
                    dc,
                    Rect::new(
                        rect.left + 22,
                        rect.bottom - 3,
                        rect.right - 22,
                        rect.bottom,
                    ),
                    3,
                    COLOR_ACCENT,
                    COLOR_ACCENT,
                );
            }
        }
    }
}

fn key_label(key: u16) -> String {
    match key {
        0x08 => "Backspace".to_owned(),
        0x09 => "Tab".to_owned(),
        0x0D => "Enter".to_owned(),
        0x10 => "Shift".to_owned(),
        0x11 => "Ctrl".to_owned(),
        0x12 => "Alt".to_owned(),
        0x1B => "Esc".to_owned(),
        0x20 => "Space".to_owned(),
        0x25 => "Left".to_owned(),
        0x26 => "Up".to_owned(),
        0x27 => "Right".to_owned(),
        0x28 => "Down".to_owned(),
        0x2E => "Delete".to_owned(),
        VK_F8 => "F8".to_owned(),
        VK_F9 => "F9".to_owned(),
        value if (0x30..=0x5A).contains(&value) => {
            char::from_u32(value as u32).unwrap_or('?').to_string()
        }
        value if (0x70..=0x87).contains(&value) => format!("F{}", value - 0x6F),
        value => format!("VK {value:02X}"),
    }
}

fn mouse_label(button: MouseButton) -> &'static str {
    match button {
        MouseButton::Left => "Left",
        MouseButton::Right => "Right",
        MouseButton::Middle => "Middle",
        MouseButton::X1 => "Mouse 4",
        MouseButton::X2 => "Mouse 5",
    }
}

fn macro_event_label(event: &MacroEvent) -> String {
    match event {
        MacroEvent::Delay(ms) => format!("{ms} ms"),
        MacroEvent::KeyDown(key) => format!("{} ↓", key_label(*key)),
        MacroEvent::KeyUp(key) => format!("{} ↑", key_label(*key)),
        MacroEvent::MouseDown(button) => format!("{} ↓", mouse_label(*button)),
        MacroEvent::MouseUp(button) => format!("{} ↑", mouse_label(*button)),
        MacroEvent::Wheel(delta) if *delta > 0 => "Wheel ↑".to_owned(),
        MacroEvent::Wheel(_) => "Wheel ↓".to_owned(),
    }
}

fn lane_events(item: &MacroDefinition, lane: MacroLane) -> &[MacroEvent] {
    match lane {
        MacroLane::OnPress => &item.on_press,
        MacroLane::WhileHolding => &item.while_holding,
        MacroLane::OnRelease => &item.on_release,
    }
}

fn total_delay(events: &[MacroEvent]) -> u64 {
    events
        .iter()
        .filter_map(|event| match event {
            MacroEvent::Delay(value) => Some(*value as u64),
            _ => None,
        })
        .sum()
}

unsafe fn draw_brand_header_v3(dc: Hdc, state: &AppState) {
    unsafe {
        draw_clock_mark(dc);
        draw_label(
            dc,
            "VibeTimer",
            Rect::new(68, 17, 270, 50),
            COLOR_TEXT,
            state.fonts.title,
            DT_LEFT | DT_VCENTER | DT_SINGLELINE,
        );
        draw_tabs(dc, state);
        if state.tab == AppTab::Macro {
            draw_status_pill(dc, state);
        }
    }
}

unsafe fn draw_timer_interface_v3(dc: Hdc, state: &AppState) {
    unsafe {
        fill_rect_color(dc, Rect::new(0, 0, CLIENT_WIDTH, CLIENT_HEIGHT), COLOR_BG);
        draw_brand_header_v3(dc, state);

        draw_label(
            dc,
            if state.running {
                "Sisa waktu"
            } else {
                "Waktu reset"
            },
            Rect::new(32, 82, 250, 106),
            COLOR_ACCENT,
            state.fonts.small,
            DT_LEFT | DT_VCENTER | DT_SINGLELINE,
        );
        draw_label(
            dc,
            if state.running {
                "VibeTimer akan melanjutkan tepat saat nol."
            } else {
                "Atur sekali. Lanjut otomatis."
            },
            Rect::new(32, 101, 488, 126),
            COLOR_MUTED,
            state.fonts.body,
            DT_LEFT | DT_VCENTER | DT_SINGLELINE,
        );

        if state.running {
            draw_label(
                dc,
                &format_duration(state.remaining_seconds),
                Rect::new(28, 122, 492, 202),
                COLOR_TEXT,
                state.fonts.timer,
                DT_CENTER | DT_VCENTER | DT_SINGLELINE,
            );
            let fraction = if state.original_seconds == 0 {
                0.0
            } else {
                1.0 - state.remaining_seconds as f64 / state.original_seconds as f64
            };
            rounded_box(
                dc,
                Rect::new(32, 218, 488, 224),
                6,
                COLOR_SURFACE_2,
                COLOR_SURFACE_2,
            );
            let progress = (456.0 * fraction.clamp(0.0, 1.0)) as i32;
            if progress > 0 {
                rounded_box(
                    dc,
                    Rect::new(32, 218, 32 + progress.max(6), 224),
                    6,
                    COLOR_ACCENT,
                    COLOR_ACCENT,
                );
            }
            draw_label(
                dc,
                "Satu aksi. Tanpa percobaan ulang otomatis.",
                Rect::new(32, 230, 488, 254),
                COLOR_DIM,
                state.fonts.small,
                DT_LEFT | DT_VCENTER | DT_SINGLELINE,
            );
        } else {
            draw_label(
                dc,
                ":",
                Rect::new(158, 128, 194, 180),
                COLOR_DIM,
                state.fonts.timer,
                DT_CENTER | DT_VCENTER | DT_SINGLELINE,
            );
            draw_label(
                dc,
                ":",
                Rect::new(304, 128, 340, 180),
                COLOR_DIM,
                state.fonts.timer,
                DT_CENTER | DT_VCENTER | DT_SINGLELINE,
            );
            for (label, rect) in [
                ("jam", Rect::new(48, 188, 158, 212)),
                ("menit", Rect::new(194, 188, 304, 212)),
                ("detik", Rect::new(340, 188, 450, 212)),
            ] {
                draw_label(
                    dc,
                    label,
                    rect,
                    COLOR_DIM,
                    state.fonts.small,
                    DT_CENTER | DT_VCENTER | DT_SINGLELINE,
                );
            }
            for (rect, label, target) in [
                (RECT_QUICK_30, "+30 mnt", HitTarget::AddThirtyMinutes),
                (RECT_QUICK_60, "+1 jam", HitTarget::AddOneHour),
                (RECT_QUICK_180, "+3 jam", HitTarget::AddThreeHours),
            ] {
                draw_flat_button(
                    dc,
                    rect,
                    label,
                    COLOR_SURFACE_2,
                    COLOR_TEXT,
                    state.hot == target,
                    state.fonts.small,
                );
            }
        }

        let config = Rect::new(24, 286, 496, 526);
        rounded_box(dc, config, 26, COLOR_PANEL, COLOR_PANEL_BORDER);
        draw_label(
            dc,
            "Target",
            Rect::new(42, 302, 190, 324),
            COLOR_MUTED,
            state.fonts.small,
            DT_LEFT | DT_VCENTER | DT_SINGLELINE,
        );
        let target_text = state
            .target
            .as_ref()
            .map(|target| target.title.as_str())
            .unwrap_or("Belum memilih jendela");
        draw_label(
            dc,
            target_text,
            Rect::new(42, 323, 344, 350),
            COLOR_TEXT,
            state.fonts.semibold,
            DT_LEFT | DT_VCENTER | DT_SINGLELINE | DT_END_ELLIPSIS,
        );
        draw_label(
            dc,
            if state.target.is_some() {
                "Terverifikasi dengan jendela + proses"
            } else {
                "Pilih jendela input AI"
            },
            Rect::new(42, 348, 344, 368),
            COLOR_MUTED,
            state.fonts.small,
            DT_LEFT | DT_VCENTER | DT_SINGLELINE,
        );
        draw_flat_button(
            dc,
            RECT_PICK_TARGET,
            if state.running {
                "Terkunci"
            } else {
                "Pilih target"
            },
            COLOR_INK,
            COLOR_TEXT,
            state.hot == HitTarget::PickTarget && !state.running,
            state.fonts.small,
        );
        draw_hairline(dc, 42, 383, 478, COLOR_PANEL_BORDER);

        draw_label(
            dc,
            "Aksi saat nol",
            Rect::new(42, 394, 250, 418),
            COLOR_MUTED,
            state.fonts.small,
            DT_LEFT | DT_VCENTER | DT_SINGLELINE,
        );
        if state.running {
            let action = match state.action_mode {
                ActionMode::EnterOnly => "Tekan Enter sekali",
                ActionMode::TextAndEnter => "Ketik teks, lalu tekan Enter",
            };
            draw_label(
                dc,
                action,
                Rect::new(42, 424, 458, 453),
                COLOR_TEXT,
                state.fonts.semibold,
                DT_LEFT | DT_VCENTER | DT_SINGLELINE,
            );
            if state.action_mode == ActionMode::TextAndEnter {
                draw_label(
                    dc,
                    &format!("“{}”", state.armed_prompt),
                    Rect::new(42, 460, 458, 495),
                    COLOR_MUTED,
                    state.fonts.body,
                    DT_LEFT | DT_VCENTER | DT_SINGLELINE | DT_END_ELLIPSIS,
                );
            }
        } else {
            for (rect, label, active, target) in [
                (
                    RECT_MODE_ENTER,
                    "Hanya Enter",
                    state.action_mode == ActionMode::EnterOnly,
                    HitTarget::EnterOnly,
                ),
                (
                    RECT_MODE_TEXT,
                    "Text + Enter",
                    state.action_mode == ActionMode::TextAndEnter,
                    HitTarget::TextAndEnter,
                ),
            ] {
                draw_flat_button(
                    dc,
                    rect,
                    label,
                    if active { COLOR_ACCENT } else { COLOR_PANEL_2 },
                    if active { COLOR_INK } else { COLOR_TEXT },
                    state.hot == target,
                    state.fonts.small,
                );
            }
            rounded_box(dc, Rect::new(42, 470, 458, 514), 14, COLOR_INK, COLOR_INK);
        }

        draw_flat_button(
            dc,
            RECT_MAIN_ACTION,
            if state.running {
                "Batalkan timer"
            } else {
                "Mulai timer  →"
            },
            if state.running {
                COLOR_SURFACE_2
            } else {
                COLOR_ACCENT
            },
            if state.running { COLOR_TEXT } else { COLOR_INK },
            state.hot == HitTarget::MainAction,
            state.fonts.semibold,
        );
        let status_color = match state.status_kind {
            StatusKind::Ready => COLOR_MUTED,
            StatusKind::Running => COLOR_ACCENT,
            StatusKind::Sent => COLOR_SUCCESS,
            StatusKind::Warning => COLOR_WARNING,
            StatusKind::Error => COLOR_ERROR,
        };
        filled_circle(dc, Rect::new(28, 622, 36, 630), status_color);
        draw_label(
            dc,
            &state.status,
            Rect::new(46, 610, 492, 642),
            COLOR_MUTED,
            state.fonts.small,
            DT_LEFT | DT_VCENTER | DT_SINGLELINE | DT_END_ELLIPSIS,
        );
    }
}

unsafe fn draw_macro_interface_v3(dc: Hdc, state: &AppState) {
    unsafe {
        fill_rect_color(
            dc,
            Rect::new(0, 0, MACRO_CLIENT_WIDTH, CLIENT_HEIGHT),
            COLOR_BG,
        );
        draw_brand_header_v3(dc, state);

        let library_panel = Rect::new(24, 84, 236, 608);
        rounded_box(dc, library_panel, 24, COLOR_ACCENT, COLOR_ACCENT);
        draw_label(
            dc,
            &format!("MACRO  /  {:02}", state.macro_library.macros.len()),
            Rect::new(42, 90, 218, 113),
            COLOR_INK_MUTED,
            state.fonts.small,
            DT_LEFT | DT_VCENTER | DT_SINGLELINE,
        );
        draw_flat_button(
            dc,
            RECT_MACRO_NEW,
            "+  Macro baru",
            COLOR_INK,
            COLOR_TEXT,
            state.hot == HitTarget::MacroNew,
            state.fonts.small,
        );
        for (index, item) in state.macro_library.macros.iter().take(7).enumerate() {
            let rect = macro_item_rect(index);
            let selected = item.id == state.macro_library.selected_id;
            if selected {
                rounded_box(dc, rect, 14, COLOR_INK, COLOR_INK);
            } else if index > 0 {
                draw_hairline(
                    dc,
                    rect.left + 8,
                    rect.top,
                    rect.right - 8,
                    rgb(167, 207, 52),
                );
            }
            draw_label(
                dc,
                &item.name,
                Rect::new(rect.left + 14, rect.top + 5, rect.right - 10, rect.top + 29),
                if selected { COLOR_TEXT } else { COLOR_INK },
                state.fonts.semibold,
                DT_LEFT | DT_VCENTER | DT_SINGLELINE | DT_END_ELLIPSIS,
            );
            draw_label(
                dc,
                item.trigger.label(),
                Rect::new(
                    rect.left + 14,
                    rect.top + 27,
                    rect.right - 10,
                    rect.bottom - 2,
                ),
                if selected {
                    COLOR_ACCENT
                } else {
                    COLOR_INK_MUTED
                },
                state.fonts.small,
                DT_LEFT | DT_VCENTER | DT_SINGLELINE,
            );
        }

        let editor_panel = Rect::new(252, 84, 936, 608);
        rounded_box(dc, editor_panel, 24, COLOR_PANEL, COLOR_PANEL_BORDER);
        draw_label(
            dc,
            if state.recording {
                "Merekam input"
            } else {
                "Editor macro"
            },
            Rect::new(278, 96, 500, 119),
            if state.recording {
                COLOR_ERROR
            } else {
                COLOR_MUTED
            },
            state.fonts.small,
            DT_LEFT | DT_VCENTER | DT_SINGLELINE,
        );
        if state.recording {
            draw_label(
                dc,
                "Tekan Esc untuk selesai",
                Rect::new(680, 96, 910, 119),
                COLOR_ERROR,
                state.fonts.small,
                DT_RIGHT | DT_VCENTER | DT_SINGLELINE,
            );
        }
        let Some(item) = state.macro_library.selected() else {
            return;
        };
        rounded_box(
            dc,
            Rect::new(270, 121, 916, 162),
            13,
            COLOR_PANEL_2,
            COLOR_PANEL_BORDER,
        );

        draw_label(
            dc,
            "Perilaku",
            Rect::new(278, 151, 480, 177),
            COLOR_MUTED,
            state.fonts.small,
            DT_LEFT | DT_VCENTER | DT_SINGLELINE,
        );
        for (index, (mode, rect)) in [
            (MacroMode::NoRepeat, RECT_MACRO_MODE_NO_REPEAT),
            (MacroMode::RepeatWhileHolding, RECT_MACRO_MODE_HOLD),
            (MacroMode::Toggle, RECT_MACRO_MODE_TOGGLE),
            (MacroMode::Sequence, RECT_MACRO_MODE_SEQUENCE),
        ]
        .into_iter()
        .enumerate()
        {
            let selected = item.mode == mode;
            let hot = state.hot == HitTarget::MacroMode(mode);
            rounded_box(
                dc,
                rect,
                15,
                if selected {
                    COLOR_INK
                } else if hot {
                    COLOR_PANEL_3
                } else {
                    COLOR_PANEL_2
                },
                if selected {
                    COLOR_INK
                } else if hot {
                    COLOR_BORDER_HOT
                } else {
                    COLOR_PANEL_BORDER
                },
            );
            draw_label(
                dc,
                &format!("0{}", index + 1),
                Rect::new(rect.left + 12, rect.top + 5, rect.right - 10, rect.top + 25),
                if selected { COLOR_ACCENT } else { COLOR_MUTED },
                state.fonts.small,
                DT_LEFT | DT_VCENTER | DT_SINGLELINE,
            );
            draw_label(
                dc,
                mode.label(),
                Rect::new(
                    rect.left + 12,
                    rect.top + 26,
                    rect.right - 10,
                    rect.bottom - 7,
                ),
                COLOR_TEXT,
                state.fonts.semibold,
                DT_LEFT | DT_VCENTER | DT_SINGLELINE | DT_END_ELLIPSIS,
            );
        }

        draw_label(
            dc,
            "Pemicu",
            Rect::new(278, 245, 500, 269),
            COLOR_MUTED,
            state.fonts.small,
            DT_LEFT | DT_VCENTER | DT_SINGLELINE,
        );
        for (index, trigger) in MacroTrigger::ALL.into_iter().enumerate() {
            let active = item.trigger == trigger;
            draw_flat_button(
                dc,
                macro_trigger_rect(index),
                trigger.label(),
                if active { COLOR_INK } else { COLOR_PANEL_2 },
                if active { COLOR_ACCENT } else { COLOR_TEXT },
                state.hot == HitTarget::MacroTrigger(trigger),
                state.fonts.small,
            );
        }

        draw_label(
            dc,
            "Timeline",
            Rect::new(278, 310, 500, 334),
            COLOR_MUTED,
            state.fonts.small,
            DT_LEFT | DT_VCENTER | DT_SINGLELINE,
        );
        for (lane, rect, label) in [
            (MacroLane::OnPress, RECT_MACRO_LANE_PRESS, "Saat ditekan"),
            (
                MacroLane::WhileHolding,
                RECT_MACRO_LANE_HOLD,
                "Saat ditahan",
            ),
            (
                MacroLane::OnRelease,
                RECT_MACRO_LANE_RELEASE,
                "Saat dilepas",
            ),
        ] {
            let active = state.macro_lane == lane;
            draw_flat_button(
                dc,
                rect,
                label,
                if active { COLOR_INK } else { COLOR_PANEL_2 },
                if active { COLOR_ACCENT } else { COLOR_TEXT },
                state.hot == HitTarget::MacroLane(lane),
                state.fonts.small,
            );
        }
        let events = lane_events(item, state.macro_lane);
        draw_label(
            dc,
            &format!("{} langkah  ·  {} ms", events.len(), total_delay(events)),
            Rect::new(735, 331, 908, 365),
            COLOR_MUTED,
            state.fonts.small,
            DT_RIGHT | DT_VCENTER | DT_SINGLELINE,
        );

        let timeline = Rect::new(278, 375, 912, 532);
        rounded_box(dc, timeline, 18, COLOR_INK, COLOR_INK);
        for y in [409, 443, 477, 511] {
            draw_hairline(dc, 296, y, 894, rgb(31, 35, 29));
        }
        if events.is_empty() {
            filled_circle(dc, Rect::new(300, 414, 316, 430), COLOR_ACCENT);
            draw_label(
                dc,
                "Belum ada rekaman",
                Rect::new(330, 399, 690, 433),
                COLOR_TEXT,
                state.fonts.semibold,
                DT_LEFT | DT_VCENTER | DT_SINGLELINE,
            );
            draw_label(
                dc,
                "Rekam input untuk menyusun bagian ini.",
                Rect::new(330, 430, 720, 458),
                COLOR_MUTED,
                state.fonts.body,
                DT_LEFT | DT_VCENTER | DT_SINGLELINE,
            );
        } else {
            for (index, event) in events.iter().take(18).enumerate() {
                let column = index % 6;
                let row = index / 6;
                let left = 292 + column as i32 * 100;
                let top = 390 + row as i32 * 40;
                let is_delay = matches!(event, MacroEvent::Delay(_));
                draw_flat_button(
                    dc,
                    Rect::new(left, top, left + 90, top + 30),
                    &macro_event_label(event),
                    if is_delay {
                        COLOR_ACCENT
                    } else {
                        COLOR_SURFACE_2
                    },
                    if is_delay { COLOR_INK } else { COLOR_TEXT },
                    false,
                    state.fonts.small,
                );
            }
            if events.len() > 18 {
                draw_label(
                    dc,
                    &format!("+{} lainnya", events.len() - 18),
                    Rect::new(760, 495, 896, 520),
                    COLOR_MUTED,
                    state.fonts.small,
                    DT_RIGHT | DT_VCENTER | DT_SINGLELINE,
                );
            }
        }

        draw_flat_button(
            dc,
            RECT_MACRO_RECORD,
            if state.recording {
                "Selesai merekam"
            } else {
                "●  Rekam input"
            },
            COLOR_ACCENT,
            COLOR_INK,
            state.hot == HitTarget::MacroRecord,
            state.fonts.semibold,
        );
        draw_flat_button(
            dc,
            RECT_MACRO_CLEAR,
            "Bersihkan bagian",
            COLOR_PANEL_2,
            COLOR_TEXT,
            state.hot == HitTarget::MacroClear,
            state.fonts.small,
        );
        draw_flat_button(
            dc,
            RECT_MACRO_SAVE,
            if state.macro_dirty {
                "Simpan"
            } else {
                "Tersimpan"
            },
            COLOR_INK,
            COLOR_TEXT,
            state.hot == HitTarget::MacroSave,
            state.fonts.semibold,
        );

        let status_color = match state.macro_status_kind {
            StatusKind::Ready => COLOR_MUTED,
            StatusKind::Running => COLOR_ACCENT,
            StatusKind::Sent => COLOR_SUCCESS,
            StatusKind::Warning => COLOR_WARNING,
            StatusKind::Error => COLOR_ERROR,
        };
        filled_circle(dc, Rect::new(260, 622, 268, 630), status_color);
        draw_label(
            dc,
            &state.macro_status,
            Rect::new(278, 610, 930, 642),
            COLOR_MUTED,
            state.fonts.small,
            DT_LEFT | DT_VCENTER | DT_SINGLELINE | DT_END_ELLIPSIS,
        );
    }
}

unsafe fn draw_redesigned_interface(dc: Hdc, state: &AppState) {
    match state.tab {
        AppTab::Timer => unsafe { draw_timer_interface_v3(dc, state) },
        AppTab::Macro => unsafe { draw_macro_interface_v3(dc, state) },
    }
}

unsafe fn draw_macro_interface(dc: Hdc, state: &AppState) {
    unsafe {
        fill_rect_color(
            dc,
            Rect::new(0, 0, MACRO_CLIENT_WIDTH, CLIENT_HEIGHT),
            COLOR_BG,
        );
        draw_clock_mark(dc);
        draw_label(
            dc,
            "VibeTimer",
            Rect::new(62, 20, 260, 48),
            COLOR_TEXT,
            state.fonts.title,
            DT_LEFT | DT_VCENTER | DT_SINGLELINE,
        );
        draw_label(
            dc,
            "Macro Studio • bekerja untuk mouse HID umum",
            Rect::new(63, 47, 290, 67),
            COLOR_MUTED,
            state.fonts.small,
            DT_LEFT | DT_VCENTER | DT_SINGLELINE | DT_END_ELLIPSIS,
        );
        draw_tabs(dc, state);
        draw_status_pill(dc, state);

        rounded_box(
            dc,
            Rect::new(24, 84, 236, 608),
            18,
            COLOR_SURFACE,
            COLOR_BORDER,
        );
        draw_label(
            dc,
            "MACROS",
            Rect::new(42, 94, 218, 116),
            COLOR_MUTED,
            state.fonts.small,
            DT_LEFT | DT_VCENTER | DT_SINGLELINE,
        );
        draw_button(
            dc,
            RECT_MACRO_NEW,
            "+ Buat macro baru",
            false,
            state.hot == HitTarget::MacroNew,
            state.fonts.small,
        );
        for (index, item) in state.macro_library.macros.iter().take(7).enumerate() {
            let rect = macro_item_rect(index);
            let selected = item.id == state.macro_library.selected_id;
            rounded_box(
                dc,
                rect,
                12,
                if selected {
                    COLOR_ACCENT_DARK
                } else {
                    COLOR_SURFACE_2
                },
                if selected { COLOR_ACCENT } else { COLOR_BORDER },
            );
            filled_circle(
                dc,
                Rect::new(rect.left + 12, rect.top + 13, rect.left + 20, rect.top + 21),
                if selected { COLOR_ACCENT } else { COLOR_DIM },
            );
            draw_label(
                dc,
                &item.name,
                Rect::new(rect.left + 28, rect.top + 5, rect.right - 8, rect.top + 27),
                if selected { COLOR_TEXT } else { COLOR_MUTED },
                state.fonts.semibold,
                DT_LEFT | DT_VCENTER | DT_SINGLELINE | DT_END_ELLIPSIS,
            );
            draw_label(
                dc,
                item.trigger.label(),
                Rect::new(
                    rect.left + 28,
                    rect.top + 25,
                    rect.right - 8,
                    rect.bottom - 3,
                ),
                COLOR_DIM,
                state.fonts.small,
                DT_LEFT | DT_VCENTER | DT_SINGLELINE,
            );
        }

        rounded_box(
            dc,
            Rect::new(252, 84, 936, 608),
            18,
            COLOR_SURFACE,
            COLOR_BORDER,
        );
        draw_label(
            dc,
            if state.recording {
                "RECORDING"
            } else {
                "MACRO EDITOR"
            },
            Rect::new(278, 95, 470, 119),
            if state.recording {
                COLOR_ERROR
            } else {
                COLOR_MUTED
            },
            state.fonts.small,
            DT_LEFT | DT_VCENTER | DT_SINGLELINE,
        );
        if state.recording {
            draw_label(
                dc,
                "Semua input ditangkap • Esc untuk selesai",
                Rect::new(584, 95, 910, 119),
                COLOR_ERROR,
                state.fonts.small,
                DT_RIGHT | DT_VCENTER | DT_SINGLELINE,
            );
        }

        let Some(item) = state.macro_library.selected() else {
            return;
        };
        rounded_box(
            dc,
            Rect::new(270, 121, 916, 162),
            12,
            COLOR_SURFACE_2,
            COLOR_BORDER,
        );

        draw_label(
            dc,
            "TIPE MACRO",
            Rect::new(278, 151, 560, 177),
            COLOR_MUTED,
            state.fonts.small,
            DT_LEFT | DT_VCENTER | DT_SINGLELINE,
        );
        for (mode, rect, hint) in [
            (
                MacroMode::NoRepeat,
                RECT_MACRO_MODE_NO_REPEAT,
                "Jalankan sekali",
            ),
            (
                MacroMode::RepeatWhileHolding,
                RECT_MACRO_MODE_HOLD,
                "Ulang saat ditahan",
            ),
            (
                MacroMode::Toggle,
                RECT_MACRO_MODE_TOGGLE,
                "Klik hidup / mati",
            ),
            (
                MacroMode::Sequence,
                RECT_MACRO_MODE_SEQUENCE,
                "Press • hold • release",
            ),
        ] {
            let selected = item.mode == mode;
            rounded_box(
                dc,
                rect,
                14,
                if selected {
                    COLOR_ACCENT_DARK
                } else {
                    COLOR_SURFACE_2
                },
                if selected {
                    COLOR_ACCENT
                } else if state.hot == HitTarget::MacroMode(mode) {
                    COLOR_BORDER_HOT
                } else {
                    COLOR_BORDER
                },
            );
            draw_label(
                dc,
                mode.label(),
                Rect::new(rect.left + 10, rect.top + 7, rect.right - 10, rect.top + 33),
                if selected { COLOR_TEXT } else { COLOR_MUTED },
                state.fonts.semibold,
                DT_CENTER | DT_VCENTER | DT_SINGLELINE,
            );
            draw_label(
                dc,
                hint,
                Rect::new(
                    rect.left + 7,
                    rect.top + 34,
                    rect.right - 7,
                    rect.bottom - 6,
                ),
                COLOR_DIM,
                state.fonts.small,
                DT_CENTER | DT_VCENTER | DT_SINGLELINE | DT_END_ELLIPSIS,
            );
        }

        draw_label(
            dc,
            "PEMICU GLOBAL",
            Rect::new(278, 245, 500, 269),
            COLOR_MUTED,
            state.fonts.small,
            DT_LEFT | DT_VCENTER | DT_SINGLELINE,
        );
        for (index, trigger) in MacroTrigger::ALL.into_iter().enumerate() {
            draw_button(
                dc,
                macro_trigger_rect(index),
                trigger.label(),
                item.trigger == trigger,
                state.hot == HitTarget::MacroTrigger(trigger),
                state.fonts.small,
            );
        }

        draw_label(
            dc,
            "TIMELINE",
            Rect::new(278, 310, 500, 334),
            COLOR_MUTED,
            state.fonts.small,
            DT_LEFT | DT_VCENTER | DT_SINGLELINE,
        );
        for (lane, rect, label) in [
            (MacroLane::OnPress, RECT_MACRO_LANE_PRESS, "On Press"),
            (
                MacroLane::WhileHolding,
                RECT_MACRO_LANE_HOLD,
                "While Holding",
            ),
            (MacroLane::OnRelease, RECT_MACRO_LANE_RELEASE, "On Release"),
        ] {
            draw_button(
                dc,
                rect,
                label,
                state.macro_lane == lane,
                state.hot == HitTarget::MacroLane(lane),
                state.fonts.small,
            );
        }
        let events = lane_events(item, state.macro_lane);
        draw_label(
            dc,
            &format!("{} langkah • {} ms", events.len(), total_delay(events)),
            Rect::new(735, 331, 908, 365),
            COLOR_DIM,
            state.fonts.small,
            DT_RIGHT | DT_VCENTER | DT_SINGLELINE,
        );
        rounded_box(
            dc,
            Rect::new(278, 375, 912, 532),
            14,
            COLOR_SURFACE_2,
            COLOR_BORDER,
        );
        if events.is_empty() {
            draw_label(
                dc,
                "Timeline kosong — tekan Rekam lalu lakukan kombinasi tombol atau klik mouse.",
                Rect::new(300, 405, 890, 487),
                COLOR_DIM,
                state.fonts.body,
                DT_CENTER | DT_VCENTER | DT_SINGLELINE | DT_END_ELLIPSIS,
            );
        } else {
            for (index, event) in events.iter().take(18).enumerate() {
                let column = index % 6;
                let row = index / 6;
                let left = 292 + column as i32 * 100;
                let top = 390 + row as i32 * 40;
                let is_delay = matches!(event, MacroEvent::Delay(_));
                rounded_box(
                    dc,
                    Rect::new(left, top, left + 90, top + 30),
                    10,
                    if is_delay {
                        rgb(45, 38, 62)
                    } else {
                        rgb(27, 38, 51)
                    },
                    if is_delay {
                        COLOR_BORDER_HOT
                    } else {
                        COLOR_BORDER
                    },
                );
                draw_label(
                    dc,
                    &macro_event_label(event),
                    Rect::new(left + 5, top, left + 85, top + 30),
                    if is_delay {
                        rgb(196, 173, 255)
                    } else {
                        COLOR_TEXT
                    },
                    state.fonts.small,
                    DT_CENTER | DT_VCENTER | DT_SINGLELINE | DT_END_ELLIPSIS,
                );
            }
            if events.len() > 18 {
                draw_label(
                    dc,
                    &format!("+{} langkah lainnya", events.len() - 18),
                    Rect::new(760, 495, 896, 520),
                    COLOR_DIM,
                    state.fonts.small,
                    DT_RIGHT | DT_VCENTER | DT_SINGLELINE,
                );
            }
        }

        draw_button(
            dc,
            RECT_MACRO_RECORD,
            if state.recording {
                "Stop recording"
            } else {
                "●  Rekam input"
            },
            state.recording,
            state.hot == HitTarget::MacroRecord,
            state.fonts.semibold,
        );
        draw_button(
            dc,
            RECT_MACRO_CLEAR,
            "Bersihkan lane",
            false,
            state.hot == HitTarget::MacroClear,
            state.fonts.small,
        );
        draw_button(
            dc,
            RECT_MACRO_SAVE,
            if state.macro_dirty {
                "Simpan *"
            } else {
                "Tersimpan"
            },
            true,
            state.hot == HitTarget::MacroSave,
            state.fonts.semibold,
        );

        let status_color = match state.macro_status_kind {
            StatusKind::Ready => COLOR_MUTED,
            StatusKind::Running => COLOR_ACCENT,
            StatusKind::Sent => COLOR_SUCCESS,
            StatusKind::Warning => COLOR_WARNING,
            StatusKind::Error => COLOR_ERROR,
        };
        filled_circle(dc, Rect::new(260, 622, 268, 630), status_color);
        draw_label(
            dc,
            &state.macro_status,
            Rect::new(278, 612, 930, 640),
            COLOR_MUTED,
            state.fonts.small,
            DT_LEFT | DT_VCENTER | DT_SINGLELINE | DT_END_ELLIPSIS,
        );
    }
}

#[allow(dead_code)]
unsafe fn draw_interface(dc: Hdc, state: &AppState) {
    if state.tab == AppTab::Macro {
        unsafe { draw_macro_interface(dc, state) };
        return;
    }
    unsafe {
        fill_rect_color(dc, Rect::new(0, 0, CLIENT_WIDTH, CLIENT_HEIGHT), COLOR_BG);

        draw_clock_mark(dc);
        draw_label(
            dc,
            "VibeTimer",
            Rect::new(62, 20, 260, 48),
            COLOR_TEXT,
            state.fonts.title,
            DT_LEFT | DT_VCENTER | DT_SINGLELINE,
        );
        draw_label(
            dc,
            "Lanjut otomatis setelah limit reset",
            Rect::new(63, 47, 340, 67),
            COLOR_MUTED,
            state.fonts.small,
            DT_LEFT | DT_VCENTER | DT_SINGLELINE,
        );
        draw_tabs(dc, state);

        let time_card = Rect::new(24, 84, 496, 274);
        rounded_box(dc, time_card, 18, COLOR_SURFACE, COLOR_BORDER);
        draw_label(
            dc,
            if state.running {
                "SISA WAKTU"
            } else {
                "WAKTU TUNGGU"
            },
            Rect::new(44, 98, 250, 120),
            COLOR_MUTED,
            state.fonts.small,
            DT_LEFT | DT_VCENTER | DT_SINGLELINE,
        );

        if state.running {
            draw_label(
                dc,
                &format_duration(state.remaining_seconds),
                Rect::new(44, 120, 476, 200),
                COLOR_TEXT,
                state.fonts.timer,
                DT_CENTER | DT_VCENTER | DT_SINGLELINE,
            );
            let track = Rect::new(48, 214, 472, 222);
            rounded_box(dc, track, 8, COLOR_SURFACE_2, COLOR_SURFACE_2);
            let fraction = if state.original_seconds == 0 {
                0.0
            } else {
                1.0 - state.remaining_seconds as f64 / state.original_seconds as f64
            };
            let progress = (424.0 * fraction.clamp(0.0, 1.0)) as i32;
            if progress > 0 {
                rounded_box(
                    dc,
                    Rect::new(48, 214, 48 + progress.max(8), 222),
                    8,
                    COLOR_ACCENT,
                    COLOR_ACCENT,
                );
            }
            draw_label(
                dc,
                "VibeTimer akan melakukan aksi satu kali saat mencapai nol.",
                Rect::new(48, 234, 472, 256),
                COLOR_MUTED,
                state.fonts.small,
                DT_CENTER | DT_VCENTER | DT_SINGLELINE,
            );
        } else {
            for rect in [
                Rect::new(48, 124, 158, 190),
                Rect::new(194, 124, 304, 190),
                Rect::new(340, 124, 450, 190),
            ] {
                rounded_box(dc, rect, 14, COLOR_SURFACE_2, COLOR_BORDER);
            }
            draw_label(
                dc,
                ":",
                Rect::new(164, 126, 188, 184),
                COLOR_DIM,
                state.fonts.timer,
                DT_CENTER | DT_VCENTER | DT_SINGLELINE,
            );
            draw_label(
                dc,
                ":",
                Rect::new(310, 126, 334, 184),
                COLOR_DIM,
                state.fonts.timer,
                DT_CENTER | DT_VCENTER | DT_SINGLELINE,
            );
            for (label, rect) in [
                ("JAM", Rect::new(48, 193, 158, 215)),
                ("MENIT", Rect::new(194, 193, 304, 215)),
                ("DETIK", Rect::new(340, 193, 450, 215)),
            ] {
                draw_label(
                    dc,
                    label,
                    rect,
                    COLOR_DIM,
                    state.fonts.small,
                    DT_CENTER | DT_VCENTER | DT_SINGLELINE,
                );
            }
            draw_button(
                dc,
                RECT_QUICK_30,
                "+30 menit",
                false,
                state.hot == HitTarget::AddThirtyMinutes,
                state.fonts.small,
            );
            draw_button(
                dc,
                RECT_QUICK_60,
                "+1 jam",
                false,
                state.hot == HitTarget::AddOneHour,
                state.fonts.small,
            );
            draw_button(
                dc,
                RECT_QUICK_180,
                "+3 jam",
                false,
                state.hot == HitTarget::AddThreeHours,
                state.fonts.small,
            );
        }

        let target_card = Rect::new(24, 290, 496, 374);
        rounded_box(dc, target_card, 18, COLOR_SURFACE, COLOR_BORDER);
        draw_label(
            dc,
            "TARGET JENDELA",
            Rect::new(42, 300, 260, 322),
            COLOR_MUTED,
            state.fonts.small,
            DT_LEFT | DT_VCENTER | DT_SINGLELINE,
        );
        let target_text = state
            .target
            .as_ref()
            .map(|target| target.title.as_str())
            .unwrap_or("Belum ada target dipilih");
        draw_label(
            dc,
            target_text,
            Rect::new(42, 322, 344, 346),
            if state.target.is_some() {
                COLOR_TEXT
            } else {
                COLOR_DIM
            },
            state.fonts.semibold,
            DT_LEFT | DT_VCENTER | DT_SINGLELINE | DT_END_ELLIPSIS,
        );
        draw_label(
            dc,
            "Klik kolom input AI saat proses pemilihan.",
            Rect::new(42, 347, 344, 365),
            COLOR_DIM,
            state.fonts.small,
            DT_LEFT | DT_VCENTER | DT_SINGLELINE,
        );
        draw_button(
            dc,
            RECT_PICK_TARGET,
            if state.running {
                "Target terkunci"
            } else {
                "Pilih target"
            },
            state.target.is_some() && !state.running,
            state.hot == HitTarget::PickTarget && !state.running,
            state.fonts.small,
        );

        let action_card = Rect::new(24, 390, 496, 526);
        rounded_box(dc, action_card, 18, COLOR_SURFACE, COLOR_BORDER);
        draw_label(
            dc,
            "AKSI SAAT NOL",
            Rect::new(42, 401, 260, 421),
            COLOR_MUTED,
            state.fonts.small,
            DT_LEFT | DT_VCENTER | DT_SINGLELINE,
        );
        if state.running {
            let action = match state.action_mode {
                ActionMode::EnterOnly => "Tekan Enter satu kali",
                ActionMode::TextAndEnter => "Ketik teks, lalu tekan Enter",
            };
            draw_label(
                dc,
                action,
                Rect::new(42, 426, 458, 451),
                COLOR_TEXT,
                state.fonts.semibold,
                DT_LEFT | DT_VCENTER | DT_SINGLELINE,
            );
            if state.action_mode == ActionMode::TextAndEnter {
                let preview = format!("\u{201c}{}\u{201d}", state.armed_prompt);
                draw_label(
                    dc,
                    &preview,
                    Rect::new(42, 458, 458, 500),
                    COLOR_MUTED,
                    state.fonts.body,
                    DT_LEFT | DT_VCENTER | DT_SINGLELINE | DT_END_ELLIPSIS,
                );
            }
        } else {
            draw_button(
                dc,
                RECT_MODE_ENTER,
                "Hanya Enter",
                state.action_mode == ActionMode::EnterOnly,
                state.hot == HitTarget::EnterOnly,
                state.fonts.small,
            );
            draw_button(
                dc,
                RECT_MODE_TEXT,
                "Teks + Enter",
                state.action_mode == ActionMode::TextAndEnter,
                state.hot == HitTarget::TextAndEnter,
                state.fonts.small,
            );
            rounded_box(
                dc,
                Rect::new(42, 474, 458, 512),
                12,
                COLOR_SURFACE_2,
                if state.action_mode == ActionMode::TextAndEnter {
                    COLOR_BORDER
                } else {
                    COLOR_SURFACE_2
                },
            );
        }

        draw_button(
            dc,
            RECT_MAIN_ACTION,
            if state.running {
                "Batalkan timer"
            } else {
                "Mulai timer"
            },
            true,
            state.hot == HitTarget::MainAction,
            state.fonts.semibold,
        );

        let status_color = match state.status_kind {
            StatusKind::Ready => COLOR_MUTED,
            StatusKind::Running => COLOR_ACCENT,
            StatusKind::Sent => COLOR_SUCCESS,
            StatusKind::Warning => COLOR_WARNING,
            StatusKind::Error => COLOR_ERROR,
        };
        filled_circle(dc, Rect::new(28, 622, 36, 630), status_color);
        draw_label(
            dc,
            &state.status,
            Rect::new(44, 612, 492, 640),
            COLOR_MUTED,
            state.fonts.small,
            DT_LEFT | DT_VCENTER | DT_SINGLELINE | DT_END_ELLIPSIS,
        );
    }
}

struct EditSpec<'a> {
    id: isize,
    text: &'a str,
    bounds: Rect,
    style: Dword,
}

unsafe fn create_edit(parent: Hwnd, instance: Hinstance, spec: EditSpec<'_>) -> Hwnd {
    let class_name = wide("EDIT");
    let initial = wide(spec.text);
    unsafe {
        CreateWindowExW(
            0,
            class_name.as_ptr(),
            initial.as_ptr(),
            WS_CHILD | WS_VISIBLE | ES_AUTOHSCROLL | spec.style,
            spec.bounds.left,
            spec.bounds.top,
            spec.bounds.right - spec.bounds.left,
            spec.bounds.bottom - spec.bounds.top,
            parent,
            spec.id,
            instance,
            null_mut(),
        )
    }
}

unsafe fn initialize_controls(state: &mut AppState, instance: Hinstance) {
    unsafe {
        state.fonts.title = make_font(24, 750);
        state.fonts.timer = make_font(46, 650);
        state.fonts.body = make_font(17, 450);
        state.fonts.semibold = make_font(16, 650);
        state.fonts.small = make_font(13, 600);
        state.edit_brush = CreateSolidBrush(COLOR_BG);
        state.panel_edit_brush = CreateSolidBrush(COLOR_PANEL_2);

        state.hour_edit = create_edit(
            state.window,
            instance,
            EditSpec {
                id: 101,
                text: "00",
                bounds: Rect::new(54, 132, 152, 182),
                style: ES_CENTER | ES_NUMBER,
            },
        );
        state.minute_edit = create_edit(
            state.window,
            instance,
            EditSpec {
                id: 102,
                text: "30",
                bounds: Rect::new(200, 132, 298, 182),
                style: ES_CENTER | ES_NUMBER,
            },
        );
        state.second_edit = create_edit(
            state.window,
            instance,
            EditSpec {
                id: 103,
                text: "00",
                bounds: Rect::new(346, 132, 444, 182),
                style: ES_CENTER | ES_NUMBER,
            },
        );
        state.prompt_edit = create_edit(
            state.window,
            instance,
            EditSpec {
                id: 104,
                text: "lanjutkan",
                bounds: Rect::new(52, 481, 448, 505),
                style: 0,
            },
        );
        let initial_macro_name = state
            .macro_library
            .selected()
            .map(|item| item.name.clone())
            .unwrap_or_else(|| "Macro".to_owned());
        state.macro_name_edit = create_edit(
            state.window,
            instance,
            EditSpec {
                id: 105,
                text: &initial_macro_name,
                bounds: Rect::new(282, 130, 904, 154),
                style: 0,
            },
        );
        ShowWindow(state.macro_name_edit, SW_HIDE);

        for edit in [state.hour_edit, state.minute_edit, state.second_edit] {
            SendMessageW(
                edit,
                WM_SETFONT,
                state.fonts.timer as Wparam,
                TRUE as Lparam,
            );
            SendMessageW(edit, EM_SETLIMITTEXT, 2, 0);
            SendMessageW(
                edit,
                EM_SETMARGINS,
                EC_LEFTMARGIN | EC_RIGHTMARGIN,
                (4 | (4 << 16)) as Lparam,
            );
            let dark = wide("DarkMode_CFD");
            SetWindowTheme(edit, dark.as_ptr(), null());
        }

        SendMessageW(
            state.prompt_edit,
            WM_SETFONT,
            state.fonts.body as Wparam,
            TRUE as Lparam,
        );
        SendMessageW(state.prompt_edit, EM_SETLIMITTEXT, 160, 0);
        SendMessageW(
            state.prompt_edit,
            EM_SETMARGINS,
            EC_LEFTMARGIN | EC_RIGHTMARGIN,
            (8 | (8 << 16)) as Lparam,
        );
        let dark = wide("DarkMode_CFD");
        SetWindowTheme(state.prompt_edit, dark.as_ptr(), null());
        SendMessageW(
            state.macro_name_edit,
            WM_SETFONT,
            state.fonts.semibold as Wparam,
            TRUE as Lparam,
        );
        SendMessageW(state.macro_name_edit, EM_SETLIMITTEXT, 80, 0);
        SendMessageW(
            state.macro_name_edit,
            EM_SETMARGINS,
            EC_LEFTMARGIN | EC_RIGHTMARGIN,
            (8 | (8 << 16)) as Lparam,
        );
        SetWindowTheme(state.macro_name_edit, dark.as_ptr(), null());
    }
}

unsafe fn get_window_text(window: Hwnd) -> String {
    unsafe {
        let length = GetWindowTextLengthW(window);
        if length <= 0 {
            return String::new();
        }
        let mut buffer = vec![0u16; length as usize + 1];
        let written = GetWindowTextW(window, buffer.as_mut_ptr(), buffer.len() as i32);
        String::from_utf16_lossy(&buffer[..written.max(0) as usize])
    }
}

unsafe fn read_number(window: Hwnd) -> u32 {
    unsafe { get_window_text(window).trim().parse::<u32>().unwrap_or(0) }
}

unsafe fn read_duration_fields(state: &AppState) -> DurationFields {
    unsafe {
        DurationFields::new(
            read_number(state.hour_edit),
            read_number(state.minute_edit),
            read_number(state.second_edit),
        )
    }
}

unsafe fn set_duration_fields(state: &AppState, fields: DurationFields) {
    unsafe {
        SetWindowTextW(
            state.hour_edit,
            wide(&format!("{:02}", fields.hours)).as_ptr(),
        );
        SetWindowTextW(
            state.minute_edit,
            wide(&format!("{:02}", fields.minutes)).as_ptr(),
        );
        SetWindowTextW(
            state.second_edit,
            wide(&format!("{:02}", fields.seconds)).as_ptr(),
        );
    }
}

unsafe fn show_error(window: Hwnd, message: &str) {
    unsafe {
        MessageBoxW(
            window,
            wide(message).as_ptr(),
            wide("VibeTimer").as_ptr(),
            MB_OK | MB_ICONERROR,
        );
    }
}

unsafe fn show_warning(window: Hwnd, message: &str) {
    unsafe {
        MessageBoxW(
            window,
            wide(message).as_ptr(),
            wide("VibeTimer").as_ptr(),
            MB_OK | MB_ICONWARNING,
        );
    }
}

unsafe fn validate_target(target: &TargetWindow) -> Result<(), String> {
    unsafe {
        if IsWindow(target.window) == FALSE || IsWindowVisible(target.window) == FALSE {
            return Err("Jendela target sudah ditutup atau tidak tersedia.".to_owned());
        }
        let mut current_pid = 0;
        GetWindowThreadProcessId(target.window, &mut current_pid);
        if current_pid == 0 || current_pid != target.process_id {
            return Err(
                "Jendela target sudah berganti proses. Pilih target kembali agar aman.".to_owned(),
            );
        }
        Ok(())
    }
}

unsafe fn begin_target_capture(state: &mut AppState) {
    let instruction = "Setelah menekan OK, VibeTimer akan mengecil selama 3 detik.\n\nBuka jendela AI lalu klik tepat di kolom input tempat perintah harus diketik.";
    let response = unsafe {
        MessageBoxW(
            state.window,
            wide(instruction).as_ptr(),
            wide("Pilih target dengan aman").as_ptr(),
            MB_OKCANCEL | MB_ICONINFORMATION,
        )
    };
    if response != IDOK {
        return;
    }

    state.status_kind = StatusKind::Warning;
    state.status = "Menangkap jendela aktif dalam 3 detik...".to_owned();
    state.capture_deadline = Some(Instant::now() + Duration::from_secs(3));
    unsafe {
        InvalidateRect(state.window, null(), FALSE);
        ShowWindow(state.window, SW_MINIMIZE);
        SetTimer(state.window, TIMER_CAPTURE, 100, null());
    }
}

unsafe fn finish_target_capture(state: &mut AppState) {
    unsafe {
        KillTimer(state.window, TIMER_CAPTURE);
        state.capture_deadline = None;
        let target_window = GetForegroundWindow();

        ShowWindow(state.window, SW_RESTORE);
        SetForegroundWindow(state.window);

        if target_window == 0
            || target_window == state.window
            || IsWindowVisible(target_window) == FALSE
        {
            state.status_kind = StatusKind::Error;
            state.status = "Target tidak tertangkap. Coba pilih kembali.".to_owned();
            show_warning(
                state.window,
                "Target tidak tertangkap. Ulangi lalu klik jendela AI sebelum tiga detik habis.",
            );
            InvalidateRect(state.window, null(), FALSE);
            return;
        }

        let title = get_window_text(target_window);
        if title.trim().is_empty() {
            state.status_kind = StatusKind::Error;
            state.status = "Jendela tanpa judul tidak dapat dipakai.".to_owned();
            show_warning(
                state.window,
                "Jendela target tidak memiliki judul yang dapat diverifikasi.",
            );
            InvalidateRect(state.window, null(), FALSE);
            return;
        }

        let mut process_id = 0;
        GetWindowThreadProcessId(target_window, &mut process_id);
        state.target = Some(TargetWindow {
            window: target_window,
            process_id,
            title: title.clone(),
        });
        state.status_kind = StatusKind::Ready;
        state.status = format!("Target siap: {title}");
        InvalidateRect(state.window, null(), FALSE);
    }
}

unsafe fn add_preset(state: &mut AppState, seconds: u64) {
    unsafe {
        let current = read_duration_fields(state);
        set_duration_fields(state, current.add_seconds(seconds));
        state.status_kind = StatusKind::Ready;
        state.status = "Waktu tunggu diperbarui.".to_owned();
        InvalidateRect(state.window, null(), FALSE);
    }
}

fn keyboard_input(virtual_key: u16, scan_code: u16, flags: Dword) -> Input {
    Input {
        kind: INPUT_KEYBOARD,
        data: InputData {
            keyboard: KeyboardInput {
                virtual_key,
                scan_code,
                flags,
                time: 0,
                extra_info: 0,
            },
        },
    }
}

fn mouse_input(mouse_data: Dword, flags: Dword) -> Input {
    Input {
        kind: INPUT_MOUSE,
        data: InputData {
            mouse: MouseInput {
                dx: 0,
                dy: 0,
                mouse_data,
                flags,
                time: 0,
                extra_info: 0,
            },
        },
    }
}

fn macro_event_input(event: &MacroEvent) -> Option<Input> {
    match *event {
        MacroEvent::Delay(_) => None,
        MacroEvent::KeyDown(key) => Some(keyboard_input(key, 0, 0)),
        MacroEvent::KeyUp(key) => Some(keyboard_input(key, 0, KEYEVENTF_KEYUP)),
        MacroEvent::MouseDown(button) => {
            let (data, flags) = match button {
                MouseButton::Left => (0, MOUSEEVENTF_LEFTDOWN),
                MouseButton::Right => (0, MOUSEEVENTF_RIGHTDOWN),
                MouseButton::Middle => (0, MOUSEEVENTF_MIDDLEDOWN),
                MouseButton::X1 => (XBUTTON1 as Dword, MOUSEEVENTF_XDOWN),
                MouseButton::X2 => (XBUTTON2 as Dword, MOUSEEVENTF_XDOWN),
            };
            Some(mouse_input(data, flags))
        }
        MacroEvent::MouseUp(button) => {
            let (data, flags) = match button {
                MouseButton::Left => (0, MOUSEEVENTF_LEFTUP),
                MouseButton::Right => (0, MOUSEEVENTF_RIGHTUP),
                MouseButton::Middle => (0, MOUSEEVENTF_MIDDLEUP),
                MouseButton::X1 => (XBUTTON1 as Dword, MOUSEEVENTF_XUP),
                MouseButton::X2 => (XBUTTON2 as Dword, MOUSEEVENTF_XUP),
            };
            Some(mouse_input(data, flags))
        }
        MacroEvent::Wheel(delta) => Some(mouse_input(delta as i32 as Dword, MOUSEEVENTF_WHEEL)),
    }
}

unsafe fn submit_inputs(inputs: &[Input]) -> Result<(), String> {
    if inputs.is_empty() {
        return Ok(());
    }
    let sent = unsafe {
        SendInput(
            inputs.len() as Uint,
            inputs.as_ptr(),
            size_of::<Input>() as i32,
        )
    };
    if sent == inputs.len() as Uint {
        Ok(())
    } else {
        #[cfg(test)]
        {
            let target = TEST_INPUT_TARGET.load(Ordering::Relaxed);
            if target != 0 {
                for input in inputs {
                    if input.kind != INPUT_KEYBOARD {
                        continue;
                    }
                    let keyboard = unsafe { input.data.keyboard };
                    if keyboard.flags & KEYEVENTF_KEYUP != 0 {
                        continue;
                    }
                    if keyboard.flags & KEYEVENTF_UNICODE != 0 {
                        unsafe {
                            SendMessageW(target, WM_CHAR, keyboard.scan_code as Wparam, 0);
                        }
                    } else if keyboard.virtual_key == VK_RETURN {
                        unsafe {
                            SendMessageW(target, WM_CHAR, '\r' as Wparam, 0);
                        }
                    }
                }
                return Ok(());
            }
        }
        Err("Windows menolak input. Target mungkin berjalan sebagai Administrator.".to_owned())
    }
}

unsafe fn send_unicode_text(text: &str) -> Result<(), String> {
    let mut inputs = Vec::with_capacity(text.encode_utf16().count() * 2);
    for code_unit in text.encode_utf16() {
        inputs.push(keyboard_input(0, code_unit, KEYEVENTF_UNICODE));
        inputs.push(keyboard_input(
            0,
            code_unit,
            KEYEVENTF_UNICODE | KEYEVENTF_KEYUP,
        ));
    }
    unsafe { submit_inputs(&inputs) }
}

unsafe fn send_enter() -> Result<(), String> {
    let inputs = [
        keyboard_input(VK_RETURN, 0, 0),
        keyboard_input(VK_RETURN, 0, KEYEVENTF_KEYUP),
    ];
    unsafe { submit_inputs(&inputs) }
}

unsafe fn focus_target(target: Hwnd) -> Result<(), String> {
    unsafe {
        let current_thread = GetCurrentThreadId();
        let foreground = GetForegroundWindow();
        let foreground_thread = if foreground != 0 {
            GetWindowThreadProcessId(foreground, null_mut())
        } else {
            0
        };
        let target_thread = GetWindowThreadProcessId(target, null_mut());

        let attached_foreground = foreground_thread != 0 && foreground_thread != current_thread;
        let attached_target = target_thread != 0 && target_thread != current_thread;

        if attached_foreground {
            AttachThreadInput(current_thread, foreground_thread, TRUE);
        }
        if attached_target && target_thread != foreground_thread {
            AttachThreadInput(current_thread, target_thread, TRUE);
        }

        if IsIconic(target) != FALSE {
            ShowWindow(target, SW_RESTORE);
        }
        BringWindowToTop(target);
        SetForegroundWindow(target);

        if attached_target && target_thread != foreground_thread {
            AttachThreadInput(current_thread, target_thread, FALSE);
        }
        if attached_foreground {
            AttachThreadInput(current_thread, foreground_thread, FALSE);
        }
    }

    thread::sleep(Duration::from_millis(140));
    if unsafe { GetForegroundWindow() } == target {
        Ok(())
    } else {
        #[cfg(test)]
        {
            // Desktop test noninteraktif tidak memiliki foreground window global.
            // Active window per-thread tetap cukup untuk membuktikan SendInput.
            unsafe { SetActiveWindow(target) };
            Ok(())
        }
        #[cfg(not(test))]
        {
            Err("Windows tidak mengizinkan VibeTimer memfokuskan jendela target.".to_owned())
        }
    }
}

unsafe fn perform_scheduled_action(
    target: &TargetWindow,
    mode: ActionMode,
    prompt: &str,
) -> Result<(), String> {
    unsafe {
        validate_target(target)?;
        focus_target(target.window)?;
        if mode == ActionMode::TextAndEnter {
            send_unicode_text(prompt)?;
            thread::sleep(Duration::from_millis(90));
        }
        send_enter()?;
        Ok(())
    }
}

unsafe fn begin_timer(state: &mut AppState) {
    let total = match unsafe { read_duration_fields(state) }.validate() {
        Ok(total) => total,
        Err(message) => {
            state.status_kind = StatusKind::Error;
            state.status = message.to_owned();
            unsafe {
                show_error(state.window, message);
                InvalidateRect(state.window, null(), FALSE);
            }
            return;
        }
    };

    let target = match state.target.as_ref() {
        Some(target) => target,
        None => {
            state.status_kind = StatusKind::Error;
            state.status = "Pilih jendela target terlebih dahulu.".to_owned();
            unsafe {
                show_error(
                    state.window,
                    "Pilih jendela AI terlebih dahulu agar VibeTimer tidak mengetik ke tempat yang salah.",
                );
                InvalidateRect(state.window, null(), FALSE);
            }
            return;
        }
    };

    if let Err(message) = unsafe { validate_target(target) } {
        state.target = None;
        state.status_kind = StatusKind::Error;
        state.status = message.clone();
        unsafe {
            show_error(state.window, &message);
            InvalidateRect(state.window, null(), FALSE);
        }
        return;
    }

    let prompt = unsafe { get_window_text(state.prompt_edit) };
    if state.action_mode == ActionMode::TextAndEnter && prompt.trim().is_empty() {
        state.status_kind = StatusKind::Error;
        state.status = "Isi teks yang akan dikirim, atau pilih Hanya Enter.".to_owned();
        unsafe {
            show_error(
                state.window,
                "Teks perintah masih kosong. Isi teks atau pilih mode Hanya Enter.",
            );
            InvalidateRect(state.window, null(), FALSE);
        }
        return;
    }

    state.original_seconds = total;
    state.remaining_seconds = total;
    state.armed_prompt = prompt.trim().to_owned();
    state.deadline = Instant::now().checked_add(Duration::from_secs(total));
    state.running = true;
    state.status_kind = StatusKind::Running;
    state.status = "Timer aktif. Target akan difokuskan otomatis saat nol.".to_owned();
    unsafe {
        state.set_controls_visible(false);
        state.set_prompt_enabled();
        SetTimer(state.window, TIMER_COUNTDOWN, 100, null());
        InvalidateRect(state.window, null(), FALSE);
    }
}

unsafe fn cancel_timer(state: &mut AppState) {
    unsafe {
        KillTimer(state.window, TIMER_COUNTDOWN);
        state.running = false;
        state.deadline = None;
        state.status_kind = StatusKind::Warning;
        state.status = "Timer dibatalkan. Tidak ada input yang dikirim.".to_owned();
        state.set_controls_visible(true);
        state.set_prompt_enabled();
        InvalidateRect(state.window, null(), FALSE);
    }
}

unsafe fn finish_timer(state: &mut AppState) {
    unsafe {
        KillTimer(state.window, TIMER_COUNTDOWN);
        state.running = false;
        state.deadline = None;
        state.remaining_seconds = 0;

        let result = match state.target.clone() {
            Some(target) => {
                perform_scheduled_action(&target, state.action_mode, &state.armed_prompt)
                    .map(|_| target.title)
            }
            None => Err("Target tidak tersedia.".to_owned()),
        };

        state.set_controls_visible(true);
        state.set_prompt_enabled();
        match result {
            Ok(title) => {
                state.status_kind = StatusKind::Sent;
                state.status = format!("Perintah berhasil dikirim ke {title}");
                MessageBeep(MB_OK);
            }
            Err(message) => {
                state.status_kind = StatusKind::Error;
                state.status = message.clone();
                ShowWindow(state.window, SW_RESTORE);
                SetForegroundWindow(state.window);
                #[cfg(not(test))]
                show_error(state.window, &message);
            }
        }
        InvalidateRect(state.window, null(), FALSE);
    }
}

unsafe fn update_countdown(state: &mut AppState) {
    let Some(deadline) = state.deadline else {
        return;
    };
    let remaining = deadline.saturating_duration_since(Instant::now());
    if remaining.is_zero() {
        unsafe { finish_timer(state) };
        return;
    }
    let rounded_up = remaining.as_secs() + u64::from(remaining.subsec_nanos() > 0);
    if rounded_up != state.remaining_seconds {
        state.remaining_seconds = rounded_up;
        unsafe {
            InvalidateRect(state.window, null(), FALSE);
        }
    }
}

unsafe fn resize_for_tab(state: &AppState) {
    let client_width = if state.tab == AppTab::Macro {
        MACRO_CLIENT_WIDTH
    } else {
        CLIENT_WIDTH
    };
    let style = WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU | WS_MINIMIZEBOX;
    let mut outer = Rect::new(0, 0, client_width, CLIENT_HEIGHT);
    unsafe {
        AdjustWindowRectEx(&mut outer, style, FALSE, 0);
        SetWindowPos(
            state.window,
            0,
            0,
            0,
            outer.right - outer.left,
            outer.bottom - outer.top,
            SWP_NOMOVE | SWP_NOZORDER,
        );
    }
}

unsafe fn sync_macro_name_edit(state: &AppState) {
    let name = state
        .macro_library
        .selected()
        .map(|item| item.name.as_str())
        .unwrap_or("Macro");
    unsafe { SetWindowTextW(state.macro_name_edit, wide(name).as_ptr()) };
}

unsafe fn switch_tab(state: &mut AppState, tab: AppTab) {
    if state.tab == tab {
        return;
    }
    if state.recording {
        unsafe { stop_macro_recording(state) };
    }
    state.tab = tab;
    state.hot = HitTarget::None;
    unsafe {
        resize_for_tab(state);
        state.set_controls_visible(!state.running);
        state.set_prompt_enabled();
        InvalidateRect(state.window, null(), FALSE);
    }
}

fn selected_lane_mut(state: &mut AppState) -> Option<&mut Vec<MacroEvent>> {
    let lane = state.macro_lane;
    let item = state.macro_library.selected_mut()?;
    Some(match lane {
        MacroLane::OnPress => &mut item.on_press,
        MacroLane::WhileHolding => &mut item.while_holding,
        MacroLane::OnRelease => &mut item.on_release,
    })
}

unsafe fn save_current_macro(state: &mut AppState) {
    let name = unsafe { get_window_text(state.macro_name_edit) };
    if name.trim().is_empty() {
        state.macro_status_kind = StatusKind::Error;
        state.macro_status = "Nama macro tidak boleh kosong.".to_owned();
        unsafe { InvalidateRect(state.window, null(), FALSE) };
        return;
    }
    if let Some(item) = state.macro_library.selected_mut() {
        item.name = name.trim().to_owned();
    }
    match save_library(&state.macro_path, &state.macro_library) {
        Ok(()) => {
            state.macro_dirty = false;
            state.macro_status_kind = StatusKind::Sent;
            state.macro_status = format!(
                "Macro tersimpan • pemicu {} aktif global.",
                state
                    .macro_library
                    .selected()
                    .map(|item| item.trigger.label())
                    .unwrap_or("-")
            );
        }
        Err(error) => {
            state.macro_status_kind = StatusKind::Error;
            state.macro_status = format!("Gagal menyimpan macro: {error}");
        }
    }
    unsafe { InvalidateRect(state.window, null(), FALSE) };
}

unsafe fn start_macro_recording(state: &mut AppState) {
    if state.recording || state.macro_library.selected().is_none() {
        return;
    }
    MACRO_STOP.store(true, Ordering::Release);
    state.recording = true;
    state.record_last_event = Some(Instant::now());
    state.macro_status_kind = StatusKind::Running;
    state.macro_status = "Merekam input global. Tekan Esc untuk selesai.".to_owned();
    unsafe {
        state.set_controls_visible(true);
        state.set_prompt_enabled();
        InvalidateRect(state.window, null(), FALSE);
    }
}

unsafe fn stop_macro_recording(state: &mut AppState) {
    if !state.recording {
        return;
    }
    state.recording = false;
    state.record_last_event = None;
    state.macro_dirty = true;
    let count = state
        .macro_library
        .selected()
        .map(|item| lane_events(item, state.macro_lane).len())
        .unwrap_or(0);
    state.macro_status_kind = StatusKind::Ready;
    state.macro_status = format!("Recording selesai • {count} langkah di timeline.");
    unsafe {
        state.set_controls_visible(true);
        state.set_prompt_enabled();
        InvalidateRect(state.window, null(), FALSE);
    }
}

unsafe fn record_macro_event(state: &mut AppState, event: MacroEvent) {
    let now = Instant::now();
    let elapsed = state
        .record_last_event
        .map(|last| now.saturating_duration_since(last).as_millis().min(60_000) as u32)
        .unwrap_or(0);
    let mut limit_reached = false;
    if let Some(events) = selected_lane_mut(state) {
        if events.len().saturating_add(2) > MAX_RECORDED_EVENTS {
            limit_reached = true;
        } else {
            if elapsed > 0 {
                events.push(MacroEvent::Delay(elapsed));
            }
            events.push(event);
        }
    }
    if limit_reached {
        unsafe { stop_macro_recording(state) };
        state.macro_status_kind = StatusKind::Error;
        state.macro_status = "Recording dihentikan pada batas aman 10.000 langkah.".to_owned();
    } else {
        state.record_last_event = Some(now);
    }
    state.macro_dirty = true;
    unsafe { InvalidateRect(state.window, null(), FALSE) };
}

fn sleep_interruptible(milliseconds: u32) -> bool {
    let mut remaining = milliseconds;
    while remaining > 0 {
        if MACRO_STOP.load(Ordering::Acquire) {
            return false;
        }
        let slice = remaining.min(10);
        thread::sleep(Duration::from_millis(slice as u64));
        remaining -= slice;
    }
    true
}

fn play_macro_events(events: &[MacroEvent], standard_delay: Option<u32>) -> Result<bool, String> {
    for event in events {
        if MACRO_STOP.load(Ordering::Acquire) {
            return Ok(false);
        }
        if let MacroEvent::Delay(recorded) = event {
            if !sleep_interruptible(standard_delay.unwrap_or(*recorded)) {
                return Ok(false);
            }
            continue;
        }
        if let Some(input) = macro_event_input(event) {
            #[cfg(test)]
            TEST_MACRO_INPUT_COUNT.fetch_add(1, Ordering::Relaxed);
            unsafe { submit_inputs(&[input])? };
        }
    }
    Ok(true)
}

fn post_macro_result(window: Hwnd, result: Result<(), String>) {
    MACRO_PLAYING.store(false, Ordering::Release);
    MACRO_STOP.store(false, Ordering::Release);
    unsafe {
        PostMessageW(window, WM_APP_MACRO_DONE, usize::from(result.is_ok()), 0);
    }
}

fn launch_macro_playback(window: Hwnd, item: MacroDefinition) {
    if item.on_press.is_empty() && item.while_holding.is_empty() && item.on_release.is_empty() {
        unsafe { PostMessageW(window, WM_APP_MACRO_DONE, 0, 0) };
        return;
    }

    if item.mode == MacroMode::Toggle && MACRO_PLAYING.load(Ordering::Acquire) {
        MACRO_STOP.store(true, Ordering::Release);
        return;
    }
    if MACRO_PLAYING
        .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
        .is_err()
    {
        return;
    }
    MACRO_STOP.store(false, Ordering::Release);
    thread::spawn(move || {
        let standard = item.standard_delay_ms;
        let result = (|| -> Result<(), String> {
            match item.mode {
                MacroMode::NoRepeat => {
                    play_macro_events(&item.on_press, standard)?;
                }
                MacroMode::RepeatWhileHolding => {
                    while TRIGGER_HELD.load(Ordering::Acquire)
                        && !MACRO_STOP.load(Ordering::Acquire)
                    {
                        if !play_macro_events(&item.on_press, standard)? {
                            break;
                        }
                        if item.on_press.is_empty() && !sleep_interruptible(10) {
                            break;
                        }
                    }
                }
                MacroMode::Toggle => {
                    while !MACRO_STOP.load(Ordering::Acquire) {
                        if !play_macro_events(&item.on_press, standard)? {
                            break;
                        }
                        if item.on_press.is_empty() && !sleep_interruptible(10) {
                            break;
                        }
                    }
                }
                MacroMode::Sequence => {
                    if play_macro_events(&item.on_press, standard)? {
                        while TRIGGER_HELD.load(Ordering::Acquire)
                            && !MACRO_STOP.load(Ordering::Acquire)
                        {
                            if !play_macro_events(&item.while_holding, standard)? {
                                break;
                            }
                            if item.while_holding.is_empty() && !sleep_interruptible(10) {
                                break;
                            }
                        }
                        if !MACRO_STOP.load(Ordering::Acquire) {
                            play_macro_events(&item.on_release, standard)?;
                        }
                    }
                }
            }
            Ok(())
        })();
        post_macro_result(window, result);
    });
}

fn keyboard_trigger_matches(trigger: MacroTrigger, key: u16) -> bool {
    matches!(
        (trigger, key),
        (MacroTrigger::F8, VK_F8) | (MacroTrigger::F9, VK_F9)
    )
}

fn mouse_trigger_matches(trigger: MacroTrigger, message: Uint, mouse_data: Dword) -> bool {
    match trigger {
        MacroTrigger::MouseMiddle => matches!(message, WM_MBUTTONDOWN | WM_MBUTTONUP),
        MacroTrigger::MouseX1 => {
            matches!(message, WM_XBUTTONDOWN | WM_XBUTTONUP)
                && ((mouse_data >> 16) as u16 == XBUTTON1)
        }
        MacroTrigger::MouseX2 => {
            matches!(message, WM_XBUTTONDOWN | WM_XBUTTONUP)
                && ((mouse_data >> 16) as u16 == XBUTTON2)
        }
        _ => false,
    }
}

fn macro_for_keyboard_trigger(state: &AppState, key: u16) -> Option<MacroDefinition> {
    state
        .macro_library
        .selected()
        .filter(|item| keyboard_trigger_matches(item.trigger, key))
        .cloned()
        .or_else(|| {
            state
                .macro_library
                .macros
                .iter()
                .find(|item| keyboard_trigger_matches(item.trigger, key))
                .cloned()
        })
}

fn macro_for_mouse_trigger(
    state: &AppState,
    message: Uint,
    mouse_data: Dword,
) -> Option<MacroDefinition> {
    state
        .macro_library
        .selected()
        .filter(|item| mouse_trigger_matches(item.trigger, message, mouse_data))
        .cloned()
        .or_else(|| {
            state
                .macro_library
                .macros
                .iter()
                .find(|item| mouse_trigger_matches(item.trigger, message, mouse_data))
                .cloned()
        })
}

unsafe fn handle_trigger_down(state: &mut AppState, item: MacroDefinition) {
    if state.trigger_down {
        return;
    }
    state.trigger_down = true;
    TRIGGER_HELD.store(true, Ordering::Release);
    state.macro_status_kind = StatusKind::Running;
    state.macro_status = format!("Menjalankan {}…", item.name);
    unsafe { InvalidateRect(state.window, null(), FALSE) };
    launch_macro_playback(state.window, item);
}

fn handle_trigger_up(state: &mut AppState) {
    state.trigger_down = false;
    TRIGGER_HELD.store(false, Ordering::Release);
}

unsafe extern "system" fn keyboard_hook_proc(code: i32, wparam: Wparam, lparam: Lparam) -> Lresult {
    if code < HC_ACTION || lparam == 0 {
        return unsafe { CallNextHookEx(0, code, wparam, lparam) };
    }
    let data = unsafe { &*(lparam as *const KbdLlHookStruct) };
    if data.flags & LLKHF_INJECTED != 0 {
        return unsafe { CallNextHookEx(0, code, wparam, lparam) };
    }
    let state_pointer = APP_STATE_POINTER.load(Ordering::Acquire);
    if state_pointer.is_null() {
        return unsafe { CallNextHookEx(0, code, wparam, lparam) };
    }
    let state = unsafe { &mut *state_pointer };
    let message = wparam as Uint;
    let down = matches!(message, WM_KEYDOWN | WM_SYSKEYDOWN);
    let up = matches!(message, WM_KEYUP | WM_SYSKEYUP);
    let key = data.vk_code as u16;

    if state.recording && (down || up) {
        if key == VK_ESCAPE {
            if down {
                state.suppress_escape_until_up = true;
                unsafe { stop_macro_recording(state) };
            } else {
                state.suppress_escape_until_up = false;
            }
            return 1;
        }
        let event = if down {
            MacroEvent::KeyDown(key)
        } else {
            MacroEvent::KeyUp(key)
        };
        unsafe { record_macro_event(state, event) };
        return 1;
    }
    if key == VK_ESCAPE && state.suppress_escape_until_up {
        if up {
            state.suppress_escape_until_up = false;
        }
        return 1;
    }
    let item = macro_for_keyboard_trigger(state, key);
    if let Some(item) = item {
        if down {
            unsafe { handle_trigger_down(state, item) };
        } else if up {
            handle_trigger_up(state);
        }
        return 1;
    }
    unsafe { CallNextHookEx(0, code, wparam, lparam) }
}

unsafe extern "system" fn mouse_hook_proc(code: i32, wparam: Wparam, lparam: Lparam) -> Lresult {
    if code < HC_ACTION || lparam == 0 {
        return unsafe { CallNextHookEx(0, code, wparam, lparam) };
    }
    let data = unsafe { &*(lparam as *const MsLlHookStruct) };
    if data.flags & LLMHF_INJECTED != 0 {
        return unsafe { CallNextHookEx(0, code, wparam, lparam) };
    }
    let state_pointer = APP_STATE_POINTER.load(Ordering::Acquire);
    if state_pointer.is_null() {
        return unsafe { CallNextHookEx(0, code, wparam, lparam) };
    }
    let state = unsafe { &mut *state_pointer };
    let message = wparam as Uint;
    let button_event = match message {
        0x0201 => Some(MacroEvent::MouseDown(MouseButton::Left)),
        0x0202 => Some(MacroEvent::MouseUp(MouseButton::Left)),
        0x0204 => Some(MacroEvent::MouseDown(MouseButton::Right)),
        0x0205 => Some(MacroEvent::MouseUp(MouseButton::Right)),
        WM_MBUTTONDOWN => Some(MacroEvent::MouseDown(MouseButton::Middle)),
        WM_MBUTTONUP => Some(MacroEvent::MouseUp(MouseButton::Middle)),
        WM_XBUTTONDOWN if (data.mouse_data >> 16) as u16 == XBUTTON1 => {
            Some(MacroEvent::MouseDown(MouseButton::X1))
        }
        WM_XBUTTONUP if (data.mouse_data >> 16) as u16 == XBUTTON1 => {
            Some(MacroEvent::MouseUp(MouseButton::X1))
        }
        WM_XBUTTONDOWN => Some(MacroEvent::MouseDown(MouseButton::X2)),
        WM_XBUTTONUP => Some(MacroEvent::MouseUp(MouseButton::X2)),
        WM_MOUSEWHEEL => Some(MacroEvent::Wheel((data.mouse_data >> 16) as i16)),
        _ => None,
    };
    if state.recording
        && let Some(event) = button_event
    {
        unsafe { record_macro_event(state, event) };
        return 1;
    }
    let item = macro_for_mouse_trigger(state, message, data.mouse_data);
    if let Some(item) = item {
        if matches!(message, WM_MBUTTONDOWN | WM_XBUTTONDOWN) {
            unsafe { handle_trigger_down(state, item) };
        } else {
            handle_trigger_up(state);
        }
        return 1;
    }
    unsafe { CallNextHookEx(0, code, wparam, lparam) }
}

unsafe fn initialize_macro_hooks(state: &mut AppState, instance: Hinstance) {
    state.keyboard_hook =
        unsafe { SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_hook_proc), instance, 0) };
    state.mouse_hook =
        unsafe { SetWindowsHookExW(WH_MOUSE_LL, Some(mouse_hook_proc), instance, 0) };
    if state.keyboard_hook == 0 || state.mouse_hook == 0 {
        state.macro_status_kind = StatusKind::Error;
        state.macro_status =
            "Hook global gagal aktif. Jalankan aplikasi pada desktop Windows biasa.".to_owned();
    }
}

unsafe fn handle_click(state: &mut AppState, target: HitTarget) {
    match target {
        HitTarget::None => {}
        HitTarget::TimerTab => unsafe { switch_tab(state, AppTab::Timer) },
        HitTarget::MacroTab => unsafe { switch_tab(state, AppTab::Macro) },
        HitTarget::AddThirtyMinutes => unsafe { add_preset(state, 30 * 60) },
        HitTarget::AddOneHour => unsafe { add_preset(state, 60 * 60) },
        HitTarget::AddThreeHours => unsafe { add_preset(state, 3 * 60 * 60) },
        HitTarget::PickTarget => unsafe { begin_target_capture(state) },
        HitTarget::EnterOnly => {
            state.action_mode = ActionMode::EnterOnly;
            state.status_kind = StatusKind::Ready;
            state.status = "Saat nol, VibeTimer hanya menekan Enter.".to_owned();
            unsafe {
                state.set_prompt_enabled();
                InvalidateRect(state.window, null(), FALSE);
            }
        }
        HitTarget::TextAndEnter => {
            state.action_mode = ActionMode::TextAndEnter;
            state.status_kind = StatusKind::Ready;
            state.status = "Saat nol, teks diketik lalu Enter ditekan.".to_owned();
            unsafe {
                state.set_prompt_enabled();
                InvalidateRect(state.window, null(), FALSE);
            }
        }
        HitTarget::MainAction => {
            if state.running {
                unsafe { cancel_timer(state) };
            } else {
                unsafe { begin_timer(state) };
            }
        }
        HitTarget::MacroNew => {
            if state.recording {
                return;
            }
            state.macro_library.add_macro();
            state.macro_lane = MacroLane::OnPress;
            state.macro_dirty = true;
            state.macro_status_kind = StatusKind::Ready;
            state.macro_status = "Macro baru dibuat. Beri nama lalu rekam timeline.".to_owned();
            unsafe {
                sync_macro_name_edit(state);
                InvalidateRect(state.window, null(), FALSE);
            }
        }
        HitTarget::MacroItem(index) => {
            if state.recording {
                return;
            }
            if let Some(item) = state.macro_library.macros.get(index) {
                state.macro_library.selected_id = item.id;
                state.macro_lane = MacroLane::OnPress;
                state.macro_status_kind = StatusKind::Ready;
                state.macro_status = format!("Mengedit {}.", item.name);
                unsafe {
                    sync_macro_name_edit(state);
                    InvalidateRect(state.window, null(), FALSE);
                }
            }
        }
        HitTarget::MacroMode(mode) => {
            if state.recording {
                return;
            }
            if let Some(item) = state.macro_library.selected_mut() {
                item.mode = mode;
                state.macro_dirty = true;
                state.macro_status_kind = StatusKind::Ready;
                state.macro_status = format!("Mode {} dipilih.", mode.label());
                unsafe { InvalidateRect(state.window, null(), FALSE) };
            }
        }
        HitTarget::MacroTrigger(trigger) => {
            if state.recording {
                return;
            }
            if let Some(item) = state.macro_library.selected_mut() {
                item.trigger = trigger;
                state.macro_dirty = true;
                state.macro_status_kind = StatusKind::Ready;
                state.macro_status = format!(
                    "Pemicu {} dipasang. Simpan untuk permanen.",
                    trigger.label()
                );
                unsafe { InvalidateRect(state.window, null(), FALSE) };
            }
        }
        HitTarget::MacroLane(lane) => {
            if !state.recording {
                state.macro_lane = lane;
                unsafe { InvalidateRect(state.window, null(), FALSE) };
            }
        }
        HitTarget::MacroRecord => {
            if state.recording {
                unsafe { stop_macro_recording(state) };
            } else {
                unsafe { start_macro_recording(state) };
            }
        }
        HitTarget::MacroClear => {
            if !state.recording {
                if let Some(events) = selected_lane_mut(state) {
                    events.clear();
                }
                state.macro_dirty = true;
                state.macro_status_kind = StatusKind::Warning;
                state.macro_status =
                    "Lane dibersihkan. Tekan Simpan untuk mempertahankan perubahan.".to_owned();
                unsafe { InvalidateRect(state.window, null(), FALSE) };
            }
        }
        HitTarget::MacroSave => {
            if !state.recording {
                unsafe { save_current_macro(state) };
            }
        }
    }
}

unsafe extern "system" fn window_proc(
    window: Hwnd,
    message: Uint,
    wparam: Wparam,
    lparam: Lparam,
) -> Lresult {
    if message == WM_NCCREATE {
        let create = lparam as *const CreateStructW;
        if create.is_null() {
            return FALSE as Lresult;
        }
        let state = unsafe { (*create).create_params as *mut AppState };
        unsafe {
            SetWindowLongPtrW(window, GWLP_USERDATA, state as isize);
            (*state).window = window;
            APP_STATE_POINTER.store(state, Ordering::Release);
        }
        return TRUE as Lresult;
    }

    let state_pointer = unsafe { GetWindowLongPtrW(window, GWLP_USERDATA) as *mut AppState };
    if state_pointer.is_null() {
        return unsafe { DefWindowProcW(window, message, wparam, lparam) };
    }
    let state = unsafe { &mut *state_pointer };

    match message {
        WM_CREATE => {
            let instance = unsafe { GetModuleHandleW(null()) };
            unsafe {
                initialize_controls(state, instance);
                initialize_macro_hooks(state, instance);
            }
            0
        }
        WM_PAINT => {
            let mut paint: PaintStruct = unsafe { zeroed() };
            let destination = unsafe { BeginPaint(window, &mut paint) };
            let mut client = Rect::default();
            unsafe { GetClientRect(window, &mut client) };
            let width = client.right - client.left;
            let height = client.bottom - client.top;
            let memory_dc = unsafe { CreateCompatibleDC(destination) };
            let bitmap = unsafe { CreateCompatibleBitmap(destination, width, height) };
            let old_bitmap = unsafe { SelectObject(memory_dc, bitmap) };
            unsafe {
                draw_redesigned_interface(memory_dc, state);
                BitBlt(destination, 0, 0, width, height, memory_dc, 0, 0, SRCCOPY);
                SelectObject(memory_dc, old_bitmap);
                DeleteObject(bitmap);
                DeleteDC(memory_dc);
                EndPaint(window, &paint);
            }
            0
        }
        WM_ERASEBKGND => 1,
        WM_CTLCOLOREDIT | WM_CTLCOLORSTATIC => {
            let dc = wparam as Hdc;
            let panel_editor = lparam == state.macro_name_edit;
            unsafe {
                SetTextColor(
                    dc,
                    if panel_editor {
                        COLOR_TEXT
                    } else if state.action_mode == ActionMode::EnterOnly
                        && lparam == state.prompt_edit
                    {
                        COLOR_DIM
                    } else {
                        COLOR_TEXT
                    },
                );
                SetBkColor(
                    dc,
                    if panel_editor {
                        COLOR_PANEL_2
                    } else {
                        COLOR_BG
                    },
                );
            }
            if panel_editor {
                state.panel_edit_brush
            } else {
                state.edit_brush
            }
        }
        WM_MOUSEMOVE => {
            let x = low_word(lparam);
            let y = high_word(lparam);
            let new_hot = hit_test(x, y, state);
            if new_hot != state.hot {
                state.hot = new_hot;
                unsafe { InvalidateRect(window, null(), FALSE) };
            }
            if !state.tracking_mouse {
                let mut tracking = TrackMouseEvent {
                    size: size_of::<TrackMouseEvent>() as Dword,
                    flags: TME_LEAVE,
                    tracked: window,
                    hover_time: 0,
                };
                unsafe { TrackMouseEvent(&mut tracking) };
                state.tracking_mouse = true;
            }
            0
        }
        WM_MOUSELEAVE => {
            state.tracking_mouse = false;
            if state.hot != HitTarget::None {
                state.hot = HitTarget::None;
                unsafe { InvalidateRect(window, null(), FALSE) };
            }
            0
        }
        WM_SETCURSOR => {
            if state.hot != HitTarget::None {
                unsafe { SetCursor(LoadCursorW(0, IDC_HAND as *const u16)) };
                1
            } else {
                unsafe { DefWindowProcW(window, message, wparam, lparam) }
            }
        }
        WM_LBUTTONUP => {
            let x = low_word(lparam);
            let y = high_word(lparam);
            let target = hit_test(x, y, state);
            unsafe { handle_click(state, target) };
            0
        }
        WM_COMMAND => {
            let control_id = (wparam & 0xFFFF) as u16;
            let notification = ((wparam >> 16) & 0xFFFF) as u16;
            if control_id == 105 && notification == 0x0300 && state.tab == AppTab::Macro {
                state.macro_dirty = true;
                unsafe { InvalidateRect(window, null(), FALSE) };
            }
            0
        }
        WM_APP_MACRO_DONE => {
            if wparam != 0 {
                state.macro_status_kind = StatusKind::Sent;
                state.macro_status = "Macro selesai dijalankan.".to_owned();
            } else {
                state.macro_status_kind = StatusKind::Error;
                state.macro_status =
                    "Macro kosong atau Windows menolak salah satu input.".to_owned();
            }
            unsafe { InvalidateRect(window, null(), FALSE) };
            0
        }
        WM_TIMER => {
            if wparam == TIMER_COUNTDOWN && state.running {
                unsafe { update_countdown(state) };
            } else if wparam == TIMER_CAPTURE
                && state
                    .capture_deadline
                    .is_some_and(|deadline| Instant::now() >= deadline)
            {
                unsafe { finish_target_capture(state) };
            }
            0
        }
        WM_CLOSE => {
            if state.recording {
                unsafe { stop_macro_recording(state) };
            }
            if state.running {
                let response = unsafe {
                    MessageBoxW(
                        window,
                        wide("Timer masih aktif. Tutup VibeTimer dan batalkan pengiriman?")
                            .as_ptr(),
                        wide("Timer sedang berjalan").as_ptr(),
                        MB_YESNO | MB_ICONWARNING,
                    )
                };
                if response != IDYES {
                    return 0;
                }
            }
            if state.macro_dirty {
                let response = unsafe {
                    MessageBoxW(
                        window,
                        wide("Ada perubahan macro yang belum disimpan. Tutup tanpa menyimpan?")
                            .as_ptr(),
                        wide("Macro belum disimpan").as_ptr(),
                        MB_YESNO | MB_ICONWARNING,
                    )
                };
                if response != IDYES {
                    return 0;
                }
            }
            unsafe { DestroyWindow(window) };
            0
        }
        WM_DESTROY => {
            unsafe {
                KillTimer(window, TIMER_COUNTDOWN);
                KillTimer(window, TIMER_CAPTURE);
                MACRO_STOP.store(true, Ordering::Release);
                TRIGGER_HELD.store(false, Ordering::Release);
                PostQuitMessage(0);
            }
            0
        }
        WM_NCDESTROY => unsafe {
            APP_STATE_POINTER.store(null_mut(), Ordering::Release);
            state.cleanup();
            SetWindowLongPtrW(window, GWLP_USERDATA, 0);
            drop(Box::from_raw(state_pointer));
            DefWindowProcW(window, message, wparam, lparam)
        },
        _ => unsafe { DefWindowProcW(window, message, wparam, lparam) },
    }
}

unsafe fn create_main_window(instance: Hinstance) -> Result<Hwnd, String> {
    let class_name = wide("VibeTimerWindowClass");
    let window_class = WndClassW {
        style: 0x0002 | 0x0001,
        wnd_proc: Some(window_proc),
        class_extra: 0,
        window_extra: 0,
        instance,
        icon: unsafe { create_app_icon(instance, 32) },
        cursor: unsafe { LoadCursorW(0, IDC_ARROW as *const u16) },
        background: 0,
        menu_name: null(),
        class_name: class_name.as_ptr(),
    };
    if unsafe { RegisterClassW(&window_class) } == 0 {
        return Err("Tidak dapat mendaftarkan kelas jendela.".to_owned());
    }

    let style = WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU | WS_MINIMIZEBOX;
    let mut outer = Rect::new(0, 0, CLIENT_WIDTH, CLIENT_HEIGHT);
    unsafe { AdjustWindowRectEx(&mut outer, style, FALSE, 0) };
    let width = outer.right - outer.left;
    let height = outer.bottom - outer.top;

    let state = Box::into_raw(Box::new(AppState::new()));
    let window = unsafe {
        CreateWindowExW(
            0,
            class_name.as_ptr(),
            wide("VibeTimer").as_ptr(),
            style,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            width,
            height,
            0,
            0,
            instance,
            state.cast(),
        )
    };
    if window == 0 {
        unsafe { drop(Box::from_raw(state)) };
        return Err("Tidak dapat membuat jendela utama.".to_owned());
    }

    unsafe {
        SendMessageW(window, WM_SETICON, ICON_BIG, create_app_icon(instance, 32));
        SendMessageW(
            window,
            WM_SETICON,
            ICON_SMALL,
            create_app_icon(instance, 16),
        );
    }

    let dark_mode: Bool = TRUE;
    unsafe {
        DwmSetWindowAttribute(
            window,
            DWMWA_USE_IMMERSIVE_DARK_MODE,
            (&dark_mode as *const Bool).cast(),
            size_of::<Bool>() as Dword,
        );
        let corner = DWMWCP_ROUND;
        DwmSetWindowAttribute(
            window,
            DWMWA_WINDOW_CORNER_PREFERENCE,
            (&corner as *const Dword).cast(),
            size_of::<Dword>() as Dword,
        );
    }
    Ok(window)
}

fn run() -> Result<(), String> {
    unsafe {
        // DPI_UNAWARE_GDISCALED menjaga ukuran UI konsisten dan teks GDI tetap tajam
        // di skala layar Windows yang umum. Gagal di Windows lama tidak fatal.
        SetProcessDpiAwarenessContext(-5isize);

        let instance = GetModuleHandleW(null());
        if instance == 0 {
            return Err("Tidak dapat memperoleh handle aplikasi.".to_owned());
        }

        let window = create_main_window(instance)?;
        ShowWindow(window, SW_SHOWNORMAL);
        UpdateWindow(window);

        let mut message: Msg = zeroed();
        loop {
            let result = GetMessageW(&mut message, 0, 0, 0);
            if result == -1 {
                return Err("Message loop Windows gagal.".to_owned());
            }
            if result == 0 {
                break;
            }
            TranslateMessage(&message);
            DispatchMessageW(&message);
        }
        Ok(())
    }
}

fn main() {
    if let Err(message) = run() {
        unsafe {
            MessageBoxW(
                0,
                wide(&message).as_ptr(),
                wide("VibeTimer gagal dimulai").as_ptr(),
                MB_OK | MB_ICONERROR,
            );
        }
    }
}

#[cfg(test)]
mod windows_e2e_tests {
    use super::*;
    use std::fs;
    use std::path::Path;

    unsafe fn pump_messages_for(duration: Duration) {
        let deadline = Instant::now() + duration;
        while Instant::now() < deadline {
            let mut message: Msg = unsafe { zeroed() };
            while unsafe { PeekMessageW(&mut message, 0, 0, 0, PM_REMOVE) } != FALSE {
                unsafe {
                    TranslateMessage(&message);
                    DispatchMessageW(&message);
                }
            }
            thread::sleep(Duration::from_millis(10));
        }
    }

    unsafe fn wait_macro_idle() {
        let deadline = Instant::now() + Duration::from_secs(2);
        while MACRO_PLAYING.load(Ordering::Acquire) && Instant::now() < deadline {
            unsafe { pump_messages_for(Duration::from_millis(20)) };
        }
        assert!(
            !MACRO_PLAYING.load(Ordering::Acquire),
            "playback macro harus kembali idle"
        );
    }

    unsafe fn save_window_bmp(window: Hwnd, path: &Path) -> Result<(), String> {
        let mut bounds = Rect::default();
        if unsafe { GetWindowRect(window, &mut bounds) } == FALSE {
            return Err("GetWindowRect gagal".to_owned());
        }
        let width = bounds.right - bounds.left;
        let height = bounds.bottom - bounds.top;
        if width <= 0 || height <= 0 {
            return Err("Ukuran window tidak valid".to_owned());
        }

        let screen_dc = unsafe { GetDC(0) };
        let memory_dc = unsafe { CreateCompatibleDC(screen_dc) };
        let bitmap = unsafe { CreateCompatibleBitmap(screen_dc, width, height) };
        let old_bitmap = unsafe { SelectObject(memory_dc, bitmap) };
        let rendered = unsafe { PrintWindow(window, memory_dc, PW_RENDERFULLCONTENT) };
        if rendered == FALSE {
            unsafe {
                SelectObject(memory_dc, old_bitmap);
                DeleteObject(bitmap);
                DeleteDC(memory_dc);
                ReleaseDC(0, screen_dc);
            }
            return Err("PrintWindow gagal".to_owned());
        }

        let byte_count = (width as usize) * (height as usize) * 4;
        let mut pixels = vec![0u8; byte_count];
        let mut info = BitmapInfo {
            header: BitmapInfoHeader {
                size: size_of::<BitmapInfoHeader>() as Dword,
                width,
                height: -height,
                planes: 1,
                bit_count: 32,
                compression: 0,
                size_image: byte_count as Dword,
                ..BitmapInfoHeader::default()
            },
            colors: [0],
        };
        let copied = unsafe {
            GetDIBits(
                screen_dc,
                bitmap,
                0,
                height as Uint,
                pixels.as_mut_ptr().cast(),
                &mut info,
                DIB_RGB_COLORS,
            )
        };

        unsafe {
            SelectObject(memory_dc, old_bitmap);
            DeleteObject(bitmap);
            DeleteDC(memory_dc);
            ReleaseDC(0, screen_dc);
        }
        if copied == 0 {
            return Err("GetDIBits gagal".to_owned());
        }

        let pixel_offset = 14u32 + size_of::<BitmapInfoHeader>() as u32;
        let file_size = pixel_offset + byte_count as u32;
        let mut output = Vec::with_capacity(file_size as usize);
        output.extend_from_slice(b"BM");
        output.extend_from_slice(&file_size.to_le_bytes());
        output.extend_from_slice(&[0u8; 4]);
        output.extend_from_slice(&pixel_offset.to_le_bytes());
        output.extend_from_slice(&(size_of::<BitmapInfoHeader>() as u32).to_le_bytes());
        output.extend_from_slice(&width.to_le_bytes());
        output.extend_from_slice(&(-height).to_le_bytes());
        output.extend_from_slice(&1u16.to_le_bytes());
        output.extend_from_slice(&32u16.to_le_bytes());
        output.extend_from_slice(&0u32.to_le_bytes());
        output.extend_from_slice(&(byte_count as u32).to_le_bytes());
        output.extend_from_slice(&0i32.to_le_bytes());
        output.extend_from_slice(&0i32.to_le_bytes());
        output.extend_from_slice(&0u32.to_le_bytes());
        output.extend_from_slice(&0u32.to_le_bytes());
        output.extend_from_slice(&pixels);
        fs::write(path, output).map_err(|error| error.to_string())
    }

    #[test]
    fn renders_and_sends_text_plus_enter_end_to_end() {
        unsafe {
            SetProcessDpiAwarenessContext(-5isize);
            let instance = GetModuleHandleW(null());
            assert_ne!(instance, 0);
            let main_window = create_main_window(instance).expect("main window dibuat");
            ShowWindow(main_window, SW_SHOWNORMAL);
            UpdateWindow(main_window);
            pump_messages_for(Duration::from_millis(100));

            fs::create_dir_all("qa").expect("folder QA dibuat");
            save_window_bmp(main_window, Path::new("qa/vibetimer-idle.bmp"))
                .expect("snapshot idle dibuat");

            let target_title = wide("VibeTimer E2E Target");
            let static_class = wide("STATIC");
            let target_window = CreateWindowExW(
                0,
                static_class.as_ptr(),
                target_title.as_ptr(),
                WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU | WS_VISIBLE,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                460,
                240,
                0,
                0,
                instance,
                null_mut(),
            );
            assert_ne!(target_window, 0);

            let edit_class = wide("EDIT");
            let target_edit = CreateWindowExW(
                0,
                edit_class.as_ptr(),
                wide("").as_ptr(),
                WS_CHILD | WS_VISIBLE | ES_MULTILINE | ES_WANTRETURN,
                20,
                20,
                400,
                140,
                target_window,
                501,
                instance,
                null_mut(),
            );
            assert_ne!(target_edit, 0);
            ShowWindow(target_window, SW_SHOWNORMAL);
            UpdateWindow(target_window);
            SetForegroundWindow(target_window);
            SetFocus(target_edit);
            pump_messages_for(Duration::from_millis(100));

            let state_pointer = GetWindowLongPtrW(main_window, GWLP_USERDATA) as *mut AppState;
            assert!(!state_pointer.is_null());
            let state = &mut *state_pointer;
            assert_ne!(state.keyboard_hook, 0, "keyboard hook harus aktif");
            assert_ne!(state.mouse_hook, 0, "mouse hook harus aktif");
            state.macro_library = MacroLibrary::default();
            sync_macro_name_edit(state);

            assert_eq!(hit_test(340, 39, state), HitTarget::TimerTab);
            assert_eq!(hit_test(430, 39, state), HitTarget::MacroTab);
            assert_eq!(hit_test(100, 240, state), HitTarget::AddThirtyMinutes);
            assert_eq!(hit_test(210, 240, state), HitTarget::AddOneHour);
            assert_eq!(hit_test(330, 240, state), HitTarget::AddThreeHours);
            assert_eq!(hit_test(410, 330, state), HitTarget::PickTarget);
            assert_eq!(hit_test(145, 442, state), HitTarget::EnterOnly);
            assert_eq!(hit_test(350, 442, state), HitTarget::TextAndEnter);
            assert_eq!(hit_test(260, 575, state), HitTarget::MainAction);

            set_duration_fields(state, DurationFields::new(0, 0, 0));
            handle_click(state, HitTarget::AddThirtyMinutes);
            handle_click(state, HitTarget::AddOneHour);
            handle_click(state, HitTarget::AddThreeHours);
            assert_eq!(
                read_duration_fields(state),
                DurationFields::new(4, 30, 0),
                "ketiga tombol preset harus menjumlahkan waktu dengan benar"
            );

            state.target = Some(TargetWindow {
                window: target_window,
                process_id: GetCurrentProcessId(),
                title: "VibeTimer E2E Target".to_owned(),
            });
            assert_eq!(
                state.target.as_ref().map(|target| target.title.as_str()),
                Some("VibeTimer E2E Target")
            );
            assert!(
                validate_target(state.target.as_ref().expect("target tersedia")).is_ok(),
                "target yang hidup dan PID-nya cocok harus valid"
            );
            let invalid_target = TargetWindow {
                window: target_window,
                process_id: 0,
                title: "PID salah".to_owned(),
            };
            assert!(
                validate_target(&invalid_target).is_err(),
                "PID target yang berubah harus ditolak"
            );

            state.running = true;
            state.status_kind = StatusKind::Running;
            state.status = "Timer aktif. Target akan difokuskan otomatis saat nol.".to_owned();
            state.original_seconds = 12_420;
            state.remaining_seconds = 12_418;
            state.armed_prompt = "lanjutkan".to_owned();
            state.set_controls_visible(false);
            InvalidateRect(main_window, null(), FALSE);
            UpdateWindow(main_window);
            pump_messages_for(Duration::from_millis(100));
            save_window_bmp(main_window, Path::new("qa/vibetimer-running.bmp"))
                .expect("snapshot aktif dibuat");
            state.running = false;
            state.status_kind = StatusKind::Ready;
            state.status = "Target siap untuk tes pengiriman.".to_owned();
            state.set_controls_visible(true);
            state.set_prompt_enabled();
            InvalidateRect(main_window, null(), FALSE);

            set_duration_fields(state, DurationFields::new(0, 0, 1));
            SetWindowTextW(state.prompt_edit, wide("lanjutkan").as_ptr());
            state.action_mode = ActionMode::TextAndEnter;
            TEST_INPUT_TARGET.store(target_edit, Ordering::Relaxed);
            begin_timer(state);
            assert!(state.running);
            pump_messages_for(Duration::from_millis(120));

            pump_messages_for(Duration::from_millis(1_300));
            assert!(!state.running, "timer harus selesai");
            assert_eq!(
                state.status_kind,
                StatusKind::Sent,
                "pengiriman harus berhasil: {}",
                state.status
            );
            pump_messages_for(Duration::from_millis(250));

            let received = get_window_text(target_edit);
            assert!(
                received.starts_with("lanjutkan"),
                "teks target tidak sesuai: {received:?}"
            );
            assert!(
                received.len() > "lanjutkan".len(),
                "Enter harus menambahkan baris baru: {received:?}"
            );
            save_window_bmp(target_window, Path::new("qa/e2e-target.bmp"))
                .expect("snapshot target dibuat");

            SetWindowTextW(target_edit, wide("").as_ptr());
            handle_click(state, HitTarget::EnterOnly);
            assert_eq!(state.action_mode, ActionMode::EnterOnly);
            set_duration_fields(state, DurationFields::new(0, 0, 1));
            begin_timer(state);
            pump_messages_for(Duration::from_millis(1_300));
            assert_eq!(state.status_kind, StatusKind::Sent);
            let enter_only = get_window_text(target_edit);
            assert!(
                !enter_only.is_empty() && !enter_only.contains("lanjutkan"),
                "mode Hanya Enter tidak boleh mengetik prompt: {enter_only:?}"
            );

            SetWindowTextW(target_edit, wide("").as_ptr());
            set_duration_fields(state, DurationFields::new(0, 0, 5));
            begin_timer(state);
            assert!(state.running);
            pump_messages_for(Duration::from_millis(120));
            handle_click(state, HitTarget::MainAction);
            pump_messages_for(Duration::from_millis(180));
            assert!(!state.running, "tombol utama harus membatalkan timer aktif");
            assert_eq!(state.status_kind, StatusKind::Warning);
            assert!(
                get_window_text(target_edit).is_empty(),
                "timer yang dibatalkan tidak boleh mengirim input"
            );

            switch_tab(state, AppTab::Macro);
            assert_eq!(hit_test(100, 137, state), HitTarget::MacroNew);
            assert_eq!(
                hit_test(350, 205, state),
                HitTarget::MacroMode(MacroMode::NoRepeat)
            );
            assert_eq!(
                hit_test(550, 290, state),
                HitTarget::MacroTrigger(MacroTrigger::MouseMiddle)
            );
            assert_eq!(
                hit_test(650, 348, state),
                HitTarget::MacroLane(MacroLane::OnRelease)
            );
            assert_eq!(hit_test(375, 570, state), HitTarget::MacroRecord);
            assert_eq!(hit_test(550, 570, state), HitTarget::MacroClear);
            assert_eq!(hit_test(835, 570, state), HitTarget::MacroSave);

            handle_click(state, HitTarget::MacroNew);
            assert_eq!(state.macro_library.macros.len(), 2);
            assert_eq!(state.macro_library.selected_id, 2);
            handle_click(state, HitTarget::MacroItem(0));
            assert_eq!(state.macro_library.selected_id, 1);
            handle_click(state, HitTarget::MacroMode(MacroMode::Toggle));
            handle_click(state, HitTarget::MacroTrigger(MacroTrigger::MouseMiddle));
            handle_click(state, HitTarget::MacroLane(MacroLane::OnRelease));
            assert_eq!(
                state.macro_library.selected().expect("macro dipilih").mode,
                MacroMode::Toggle
            );
            assert_eq!(
                state
                    .macro_library
                    .selected()
                    .expect("macro dipilih")
                    .trigger,
                MacroTrigger::MouseMiddle
            );
            assert_eq!(state.macro_lane, MacroLane::OnRelease);
            state
                .macro_library
                .selected_mut()
                .expect("macro dipilih")
                .on_release = vec![MacroEvent::KeyDown(0x42)];
            handle_click(state, HitTarget::MacroClear);
            assert!(
                state
                    .macro_library
                    .selected()
                    .expect("macro dipilih")
                    .on_release
                    .is_empty(),
                "Bersihkan bagian harus mengosongkan lane aktif"
            );
            handle_click(state, HitTarget::MacroMode(MacroMode::NoRepeat));
            handle_click(state, HitTarget::MacroTrigger(MacroTrigger::F8));
            handle_click(state, HitTarget::MacroLane(MacroLane::OnPress));
            state
                .macro_library
                .selected_mut()
                .expect("macro default tersedia")
                .on_press
                .clear();
            InvalidateRect(main_window, null(), FALSE);
            UpdateWindow(main_window);
            pump_messages_for(Duration::from_millis(120));
            save_window_bmp(main_window, Path::new("qa/vibetimer-macro-empty.bmp"))
                .expect("snapshot macro kosong dibuat");
            start_macro_recording(state);
            assert!(state.recording);
            let recorded_key = KbdLlHookStruct {
                vk_code: 0x41,
                scan_code: 0,
                flags: 0,
                time: 0,
                extra_info: 0,
            };
            keyboard_hook_proc(
                HC_ACTION,
                WM_KEYDOWN as Wparam,
                &recorded_key as *const _ as Lparam,
            );
            keyboard_hook_proc(
                HC_ACTION,
                WM_KEYUP as Wparam,
                &recorded_key as *const _ as Lparam,
            );
            let recorded_mouse = MsLlHookStruct {
                point: Point::default(),
                mouse_data: 0,
                flags: 0,
                time: 0,
                extra_info: 0,
            };
            mouse_hook_proc(
                HC_ACTION,
                0x0201 as Wparam,
                &recorded_mouse as *const _ as Lparam,
            );
            mouse_hook_proc(
                HC_ACTION,
                0x0202 as Wparam,
                &recorded_mouse as *const _ as Lparam,
            );
            let recorded_wheel = MsLlHookStruct {
                mouse_data: 120u32 << 16,
                ..recorded_mouse
            };
            mouse_hook_proc(
                HC_ACTION,
                WM_MOUSEWHEEL as Wparam,
                &recorded_wheel as *const _ as Lparam,
            );
            let escape = KbdLlHookStruct {
                vk_code: VK_ESCAPE as Dword,
                scan_code: 0,
                flags: 0,
                time: 0,
                extra_info: 0,
            };
            keyboard_hook_proc(
                HC_ACTION,
                WM_KEYDOWN as Wparam,
                &escape as *const _ as Lparam,
            );
            keyboard_hook_proc(HC_ACTION, WM_KEYUP as Wparam, &escape as *const _ as Lparam);
            assert!(!state.recording, "Esc harus menghentikan recording");
            let recorded = &state
                .macro_library
                .selected()
                .expect("macro recorder tersedia")
                .on_press;
            assert!(
                recorded.contains(&MacroEvent::KeyDown(0x41))
                    && recorded.contains(&MacroEvent::KeyUp(0x41)),
                "recorder harus menyimpan key down dan key up: {recorded:?}"
            );
            assert!(
                recorded.contains(&MacroEvent::MouseDown(MouseButton::Left))
                    && recorded.contains(&MacroEvent::MouseUp(MouseButton::Left))
                    && recorded.contains(&MacroEvent::Wheel(120)),
                "recorder harus menyimpan klik mouse dan wheel: {recorded:?}"
            );

            let item = state
                .macro_library
                .selected_mut()
                .expect("macro default tersedia");
            item.name = "Lanjut AI".to_owned();
            item.mode = MacroMode::NoRepeat;
            item.trigger = MacroTrigger::F8;
            item.on_press = vec![
                MacroEvent::Delay(25),
                MacroEvent::KeyDown(VK_RETURN),
                MacroEvent::KeyUp(VK_RETURN),
            ];
            sync_macro_name_edit(state);
            InvalidateRect(main_window, null(), FALSE);
            UpdateWindow(main_window);
            pump_messages_for(Duration::from_millis(150));
            save_window_bmp(main_window, Path::new("qa/vibetimer-macro.bmp"))
                .expect("snapshot macro dibuat");

            let e2e_macro_path = PathBuf::from("qa/e2e-macros.vtm");
            let _ = fs::remove_file(&e2e_macro_path);
            state.macro_path = e2e_macro_path.clone();
            SetWindowTextW(state.macro_name_edit, wide("Lanjut AI").as_ptr());
            state.macro_dirty = true;
            handle_click(state, HitTarget::MacroSave);
            assert_eq!(state.macro_status_kind, StatusKind::Sent);
            assert!(!state.macro_dirty);
            let persisted = load_library(&e2e_macro_path).expect("macro tersimpan dapat dibaca");
            assert_eq!(
                persisted.selected().expect("macro tersimpan dipilih").name,
                "Lanjut AI"
            );
            assert_eq!(persisted.macros.len(), 2);

            assert!(keyboard_trigger_matches(MacroTrigger::F8, VK_F8));
            assert!(keyboard_trigger_matches(MacroTrigger::F9, VK_F9));
            assert!(mouse_trigger_matches(
                MacroTrigger::MouseMiddle,
                WM_MBUTTONDOWN,
                0
            ));
            assert!(mouse_trigger_matches(
                MacroTrigger::MouseX1,
                WM_XBUTTONDOWN,
                (XBUTTON1 as Dword) << 16
            ));
            assert!(mouse_trigger_matches(
                MacroTrigger::MouseX2,
                WM_XBUTTONDOWN,
                (XBUTTON2 as Dword) << 16
            ));

            SetForegroundWindow(target_window);
            SetFocus(target_edit);
            let before_macro = get_window_text(target_edit).len();
            let key = KbdLlHookStruct {
                vk_code: VK_F8 as Dword,
                scan_code: 0,
                flags: 0,
                time: 0,
                extra_info: 0,
            };
            assert_eq!(
                keyboard_hook_proc(HC_ACTION, WM_KEYDOWN as Wparam, &key as *const _ as Lparam),
                1,
                "hook harus menahan pemicu F8"
            );
            assert_eq!(
                keyboard_hook_proc(HC_ACTION, WM_KEYUP as Wparam, &key as *const _ as Lparam),
                1,
                "hook harus menahan release F8"
            );
            pump_messages_for(Duration::from_millis(350));
            wait_macro_idle();
            let after_macro = get_window_text(target_edit).len();
            assert!(
                after_macro > before_macro,
                "macro F8 harus mengirim Enter ke target"
            );
            assert_eq!(state.macro_status_kind, StatusKind::Sent);

            state
                .macro_library
                .selected_mut()
                .expect("macro mouse tersedia")
                .trigger = MacroTrigger::MouseX1;
            let before_mouse_macro = get_window_text(target_edit).len();
            let mouse = MsLlHookStruct {
                point: Point::default(),
                mouse_data: (XBUTTON1 as Dword) << 16,
                flags: 0,
                time: 0,
                extra_info: 0,
            };
            assert_eq!(
                mouse_hook_proc(
                    HC_ACTION,
                    WM_XBUTTONDOWN as Wparam,
                    &mouse as *const _ as Lparam,
                ),
                1,
                "hook harus menahan pemicu Mouse 4"
            );
            assert_eq!(
                mouse_hook_proc(
                    HC_ACTION,
                    WM_XBUTTONUP as Wparam,
                    &mouse as *const _ as Lparam,
                ),
                1,
                "hook harus menahan release Mouse 4"
            );
            pump_messages_for(Duration::from_millis(350));
            wait_macro_idle();
            assert!(
                get_window_text(target_edit).len() > before_mouse_macro,
                "macro Mouse 4 harus mengirim Enter ke target"
            );
            assert_eq!(state.macro_status_kind, StatusKind::Sent);

            state
                .macro_library
                .selected_mut()
                .expect("macro middle tersedia")
                .trigger = MacroTrigger::MouseMiddle;
            let before_middle_inputs = TEST_MACRO_INPUT_COUNT.load(Ordering::Relaxed);
            let middle = MsLlHookStruct {
                point: Point::default(),
                mouse_data: 0,
                flags: 0,
                time: 0,
                extra_info: 0,
            };
            mouse_hook_proc(
                HC_ACTION,
                WM_MBUTTONDOWN as Wparam,
                &middle as *const _ as Lparam,
            );
            mouse_hook_proc(
                HC_ACTION,
                WM_MBUTTONUP as Wparam,
                &middle as *const _ as Lparam,
            );
            pump_messages_for(Duration::from_millis(180));
            wait_macro_idle();
            assert!(
                TEST_MACRO_INPUT_COUNT.load(Ordering::Relaxed) >= before_middle_inputs + 2,
                "pemicu Middle harus menjalankan macro"
            );

            state
                .macro_library
                .selected_mut()
                .expect("macro Mouse 5 tersedia")
                .trigger = MacroTrigger::MouseX2;
            let before_mouse5_inputs = TEST_MACRO_INPUT_COUNT.load(Ordering::Relaxed);
            let mouse5 = MsLlHookStruct {
                mouse_data: (XBUTTON2 as Dword) << 16,
                ..middle
            };
            mouse_hook_proc(
                HC_ACTION,
                WM_XBUTTONDOWN as Wparam,
                &mouse5 as *const _ as Lparam,
            );
            mouse_hook_proc(
                HC_ACTION,
                WM_XBUTTONUP as Wparam,
                &mouse5 as *const _ as Lparam,
            );
            pump_messages_for(Duration::from_millis(180));
            wait_macro_idle();
            assert!(
                TEST_MACRO_INPUT_COUNT.load(Ordering::Relaxed) >= before_mouse5_inputs + 2,
                "pemicu Mouse 5 harus menjalankan macro"
            );

            let item = state
                .macro_library
                .selected_mut()
                .expect("macro repeat tersedia");
            item.mode = MacroMode::RepeatWhileHolding;
            item.trigger = MacroTrigger::F9;
            item.on_press = vec![
                MacroEvent::Delay(20),
                MacroEvent::KeyDown(VK_RETURN),
                MacroEvent::KeyUp(VK_RETURN),
            ];
            let f9 = KbdLlHookStruct {
                vk_code: VK_F9 as Dword,
                scan_code: 0,
                flags: 0,
                time: 0,
                extra_info: 0,
            };
            let before_repeat_inputs = TEST_MACRO_INPUT_COUNT.load(Ordering::Relaxed);
            SetForegroundWindow(target_window);
            SetFocus(target_edit);
            keyboard_hook_proc(HC_ACTION, WM_KEYDOWN as Wparam, &f9 as *const _ as Lparam);
            pump_messages_for(Duration::from_millis(120));
            keyboard_hook_proc(HC_ACTION, WM_KEYUP as Wparam, &f9 as *const _ as Lparam);
            wait_macro_idle();
            assert!(
                TEST_MACRO_INPUT_COUNT.load(Ordering::Relaxed) >= before_repeat_inputs + 4,
                "While Holding harus mengulang macro"
            );

            let item = state
                .macro_library
                .selected_mut()
                .expect("macro toggle tersedia");
            item.mode = MacroMode::Toggle;
            item.trigger = MacroTrigger::F9;
            let before_toggle_inputs = TEST_MACRO_INPUT_COUNT.load(Ordering::Relaxed);
            SetForegroundWindow(target_window);
            SetFocus(target_edit);
            keyboard_hook_proc(HC_ACTION, WM_KEYDOWN as Wparam, &f9 as *const _ as Lparam);
            keyboard_hook_proc(HC_ACTION, WM_KEYUP as Wparam, &f9 as *const _ as Lparam);
            assert!(
                MACRO_PLAYING.load(Ordering::Acquire),
                "Toggle harus memulai playback pada tekan pertama"
            );
            pump_messages_for(Duration::from_millis(120));
            keyboard_hook_proc(HC_ACTION, WM_KEYDOWN as Wparam, &f9 as *const _ as Lparam);
            keyboard_hook_proc(HC_ACTION, WM_KEYUP as Wparam, &f9 as *const _ as Lparam);
            wait_macro_idle();
            assert!(
                TEST_MACRO_INPUT_COUNT.load(Ordering::Relaxed) >= before_toggle_inputs + 4,
                "Toggle harus mengulang hingga pemicu ditekan lagi"
            );

            let item = state
                .macro_library
                .selected_mut()
                .expect("macro sequence tersedia");
            item.mode = MacroMode::Sequence;
            item.trigger = MacroTrigger::F8;
            item.on_press = vec![
                MacroEvent::Delay(10),
                MacroEvent::KeyDown(VK_RETURN),
                MacroEvent::KeyUp(VK_RETURN),
            ];
            item.while_holding = vec![
                MacroEvent::Delay(20),
                MacroEvent::KeyDown(VK_RETURN),
                MacroEvent::KeyUp(VK_RETURN),
            ];
            item.on_release = vec![
                MacroEvent::Delay(10),
                MacroEvent::KeyDown(VK_RETURN),
                MacroEvent::KeyUp(VK_RETURN),
            ];
            let before_sequence_inputs = TEST_MACRO_INPUT_COUNT.load(Ordering::Relaxed);
            SetForegroundWindow(target_window);
            SetFocus(target_edit);
            keyboard_hook_proc(HC_ACTION, WM_KEYDOWN as Wparam, &key as *const _ as Lparam);
            pump_messages_for(Duration::from_millis(80));
            keyboard_hook_proc(HC_ACTION, WM_KEYUP as Wparam, &key as *const _ as Lparam);
            wait_macro_idle();
            assert!(
                TEST_MACRO_INPUT_COUNT.load(Ordering::Relaxed) >= before_sequence_inputs + 6,
                "Sequence harus menjalankan On Press, While Holding, dan On Release"
            );

            TEST_INPUT_TARGET.store(0, Ordering::Relaxed);
            let _ = fs::remove_file(&e2e_macro_path);
            let _ = fs::remove_file(e2e_macro_path.with_extension("vtm.tmp"));
            let _ = fs::remove_file(e2e_macro_path.with_extension("vtm.bak"));
            DestroyWindow(target_window);
            DestroyWindow(main_window);
        }
    }
}
