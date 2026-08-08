#![windows_subsystem = "windows"]

use std::ffi::c_void;
use std::mem::{size_of, zeroed};
use std::ptr::{null, null_mut};
use std::thread;
use std::time::{Duration, Instant};

#[cfg(test)]
use std::sync::atomic::{AtomicIsize, Ordering};

use vibe_timer_core::{DurationFields, format_duration};

type Bool = i32;
type Dword = u32;
type Hbrush = isize;
type Hcursor = isize;
type Hdc = isize;
type Hgdiobj = isize;
type Hicon = isize;
type Hinstance = isize;
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
const WM_SETFONT: Uint = 0x0030;
const WM_SETICON: Uint = 0x0080;

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
const DT_VCENTER: Uint = 0x0004;
const DT_SINGLELINE: Uint = 0x0020;
const DT_NOPREFIX: Uint = 0x0800;
const DT_END_ELLIPSIS: Uint = 0x8000;

const TRANSPARENT: i32 = 1;
const PS_SOLID: i32 = 0;
const SRCCOPY: Dword = 0x00CC_0020;

const TME_LEAVE: Dword = 0x0000_0002;
const INPUT_KEYBOARD: Dword = 1;
const KEYEVENTF_KEYUP: Dword = 0x0002;
const KEYEVENTF_UNICODE: Dword = 0x0004;
const VK_RETURN: u16 = 0x0D;
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
const CLIENT_HEIGHT: i32 = 650;

const COLOR_BG: u32 = rgb(11, 15, 21);
const COLOR_SURFACE: u32 = rgb(17, 24, 33);
const COLOR_SURFACE_2: u32 = rgb(24, 33, 45);
const COLOR_BORDER: u32 = rgb(39, 52, 68);
const COLOR_BORDER_HOT: u32 = rgb(95, 75, 165);
const COLOR_TEXT: u32 = rgb(236, 241, 248);
const COLOR_MUTED: u32 = rgb(145, 159, 177);
const COLOR_DIM: u32 = rgb(94, 108, 126);
const COLOR_ACCENT: u32 = rgb(139, 92, 246);
const COLOR_ACCENT_HOT: u32 = rgb(154, 112, 248);
const COLOR_ACCENT_DARK: u32 = rgb(64, 45, 112);
const COLOR_SUCCESS: u32 = rgb(48, 210, 137);
const COLOR_WARNING: u32 = rgb(248, 180, 64);
const COLOR_ERROR: u32 = rgb(244, 91, 105);

#[cfg(test)]
static TEST_INPUT_TARGET: AtomicIsize = AtomicIsize::new(0);

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
    AddThirtyMinutes,
    AddOneHour,
    AddThreeHours,
    PickTarget,
    EnterOnly,
    TextAndEnter,
    MainAction,
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
    hour_edit: Hwnd,
    minute_edit: Hwnd,
    second_edit: Hwnd,
    prompt_edit: Hwnd,
    edit_brush: Hbrush,
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
}

impl AppState {
    fn new() -> Self {
        Self {
            window: 0,
            hour_edit: 0,
            minute_edit: 0,
            second_edit: 0,
            prompt_edit: 0,
            edit_brush: 0,
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
        }
    }

    unsafe fn set_controls_visible(&self, visible: bool) {
        let command = if visible { SW_SHOWNORMAL } else { SW_HIDE };
        unsafe {
            ShowWindow(self.hour_edit, command);
            ShowWindow(self.minute_edit, command);
            ShowWindow(self.second_edit, command);
            ShowWindow(self.prompt_edit, command);
        }
    }

    unsafe fn set_prompt_enabled(&self) {
        let enabled = self.action_mode == ActionMode::TextAndEnter && !self.running;
        unsafe {
            EnableWindow(self.prompt_edit, if enabled { TRUE } else { FALSE });
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

fn wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(std::iter::once(0)).collect()
}

fn low_word(value: Lparam) -> i32 {
    (value as u32 & 0xFFFF) as i16 as i32
}

fn high_word(value: Lparam) -> i32 {
    ((value as u32 >> 16) & 0xFFFF) as i16 as i32
}

fn hit_test(x: i32, y: i32, running: bool) -> HitTarget {
    if RECT_MAIN_ACTION.contains(x, y) {
        return HitTarget::MainAction;
    }
    if running {
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
    let face = wide("Segoe UI");
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
            COLOR_TEXT,
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

unsafe fn draw_clock_mark(dc: Hdc) {
    unsafe {
        let brush = CreateSolidBrush(COLOR_ACCENT_DARK);
        let pen = CreatePen(PS_SOLID, 2, COLOR_ACCENT);
        let old_brush = SelectObject(dc, brush);
        let old_pen = SelectObject(dc, pen);
        Ellipse(dc, 25, 24, 51, 50);
        MoveToEx(dc, 38, 30, null_mut());
        LineTo(dc, 38, 38);
        LineTo(dc, 44, 41);
        SelectObject(dc, old_brush);
        SelectObject(dc, old_pen);
        DeleteObject(brush);
        DeleteObject(pen);
    }
}

unsafe fn draw_status_pill(dc: Hdc, state: &AppState) {
    let (label, dot_color, width) = match state.status_kind {
        StatusKind::Ready => ("SIAP", COLOR_MUTED, 74),
        StatusKind::Running => ("AKTIF", COLOR_ACCENT, 82),
        StatusKind::Sent => ("TERKIRIM", COLOR_SUCCESS, 104),
        StatusKind::Warning => ("PERHATIAN", COLOR_WARNING, 118),
        StatusKind::Error => ("GAGAL", COLOR_ERROR, 82),
    };
    let rect = Rect::new(496 - width, 26, 496, 50);
    unsafe {
        rounded_box(dc, rect, 12, COLOR_SURFACE_2, COLOR_BORDER);
        let dot = Rect::new(rect.left + 10, 35, rect.left + 16, 41);
        filled_circle(dc, dot, dot_color);
        draw_label(
            dc,
            label,
            Rect::new(rect.left + 21, rect.top, rect.right - 7, rect.bottom),
            COLOR_MUTED,
            state.fonts.small,
            DT_CENTER | DT_VCENTER | DT_SINGLELINE,
        );
    }
}

unsafe fn draw_interface(dc: Hdc, state: &AppState) {
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
        draw_status_pill(dc, state);

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
        state.fonts.title = make_font(23, 700);
        state.fonts.timer = make_font(38, 600);
        state.fonts.body = make_font(17, 400);
        state.fonts.semibold = make_font(16, 600);
        state.fonts.small = make_font(13, 600);
        state.edit_brush = CreateSolidBrush(COLOR_SURFACE_2);

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

unsafe fn handle_click(state: &mut AppState, target: HitTarget) {
    match target {
        HitTarget::None => {}
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
            unsafe { initialize_controls(state, instance) };
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
                draw_interface(memory_dc, state);
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
            unsafe {
                SetTextColor(
                    dc,
                    if state.action_mode == ActionMode::EnterOnly && lparam == state.prompt_edit {
                        COLOR_DIM
                    } else {
                        COLOR_TEXT
                    },
                );
                SetBkColor(dc, COLOR_SURFACE_2);
            }
            state.edit_brush
        }
        WM_MOUSEMOVE => {
            let x = low_word(lparam);
            let y = high_word(lparam);
            let new_hot = hit_test(x, y, state.running);
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
            let target = hit_test(x, y, state.running);
            unsafe { handle_click(state, target) };
            0
        }
        WM_COMMAND => 0,
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
            unsafe { DestroyWindow(window) };
            0
        }
        WM_DESTROY => {
            unsafe {
                KillTimer(window, TIMER_COUNTDOWN);
                KillTimer(window, TIMER_CAPTURE);
                PostQuitMessage(0);
            }
            0
        }
        WM_NCDESTROY => unsafe {
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

            state.target = Some(TargetWindow {
                window: target_window,
                process_id: GetCurrentProcessId(),
                title: "VibeTimer E2E Target".to_owned(),
            });
            assert_eq!(
                state.target.as_ref().map(|target| target.title.as_str()),
                Some("VibeTimer E2E Target")
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

            TEST_INPUT_TARGET.store(0, Ordering::Relaxed);
            DestroyWindow(target_window);
            DestroyWindow(main_window);
        }
    }
}
