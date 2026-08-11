#![windows_subsystem = "windows"]

use std::collections::HashMap;
use std::ffi::c_void;
use std::mem::{size_of, zeroed};
use std::path::{Path, PathBuf};
use std::ptr::{null, null_mut};
use std::sync::atomic::{AtomicBool, AtomicPtr, AtomicU32, Ordering};
use std::thread;
use std::time::{Duration, Instant};

#[cfg(test)]
use std::sync::atomic::{AtomicIsize, AtomicUsize};

use vibe_timer_core::backup::{BackupBundle, load_backup, save_backup};
use vibe_timer_core::macro_engine::{
    MacroDefinition, MacroEvent, MacroLibrary, MacroMode, MacroTarget, MacroTrigger, MouseButton,
    default_data_path, delete_event, duplicate_event, insert_delay, load_library, move_event,
    save_library,
};
use vibe_timer_core::profiles::{
    ProfileLibrary, default_profiles_path, load_profiles, save_profiles,
};
use vibe_timer_core::settings::{
    AppSettings, EmergencyHotkey, default_settings_path, load_settings, save_settings,
};
use vibe_timer_core::smart_reset::{ClockContext, parse_reset_text};
use vibe_timer_core::timers::{
    TimerAction, TimerLibrary, TimerPhase, default_timers_path, load_timers, now_unix_ms,
    save_timers,
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
const WM_SIZE: Uint = 0x0005;
const WM_TIMER: Uint = 0x0113;
const WM_HOTKEY: Uint = 0x0312;
const WM_CTLCOLOREDIT: Uint = 0x0133;
const WM_CTLCOLORSTATIC: Uint = 0x0138;
const WM_MOUSEMOVE: Uint = 0x0200;
const WM_LBUTTONDOWN: Uint = 0x0201;
const WM_LBUTTONUP: Uint = 0x0202;
const WM_RBUTTONDOWN: Uint = 0x0204;
const WM_RBUTTONUP: Uint = 0x0205;
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
const WM_APP_TRAY: Uint = 0x8002;

const EM_SETMARGINS: Uint = 0x00D3;
const EM_SETLIMITTEXT: Uint = 0x00C5;
const EC_LEFTMARGIN: Wparam = 0x0001;
const EC_RIGHTMARGIN: Wparam = 0x0002;

const GWLP_USERDATA: i32 = -21;
const TIMER_COUNTDOWN: usize = 1;
const TIMER_CAPTURE: usize = 2;
const EMERGENCY_HOTKEY_ID: i32 = 1;
const TRAY_ICON_ID: Uint = 1;
const GA_ROOT: Uint = 2;
const MAPVK_VK_TO_VSC: Uint = 0;
const PROCESS_QUERY_LIMITED_INFORMATION: Dword = 0x1000;

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
const DT_WORDBREAK: Uint = 0x0010;
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
const VK_F12: Uint = 0x7B;
const VK_PAUSE: Uint = 0x13;
const WH_KEYBOARD_LL: i32 = 13;
const WH_MOUSE_LL: i32 = 14;
const HC_ACTION: i32 = 0;
const LLKHF_INJECTED: Dword = 0x10;
const LLMHF_INJECTED: Dword = 0x01;
const XBUTTON1: u16 = 1;
const XBUTTON2: u16 = 2;
const MOD_ALT: Uint = 0x0001;
const MOD_CONTROL: Uint = 0x0002;
const MOD_SHIFT: Uint = 0x0004;
const MOD_NOREPEAT: Uint = 0x4000;
const SIZE_MINIMIZED: Wparam = 1;
const NIM_ADD: Dword = 0x0000_0000;
const NIM_MODIFY: Dword = 0x0000_0001;
const NIM_DELETE: Dword = 0x0000_0002;
const NIF_MESSAGE: Uint = 0x0000_0001;
const NIF_ICON: Uint = 0x0000_0002;
const NIF_TIP: Uint = 0x0000_0004;
const NIF_INFO: Uint = 0x0000_0010;
const NIIF_INFO: Dword = 0x0000_0001;
const MF_STRING: Uint = 0x0000_0000;
const MF_SEPARATOR: Uint = 0x0000_0800;
const TPM_RIGHTBUTTON: Uint = 0x0002;
const TPM_BOTTOMALIGN: Uint = 0x0020;
const MENU_OPEN: usize = 9_001;
const MENU_STOP_ALL: usize = 9_002;
const MENU_EXIT: usize = 9_003;
#[cfg(not(test))]
const HKEY_CURRENT_USER: isize = 0x8000_0001u32 as isize;
#[cfg(not(test))]
const REG_SZ: Dword = 1;
const OFN_OVERWRITEPROMPT: Dword = 0x0000_0002;
const OFN_PATHMUSTEXIST: Dword = 0x0000_0800;
const OFN_FILEMUSTEXIST: Dword = 0x0000_1000;
const OFN_EXPLORER: Dword = 0x0008_0000;
const CF_UNICODETEXT: Uint = 13;
const ERROR_ALREADY_EXISTS: Dword = 183;
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

const CLIENT_WIDTH: i32 = 900;
const MACRO_CLIENT_WIDTH: i32 = 1120;
const SETTINGS_CLIENT_WIDTH: i32 = 820;
const PROFILES_CLIENT_WIDTH: i32 = 900;
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
#[cfg(test)]
static TEST_AUTOSTART_ENABLED: AtomicBool = AtomicBool::new(false);
static APP_STATE_POINTER: AtomicPtr<AppState> = AtomicPtr::new(null_mut());
static MACRO_PLAYING: AtomicBool = AtomicBool::new(false);
static MACRO_PLAYING_ID: AtomicU32 = AtomicU32::new(0);
static MACRO_STOP: AtomicBool = AtomicBool::new(false);
static TRIGGER_HELD: AtomicBool = AtomicBool::new(false);
static EMERGENCY_ACTIVE: AtomicBool = AtomicBool::new(false);

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
struct SystemTimeW {
    year: u16,
    month: u16,
    day_of_week: u16,
    day: u16,
    hour: u16,
    minute: u16,
    second: u16,
    milliseconds: u16,
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

#[repr(C)]
struct Guid {
    data1: u32,
    data2: u16,
    data3: u16,
    data4: [u8; 8],
}

#[repr(C)]
struct NotifyIconDataW {
    size: Dword,
    window: Hwnd,
    id: Uint,
    flags: Uint,
    callback_message: Uint,
    icon: Hicon,
    tip: [u16; 128],
    state: Dword,
    state_mask: Dword,
    info: [u16; 256],
    timeout_or_version: Uint,
    info_title: [u16; 64],
    info_flags: Dword,
    guid: Guid,
    balloon_icon: Hicon,
}

#[repr(C)]
struct OpenFileNameW {
    size: Dword,
    owner: Hwnd,
    instance: Hinstance,
    filter: *const u16,
    custom_filter: *mut u16,
    max_custom_filter: Dword,
    filter_index: Dword,
    file: *mut u16,
    max_file: Dword,
    file_title: *mut u16,
    max_file_title: Dword,
    initial_directory: *const u16,
    title: *const u16,
    flags: Dword,
    file_offset: u16,
    file_extension: u16,
    default_extension: *const u16,
    custom_data: Lparam,
    hook: *const c_void,
    template_name: *const u16,
    reserved: *mut c_void,
    reserved_value: Dword,
    flags_ex: Dword,
}

impl Default for NotifyIconDataW {
    fn default() -> Self {
        unsafe { zeroed() }
    }
}

struct OwnedKernelHandle(isize);

impl Drop for OwnedKernelHandle {
    fn drop(&mut self) {
        if self.0 != 0 {
            unsafe { CloseHandle(self.0) };
        }
    }
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
    fn GetCursorPos(point: *mut Point) -> Bool;
    fn WindowFromPoint(point: Point) -> Hwnd;
    fn GetAncestor(window: Hwnd, flags: Uint) -> Hwnd;
    fn ScreenToClient(window: Hwnd, point: *mut Point) -> Bool;
    fn ClientToScreen(window: Hwnd, point: *mut Point) -> Bool;
    fn EnumWindows(
        callback: Option<unsafe extern "system" fn(Hwnd, Lparam) -> Bool>,
        lparam: Lparam,
    ) -> Bool;
    fn MapVirtualKeyW(code: Uint, map_type: Uint) -> Uint;
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
    fn RegisterHotKey(window: Hwnd, id: i32, modifiers: Uint, virtual_key: Uint) -> Bool;
    fn UnregisterHotKey(window: Hwnd, id: i32) -> Bool;
    fn OpenClipboard(window: Hwnd) -> Bool;
    fn CloseClipboard() -> Bool;
    fn IsClipboardFormatAvailable(format: Uint) -> Bool;
    fn GetClipboardData(format: Uint) -> isize;
    fn FindWindowW(class_name: *const u16, window_name: *const u16) -> Hwnd;
    fn CreatePopupMenu() -> Hmenu;
    fn AppendMenuW(menu: Hmenu, flags: Uint, id: usize, text: *const u16) -> Bool;
    fn TrackPopupMenu(
        menu: Hmenu,
        flags: Uint,
        x: i32,
        y: i32,
        reserved: i32,
        window: Hwnd,
        rect: *const Rect,
    ) -> Bool;
    fn DestroyMenu(menu: Hmenu) -> Bool;
    fn DestroyIcon(icon: Hicon) -> Bool;
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
    fn GetActiveWindow() -> Hwnd;
    #[cfg(test)]
    fn PeekMessageW(message: *mut Msg, window: Hwnd, min: Uint, max: Uint, remove: Uint) -> Bool;
    #[cfg(test)]
    fn GetDC(window: Hwnd) -> Hdc;
    #[cfg(test)]
    fn ReleaseDC(window: Hwnd, dc: Hdc) -> i32;
    #[cfg(test)]
    fn PrintWindow(window: Hwnd, dc: Hdc, flags: Uint) -> Bool;
}

#[link(name = "shell32")]
unsafe extern "system" {
    fn Shell_NotifyIconW(message: Dword, data: *mut NotifyIconDataW) -> Bool;
}

#[link(name = "comdlg32")]
unsafe extern "system" {
    fn GetSaveFileNameW(data: *mut OpenFileNameW) -> Bool;
    fn GetOpenFileNameW(data: *mut OpenFileNameW) -> Bool;
}

#[cfg(not(test))]
#[link(name = "advapi32")]
unsafe extern "system" {
    fn RegSetKeyValueW(
        key: isize,
        sub_key: *const u16,
        value_name: *const u16,
        kind: Dword,
        data: *const c_void,
        data_size: Dword,
    ) -> i32;
    fn RegDeleteKeyValueW(key: isize, sub_key: *const u16, value_name: *const u16) -> i32;
}

#[link(name = "kernel32")]
unsafe extern "system" {
    fn GetModuleHandleW(module_name: *const u16) -> Hinstance;
    fn GetCurrentThreadId() -> Dword;
    #[cfg(test)]
    fn GetCurrentProcessId() -> Dword;
    fn OpenProcess(access: Dword, inherit_handle: Bool, process_id: Dword) -> isize;
    fn QueryFullProcessImageNameW(
        process: isize,
        flags: Dword,
        path: *mut u16,
        size: *mut Dword,
    ) -> Bool;
    fn CloseHandle(handle: isize) -> Bool;
    fn GlobalLock(memory: isize) -> *mut c_void;
    fn GlobalUnlock(memory: isize) -> Bool;
    fn GetLocalTime(time: *mut SystemTimeW);
    fn CreateMutexW(attributes: *mut c_void, initial_owner: Bool, name: *const u16) -> isize;
    fn GetLastError() -> Dword;
    fn GlobalSize(memory: isize) -> usize;
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
    Profiles,
    Settings,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MacroLane {
    OnPress,
    WhileHolding,
    OnRelease,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CaptureKind {
    Timer,
    Macro,
    Profile,
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
    ProfilesTab,
    SettingsTab,
    AddThirtyMinutes,
    AddOneHour,
    AddThreeHours,
    TimerNew,
    TimerItem(usize),
    TimerDuplicate,
    TimerDelete,
    TimerSave,
    SmartResetClipboard,
    SmartResetApply,
    PickTarget,
    EnterOnly,
    TextAndEnter,
    MainAction,
    MacroNew,
    MacroItem(usize),
    MacroMode(MacroMode),
    MacroTrigger(MacroTrigger),
    MacroLane(MacroLane),
    MacroEvent(usize),
    MacroScopeGlobal,
    MacroScopeTarget,
    MacroTargetPick,
    MacroDelayMinus,
    MacroDelayPlus,
    MacroDelayApply,
    MacroEventUp,
    MacroEventDown,
    MacroEventDuplicate,
    MacroEventDelete,
    MacroInsertDelay,
    MacroDuplicate,
    MacroDelete,
    MacroRecord,
    MacroClear,
    MacroSave,
    SettingMinimizeTray,
    SettingCloseTray,
    SettingAutoStart,
    SettingEmergencyHotkey(EmergencyHotkey),
    SettingEmergencyTimers,
    SettingMaxRuntime(u32),
    SettingMaxRepeats(u32),
    SettingTestEmergency,
    ProfileNew,
    ProfileItem(usize),
    ProfileDuplicate,
    ProfileDelete,
    ProfileTargetPick,
    ProfileMacro(usize),
    ProfileUseTimer,
    ProfileSave,
    BackupExport,
    BackupImport,
}

#[derive(Clone)]
struct TargetWindow {
    window: Hwnd,
    process_id: Dword,
    title: String,
    executable: String,
}

#[derive(Clone)]
struct MacroPlaybackTarget {
    root: Hwnd,
    receiver: Hwnd,
    process_id: Dword,
    title: String,
}

#[derive(Clone)]
enum PlaybackDestination {
    Foreground,
    Window(MacroPlaybackTarget),
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
    macro_delay_edit: Hwnd,
    profile_name_edit: Hwnd,
    timer_name_edit: Hwnd,
    smart_reset_edit: Hwnd,
    edit_brush: Hbrush,
    panel_edit_brush: Hbrush,
    fonts: Fonts,
    action_mode: ActionMode,
    status_kind: StatusKind,
    status: String,
    target: Option<TargetWindow>,
    running: bool,
    capture_deadline: Option<Instant>,
    capture_kind: CaptureKind,
    original_seconds: u64,
    remaining_seconds: u64,
    armed_prompt: String,
    timer_library: TimerLibrary,
    timers_path: PathBuf,
    timer_targets: HashMap<u32, TargetWindow>,
    timer_dirty: bool,
    hot: HitTarget,
    tracking_mouse: bool,
    macro_library: MacroLibrary,
    macro_path: PathBuf,
    profile_library: ProfileLibrary,
    profiles_path: PathBuf,
    profile_status_kind: StatusKind,
    profile_status: String,
    profile_dirty: bool,
    profile_targets: HashMap<u32, MacroPlaybackTarget>,
    settings: AppSettings,
    settings_path: PathBuf,
    settings_status_kind: StatusKind,
    settings_status: String,
    tray_icon: Hicon,
    tray_added: bool,
    exit_requested: bool,
    macro_status_kind: StatusKind,
    macro_status: String,
    macro_dirty: bool,
    macro_lane: MacroLane,
    macro_selected_event: Option<usize>,
    macro_targets: HashMap<u32, MacroPlaybackTarget>,
    recording: bool,
    record_last_event: Option<Instant>,
    suppress_escape_until_up: bool,
    trigger_down: bool,
    trigger_macro_id: Option<u32>,
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
        let settings_path = default_settings_path();
        let (settings, settings_status_kind, settings_status) = match load_settings(&settings_path)
        {
            Ok(settings) => (
                settings,
                StatusKind::Ready,
                "Pengaturan aktif dan disimpan otomatis.".to_owned(),
            ),
            Err(error) => (
                AppSettings::default(),
                StatusKind::Warning,
                format!("Pengaturan lama diabaikan: {error}"),
            ),
        };
        let profiles_path = default_profiles_path();
        let (mut profile_library, profile_status_kind, profile_status) =
            match load_profiles(&profiles_path) {
                Ok(profiles) => (
                    profiles,
                    StatusKind::Ready,
                    "Profil siap. Pilih target lalu tautkan macro.".to_owned(),
                ),
                Err(error) => (
                    ProfileLibrary::default(),
                    StatusKind::Warning,
                    format!("File profil tidak dapat dibaca: {error}"),
                ),
            };
        let macro_ids: Vec<u32> = macro_library.macros.iter().map(|item| item.id).collect();
        profile_library.remove_missing_macro_links(&macro_ids);
        let timers_path = default_timers_path();
        let (mut timer_library, mut timer_status_kind, mut timer_status) =
            match load_timers(&timers_path) {
                Ok(library) => (
                    library,
                    StatusKind::Ready,
                    "Multi Timer siap. Pilih target untuk mulai.".to_owned(),
                ),
                Err(error) => (
                    TimerLibrary::default(),
                    StatusKind::Warning,
                    format!("File timer tidak dapat dibaca: {error}"),
                ),
            };
        let missed = timer_library.recover_after_restart(now_unix_ms());
        if missed > 0 {
            timer_status_kind = StatusKind::Warning;
            timer_status = format!(
                "{missed} timer terlewat saat aplikasi tertutup; tidak ada input yang dikirim."
            );
            if let Err(error) = save_timers(&timers_path, &timer_library) {
                timer_status = format!("{timer_status} Status aman gagal disimpan: {error}");
            }
        } else if timer_library.running_count() > 0 {
            timer_status_kind = StatusKind::Running;
            timer_status = format!(
                "{} timer dipulihkan dan tetap berjalan.",
                timer_library.running_count()
            );
        }
        let selected_timer = timer_library
            .selected()
            .cloned()
            .unwrap_or_else(|| TimerLibrary::default().timers.into_iter().next().unwrap());
        Self {
            window: 0,
            tab: AppTab::Timer,
            hour_edit: 0,
            minute_edit: 0,
            second_edit: 0,
            prompt_edit: 0,
            macro_name_edit: 0,
            macro_delay_edit: 0,
            profile_name_edit: 0,
            timer_name_edit: 0,
            smart_reset_edit: 0,
            edit_brush: 0,
            panel_edit_brush: 0,
            fonts: Fonts::default(),
            action_mode: match selected_timer.action {
                TimerAction::EnterOnly => ActionMode::EnterOnly,
                TimerAction::TextAndEnter => ActionMode::TextAndEnter,
            },
            status_kind: timer_status_kind,
            status: timer_status,
            target: None,
            running: selected_timer.is_running(),
            capture_deadline: None,
            capture_kind: CaptureKind::Timer,
            original_seconds: selected_timer.duration_seconds,
            remaining_seconds: selected_timer.remaining_seconds,
            armed_prompt: selected_timer.prompt.clone(),
            timer_library,
            timers_path,
            timer_targets: HashMap::new(),
            timer_dirty: false,
            hot: HitTarget::None,
            tracking_mouse: false,
            macro_library,
            macro_path,
            profile_library,
            profiles_path,
            profile_status_kind,
            profile_status,
            profile_dirty: false,
            profile_targets: HashMap::new(),
            settings,
            settings_path,
            settings_status_kind,
            settings_status,
            tray_icon: 0,
            tray_added: false,
            exit_requested: false,
            macro_status_kind,
            macro_status,
            macro_dirty: false,
            macro_lane: MacroLane::OnPress,
            macro_selected_event: None,
            macro_targets: HashMap::new(),
            recording: false,
            record_last_event: None,
            suppress_escape_until_up: false,
            trigger_down: false,
            trigger_macro_id: None,
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
            ShowWindow(
                self.macro_delay_edit,
                if self.tab == AppTab::Macro && !self.recording && selected_delay(self).is_some() {
                    SW_SHOWNORMAL
                } else {
                    SW_HIDE
                },
            );
            ShowWindow(
                self.profile_name_edit,
                if self.tab == AppTab::Profiles {
                    SW_SHOWNORMAL
                } else {
                    SW_HIDE
                },
            );
            ShowWindow(
                self.timer_name_edit,
                if self.tab == AppTab::Timer {
                    SW_SHOWNORMAL
                } else {
                    SW_HIDE
                },
            );
            ShowWindow(
                self.smart_reset_edit,
                if self.tab == AppTab::Timer && !self.running {
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
            EnableWindow(
                self.macro_delay_edit,
                if !self.recording { TRUE } else { FALSE },
            );
            EnableWindow(self.profile_name_edit, TRUE);
            EnableWindow(
                self.timer_name_edit,
                if self.running { FALSE } else { TRUE },
            );
            EnableWindow(
                self.smart_reset_edit,
                if self.running { FALSE } else { TRUE },
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
            UnregisterHotKey(self.window, EMERGENCY_HOTKEY_ID);
            if self.tray_added {
                remove_tray_icon(self);
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
const RECT_TIMER_NEW: Rect = Rect::new(520, 117, 646, 157);
const RECT_TIMER_DUPLICATE: Rect = Rect::new(520, 510, 632, 548);
const RECT_TIMER_DELETE: Rect = Rect::new(640, 510, 752, 548);
const RECT_TIMER_SAVE: Rect = Rect::new(760, 510, 876, 548);
const RECT_SMART_CLIPBOARD: Rect = Rect::new(520, 614, 694, 642);
const RECT_SMART_APPLY: Rect = Rect::new(702, 614, 876, 642);
const RECT_TAB_TIMER: Rect = Rect::new(202, 24, 258, 54);
const RECT_TAB_MACRO: Rect = Rect::new(262, 24, 322, 54);
const RECT_TAB_PROFILES: Rect = Rect::new(326, 24, 402, 54);
const RECT_TAB_SETTINGS: Rect = Rect::new(406, 24, 496, 54);
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
const RECT_MACRO_SCOPE_GLOBAL: Rect = Rect::new(946, 169, 1008, 205);
const RECT_MACRO_SCOPE_TARGET: Rect = Rect::new(1016, 169, 1080, 205);
const RECT_MACRO_TARGET_PICK: Rect = Rect::new(946, 216, 1080, 256);
const RECT_MACRO_DELAY_MINUS: Rect = Rect::new(946, 393, 982, 429);
const RECT_MACRO_DELAY_PLUS: Rect = Rect::new(1044, 393, 1080, 429);
const RECT_MACRO_DELAY_APPLY: Rect = Rect::new(946, 440, 1080, 480);
const RECT_MACRO_EVENT_UP: Rect = Rect::new(946, 486, 1008, 522);
const RECT_MACRO_EVENT_DOWN: Rect = Rect::new(1016, 486, 1080, 522);
const RECT_MACRO_EVENT_DUPLICATE: Rect = Rect::new(946, 530, 1008, 566);
const RECT_MACRO_EVENT_DELETE: Rect = Rect::new(1016, 530, 1080, 566);
const RECT_MACRO_INSERT_DELAY: Rect = Rect::new(644, 548, 748, 596);
const RECT_MACRO_DUPLICATE: Rect = Rect::new(42, 548, 126, 590);
const RECT_MACRO_DELETE: Rect = Rect::new(134, 548, 218, 590);

const RECT_SETTING_MINIMIZE_TRAY: Rect = Rect::new(42, 178, 382, 224);
const RECT_SETTING_CLOSE_TRAY: Rect = Rect::new(42, 238, 382, 284);
const RECT_SETTING_AUTO_START: Rect = Rect::new(42, 298, 382, 344);
const RECT_SETTING_HOTKEY_1: Rect = Rect::new(438, 178, 772, 218);
const RECT_SETTING_HOTKEY_2: Rect = Rect::new(438, 226, 772, 266);
const RECT_SETTING_HOTKEY_3: Rect = Rect::new(438, 274, 772, 314);
const RECT_SETTING_STOP_TIMERS: Rect = Rect::new(438, 332, 772, 376);
const RECT_SETTING_RUNTIME_5: Rect = Rect::new(438, 411, 514, 449);
const RECT_SETTING_RUNTIME_30: Rect = Rect::new(522, 411, 598, 449);
const RECT_SETTING_RUNTIME_60: Rect = Rect::new(606, 411, 682, 449);
const RECT_SETTING_RUNTIME_OFF: Rect = Rect::new(690, 411, 772, 449);
const RECT_SETTING_REPEAT_100: Rect = Rect::new(438, 493, 514, 531);
const RECT_SETTING_REPEAT_1000: Rect = Rect::new(522, 493, 598, 531);
const RECT_SETTING_REPEAT_10000: Rect = Rect::new(606, 493, 682, 531);
const RECT_SETTING_REPEAT_OFF: Rect = Rect::new(690, 493, 772, 531);
const RECT_SETTING_TEST_EMERGENCY: Rect = Rect::new(24, 548, 796, 604);
const RECT_PROFILE_NEW: Rect = Rect::new(42, 117, 218, 157);
const RECT_PROFILE_DUPLICATE: Rect = Rect::new(42, 548, 126, 590);
const RECT_PROFILE_DELETE: Rect = Rect::new(134, 548, 218, 590);
const RECT_PROFILE_USE_TIMER: Rect = Rect::new(278, 242, 548, 282);
const RECT_PROFILE_TARGET_PICK: Rect = Rect::new(560, 242, 850, 282);
const RECT_PROFILE_EXPORT: Rect = Rect::new(278, 548, 446, 596);
const RECT_PROFILE_IMPORT: Rect = Rect::new(456, 548, 624, 596);
const RECT_PROFILE_SAVE: Rect = Rect::new(634, 548, 850, 596);

fn macro_trigger_rect(index: usize) -> Rect {
    let left = 278 + index as i32 * 116;
    Rect::new(left, 272, left + 106, 310)
}

fn macro_item_rect(index: usize) -> Rect {
    let top = 169 + index as i32 * 58;
    Rect::new(42, top, 218, top + 48)
}

fn profile_item_rect(index: usize) -> Rect {
    let top = 169 + index as i32 * 58;
    Rect::new(42, top, 218, top + 48)
}

fn profile_macro_rect(index: usize) -> Rect {
    let column = index % 2;
    let row = index / 2;
    let left = 278 + column as i32 * 286;
    let top = 340 + row as i32 * 58;
    Rect::new(left, top, left + 272, top + 48)
}

fn timer_item_rect(index: usize) -> Rect {
    let top = 170 + index as i32 * 56;
    Rect::new(520, top, 876, top + 48)
}

fn macro_event_rect(index: usize) -> Rect {
    let column = index % 6;
    let row = index / 6;
    let left = 292 + column as i32 * 100;
    let top = 390 + row as i32 * 40;
    Rect::new(left, top, left + 90, top + 30)
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

fn copy_wide<const N: usize>(destination: &mut [u16; N], value: &str) {
    destination.fill(0);
    for (slot, character) in destination
        .iter_mut()
        .take(N.saturating_sub(1))
        .zip(value.encode_utf16())
    {
        *slot = character;
    }
}

fn hit_test(x: i32, y: i32, state: &AppState) -> HitTarget {
    if RECT_TAB_TIMER.contains(x, y) {
        return HitTarget::TimerTab;
    }
    if RECT_TAB_MACRO.contains(x, y) {
        return HitTarget::MacroTab;
    }
    if RECT_TAB_PROFILES.contains(x, y) {
        return HitTarget::ProfilesTab;
    }
    if RECT_TAB_SETTINGS.contains(x, y) {
        return HitTarget::SettingsTab;
    }
    if state.tab == AppTab::Settings {
        for (rect, target) in [
            (RECT_SETTING_MINIMIZE_TRAY, HitTarget::SettingMinimizeTray),
            (RECT_SETTING_CLOSE_TRAY, HitTarget::SettingCloseTray),
            (RECT_SETTING_AUTO_START, HitTarget::SettingAutoStart),
            (
                RECT_SETTING_HOTKEY_1,
                HitTarget::SettingEmergencyHotkey(EmergencyHotkey::CtrlAltF12),
            ),
            (
                RECT_SETTING_HOTKEY_2,
                HitTarget::SettingEmergencyHotkey(EmergencyHotkey::CtrlShiftF12),
            ),
            (
                RECT_SETTING_HOTKEY_3,
                HitTarget::SettingEmergencyHotkey(EmergencyHotkey::CtrlAltPause),
            ),
            (RECT_SETTING_STOP_TIMERS, HitTarget::SettingEmergencyTimers),
            (RECT_SETTING_RUNTIME_5, HitTarget::SettingMaxRuntime(5 * 60)),
            (
                RECT_SETTING_RUNTIME_30,
                HitTarget::SettingMaxRuntime(30 * 60),
            ),
            (
                RECT_SETTING_RUNTIME_60,
                HitTarget::SettingMaxRuntime(60 * 60),
            ),
            (RECT_SETTING_RUNTIME_OFF, HitTarget::SettingMaxRuntime(0)),
            (RECT_SETTING_REPEAT_100, HitTarget::SettingMaxRepeats(100)),
            (
                RECT_SETTING_REPEAT_1000,
                HitTarget::SettingMaxRepeats(1_000),
            ),
            (
                RECT_SETTING_REPEAT_10000,
                HitTarget::SettingMaxRepeats(10_000),
            ),
            (RECT_SETTING_REPEAT_OFF, HitTarget::SettingMaxRepeats(0)),
            (RECT_SETTING_TEST_EMERGENCY, HitTarget::SettingTestEmergency),
        ] {
            if rect.contains(x, y) {
                return target;
            }
        }
        return HitTarget::None;
    }
    if state.tab == AppTab::Profiles {
        if RECT_PROFILE_NEW.contains(x, y) {
            return HitTarget::ProfileNew;
        }
        for (index, _) in state.profile_library.profiles.iter().take(6).enumerate() {
            if profile_item_rect(index).contains(x, y) {
                return HitTarget::ProfileItem(index);
            }
        }
        for (rect, target) in [
            (RECT_PROFILE_DUPLICATE, HitTarget::ProfileDuplicate),
            (RECT_PROFILE_DELETE, HitTarget::ProfileDelete),
            (RECT_PROFILE_TARGET_PICK, HitTarget::ProfileTargetPick),
            (RECT_PROFILE_USE_TIMER, HitTarget::ProfileUseTimer),
            (RECT_PROFILE_EXPORT, HitTarget::BackupExport),
            (RECT_PROFILE_IMPORT, HitTarget::BackupImport),
            (RECT_PROFILE_SAVE, HitTarget::ProfileSave),
        ] {
            if rect.contains(x, y) {
                return target;
            }
        }
        for (index, _) in state.macro_library.macros.iter().take(6).enumerate() {
            if profile_macro_rect(index).contains(x, y) {
                return HitTarget::ProfileMacro(index);
            }
        }
        return HitTarget::None;
    }
    if state.tab == AppTab::Macro {
        if RECT_MACRO_NEW.contains(x, y) {
            return HitTarget::MacroNew;
        }
        for (index, _) in state.macro_library.macros.iter().take(6).enumerate() {
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
        if let Some(item) = state.macro_library.selected() {
            for (index, _event) in lane_events(item, state.macro_lane)
                .iter()
                .take(18)
                .enumerate()
            {
                if macro_event_rect(index).contains(x, y) {
                    return HitTarget::MacroEvent(index);
                }
            }
        }
        if RECT_MACRO_SCOPE_GLOBAL.contains(x, y) {
            return HitTarget::MacroScopeGlobal;
        }
        if RECT_MACRO_SCOPE_TARGET.contains(x, y) {
            return HitTarget::MacroScopeTarget;
        }
        if RECT_MACRO_TARGET_PICK.contains(x, y) {
            return HitTarget::MacroTargetPick;
        }
        if RECT_MACRO_DELAY_MINUS.contains(x, y) {
            return HitTarget::MacroDelayMinus;
        }
        if RECT_MACRO_DELAY_PLUS.contains(x, y) {
            return HitTarget::MacroDelayPlus;
        }
        if RECT_MACRO_DELAY_APPLY.contains(x, y) {
            return HitTarget::MacroDelayApply;
        }
        for (rect, target) in [
            (RECT_MACRO_EVENT_UP, HitTarget::MacroEventUp),
            (RECT_MACRO_EVENT_DOWN, HitTarget::MacroEventDown),
            (RECT_MACRO_EVENT_DUPLICATE, HitTarget::MacroEventDuplicate),
            (RECT_MACRO_EVENT_DELETE, HitTarget::MacroEventDelete),
            (RECT_MACRO_INSERT_DELAY, HitTarget::MacroInsertDelay),
            (RECT_MACRO_DUPLICATE, HitTarget::MacroDuplicate),
            (RECT_MACRO_DELETE, HitTarget::MacroDelete),
        ] {
            if rect.contains(x, y) {
                return target;
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
    if state.tab == AppTab::Timer {
        if RECT_TIMER_NEW.contains(x, y) {
            return HitTarget::TimerNew;
        }
        for (index, _) in state.timer_library.timers.iter().enumerate() {
            if timer_item_rect(index).contains(x, y) {
                return HitTarget::TimerItem(index);
            }
        }
        for (rect, target) in [
            (RECT_TIMER_DUPLICATE, HitTarget::TimerDuplicate),
            (RECT_TIMER_DELETE, HitTarget::TimerDelete),
            (RECT_TIMER_SAVE, HitTarget::TimerSave),
            (RECT_SMART_CLIPBOARD, HitTarget::SmartResetClipboard),
            (RECT_SMART_APPLY, HitTarget::SmartResetApply),
        ] {
            if rect.contains(x, y) {
                return target;
            }
        }
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
    let kind = match state.tab {
        AppTab::Timer => state.status_kind,
        AppTab::Macro => state.macro_status_kind,
        AppTab::Profiles => state.profile_status_kind,
        AppTab::Settings => state.settings_status_kind,
    };
    let (label, dot_color, width) = match kind {
        StatusKind::Ready => ("Siap", COLOR_MUTED, 58),
        StatusKind::Running => ("Aktif", COLOR_ACCENT, 62),
        StatusKind::Sent => ("Selesai", COLOR_SUCCESS, 76),
        StatusKind::Warning => ("Periksa", COLOR_WARNING, 76),
        StatusKind::Error => ("Gagal", COLOR_ERROR, 64),
    };
    let right = match state.tab {
        AppTab::Timer => 496,
        AppTab::Macro => 1096,
        AppTab::Profiles => 876,
        AppTab::Settings => 796,
    };
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
            (
                RECT_TAB_PROFILES,
                "Profiles",
                HitTarget::ProfilesTab,
                state.tab == AppTab::Profiles,
            ),
            (
                RECT_TAB_SETTINGS,
                "Settings",
                HitTarget::SettingsTab,
                state.tab == AppTab::Settings,
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
        MacroEvent::MouseDownAt(button, _, _) => format!("{} ↓", mouse_label(*button)),
        MacroEvent::MouseUpAt(button, _, _) => format!("{} ↑", mouse_label(*button)),
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

fn lane_events_mut(item: &mut MacroDefinition, lane: MacroLane) -> &mut Vec<MacroEvent> {
    match lane {
        MacroLane::OnPress => &mut item.on_press,
        MacroLane::WhileHolding => &mut item.while_holding,
        MacroLane::OnRelease => &mut item.on_release,
    }
}

fn selected_delay(state: &AppState) -> Option<u32> {
    let index = state.macro_selected_event?;
    let item = state.macro_library.selected()?;
    match lane_events(item, state.macro_lane).get(index) {
        Some(MacroEvent::Delay(value)) => Some(*value),
        _ => None,
    }
}

unsafe fn sync_delay_edit(state: &AppState) {
    let delay = selected_delay(state);
    unsafe {
        if let Some(value) = delay {
            SetWindowTextW(state.macro_delay_edit, wide(&value.to_string()).as_ptr());
        }
        ShowWindow(
            state.macro_delay_edit,
            if state.tab == AppTab::Macro && !state.recording && delay.is_some() {
                SW_SHOWNORMAL
            } else {
                SW_HIDE
            },
        );
    }
}

unsafe fn set_selected_delay(state: &mut AppState, value: u32) {
    let Some(index) = state.macro_selected_event else {
        return;
    };
    let lane = state.macro_lane;
    let Some(item) = state.macro_library.selected_mut() else {
        return;
    };
    let Some(event) = lane_events_mut(item, lane).get_mut(index) else {
        return;
    };
    if !matches!(event, MacroEvent::Delay(_)) {
        return;
    }
    *event = MacroEvent::Delay(value.min(60_000));
    item.standard_delay_ms = None;
    state.macro_dirty = true;
    state.macro_status_kind = StatusKind::Ready;
    state.macro_status = format!("Delay langkah diubah menjadi {} ms.", value.min(60_000));
    unsafe {
        sync_delay_edit(state);
        InvalidateRect(state.window, null(), FALSE);
    }
}

unsafe fn apply_delay_edit(state: &mut AppState) {
    let text = unsafe { get_window_text(state.macro_delay_edit) };
    match text.trim().parse::<u32>() {
        Ok(value) if value <= 60_000 => unsafe { set_selected_delay(state, value) },
        _ => {
            state.macro_status_kind = StatusKind::Error;
            state.macro_status = "Delay harus berupa angka 0 sampai 60000 ms.".to_owned();
            unsafe {
                sync_delay_edit(state);
                InvalidateRect(state.window, null(), FALSE);
            }
        }
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
        if state.tab != AppTab::Timer {
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
        let stored_target = state
            .timer_library
            .selected()
            .and_then(|timer| timer.target.as_ref());
        let target_text = state
            .target
            .as_ref()
            .map(|target| target.title.as_str())
            .or_else(|| stored_target.map(|target| target.window_title.as_str()))
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
            } else if stored_target.is_some() {
                "Target tersimpan; diverifikasi ulang saat nol"
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
        draw_timer_sidebar(dc, state);
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

unsafe fn draw_timer_sidebar(dc: Hdc, state: &AppState) {
    unsafe {
        rounded_box(
            dc,
            Rect::new(508, 84, 888, 648),
            24,
            COLOR_PANEL,
            COLOR_PANEL_BORDER,
        );
        draw_label(
            dc,
            &format!(
                "MULTI TIMER  /  {} AKTIF",
                state.timer_library.running_count()
            ),
            Rect::new(520, 90, 876, 112),
            COLOR_ACCENT,
            state.fonts.small,
            DT_LEFT | DT_VCENTER | DT_SINGLELINE,
        );
        draw_flat_button(
            dc,
            RECT_TIMER_NEW,
            "+ Timer",
            COLOR_ACCENT,
            COLOR_INK,
            state.hot == HitTarget::TimerNew,
            state.fonts.small,
        );
        rounded_box(
            dc,
            Rect::new(654, 117, 876, 157),
            12,
            COLOR_PANEL_2,
            COLOR_PANEL_BORDER,
        );

        for (index, timer) in state.timer_library.timers.iter().enumerate() {
            let rect = timer_item_rect(index);
            let selected = timer.id == state.timer_library.selected_id;
            draw_flat_button(
                dc,
                rect,
                "",
                if selected {
                    COLOR_ACCENT
                } else {
                    COLOR_PANEL_2
                },
                if selected { COLOR_INK } else { COLOR_TEXT },
                state.hot == HitTarget::TimerItem(index),
                state.fonts.small,
            );
            draw_label(
                dc,
                &timer.name,
                Rect::new(
                    rect.left + 14,
                    rect.top + 4,
                    rect.right - 112,
                    rect.top + 27,
                ),
                if selected { COLOR_INK } else { COLOR_TEXT },
                state.fonts.semibold,
                DT_LEFT | DT_VCENTER | DT_SINGLELINE | DT_END_ELLIPSIS,
            );
            let detail = if timer.is_running() {
                format_duration(timer.remaining_seconds)
            } else {
                timer.phase.label().to_owned()
            };
            draw_label(
                dc,
                &detail,
                Rect::new(
                    rect.right - 106,
                    rect.top + 4,
                    rect.right - 14,
                    rect.top + 27,
                ),
                if timer.is_running() {
                    if selected { COLOR_INK } else { COLOR_ACCENT }
                } else if selected {
                    COLOR_INK_MUTED
                } else {
                    COLOR_MUTED
                },
                state.fonts.small,
                DT_RIGHT | DT_VCENTER | DT_SINGLELINE | DT_END_ELLIPSIS,
            );
            let target = timer
                .target
                .as_ref()
                .map(|target| target.executable.as_str())
                .unwrap_or("target belum dipilih");
            draw_label(
                dc,
                target,
                Rect::new(
                    rect.left + 14,
                    rect.top + 25,
                    rect.right - 14,
                    rect.bottom - 3,
                ),
                if selected {
                    COLOR_INK_MUTED
                } else {
                    COLOR_MUTED
                },
                state.fonts.small,
                DT_LEFT | DT_VCENTER | DT_SINGLELINE | DT_END_ELLIPSIS,
            );
        }

        for (rect, label, target, danger) in [
            (
                RECT_TIMER_DUPLICATE,
                "Salin",
                HitTarget::TimerDuplicate,
                false,
            ),
            (RECT_TIMER_DELETE, "Hapus", HitTarget::TimerDelete, true),
            (RECT_TIMER_SAVE, "Simpan", HitTarget::TimerSave, false),
        ] {
            draw_flat_button(
                dc,
                rect,
                label,
                COLOR_BG,
                if danger { COLOR_ERROR } else { COLOR_TEXT },
                state.hot == target,
                state.fonts.small,
            );
        }
        draw_label(
            dc,
            "SMART RESET CAPTURE",
            Rect::new(520, 552, 876, 572),
            COLOR_ACCENT,
            state.fonts.small,
            DT_LEFT | DT_VCENTER | DT_SINGLELINE,
        );
        draw_label(
            dc,
            "Paste teks reset, atau baca clipboard satu kali.",
            Rect::new(520, 568, 876, 584),
            COLOR_MUTED,
            state.fonts.small,
            DT_LEFT | DT_VCENTER | DT_SINGLELINE,
        );
        rounded_box(
            dc,
            Rect::new(520, 584, 876, 612),
            10,
            COLOR_PANEL_2,
            COLOR_PANEL_BORDER,
        );
        draw_flat_button(
            dc,
            RECT_SMART_CLIPBOARD,
            "Baca clipboard",
            COLOR_PANEL_2,
            COLOR_TEXT,
            state.hot == HitTarget::SmartResetClipboard,
            state.fonts.small,
        );
        draw_flat_button(
            dc,
            RECT_SMART_APPLY,
            "Terapkan",
            COLOR_ACCENT,
            COLOR_INK,
            state.hot == HitTarget::SmartResetApply,
            state.fonts.small,
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
        for (index, item) in state.macro_library.macros.iter().take(6).enumerate() {
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
        draw_flat_button(
            dc,
            RECT_MACRO_DUPLICATE,
            "Salin",
            COLOR_INK,
            COLOR_TEXT,
            state.hot == HitTarget::MacroDuplicate,
            state.fonts.small,
        );
        draw_flat_button(
            dc,
            RECT_MACRO_DELETE,
            "Hapus",
            COLOR_INK,
            COLOR_ERROR,
            state.hot == HitTarget::MacroDelete,
            state.fonts.small,
        );

        let editor_panel = Rect::new(252, 84, 1096, 608);
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
                let is_delay = matches!(event, MacroEvent::Delay(_));
                let rect = macro_event_rect(index);
                draw_flat_button(
                    dc,
                    rect,
                    &macro_event_label(event),
                    if is_delay {
                        COLOR_ACCENT
                    } else {
                        COLOR_SURFACE_2
                    },
                    if is_delay { COLOR_INK } else { COLOR_TEXT },
                    state.hot == HitTarget::MacroEvent(index)
                        || state.macro_selected_event == Some(index),
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
            RECT_MACRO_INSERT_DELAY,
            "+ Delay",
            COLOR_PANEL_2,
            COLOR_TEXT,
            state.hot == HitTarget::MacroInsertDelay,
            state.fonts.small,
        );

        rounded_box(
            dc,
            Rect::new(927, 121, 929, 596),
            2,
            COLOR_PANEL_BORDER,
            COLOR_PANEL_BORDER,
        );
        draw_label(
            dc,
            "Output macro",
            Rect::new(946, 126, 1080, 151),
            COLOR_MUTED,
            state.fonts.small,
            DT_LEFT | DT_VCENTER | DT_SINGLELINE,
        );
        let window_scoped = item.target.is_some();
        for (rect, label, active, target) in [
            (
                RECT_MACRO_SCOPE_GLOBAL,
                "Global",
                !window_scoped,
                HitTarget::MacroScopeGlobal,
            ),
            (
                RECT_MACRO_SCOPE_TARGET,
                "Window",
                window_scoped,
                HitTarget::MacroScopeTarget,
            ),
        ] {
            draw_flat_button(
                dc,
                rect,
                label,
                if active { COLOR_INK } else { COLOR_PANEL_2 },
                if active { COLOR_ACCENT } else { COLOR_TEXT },
                state.hot == target,
                state.fonts.small,
            );
        }
        draw_flat_button(
            dc,
            RECT_MACRO_TARGET_PICK,
            if window_scoped {
                "Ganti target"
            } else {
                "Pilih window"
            },
            if window_scoped {
                COLOR_ACCENT
            } else {
                COLOR_PANEL_2
            },
            if window_scoped { COLOR_INK } else { COLOR_TEXT },
            state.hot == HitTarget::MacroTargetPick,
            state.fonts.small,
        );
        let target_name = item
            .target
            .as_ref()
            .map(|target| target.window_title.as_str())
            .unwrap_or("Belum dibatasi");
        draw_label(
            dc,
            target_name,
            Rect::new(946, 264, 1080, 288),
            if window_scoped { COLOR_TEXT } else { COLOR_DIM },
            state.fonts.small,
            DT_LEFT | DT_VCENTER | DT_SINGLELINE | DT_END_ELLIPSIS,
        );
        draw_label(
            dc,
            if window_scoped {
                "Alt+Tab tetap aman"
            } else {
                "Mengikuti app aktif"
            },
            Rect::new(946, 288, 1080, 322),
            if window_scoped {
                COLOR_ACCENT
            } else {
                COLOR_DIM
            },
            state.fonts.small,
            DT_LEFT | DT_VCENTER | DT_SINGLELINE,
        );

        draw_hairline(dc, 946, 334, 1080, COLOR_PANEL_BORDER);
        draw_label(
            dc,
            "Delay langkah",
            Rect::new(946, 342, 1080, 366),
            COLOR_MUTED,
            state.fonts.small,
            DT_LEFT | DT_VCENTER | DT_SINGLELINE,
        );
        draw_label(
            dc,
            if state.macro_selected_event.is_some() {
                if selected_delay(state).is_some() {
                    "Edit delay terpilih"
                } else {
                    "Langkah terpilih"
                }
            } else {
                "Klik langkah timeline"
            },
            Rect::new(946, 364, 1080, 387),
            if state.macro_selected_event.is_some() {
                COLOR_TEXT
            } else {
                COLOR_DIM
            },
            state.fonts.small,
            DT_LEFT | DT_VCENTER | DT_SINGLELINE,
        );
        let delay_active = selected_delay(state).is_some();
        for (rect, label, target) in [
            (RECT_MACRO_DELAY_MINUS, "−", HitTarget::MacroDelayMinus),
            (RECT_MACRO_DELAY_PLUS, "+", HitTarget::MacroDelayPlus),
        ] {
            draw_flat_button(
                dc,
                rect,
                label,
                if delay_active {
                    COLOR_PANEL_2
                } else {
                    COLOR_BG
                },
                if delay_active { COLOR_TEXT } else { COLOR_DIM },
                delay_active && state.hot == target,
                state.fonts.semibold,
            );
        }
        rounded_box(
            dc,
            Rect::new(986, 393, 1040, 429),
            11,
            if delay_active {
                COLOR_PANEL_2
            } else {
                COLOR_BG
            },
            if delay_active {
                COLOR_PANEL_BORDER
            } else {
                COLOR_BG
            },
        );
        draw_flat_button(
            dc,
            RECT_MACRO_DELAY_APPLY,
            "Terapkan ms",
            if delay_active { COLOR_ACCENT } else { COLOR_BG },
            if delay_active { COLOR_INK } else { COLOR_DIM },
            delay_active && state.hot == HitTarget::MacroDelayApply,
            state.fonts.small,
        );
        let event_active = state.macro_selected_event.is_some();
        for (rect, label, target) in [
            (RECT_MACRO_EVENT_UP, "↑", HitTarget::MacroEventUp),
            (RECT_MACRO_EVENT_DOWN, "↓", HitTarget::MacroEventDown),
            (
                RECT_MACRO_EVENT_DUPLICATE,
                "Salin",
                HitTarget::MacroEventDuplicate,
            ),
            (
                RECT_MACRO_EVENT_DELETE,
                "Hapus",
                HitTarget::MacroEventDelete,
            ),
        ] {
            draw_flat_button(
                dc,
                rect,
                label,
                if event_active {
                    COLOR_PANEL_2
                } else {
                    COLOR_BG
                },
                if event_active { COLOR_TEXT } else { COLOR_DIM },
                event_active && state.hot == target,
                state.fonts.small,
            );
        }
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
            Rect::new(278, 610, 1090, 642),
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
        AppTab::Profiles => unsafe { draw_profiles_interface(dc, state) },
        AppTab::Settings => unsafe { draw_settings_interface(dc, state) },
    }
}

unsafe fn draw_profiles_interface(dc: Hdc, state: &AppState) {
    unsafe {
        fill_rect_color(
            dc,
            Rect::new(0, 0, PROFILES_CLIENT_WIDTH, CLIENT_HEIGHT),
            COLOR_BG,
        );
        draw_brand_header_v3(dc, state);

        rounded_box(
            dc,
            Rect::new(24, 84, 236, 608),
            24,
            COLOR_ACCENT,
            COLOR_ACCENT,
        );
        draw_label(
            dc,
            &format!("PROFILES  /  {:02}", state.profile_library.profiles.len()),
            Rect::new(42, 90, 218, 113),
            COLOR_INK_MUTED,
            state.fonts.small,
            DT_LEFT | DT_VCENTER | DT_SINGLELINE,
        );
        draw_flat_button(
            dc,
            RECT_PROFILE_NEW,
            "+  Profil baru",
            COLOR_INK,
            COLOR_TEXT,
            state.hot == HitTarget::ProfileNew,
            state.fonts.small,
        );
        for (index, profile) in state.profile_library.profiles.iter().take(6).enumerate() {
            let rect = profile_item_rect(index);
            let selected = profile.id == state.profile_library.selected_id;
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
                &profile.name,
                Rect::new(rect.left + 14, rect.top + 5, rect.right - 10, rect.top + 29),
                if selected { COLOR_TEXT } else { COLOR_INK },
                state.fonts.semibold,
                DT_LEFT | DT_VCENTER | DT_SINGLELINE | DT_END_ELLIPSIS,
            );
            draw_label(
                dc,
                &format!("{} macro", profile.macro_ids.len()),
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
        draw_flat_button(
            dc,
            RECT_PROFILE_DUPLICATE,
            "Salin",
            COLOR_INK,
            COLOR_TEXT,
            state.hot == HitTarget::ProfileDuplicate,
            state.fonts.small,
        );
        draw_flat_button(
            dc,
            RECT_PROFILE_DELETE,
            "Hapus",
            COLOR_INK,
            COLOR_ERROR,
            state.hot == HitTarget::ProfileDelete,
            state.fonts.small,
        );

        rounded_box(
            dc,
            Rect::new(252, 84, 876, 608),
            24,
            COLOR_PANEL,
            COLOR_PANEL_BORDER,
        );
        draw_label(
            dc,
            "App Profile",
            Rect::new(278, 96, 520, 120),
            COLOR_MUTED,
            state.fonts.small,
            DT_LEFT | DT_VCENTER | DT_SINGLELINE,
        );
        let Some(profile) = state.profile_library.selected() else {
            return;
        };
        rounded_box(
            dc,
            Rect::new(270, 121, 858, 164),
            13,
            COLOR_PANEL_2,
            COLOR_PANEL_BORDER,
        );
        draw_label(
            dc,
            "TARGET APLIKASI",
            Rect::new(278, 181, 520, 205),
            COLOR_ACCENT,
            state.fonts.small,
            DT_LEFT | DT_VCENTER | DT_SINGLELINE,
        );
        let target_title = profile
            .target
            .as_ref()
            .map(|target| target.window_title.as_str())
            .unwrap_or("Belum memilih target");
        let target_executable = profile
            .target
            .as_ref()
            .map(|target| target.executable.as_str())
            .unwrap_or("Pilih window aplikasi atau game");
        draw_label(
            dc,
            target_title,
            Rect::new(278, 203, 850, 226),
            COLOR_TEXT,
            state.fonts.semibold,
            DT_LEFT | DT_VCENTER | DT_SINGLELINE | DT_END_ELLIPSIS,
        );
        draw_label(
            dc,
            target_executable,
            Rect::new(278, 224, 850, 243),
            if profile.target.is_some() {
                COLOR_MUTED
            } else {
                COLOR_DIM
            },
            state.fonts.small,
            DT_LEFT | DT_VCENTER | DT_SINGLELINE | DT_END_ELLIPSIS,
        );
        draw_flat_button(
            dc,
            RECT_PROFILE_USE_TIMER,
            "Gunakan target untuk Timer",
            COLOR_PANEL_2,
            COLOR_TEXT,
            state.hot == HitTarget::ProfileUseTimer,
            state.fonts.small,
        );
        draw_flat_button(
            dc,
            RECT_PROFILE_TARGET_PICK,
            if profile.target.is_some() {
                "Ganti target window"
            } else {
                "Pilih target window"
            },
            if profile.target.is_some() {
                COLOR_ACCENT
            } else {
                COLOR_PANEL_2
            },
            if profile.target.is_some() {
                COLOR_INK
            } else {
                COLOR_TEXT
            },
            state.hot == HitTarget::ProfileTargetPick,
            state.fonts.small,
        );

        draw_label(
            dc,
            "MACRO DALAM PROFIL",
            Rect::new(278, 302, 620, 330),
            COLOR_MUTED,
            state.fonts.small,
            DT_LEFT | DT_VCENTER | DT_SINGLELINE,
        );
        draw_label(
            dc,
            "Macro tertaut otomatis memakai target profil",
            Rect::new(520, 302, 850, 330),
            COLOR_DIM,
            state.fonts.small,
            DT_RIGHT | DT_VCENTER | DT_SINGLELINE,
        );
        for (index, item) in state.macro_library.macros.iter().take(6).enumerate() {
            let linked = profile.contains_macro(item.id);
            let rect = profile_macro_rect(index);
            draw_flat_button(
                dc,
                rect,
                &format!(
                    "{}    {}",
                    item.name,
                    if linked { "TERTAUT" } else { "+ TAUTKAN" }
                ),
                if linked {
                    COLOR_ACCENT_DARK
                } else {
                    COLOR_PANEL_2
                },
                if linked { COLOR_ACCENT } else { COLOR_TEXT },
                state.hot == HitTarget::ProfileMacro(index),
                state.fonts.small,
            );
        }
        draw_flat_button(
            dc,
            RECT_PROFILE_EXPORT,
            "Export backup",
            COLOR_PANEL_2,
            COLOR_TEXT,
            state.hot == HitTarget::BackupExport,
            state.fonts.small,
        );
        draw_flat_button(
            dc,
            RECT_PROFILE_IMPORT,
            "Import backup",
            COLOR_PANEL_2,
            COLOR_TEXT,
            state.hot == HitTarget::BackupImport,
            state.fonts.small,
        );
        draw_flat_button(
            dc,
            RECT_PROFILE_SAVE,
            if state.profile_dirty {
                "Simpan profil"
            } else {
                "Tersimpan"
            },
            COLOR_INK,
            COLOR_TEXT,
            state.hot == HitTarget::ProfileSave,
            state.fonts.semibold,
        );

        let status_color = match state.profile_status_kind {
            StatusKind::Ready => COLOR_MUTED,
            StatusKind::Running => COLOR_ACCENT,
            StatusKind::Sent => COLOR_SUCCESS,
            StatusKind::Warning => COLOR_WARNING,
            StatusKind::Error => COLOR_ERROR,
        };
        filled_circle(dc, Rect::new(260, 622, 268, 630), status_color);
        draw_label(
            dc,
            &state.profile_status,
            Rect::new(278, 610, 870, 642),
            COLOR_MUTED,
            state.fonts.small,
            DT_LEFT | DT_VCENTER | DT_SINGLELINE | DT_END_ELLIPSIS,
        );
    }
}

unsafe fn draw_settings_interface(dc: Hdc, state: &AppState) {
    unsafe {
        fill_rect_color(
            dc,
            Rect::new(0, 0, SETTINGS_CLIENT_WIDTH, CLIENT_HEIGHT),
            COLOR_BG,
        );
        draw_brand_header_v3(dc, state);
        draw_label(
            dc,
            "Kontrol & keselamatan",
            Rect::new(24, 78, 500, 112),
            COLOR_TEXT,
            state.fonts.title,
            DT_LEFT | DT_VCENTER | DT_SINGLELINE,
        );
        draw_label(
            dc,
            "Semua perubahan disimpan otomatis. Hotkey darurat selalu menghentikan macro.",
            Rect::new(24, 107, 790, 132),
            COLOR_MUTED,
            state.fonts.small,
            DT_LEFT | DT_VCENTER | DT_SINGLELINE,
        );

        rounded_box(
            dc,
            Rect::new(24, 136, 400, 536),
            24,
            COLOR_PANEL,
            COLOR_PANEL_BORDER,
        );
        draw_label(
            dc,
            "TRAY & STARTUP",
            Rect::new(42, 146, 382, 170),
            COLOR_ACCENT,
            state.fonts.small,
            DT_LEFT | DT_VCENTER | DT_SINGLELINE,
        );
        for (rect, label, active, target) in [
            (
                RECT_SETTING_MINIMIZE_TRAY,
                "Minimize ke system tray",
                state.settings.minimize_to_tray,
                HitTarget::SettingMinimizeTray,
            ),
            (
                RECT_SETTING_CLOSE_TRAY,
                "Tombol X tetap di tray",
                state.settings.close_to_tray,
                HitTarget::SettingCloseTray,
            ),
            (
                RECT_SETTING_AUTO_START,
                "Mulai bersama Windows",
                state.settings.auto_start,
                HitTarget::SettingAutoStart,
            ),
        ] {
            draw_flat_button(
                dc,
                rect,
                &format!("{}    {}", label, if active { "AKTIF" } else { "NONAKTIF" }),
                if active {
                    COLOR_ACCENT_DARK
                } else {
                    COLOR_PANEL_2
                },
                if active { COLOR_ACCENT } else { COLOR_TEXT },
                state.hot == target,
                state.fonts.small,
            );
        }
        draw_hairline(dc, 42, 370, 382, COLOR_PANEL_BORDER);
        draw_label(
            dc,
            "Tray menjaga timer dan macro tetap berjalan saat jendela disembunyikan.",
            Rect::new(42, 390, 382, 438),
            COLOR_MUTED,
            state.fonts.body,
            DT_LEFT | DT_WORDBREAK,
        );
        draw_label(
            dc,
            "Auto Start memakai registry Current User—tidak memerlukan Administrator.",
            Rect::new(42, 456, 382, 510),
            COLOR_DIM,
            state.fonts.small,
            DT_LEFT | DT_WORDBREAK,
        );

        rounded_box(
            dc,
            Rect::new(416, 136, 796, 536),
            24,
            COLOR_PANEL,
            COLOR_PANEL_BORDER,
        );
        draw_label(
            dc,
            "EMERGENCY STOP",
            Rect::new(438, 146, 772, 170),
            COLOR_ERROR,
            state.fonts.small,
            DT_LEFT | DT_VCENTER | DT_SINGLELINE,
        );
        for (index, hotkey) in EmergencyHotkey::ALL.into_iter().enumerate() {
            let rect = [
                RECT_SETTING_HOTKEY_1,
                RECT_SETTING_HOTKEY_2,
                RECT_SETTING_HOTKEY_3,
            ][index];
            let active = state.settings.emergency_hotkey == hotkey;
            draw_flat_button(
                dc,
                rect,
                hotkey.label(),
                if active { COLOR_ACCENT } else { COLOR_PANEL_2 },
                if active { COLOR_INK } else { COLOR_TEXT },
                state.hot == HitTarget::SettingEmergencyHotkey(hotkey),
                state.fonts.small,
            );
        }
        draw_flat_button(
            dc,
            RECT_SETTING_STOP_TIMERS,
            if state.settings.emergency_stops_timers {
                "Emergency juga membatalkan semua timer"
            } else {
                "Emergency hanya menghentikan macro"
            },
            if state.settings.emergency_stops_timers {
                COLOR_ACCENT_DARK
            } else {
                COLOR_PANEL_2
            },
            if state.settings.emergency_stops_timers {
                COLOR_ACCENT
            } else {
                COLOR_TEXT
            },
            state.hot == HitTarget::SettingEmergencyTimers,
            state.fonts.small,
        );
        draw_label(
            dc,
            "Batas durasi Toggle",
            Rect::new(438, 382, 772, 407),
            COLOR_MUTED,
            state.fonts.small,
            DT_LEFT | DT_VCENTER | DT_SINGLELINE,
        );
        for (rect, label, value) in [
            (RECT_SETTING_RUNTIME_5, "5 mnt", 5 * 60),
            (RECT_SETTING_RUNTIME_30, "30 mnt", 30 * 60),
            (RECT_SETTING_RUNTIME_60, "60 mnt", 60 * 60),
            (RECT_SETTING_RUNTIME_OFF, "Tanpa", 0),
        ] {
            let active = state.settings.max_macro_runtime_seconds == value;
            draw_flat_button(
                dc,
                rect,
                label,
                if active { COLOR_INK } else { COLOR_PANEL_2 },
                if active { COLOR_ACCENT } else { COLOR_TEXT },
                state.hot == HitTarget::SettingMaxRuntime(value),
                state.fonts.small,
            );
        }
        draw_label(
            dc,
            "Batas pengulangan",
            Rect::new(438, 457, 772, 486),
            COLOR_MUTED,
            state.fonts.small,
            DT_LEFT | DT_VCENTER | DT_SINGLELINE,
        );
        for (rect, label, value) in [
            (RECT_SETTING_REPEAT_100, "100×", 100),
            (RECT_SETTING_REPEAT_1000, "1K×", 1_000),
            (RECT_SETTING_REPEAT_10000, "10K×", 10_000),
            (RECT_SETTING_REPEAT_OFF, "Tanpa", 0),
        ] {
            let active = state.settings.max_macro_repeats == value;
            draw_flat_button(
                dc,
                rect,
                label,
                if active { COLOR_INK } else { COLOR_PANEL_2 },
                if active { COLOR_ACCENT } else { COLOR_TEXT },
                state.hot == HitTarget::SettingMaxRepeats(value),
                state.fonts.small,
            );
        }

        draw_flat_button(
            dc,
            RECT_SETTING_TEST_EMERGENCY,
            "Hentikan semua sekarang",
            COLOR_ERROR,
            COLOR_TEXT,
            state.hot == HitTarget::SettingTestEmergency,
            state.fonts.semibold,
        );
        let status_color = match state.settings_status_kind {
            StatusKind::Ready => COLOR_MUTED,
            StatusKind::Running => COLOR_ACCENT,
            StatusKind::Sent => COLOR_SUCCESS,
            StatusKind::Warning => COLOR_WARNING,
            StatusKind::Error => COLOR_ERROR,
        };
        filled_circle(dc, Rect::new(28, 622, 36, 630), status_color);
        draw_label(
            dc,
            &state.settings_status,
            Rect::new(46, 610, 792, 642),
            COLOR_MUTED,
            state.fonts.small,
            DT_LEFT | DT_VCENTER | DT_SINGLELINE | DT_END_ELLIPSIS,
        );
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
        state.macro_delay_edit = create_edit(
            state.window,
            instance,
            EditSpec {
                id: 106,
                text: "0",
                bounds: Rect::new(990, 399, 1036, 423),
                style: ES_CENTER | ES_NUMBER,
            },
        );
        ShowWindow(state.macro_delay_edit, SW_HIDE);
        let initial_profile_name = state
            .profile_library
            .selected()
            .map(|profile| profile.name.clone())
            .unwrap_or_else(|| "Profil".to_owned());
        state.profile_name_edit = create_edit(
            state.window,
            instance,
            EditSpec {
                id: 107,
                text: &initial_profile_name,
                bounds: Rect::new(278, 130, 850, 158),
                style: 0,
            },
        );
        ShowWindow(state.profile_name_edit, SW_HIDE);
        let initial_timer_name = state
            .timer_library
            .selected()
            .map(|timer| timer.name.clone())
            .unwrap_or_else(|| "Timer".to_owned());
        state.timer_name_edit = create_edit(
            state.window,
            instance,
            EditSpec {
                id: 108,
                text: &initial_timer_name,
                bounds: Rect::new(662, 125, 868, 149),
                style: 0,
            },
        );
        state.smart_reset_edit = create_edit(
            state.window,
            instance,
            EditSpec {
                id: 109,
                text: "Resets in 3 h 27 min",
                bounds: Rect::new(528, 587, 868, 607),
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
            SendMessageW(
                edit,
                EM_SETLIMITTEXT,
                if edit == state.hour_edit { 3 } else { 2 },
                0,
            );
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
        SendMessageW(
            state.macro_delay_edit,
            WM_SETFONT,
            state.fonts.small as Wparam,
            TRUE as Lparam,
        );
        SendMessageW(state.macro_delay_edit, EM_SETLIMITTEXT, 5, 0);
        SendMessageW(
            state.macro_delay_edit,
            EM_SETMARGINS,
            EC_LEFTMARGIN | EC_RIGHTMARGIN,
            (4 | (4 << 16)) as Lparam,
        );
        SetWindowTheme(state.macro_delay_edit, dark.as_ptr(), null());
        SendMessageW(
            state.profile_name_edit,
            WM_SETFONT,
            state.fonts.semibold as Wparam,
            TRUE as Lparam,
        );
        SendMessageW(state.profile_name_edit, EM_SETLIMITTEXT, 80, 0);
        SendMessageW(
            state.profile_name_edit,
            EM_SETMARGINS,
            EC_LEFTMARGIN | EC_RIGHTMARGIN,
            (8 | (8 << 16)) as Lparam,
        );
        SetWindowTheme(state.profile_name_edit, dark.as_ptr(), null());
        for edit in [state.timer_name_edit, state.smart_reset_edit] {
            SendMessageW(
                edit,
                WM_SETFONT,
                state.fonts.small as Wparam,
                TRUE as Lparam,
            );
            SendMessageW(edit, EM_SETLIMITTEXT, 512, 0);
            SendMessageW(
                edit,
                EM_SETMARGINS,
                EC_LEFTMARGIN | EC_RIGHTMARGIN,
                (8 | (8 << 16)) as Lparam,
            );
            SetWindowTheme(edit, dark.as_ptr(), null());
        }
        sync_selected_timer_to_controls(state);
    }
}

fn hotkey_parts(hotkey: EmergencyHotkey) -> (Uint, Uint) {
    match hotkey {
        EmergencyHotkey::CtrlAltF12 => (MOD_CONTROL | MOD_ALT | MOD_NOREPEAT, VK_F12),
        EmergencyHotkey::CtrlShiftF12 => (MOD_CONTROL | MOD_SHIFT | MOD_NOREPEAT, VK_F12),
        EmergencyHotkey::CtrlAltPause => (MOD_CONTROL | MOD_ALT | MOD_NOREPEAT, VK_PAUSE),
    }
}

unsafe fn register_emergency_hotkey(state: &mut AppState) -> Result<(), String> {
    unsafe { UnregisterHotKey(state.window, EMERGENCY_HOTKEY_ID) };
    let (modifiers, key) = hotkey_parts(state.settings.emergency_hotkey);
    if unsafe { RegisterHotKey(state.window, EMERGENCY_HOTKEY_ID, modifiers, key) } == FALSE {
        return Err(format!(
            "Hotkey {} sedang dipakai aplikasi lain.",
            state.settings.emergency_hotkey.label()
        ));
    }
    Ok(())
}

#[cfg(test)]
unsafe fn configure_auto_start(enabled: bool) -> Result<(), String> {
    TEST_AUTOSTART_ENABLED.store(enabled, Ordering::Release);
    Ok(())
}

#[cfg(not(test))]
unsafe fn configure_auto_start(enabled: bool) -> Result<(), String> {
    let key = wide("Software\\Microsoft\\Windows\\CurrentVersion\\Run");
    let value_name = wide("VibeTimer");
    let status = if enabled {
        let executable = std::env::current_exe()
            .map_err(|error| format!("Lokasi VibeTimer tidak ditemukan: {error}"))?;
        let command = format!("\"{}\" --background", executable.display());
        let data = wide(&command);
        unsafe {
            RegSetKeyValueW(
                HKEY_CURRENT_USER,
                key.as_ptr(),
                value_name.as_ptr(),
                REG_SZ,
                data.as_ptr().cast(),
                (data.len() * size_of::<u16>()) as Dword,
            )
        }
    } else {
        unsafe { RegDeleteKeyValueW(HKEY_CURRENT_USER, key.as_ptr(), value_name.as_ptr()) }
    };
    if status == 0 || (!enabled && status == 2) {
        Ok(())
    } else {
        Err(format!(
            "Windows menolak perubahan Auto Start (kode {status})."
        ))
    }
}

unsafe fn persist_settings(state: &mut AppState, message: &str) -> bool {
    match save_settings(&state.settings_path, &state.settings) {
        Ok(()) => {
            state.settings_status_kind = StatusKind::Sent;
            state.settings_status = message.to_owned();
            unsafe { InvalidateRect(state.window, null(), FALSE) };
            true
        }
        Err(error) => {
            state.settings_status_kind = StatusKind::Error;
            state.settings_status = format!("Pengaturan gagal disimpan: {error}");
            unsafe { InvalidateRect(state.window, null(), FALSE) };
            false
        }
    }
}

unsafe fn add_tray_icon(state: &mut AppState) -> Result<(), String> {
    if state.tray_added {
        return Ok(());
    }
    state.tray_icon = unsafe { create_app_icon(GetModuleHandleW(null()), 16) };
    let mut data = NotifyIconDataW {
        size: size_of::<NotifyIconDataW>() as Dword,
        window: state.window,
        id: TRAY_ICON_ID,
        flags: NIF_MESSAGE | NIF_ICON | NIF_TIP,
        callback_message: WM_APP_TRAY,
        icon: state.tray_icon,
        ..NotifyIconDataW::default()
    };
    copy_wide(&mut data.tip, "VibeTimer — siap");
    if unsafe { Shell_NotifyIconW(NIM_ADD, &mut data) } == FALSE {
        if state.tray_icon != 0 {
            unsafe { DestroyIcon(state.tray_icon) };
            state.tray_icon = 0;
        }
        return Err("System tray Windows menolak ikon VibeTimer.".to_owned());
    }
    state.tray_added = true;
    Ok(())
}

unsafe fn remove_tray_icon(state: &mut AppState) {
    if state.tray_added {
        let mut data = NotifyIconDataW {
            size: size_of::<NotifyIconDataW>() as Dword,
            window: state.window,
            id: TRAY_ICON_ID,
            ..NotifyIconDataW::default()
        };
        unsafe { Shell_NotifyIconW(NIM_DELETE, &mut data) };
        state.tray_added = false;
    }
    if state.tray_icon != 0 {
        unsafe { DestroyIcon(state.tray_icon) };
        state.tray_icon = 0;
    }
}

unsafe fn show_tray_notification(state: &AppState, title: &str, message: &str) {
    if !state.tray_added {
        return;
    }
    let mut data = NotifyIconDataW {
        size: size_of::<NotifyIconDataW>() as Dword,
        window: state.window,
        id: TRAY_ICON_ID,
        flags: NIF_INFO,
        info_flags: NIIF_INFO,
        timeout_or_version: 5_000,
        ..NotifyIconDataW::default()
    };
    copy_wide(&mut data.info_title, title);
    copy_wide(&mut data.info, message);
    unsafe { Shell_NotifyIconW(NIM_MODIFY, &mut data) };
}

unsafe fn restore_from_tray(state: &AppState) {
    unsafe {
        ShowWindow(state.window, SW_RESTORE);
        SetForegroundWindow(state.window);
    }
}

unsafe fn show_tray_menu(state: &AppState) {
    let menu = unsafe { CreatePopupMenu() };
    if menu == 0 {
        return;
    }
    unsafe {
        AppendMenuW(menu, MF_STRING, MENU_OPEN, wide("Buka VibeTimer").as_ptr());
        AppendMenuW(
            menu,
            MF_STRING,
            MENU_STOP_ALL,
            wide("Emergency Stop").as_ptr(),
        );
        AppendMenuW(menu, MF_SEPARATOR, 0, null());
        AppendMenuW(menu, MF_STRING, MENU_EXIT, wide("Keluar").as_ptr());
        let mut point = Point::default();
        GetCursorPos(&mut point);
        SetForegroundWindow(state.window);
        TrackPopupMenu(
            menu,
            TPM_RIGHTBUTTON | TPM_BOTTOMALIGN,
            point.x,
            point.y,
            0,
            state.window,
            null(),
        );
        DestroyMenu(menu);
    }
}

unsafe fn choose_backup_path(window: Hwnd, save: bool) -> Option<PathBuf> {
    let mut file = [0u16; 32_768];
    if save {
        copy_wide(&mut file, "VibeTimer-backup.vtb");
    }
    let filter = wide("VibeTimer Backup (*.vtb)\0*.vtb\0Semua file (*.*)\0*.*\0");
    let title = wide(if save {
        "Export backup VibeTimer"
    } else {
        "Import backup VibeTimer"
    });
    let extension = wide("vtb");
    let mut data = OpenFileNameW {
        size: size_of::<OpenFileNameW>() as Dword,
        owner: window,
        instance: 0,
        filter: filter.as_ptr(),
        custom_filter: null_mut(),
        max_custom_filter: 0,
        filter_index: 1,
        file: file.as_mut_ptr(),
        max_file: file.len() as Dword,
        file_title: null_mut(),
        max_file_title: 0,
        initial_directory: null(),
        title: title.as_ptr(),
        flags: OFN_EXPLORER
            | OFN_PATHMUSTEXIST
            | if save {
                OFN_OVERWRITEPROMPT
            } else {
                OFN_FILEMUSTEXIST
            },
        file_offset: 0,
        file_extension: 0,
        default_extension: extension.as_ptr(),
        custom_data: 0,
        hook: null(),
        template_name: null(),
        reserved: null_mut(),
        reserved_value: 0,
        flags_ex: 0,
    };
    let accepted = if save {
        unsafe { GetSaveFileNameW(&mut data) }
    } else {
        unsafe { GetOpenFileNameW(&mut data) }
    };
    if accepted == FALSE {
        return None;
    }
    let length = file
        .iter()
        .position(|value| *value == 0)
        .unwrap_or(file.len());
    Some(PathBuf::from(String::from_utf16_lossy(&file[..length])))
}

fn synchronize_all_profile_targets(profiles: &ProfileLibrary, macros: &mut MacroLibrary) {
    for profile in &profiles.profiles {
        let Some(target) = profile.target.as_ref() else {
            continue;
        };
        for item in &mut macros.macros {
            if profile.contains_macro(item.id) {
                item.target = Some(target.clone());
            }
        }
    }
}

fn save_bundle_files(
    macro_path: &Path,
    profiles_path: &Path,
    settings_path: &Path,
    timers_path: &Path,
    bundle: &BackupBundle,
) -> Result<(), String> {
    save_library(macro_path, &bundle.macros)
        .and_then(|_| save_profiles(profiles_path, &bundle.profiles))
        .and_then(|_| save_settings(settings_path, &bundle.settings))
        .and_then(|_| save_timers(timers_path, &bundle.timers))
        .map_err(|error| error.to_string())
}

unsafe fn export_backup_to_path(state: &mut AppState, path: &Path) {
    let bundle = BackupBundle::with_timers(
        state.macro_library.clone(),
        state.profile_library.clone(),
        state.settings.clone(),
        state.timer_library.clone(),
    );
    match save_backup(path, &bundle) {
        Ok(()) => {
            state.profile_status_kind = StatusKind::Sent;
            state.profile_status = format!(
                "Backup tersimpan: {}",
                path.file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or("VibeTimer-backup.vtb")
            );
        }
        Err(error) => {
            state.profile_status_kind = StatusKind::Error;
            state.profile_status = format!("Export backup gagal: {error}");
        }
    }
    unsafe { InvalidateRect(state.window, null(), FALSE) };
}

unsafe fn import_backup_from_path(state: &mut AppState, path: &Path) {
    let mut bundle = match load_backup(path) {
        Ok(bundle) => bundle,
        Err(error) => {
            state.profile_status_kind = StatusKind::Error;
            state.profile_status = format!("Import ditolak: {error}");
            unsafe { InvalidateRect(state.window, null(), FALSE) };
            return;
        }
    };
    let available_ids: Vec<u32> = bundle.macros.macros.iter().map(|item| item.id).collect();
    bundle.profiles.remove_missing_macro_links(&available_ids);
    synchronize_all_profile_targets(&bundle.profiles, &mut bundle.macros);
    // Import tidak pernah meneruskan timer aktif dari komputer/sesi lain.
    bundle.timers.cancel_all();
    let previous = BackupBundle::with_timers(
        state.macro_library.clone(),
        state.profile_library.clone(),
        state.settings.clone(),
        state.timer_library.clone(),
    );
    if let Err(error) = save_bundle_files(
        &state.macro_path,
        &state.profiles_path,
        &state.settings_path,
        &state.timers_path,
        &bundle,
    ) {
        let rollback = save_bundle_files(
            &state.macro_path,
            &state.profiles_path,
            &state.settings_path,
            &state.timers_path,
            &previous,
        );
        state.profile_status_kind = StatusKind::Error;
        state.profile_status = match rollback {
            Ok(()) => format!("Import gagal dan state lama dipulihkan: {error}"),
            Err(rollback_error) => format!(
                "Import gagal ({error}); pemulihan state juga gagal ({rollback_error}). Gunakan backup manual."
            ),
        };
        unsafe { InvalidateRect(state.window, null(), FALSE) };
        return;
    }
    state.macro_library = bundle.macros;
    state.profile_library = bundle.profiles;
    state.settings = bundle.settings;
    state.timer_library = bundle.timers;
    state.macro_targets.clear();
    state.profile_targets.clear();
    state.timer_targets.clear();
    state.macro_dirty = false;
    state.profile_dirty = false;
    state.timer_dirty = false;
    let _ = unsafe { configure_auto_start(state.settings.auto_start) };
    if let Err(message) = unsafe { register_emergency_hotkey(state) } {
        state.settings_status_kind = StatusKind::Warning;
        state.settings_status = message;
    }
    unsafe {
        sync_macro_name_edit(state);
        sync_profile_name_edit(state);
        sync_delay_edit(state);
        refresh_macro_hooks(state);
        KillTimer(state.window, TIMER_COUNTDOWN);
        sync_selected_timer_to_controls(state);
    }
    state.profile_status_kind = StatusKind::Sent;
    state.profile_status = format!(
        "Backup {} berhasil diimpor dan divalidasi.",
        path.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("VibeTimer")
    );
    unsafe { InvalidateRect(state.window, null(), FALSE) };
}

unsafe fn emergency_stop(state: &mut AppState, source: &str) {
    if state.recording {
        unsafe { stop_macro_recording(state) };
    }
    if MACRO_PLAYING.load(Ordering::Acquire) {
        EMERGENCY_ACTIVE.store(true, Ordering::Release);
    }
    MACRO_STOP.store(true, Ordering::Release);
    TRIGGER_HELD.store(false, Ordering::Release);
    state.trigger_down = false;
    state.trigger_macro_id = None;
    if state.settings.emergency_stops_timers && state.timer_library.running_count() > 0 {
        state.timer_library.cancel_all();
        let _ = save_timers(&state.timers_path, &state.timer_library);
        unsafe {
            KillTimer(state.window, TIMER_COUNTDOWN);
            sync_selected_timer_to_controls(state);
        }
        state.status_kind = StatusKind::Warning;
        state.status = "Semua timer dibatalkan oleh Emergency Stop.".to_owned();
    }
    state.macro_status_kind = StatusKind::Warning;
    state.macro_status = format!("Emergency Stop dari {source}: semua macro dihentikan.");
    state.settings_status_kind = StatusKind::Warning;
    state.settings_status = format!("Emergency Stop dijalankan dari {source}.");
    unsafe {
        show_tray_notification(
            state,
            "VibeTimer dihentikan",
            "Semua macro berhenti. Timer mengikuti pengaturan Emergency Stop.",
        );
        MessageBeep(MB_ICONWARNING);
        InvalidateRect(state.window, null(), FALSE);
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

unsafe fn process_executable_name(process_id: Dword) -> Result<String, String> {
    let process = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, FALSE, process_id) };
    if process == 0 {
        return Err("Proses target tidak dapat dibaca.".to_owned());
    }
    let mut buffer = vec![0u16; 32_768];
    let mut length = buffer.len() as Dword;
    let queried =
        unsafe { QueryFullProcessImageNameW(process, 0, buffer.as_mut_ptr(), &mut length) };
    unsafe { CloseHandle(process) };
    if queried == FALSE || length == 0 {
        return Err("Nama executable target tidak dapat dibaca.".to_owned());
    }
    let path = String::from_utf16_lossy(&buffer[..length as usize]);
    Ok(PathBuf::from(path)
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default())
}

struct MacroTargetSearch<'a> {
    specification: &'a MacroTarget,
    found: Hwnd,
}

unsafe extern "system" fn find_macro_target_callback(window: Hwnd, lparam: Lparam) -> Bool {
    let search = unsafe { &mut *(lparam as *mut MacroTargetSearch<'_>) };
    if unsafe { IsWindowVisible(window) } == FALSE {
        return TRUE;
    }
    let title = unsafe { get_window_text(window) };
    if !title.eq_ignore_ascii_case(&search.specification.window_title) {
        return TRUE;
    }
    let mut process_id = 0;
    unsafe { GetWindowThreadProcessId(window, &mut process_id) };
    let executable = unsafe { process_executable_name(process_id) };
    if executable
        .as_deref()
        .is_ok_and(|value| value.eq_ignore_ascii_case(&search.specification.executable))
    {
        search.found = window;
        FALSE
    } else {
        TRUE
    }
}

unsafe fn find_saved_macro_target(
    specification: &MacroTarget,
) -> Result<MacroPlaybackTarget, String> {
    let mut search = MacroTargetSearch {
        specification,
        found: 0,
    };
    unsafe {
        EnumWindows(
            Some(find_macro_target_callback),
            &mut search as *mut _ as Lparam,
        );
    }
    if search.found == 0 {
        return Err(format!(
            "Target {} belum terbuka. Buka aplikasinya atau pilih ulang window.",
            specification.executable
        ));
    }
    let mut process_id = 0;
    unsafe { GetWindowThreadProcessId(search.found, &mut process_id) };
    Ok(MacroPlaybackTarget {
        root: search.found,
        receiver: search.found,
        process_id,
        title: specification.window_title.clone(),
    })
}

unsafe fn validate_macro_playback_target(target: &MacroPlaybackTarget) -> Result<(), String> {
    unsafe {
        if IsWindow(target.root) == FALSE || IsWindow(target.receiver) == FALSE {
            return Err("Window target macro sudah ditutup.".to_owned());
        }
        let mut root_pid = 0;
        let mut receiver_pid = 0;
        GetWindowThreadProcessId(target.root, &mut root_pid);
        GetWindowThreadProcessId(target.receiver, &mut receiver_pid);
        if root_pid == 0 || root_pid != target.process_id || receiver_pid != target.process_id {
            return Err("Window target macro sudah berganti proses.".to_owned());
        }
    }
    Ok(())
}

unsafe fn resolve_playback_destination(
    state: &mut AppState,
    item: &MacroDefinition,
) -> Result<PlaybackDestination, String> {
    let Some(specification) = item.target.as_ref() else {
        return Ok(PlaybackDestination::Foreground);
    };
    if let Some(target) = state.macro_targets.get(&item.id)
        && unsafe { validate_macro_playback_target(target) }.is_ok()
    {
        return Ok(PlaybackDestination::Window(target.clone()));
    }
    state.macro_targets.remove(&item.id);
    let target = unsafe { find_saved_macro_target(specification) }?;
    state.macro_targets.insert(item.id, target.clone());
    Ok(PlaybackDestination::Window(target))
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
    state.capture_kind = CaptureKind::Timer;
    state.capture_deadline = Some(Instant::now() + Duration::from_secs(3));
    unsafe {
        InvalidateRect(state.window, null(), FALSE);
        ShowWindow(state.window, SW_MINIMIZE);
        SetTimer(state.window, TIMER_CAPTURE, 100, null());
    }
}

unsafe fn begin_macro_target_capture(state: &mut AppState) {
    if state.recording || state.macro_library.selected().is_none() {
        return;
    }
    let instruction = "Setelah menekan OK, VibeTimer mengecil selama 3 detik.\n\nKlik tepat pada area game atau aplikasi yang harus menerima macro. Posisi klik saat recording akan disimpan relatif ke window ini.";
    let response = unsafe {
        MessageBoxW(
            state.window,
            wide(instruction).as_ptr(),
            wide("Pasang target macro").as_ptr(),
            MB_OKCANCEL | MB_ICONINFORMATION,
        )
    };
    if response != IDOK {
        return;
    }
    state.capture_kind = CaptureKind::Macro;
    state.capture_deadline = Some(Instant::now() + Duration::from_secs(3));
    state.macro_status_kind = StatusKind::Warning;
    state.macro_status = "Klik area target macro dalam 3 detik…".to_owned();
    unsafe {
        InvalidateRect(state.window, null(), FALSE);
        ShowWindow(state.window, SW_MINIMIZE);
        SetTimer(state.window, TIMER_CAPTURE, 100, null());
    }
}

unsafe fn begin_profile_target_capture(state: &mut AppState) {
    if state.profile_library.selected().is_none() {
        return;
    }
    let instruction = "Setelah menekan OK, VibeTimer mengecil selama 3 detik.\n\nKlik area aplikasi atau game untuk profil ini. Semua macro yang ditautkan akan memakai target yang sama.";
    let response = unsafe {
        MessageBoxW(
            state.window,
            wide(instruction).as_ptr(),
            wide("Pilih target App Profile").as_ptr(),
            MB_OKCANCEL | MB_ICONINFORMATION,
        )
    };
    if response != IDOK {
        return;
    }
    state.capture_kind = CaptureKind::Profile;
    state.capture_deadline = Some(Instant::now() + Duration::from_secs(3));
    state.profile_status_kind = StatusKind::Warning;
    state.profile_status = "Klik area aplikasi target dalam 3 detik…".to_owned();
    unsafe {
        InvalidateRect(state.window, null(), FALSE);
        ShowWindow(state.window, SW_MINIMIZE);
        SetTimer(state.window, TIMER_CAPTURE, 100, null());
    }
}

unsafe fn finish_macro_target_capture(state: &mut AppState) {
    unsafe {
        KillTimer(state.window, TIMER_CAPTURE);
        state.capture_deadline = None;
        let mut cursor = Point::default();
        let has_cursor = GetCursorPos(&mut cursor) != FALSE;
        let pointed_window = if has_cursor {
            WindowFromPoint(cursor)
        } else {
            0
        };
        let root = GetForegroundWindow();
        let receiver = if pointed_window != 0 && GetAncestor(pointed_window, GA_ROOT) == root {
            pointed_window
        } else {
            root
        };

        ShowWindow(state.window, SW_RESTORE);
        SetForegroundWindow(state.window);

        if root == 0 || receiver == 0 || root == state.window || IsWindowVisible(root) == FALSE {
            state.macro_status_kind = StatusKind::Error;
            state.macro_status = "Target macro tidak tertangkap. Coba pilih ulang.".to_owned();
            show_warning(
                state.window,
                "Target macro tidak tertangkap. Ulangi dan klik langsung area game/aplikasi.",
            );
            InvalidateRect(state.window, null(), FALSE);
            return;
        }

        let title = get_window_text(root);
        let mut process_id = 0;
        GetWindowThreadProcessId(root, &mut process_id);
        let executable = match process_executable_name(process_id) {
            Ok(value) if !value.is_empty() => value,
            Ok(_) | Err(_) => {
                state.macro_status_kind = StatusKind::Error;
                state.macro_status = "Executable target tidak dapat diverifikasi.".to_owned();
                InvalidateRect(state.window, null(), FALSE);
                return;
            }
        };
        if title.trim().is_empty() {
            state.macro_status_kind = StatusKind::Error;
            state.macro_status = "Window target tidak memiliki judul.".to_owned();
            InvalidateRect(state.window, null(), FALSE);
            return;
        }

        let selected_id = state.macro_library.selected_id;
        if let Some(item) = state.macro_library.selected_mut() {
            item.target = Some(MacroTarget {
                executable: executable.clone(),
                window_title: title.clone(),
            });
        }
        state.macro_targets.insert(
            selected_id,
            MacroPlaybackTarget {
                root,
                receiver,
                process_id,
                title: title.clone(),
            },
        );
        state.macro_dirty = true;
        state.macro_status_kind = StatusKind::Ready;
        state.macro_status = format!(
            "Target {} dipasang. Toggle tetap berjalan saat Alt+Tab.",
            executable
        );
        InvalidateRect(state.window, null(), FALSE);
    }
}

unsafe fn finish_profile_target_capture(state: &mut AppState) {
    unsafe {
        KillTimer(state.window, TIMER_CAPTURE);
        state.capture_deadline = None;
        let mut cursor = Point::default();
        let has_cursor = GetCursorPos(&mut cursor) != FALSE;
        let pointed_window = if has_cursor {
            WindowFromPoint(cursor)
        } else {
            0
        };
        let root = GetForegroundWindow();
        let receiver = if pointed_window != 0 && GetAncestor(pointed_window, GA_ROOT) == root {
            pointed_window
        } else {
            root
        };
        ShowWindow(state.window, SW_RESTORE);
        SetForegroundWindow(state.window);
        if root == 0 || receiver == 0 || root == state.window || IsWindowVisible(root) == FALSE {
            state.profile_status_kind = StatusKind::Error;
            state.profile_status = "Target profil tidak tertangkap. Coba pilih ulang.".to_owned();
            InvalidateRect(state.window, null(), FALSE);
            return;
        }
        let title = get_window_text(root);
        let mut process_id = 0;
        GetWindowThreadProcessId(root, &mut process_id);
        let executable = match process_executable_name(process_id) {
            Ok(value) if !value.is_empty() => value,
            _ => {
                state.profile_status_kind = StatusKind::Error;
                state.profile_status =
                    "Executable target profil tidak dapat diverifikasi.".to_owned();
                InvalidateRect(state.window, null(), FALSE);
                return;
            }
        };
        if title.trim().is_empty() {
            state.profile_status_kind = StatusKind::Error;
            state.profile_status = "Window target profil tidak memiliki judul.".to_owned();
            InvalidateRect(state.window, null(), FALSE);
            return;
        }
        let selected_id = state.profile_library.selected_id;
        if let Some(profile) = state.profile_library.selected_mut() {
            profile.target = Some(MacroTarget {
                executable: executable.clone(),
                window_title: title.clone(),
            });
        }
        state.profile_targets.insert(
            selected_id,
            MacroPlaybackTarget {
                root,
                receiver,
                process_id,
                title: title.clone(),
            },
        );
        apply_selected_profile_target_to_linked_macros(state);
        state.profile_dirty = true;
        persist_profiles_and_macros(
            state,
            &format!("Target {executable} dipasang ke profil dan macro tertaut."),
        );
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
        let executable = match process_executable_name(process_id) {
            Ok(value) if !value.is_empty() => value,
            _ => {
                state.status_kind = StatusKind::Error;
                state.status = "Executable target tidak dapat diverifikasi.".to_owned();
                InvalidateRect(state.window, null(), FALSE);
                return;
            }
        };
        let target = TargetWindow {
            window: target_window,
            process_id,
            title: title.clone(),
            executable,
        };
        let selected_id = state.timer_library.selected_id;
        if let Some(timer) = state.timer_library.selected_mut() {
            timer.target = Some(target_specification(&target));
        }
        state.timer_targets.insert(selected_id, target.clone());
        state.target = Some(target);
        state.timer_dirty = true;
        let _ = save_timers(&state.timers_path, &state.timer_library);
        state.status_kind = StatusKind::Ready;
        state.status = format!("Target siap: {title}");
        InvalidateRect(state.window, null(), FALSE);
    }
}

unsafe fn add_preset(state: &mut AppState, seconds: u64) {
    unsafe {
        let current = read_duration_fields(state);
        let updated = current.add_seconds(seconds);
        set_duration_fields(state, updated);
        if let Some(timer) = state.timer_library.selected_mut()
            && let Ok(total) = updated.validate()
        {
            timer.duration_seconds = total;
            timer.remaining_seconds = total;
            state.timer_dirty = true;
        }
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
        MacroEvent::MouseDown(button) | MacroEvent::MouseDownAt(button, _, _) => {
            let (data, flags) = match button {
                MouseButton::Left => (0, MOUSEEVENTF_LEFTDOWN),
                MouseButton::Right => (0, MOUSEEVENTF_RIGHTDOWN),
                MouseButton::Middle => (0, MOUSEEVENTF_MIDDLEDOWN),
                MouseButton::X1 => (XBUTTON1 as Dword, MOUSEEVENTF_XDOWN),
                MouseButton::X2 => (XBUTTON2 as Dword, MOUSEEVENTF_XDOWN),
            };
            Some(mouse_input(data, flags))
        }
        MacroEvent::MouseUp(button) | MacroEvent::MouseUpAt(button, _, _) => {
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

fn point_lparam(x: i32, y: i32) -> Lparam {
    let x = x.clamp(i16::MIN as i32, i16::MAX as i32) as i16 as u16 as u32;
    let y = y.clamp(i16::MIN as i32, i16::MAX as i32) as i16 as u16 as u32;
    (x | (y << 16)) as Lparam
}

unsafe fn background_mouse_point(target: &MacroPlaybackTarget) -> Result<Point, String> {
    let mut rect = Rect::default();
    if unsafe { GetClientRect(target.receiver, &mut rect) } == FALSE {
        return Err("Area client target macro tidak tersedia.".to_owned());
    }
    Ok(Point {
        x: (rect.right - rect.left).max(1) / 2,
        y: (rect.bottom - rect.top).max(1) / 2,
    })
}

unsafe fn post_background_event(
    target: &MacroPlaybackTarget,
    event: &MacroEvent,
) -> Result<(), String> {
    unsafe { validate_macro_playback_target(target) }?;
    let posted = match *event {
        MacroEvent::Delay(_) => TRUE,
        MacroEvent::KeyDown(key) | MacroEvent::KeyUp(key) => {
            let is_up = matches!(event, MacroEvent::KeyUp(_));
            let scan = unsafe { MapVirtualKeyW(key as Uint, MAPVK_VK_TO_VSC) };
            let mut key_data = 1u32 | (scan << 16);
            if is_up {
                key_data |= 0xC000_0000;
            }
            unsafe {
                PostMessageW(
                    target.receiver,
                    if is_up { WM_KEYUP } else { WM_KEYDOWN },
                    key as Wparam,
                    key_data as Lparam,
                )
            }
        }
        MacroEvent::MouseDown(button)
        | MacroEvent::MouseUp(button)
        | MacroEvent::MouseDownAt(button, _, _)
        | MacroEvent::MouseUpAt(button, _, _) => {
            let down = matches!(
                event,
                MacroEvent::MouseDown(_) | MacroEvent::MouseDownAt(_, _, _)
            );
            let point = match *event {
                MacroEvent::MouseDownAt(_, x, y) | MacroEvent::MouseUpAt(_, x, y) => Point { x, y },
                _ => unsafe { background_mouse_point(target) }?,
            };
            let (message, button_state) = match (button, down) {
                (MouseButton::Left, true) => (WM_LBUTTONDOWN, 0x0001usize),
                (MouseButton::Left, false) => (WM_LBUTTONUP, 0usize),
                (MouseButton::Right, true) => (WM_RBUTTONDOWN, 0x0002usize),
                (MouseButton::Right, false) => (WM_RBUTTONUP, 0usize),
                (MouseButton::Middle, true) => (WM_MBUTTONDOWN, 0x0010usize),
                (MouseButton::Middle, false) => (WM_MBUTTONUP, 0usize),
                (MouseButton::X1, true) => (WM_XBUTTONDOWN, ((XBUTTON1 as usize) << 16) | 0x0020),
                (MouseButton::X1, false) => (WM_XBUTTONUP, (XBUTTON1 as usize) << 16),
                (MouseButton::X2, true) => (WM_XBUTTONDOWN, ((XBUTTON2 as usize) << 16) | 0x0040),
                (MouseButton::X2, false) => (WM_XBUTTONUP, (XBUTTON2 as usize) << 16),
            };
            unsafe {
                PostMessageW(
                    target.receiver,
                    message,
                    button_state,
                    point_lparam(point.x, point.y),
                )
            }
        }
        MacroEvent::Wheel(delta) => {
            let mut point = unsafe { background_mouse_point(target) }?;
            unsafe { ClientToScreen(target.receiver, &mut point) };
            unsafe {
                PostMessageW(
                    target.receiver,
                    WM_MOUSEWHEEL,
                    ((delta as u16 as u32) << 16) as Wparam,
                    point_lparam(point.x, point.y),
                )
            }
        }
    };
    if posted == FALSE {
        Err(format!(
            "Window {} menolak pesan macro background.",
            target.title
        ))
    } else {
        Ok(())
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

    let target = match state.target.clone() {
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

    if let Err(message) = unsafe { validate_target(&target) } {
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

    if let Err(message) = unsafe { update_selected_timer_from_controls(state) } {
        state.status_kind = StatusKind::Error;
        state.status = message;
        unsafe {
            InvalidateRect(state.window, null(), FALSE);
        }
        return;
    }
    let selected_id = state.timer_library.selected_id;
    let start_result = state
        .timer_library
        .selected_mut()
        .ok_or_else(|| "Timer terpilih tidak ditemukan.".to_owned())
        .and_then(|timer| {
            timer
                .start(
                    now_unix_ms(),
                    total,
                    timer_action(state.action_mode),
                    prompt.trim().to_owned(),
                    target_specification(&target),
                )
                .map_err(str::to_owned)
        });
    if let Err(message) = start_result {
        state.status_kind = StatusKind::Error;
        state.status = message;
        unsafe { InvalidateRect(state.window, null(), FALSE) };
        return;
    }
    if let Err(error) = save_timers(&state.timers_path, &state.timer_library) {
        if let Some(timer) = state.timer_library.selected_mut() {
            timer.cancel();
        }
        state.status_kind = StatusKind::Error;
        state.status = format!("Timer tidak dimulai karena gagal disimpan: {error}");
        unsafe { InvalidateRect(state.window, null(), FALSE) };
        return;
    }
    state.timer_targets.insert(selected_id, target);
    state.original_seconds = total;
    state.remaining_seconds = total;
    state.armed_prompt = prompt.trim().to_owned();
    state.running = true;
    state.timer_dirty = false;
    state.status_kind = StatusKind::Running;
    state.status = format!(
        "Timer aktif. {} timer berjalan bersamaan.",
        state.timer_library.running_count()
    );
    unsafe {
        state.set_controls_visible(false);
        state.set_prompt_enabled();
        SetTimer(state.window, TIMER_COUNTDOWN, 100, null());
        InvalidateRect(state.window, null(), FALSE);
    }
}

unsafe fn cancel_timer(state: &mut AppState) {
    unsafe {
        if let Some(timer) = state.timer_library.selected_mut() {
            timer.cancel();
        }
        let _ = save_timers(&state.timers_path, &state.timer_library);
        if state.timer_library.running_count() == 0 {
            KillTimer(state.window, TIMER_COUNTDOWN);
        }
        state.running = false;
        state.status_kind = StatusKind::Warning;
        state.status = "Timer dibatalkan. Tidak ada input yang dikirim.".to_owned();
        state.set_controls_visible(true);
        state.set_prompt_enabled();
        InvalidateRect(state.window, null(), FALSE);
    }
}

unsafe fn finish_timer(state: &mut AppState, timer_id: u32) {
    unsafe {
        let Some(timer) = state
            .timer_library
            .timers
            .iter()
            .find(|timer| timer.id == timer_id)
            .cloned()
        else {
            return;
        };
        let result = match timer.target.as_ref() {
            Some(specification) => {
                resolve_timer_target(state, timer_id, specification).and_then(|target| {
                    perform_scheduled_action(&target, action_mode(timer.action), &timer.prompt)
                        .map(|_| target.title)
                })
            }
            None => Err("Target timer tidak tersedia.".to_owned()),
        };
        state.timer_library.mark_result(timer_id, result.is_ok());
        let _ = save_timers(&state.timers_path, &state.timer_library);
        if state.timer_library.running_count() == 0 {
            KillTimer(state.window, TIMER_COUNTDOWN);
        }
        let is_selected = timer_id == state.timer_library.selected_id;
        if is_selected {
            sync_selected_timer_to_controls(state);
        }
        match result {
            Ok(title) => {
                state.status_kind = StatusKind::Sent;
                state.status = format!("{} berhasil dikirim ke {title}", timer.name);
                show_tray_notification(
                    state,
                    "Timer selesai",
                    &format!("{} berhasil menjalankan aksi satu kali.", timer.name),
                );
                MessageBeep(MB_OK);
            }
            Err(message) => {
                state.status_kind = StatusKind::Error;
                state.status = format!("{} gagal: {message}", timer.name);
                show_tray_notification(
                    state,
                    "Timer gagal",
                    &format!("{} tidak mengirim input. {message}", timer.name),
                );
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
    let previous: Vec<(u32, u64, TimerPhase)> = state
        .timer_library
        .timers
        .iter()
        .map(|timer| (timer.id, timer.remaining_seconds, timer.phase))
        .collect();
    let due = state.timer_library.refresh_due(now_unix_ms());
    if let Some(selected) = state.timer_library.selected() {
        state.running = selected.is_running();
        state.original_seconds = selected.duration_seconds;
        state.remaining_seconds = selected.remaining_seconds;
    }
    for timer_id in due {
        unsafe { finish_timer(state, timer_id) };
    }
    let changed = previous.iter().any(|(id, remaining, phase)| {
        state
            .timer_library
            .timers
            .iter()
            .find(|timer| timer.id == *id)
            .is_none_or(|timer| timer.remaining_seconds != *remaining || timer.phase != *phase)
    });
    if changed {
        unsafe { InvalidateRect(state.window, null(), FALSE) };
    }
}

unsafe fn resize_for_tab(state: &AppState) {
    let client_width = match state.tab {
        AppTab::Timer => CLIENT_WIDTH,
        AppTab::Macro => MACRO_CLIENT_WIDTH,
        AppTab::Profiles => PROFILES_CLIENT_WIDTH,
        AppTab::Settings => SETTINGS_CLIENT_WIDTH,
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

unsafe fn sync_profile_name_edit(state: &AppState) {
    let name = state
        .profile_library
        .selected()
        .map(|profile| profile.name.as_str())
        .unwrap_or("Profil");
    unsafe { SetWindowTextW(state.profile_name_edit, wide(name).as_ptr()) };
}

fn timer_action(action: ActionMode) -> TimerAction {
    match action {
        ActionMode::EnterOnly => TimerAction::EnterOnly,
        ActionMode::TextAndEnter => TimerAction::TextAndEnter,
    }
}

fn action_mode(action: TimerAction) -> ActionMode {
    match action {
        TimerAction::EnterOnly => ActionMode::EnterOnly,
        TimerAction::TextAndEnter => ActionMode::TextAndEnter,
    }
}

fn target_specification(target: &TargetWindow) -> MacroTarget {
    MacroTarget {
        executable: target.executable.clone(),
        window_title: target.title.clone(),
    }
}

unsafe fn resolve_timer_target(
    state: &mut AppState,
    timer_id: u32,
    specification: &MacroTarget,
) -> Result<TargetWindow, String> {
    if let Some(target) = state.timer_targets.get(&timer_id)
        && unsafe { validate_target(target) }.is_ok()
        && target
            .executable
            .eq_ignore_ascii_case(&specification.executable)
    {
        return Ok(target.clone());
    }
    state.timer_targets.remove(&timer_id);
    let resolved = unsafe { find_saved_macro_target(specification) }?;
    let target = TargetWindow {
        window: resolved.root,
        process_id: resolved.process_id,
        title: resolved.title,
        executable: specification.executable.clone(),
    };
    state.timer_targets.insert(timer_id, target.clone());
    Ok(target)
}

unsafe fn sync_selected_timer_to_controls(state: &mut AppState) {
    let Some(timer) = state.timer_library.selected().cloned() else {
        return;
    };
    state.running = timer.is_running();
    state.original_seconds = timer.duration_seconds;
    state.remaining_seconds = timer.remaining_seconds;
    state.armed_prompt = timer.prompt.clone();
    state.action_mode = action_mode(timer.action);
    unsafe {
        SetWindowTextW(state.timer_name_edit, wide(&timer.name).as_ptr());
        set_duration_fields(
            state,
            DurationFields::from_total_seconds(timer.duration_seconds),
        );
        SetWindowTextW(state.prompt_edit, wide(&timer.prompt).as_ptr());
    }
    state.target = match timer.target.as_ref() {
        Some(specification) => unsafe { resolve_timer_target(state, timer.id, specification) }.ok(),
        None => None,
    };
    unsafe {
        state.set_controls_visible(!state.running);
        state.set_prompt_enabled();
    }
}

unsafe fn update_selected_timer_from_controls(state: &mut AppState) -> Result<(), String> {
    if state.running {
        return Ok(());
    }
    let name = unsafe { get_window_text(state.timer_name_edit) };
    if name.trim().is_empty() {
        return Err("Nama timer tidak boleh kosong.".to_owned());
    }
    let duration = unsafe { read_duration_fields(state) }
        .validate()
        .map_err(str::to_owned)?;
    let prompt = unsafe { get_window_text(state.prompt_edit) };
    if state.action_mode == ActionMode::TextAndEnter && prompt.trim().is_empty() {
        return Err("Isi teks yang akan dikirim, atau pilih Hanya Enter.".to_owned());
    }
    let target = state.target.as_ref().map(target_specification).or_else(|| {
        state
            .timer_library
            .selected()
            .and_then(|timer| timer.target.clone())
    });
    let Some(timer) = state.timer_library.selected_mut() else {
        return Err("Timer terpilih tidak ditemukan.".to_owned());
    };
    timer.name = name.trim().to_owned();
    timer.duration_seconds = duration;
    timer.remaining_seconds = duration;
    timer.action = timer_action(state.action_mode);
    timer.prompt = prompt.trim().to_owned();
    timer.target = target;
    Ok(())
}

unsafe fn persist_timer_library(state: &mut AppState, message: &str) -> bool {
    match save_timers(&state.timers_path, &state.timer_library) {
        Ok(()) => {
            state.timer_dirty = false;
            state.status_kind = StatusKind::Sent;
            state.status = message.to_owned();
            unsafe { InvalidateRect(state.window, null(), FALSE) };
            true
        }
        Err(error) => {
            state.status_kind = StatusKind::Error;
            state.status = format!("Timer gagal disimpan: {error}");
            unsafe { InvalidateRect(state.window, null(), FALSE) };
            false
        }
    }
}

unsafe fn save_selected_timer(state: &mut AppState) -> bool {
    if let Err(message) = unsafe { update_selected_timer_from_controls(state) } {
        state.status_kind = StatusKind::Error;
        state.status = message;
        unsafe { InvalidateRect(state.window, null(), FALSE) };
        return false;
    }
    state.timer_dirty = true;
    unsafe { persist_timer_library(state, "Timer dan target tersimpan.") }
}

unsafe fn clipboard_text(owner: Hwnd) -> Result<String, String> {
    unsafe {
        if IsClipboardFormatAvailable(CF_UNICODETEXT) == FALSE {
            return Err("Clipboard tidak berisi teks Unicode.".to_owned());
        }
        if OpenClipboard(owner) == FALSE {
            return Err("Clipboard sedang dipakai aplikasi lain. Coba lagi.".to_owned());
        }
        let result = (|| {
            let memory = GetClipboardData(CF_UNICODETEXT);
            if memory == 0 {
                return Err("Teks clipboard tidak dapat dibaca.".to_owned());
            }
            let pointer = GlobalLock(memory) as *const u16;
            if pointer.is_null() {
                return Err("Teks clipboard tidak dapat dikunci.".to_owned());
            }
            let maximum_units = (GlobalSize(memory) / size_of::<u16>()).min(16_384);
            if maximum_units == 0 {
                GlobalUnlock(memory);
                return Err("Ukuran teks clipboard tidak valid.".to_owned());
            }
            let mut length = 0usize;
            while length < maximum_units && *pointer.add(length) != 0 {
                length += 1;
            }
            if length == maximum_units {
                GlobalUnlock(memory);
                return Err("Teks clipboard tidak memiliki terminator yang valid.".to_owned());
            }
            let text = String::from_utf16_lossy(std::slice::from_raw_parts(pointer, length));
            GlobalUnlock(memory);
            if text.trim().is_empty() {
                Err("Clipboard teks kosong.".to_owned())
            } else {
                Ok(text)
            }
        })();
        CloseClipboard();
        result
    }
}

unsafe fn local_clock_context() -> ClockContext {
    let mut time = SystemTimeW::default();
    unsafe { GetLocalTime(&mut time) };
    ClockContext {
        hour: time.hour.min(23) as u8,
        minute: time.minute.min(59) as u8,
        second: time.second.min(59) as u8,
        weekday: time.day_of_week.min(6) as u8,
    }
}

unsafe fn apply_smart_reset(state: &mut AppState, text: &str) -> bool {
    if state.running {
        return false;
    }
    match parse_reset_text(text, unsafe { local_clock_context() }) {
        Ok(capture) => {
            unsafe {
                set_duration_fields(state, DurationFields::from_total_seconds(capture.seconds));
                SetWindowTextW(state.smart_reset_edit, wide(text.trim()).as_ptr());
            }
            if let Some(timer) = state.timer_library.selected_mut() {
                timer.duration_seconds = capture.seconds;
                timer.remaining_seconds = capture.seconds;
            }
            state.original_seconds = capture.seconds;
            state.remaining_seconds = capture.seconds;
            state.timer_dirty = true;
            state.status_kind = StatusKind::Sent;
            state.status = capture.summary;
            unsafe { InvalidateRect(state.window, null(), FALSE) };
            true
        }
        Err(message) => {
            state.status_kind = StatusKind::Error;
            state.status = message.to_owned();
            unsafe { InvalidateRect(state.window, null(), FALSE) };
            false
        }
    }
}

fn apply_selected_profile_target_to_linked_macros(state: &mut AppState) {
    let Some(profile) = state.profile_library.selected() else {
        return;
    };
    let Some(target) = profile.target.clone() else {
        return;
    };
    let linked = profile.macro_ids.clone();
    for item in &mut state.macro_library.macros {
        if linked.contains(&item.id) {
            item.target = Some(target.clone());
            state.macro_targets.remove(&item.id);
        }
    }
}

unsafe fn persist_profiles_and_macros(state: &mut AppState, message: &str) -> bool {
    let previous_profiles = load_profiles(&state.profiles_path);
    let previous_macros = load_library(&state.macro_path);
    let profiles_result = save_profiles(&state.profiles_path, &state.profile_library);
    let macros_result = save_library(&state.macro_path, &state.macro_library);
    match profiles_result.and(macros_result) {
        Ok(()) => {
            state.profile_dirty = false;
            state.macro_dirty = false;
            state.profile_status_kind = StatusKind::Sent;
            state.profile_status = message.to_owned();
            unsafe { InvalidateRect(state.window, null(), FALSE) };
            true
        }
        Err(error) => {
            let rollback = previous_profiles.and_then(|profiles| {
                save_profiles(&state.profiles_path, &profiles).and_then(|_| {
                    previous_macros.and_then(|macros| save_library(&state.macro_path, &macros))
                })
            });
            state.profile_status_kind = StatusKind::Error;
            state.profile_status = if rollback.is_ok() {
                format!("Profil gagal disimpan; file lama dipulihkan: {error}")
            } else {
                format!("Profil gagal disimpan dan rollback tidak lengkap: {error}")
            };
            unsafe { InvalidateRect(state.window, null(), FALSE) };
            false
        }
    }
}

unsafe fn save_current_profile(state: &mut AppState) {
    let name = unsafe { get_window_text(state.profile_name_edit) };
    if name.trim().is_empty() {
        state.profile_status_kind = StatusKind::Error;
        state.profile_status = "Nama profil tidak boleh kosong.".to_owned();
        unsafe { InvalidateRect(state.window, null(), FALSE) };
        return;
    }
    if let Some(profile) = state.profile_library.selected_mut() {
        profile.name = name.trim().to_owned();
    }
    state.profile_dirty = true;
    unsafe { persist_profiles_and_macros(state, "Profil dan tautan macro tersimpan.") };
}

unsafe fn switch_tab(state: &mut AppState, tab: AppTab) {
    if state.tab == tab {
        return;
    }
    if state.tab == AppTab::Timer
        && tab != AppTab::Timer
        && state.timer_dirty
        && !state.running
        && !unsafe { save_selected_timer(state) }
    {
        return;
    }
    if state.recording {
        unsafe { stop_macro_recording(state) };
    }
    state.tab = tab;
    state.hot = HitTarget::None;
    unsafe {
        if state.tab == AppTab::Profiles {
            sync_profile_name_edit(state);
            let selected_id = state.profile_library.selected_id;
            if let Some(specification) = state
                .profile_library
                .selected()
                .and_then(|profile| profile.target.clone())
            {
                if let Ok(target) = find_saved_macro_target(&specification) {
                    state.profile_targets.insert(selected_id, target);
                } else {
                    state.profile_targets.remove(&selected_id);
                }
            }
        } else if state.tab == AppTab::Timer {
            sync_selected_timer_to_controls(state);
        }
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
            let item = state.macro_library.selected();
            state.macro_status = format!(
                "Macro tersimpan • {} • {}.",
                item.map(|item| item.trigger.label()).unwrap_or("-"),
                if item.is_some_and(|item| item.target.is_some()) {
                    "khusus window target"
                } else {
                    "aktif global"
                }
            );
        }
        Err(error) => {
            state.macro_status_kind = StatusKind::Error;
            state.macro_status = format!("Gagal menyimpan macro: {error}");
        }
    }
    unsafe {
        refresh_macro_hooks(state);
        InvalidateRect(state.window, null(), FALSE);
    };
}

unsafe fn start_macro_recording(state: &mut AppState) {
    if state.recording || state.macro_library.selected().is_none() {
        return;
    }
    let item = state
        .macro_library
        .selected()
        .expect("selected macro checked")
        .clone();
    let destination = match unsafe { resolve_playback_destination(state, &item) } {
        Ok(destination) => destination,
        Err(message) => {
            state.macro_status_kind = StatusKind::Error;
            state.macro_status = message;
            unsafe { InvalidateRect(state.window, null(), FALSE) };
            return;
        }
    };
    MACRO_STOP.store(true, Ordering::Release);
    state.recording = true;
    if !unsafe { refresh_macro_hooks(state) } {
        state.recording = false;
        unsafe { refresh_macro_hooks(state) };
        return;
    }
    state.record_last_event = Some(Instant::now());
    state.macro_status_kind = StatusKind::Running;
    state.macro_status = match destination {
        PlaybackDestination::Foreground => {
            "Merekam input global. Tekan Esc untuk selesai.".to_owned()
        }
        PlaybackDestination::Window(target) => format!(
            "Merekam posisi relatif untuk {}. Tekan Esc untuk selesai.",
            target.title
        ),
    };
    unsafe {
        state.set_controls_visible(true);
        state.set_prompt_enabled();
        InvalidateRect(state.window, null(), FALSE);
    }
}

unsafe fn recorded_mouse_event(
    state: &AppState,
    button: MouseButton,
    down: bool,
    screen_point: Point,
) -> MacroEvent {
    let selected_id = state.macro_library.selected_id;
    let window_scoped = state
        .macro_library
        .selected()
        .is_some_and(|item| item.target.is_some());
    if window_scoped && let Some(target) = state.macro_targets.get(&selected_id) {
        let mut client_point = screen_point;
        if unsafe { ScreenToClient(target.receiver, &mut client_point) } != FALSE {
            return if down {
                MacroEvent::MouseDownAt(button, client_point.x, client_point.y)
            } else {
                MacroEvent::MouseUpAt(button, client_point.x, client_point.y)
            };
        }
    }
    if down {
        MacroEvent::MouseDown(button)
    } else {
        MacroEvent::MouseUp(button)
    }
}

unsafe fn stop_macro_recording(state: &mut AppState) {
    if !state.recording {
        return;
    }
    state.recording = false;
    unsafe { refresh_macro_hooks(state) };
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

fn play_macro_events(
    events: &[MacroEvent],
    standard_delay: Option<u32>,
    destination: &PlaybackDestination,
) -> Result<bool, String> {
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
        #[cfg(test)]
        TEST_MACRO_INPUT_COUNT.fetch_add(1, Ordering::Relaxed);
        match destination {
            PlaybackDestination::Foreground => {
                if let Some(input) = macro_event_input(event) {
                    unsafe { submit_inputs(&[input])? };
                }
            }
            PlaybackDestination::Window(target) => unsafe {
                post_background_event(target, event)?;
            },
        }
    }
    Ok(true)
}

fn post_macro_result(window: Hwnd, result: Result<(), String>) {
    let emergency = EMERGENCY_ACTIVE.swap(false, Ordering::AcqRel);
    let result_code = if emergency {
        4
    } else {
        match result {
            Ok(()) => 1,
            Err(message) if message.contains("Batas aman") => 3,
            Err(message)
                if message.contains("target")
                    || message.contains("Target")
                    || message.contains("Window") =>
            {
                2
            }
            Err(_) => 0,
        }
    };
    MACRO_PLAYING.store(false, Ordering::Release);
    MACRO_PLAYING_ID.store(0, Ordering::Release);
    MACRO_STOP.store(false, Ordering::Release);
    unsafe {
        PostMessageW(window, WM_APP_MACRO_DONE, result_code, 0);
    }
}

fn playback_limit_check(
    started: Instant,
    repeats: u32,
    max_runtime_seconds: u32,
    max_repeats: u32,
) -> Result<(), String> {
    if max_runtime_seconds > 0
        && started.elapsed() >= Duration::from_secs(max_runtime_seconds as u64)
    {
        return Err(format!(
            "Batas aman durasi {} detik tercapai.",
            max_runtime_seconds
        ));
    }
    if max_repeats > 0 && repeats >= max_repeats {
        return Err(format!(
            "Batas aman pengulangan {max_repeats} kali tercapai."
        ));
    }
    Ok(())
}

fn launch_macro_playback(
    window: Hwnd,
    item: MacroDefinition,
    destination: PlaybackDestination,
    max_runtime_seconds: u32,
    max_repeats: u32,
) {
    if item.on_press.is_empty() && item.while_holding.is_empty() && item.on_release.is_empty() {
        unsafe { PostMessageW(window, WM_APP_MACRO_DONE, 0, 0) };
        return;
    }

    if item.mode == MacroMode::Toggle
        && MACRO_PLAYING_ID.load(Ordering::Acquire) == item.id
        && MACRO_PLAYING.load(Ordering::Acquire)
    {
        MACRO_STOP.store(true, Ordering::Release);
        return;
    }
    if MACRO_PLAYING
        .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
        .is_err()
    {
        return;
    }
    MACRO_PLAYING_ID.store(item.id, Ordering::Release);
    MACRO_STOP.store(false, Ordering::Release);
    thread::spawn(move || {
        let standard = item.standard_delay_ms;
        let started = Instant::now();
        let mut repeats = 0u32;
        let result = (|| -> Result<(), String> {
            match item.mode {
                MacroMode::NoRepeat => {
                    play_macro_events(&item.on_press, standard, &destination)?;
                }
                MacroMode::RepeatWhileHolding => {
                    while TRIGGER_HELD.load(Ordering::Acquire)
                        && !MACRO_STOP.load(Ordering::Acquire)
                    {
                        playback_limit_check(started, repeats, max_runtime_seconds, max_repeats)?;
                        if !play_macro_events(&item.on_press, standard, &destination)? {
                            break;
                        }
                        repeats = repeats.saturating_add(1);
                        if item.on_press.is_empty() && !sleep_interruptible(10) {
                            break;
                        }
                    }
                }
                MacroMode::Toggle => {
                    while !MACRO_STOP.load(Ordering::Acquire) {
                        playback_limit_check(started, repeats, max_runtime_seconds, max_repeats)?;
                        if !play_macro_events(&item.on_press, standard, &destination)? {
                            break;
                        }
                        repeats = repeats.saturating_add(1);
                        if item.on_press.is_empty() && !sleep_interruptible(10) {
                            break;
                        }
                    }
                }
                MacroMode::Sequence => {
                    if play_macro_events(&item.on_press, standard, &destination)? {
                        while TRIGGER_HELD.load(Ordering::Acquire)
                            && !MACRO_STOP.load(Ordering::Acquire)
                        {
                            playback_limit_check(
                                started,
                                repeats,
                                max_runtime_seconds,
                                max_repeats,
                            )?;
                            if !play_macro_events(&item.while_holding, standard, &destination)? {
                                break;
                            }
                            repeats = repeats.saturating_add(1);
                            if item.while_holding.is_empty() && !sleep_interruptible(10) {
                                break;
                            }
                        }
                        if !MACRO_STOP.load(Ordering::Acquire) {
                            play_macro_events(&item.on_release, standard, &destination)?;
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

unsafe fn macro_trigger_allowed(state: &AppState, item: &MacroDefinition) -> bool {
    let Some(specification) = item.target.as_ref() else {
        return true;
    };
    if state.trigger_macro_id == Some(item.id) {
        return true;
    }
    if item.mode == MacroMode::Toggle
        && MACRO_PLAYING.load(Ordering::Acquire)
        && MACRO_PLAYING_ID.load(Ordering::Acquire) == item.id
    {
        return true;
    }
    let foreground = unsafe { GetForegroundWindow() };
    #[cfg(test)]
    let foreground = if foreground == 0 {
        unsafe { GetActiveWindow() }
    } else {
        foreground
    };
    if foreground == 0 {
        return false;
    }
    let foreground_root = unsafe { GetAncestor(foreground, GA_ROOT) };
    if state
        .macro_targets
        .get(&item.id)
        .is_some_and(|target| target.root == foreground_root)
    {
        return true;
    }
    let title = unsafe { get_window_text(foreground_root) };
    if !title.eq_ignore_ascii_case(&specification.window_title) {
        return false;
    }
    let mut process_id = 0;
    unsafe { GetWindowThreadProcessId(foreground_root, &mut process_id) };
    unsafe { process_executable_name(process_id) }
        .as_deref()
        .is_ok_and(|value| value.eq_ignore_ascii_case(&specification.executable))
}

unsafe fn macro_for_keyboard_trigger(state: &AppState, key: u16) -> Option<MacroDefinition> {
    if let Some(selected) = state.macro_library.selected()
        && keyboard_trigger_matches(selected.trigger, key)
    {
        return unsafe { macro_trigger_allowed(state, selected) }.then(|| selected.clone());
    }
    state
        .macro_library
        .macros
        .iter()
        .find(|item| {
            keyboard_trigger_matches(item.trigger, key)
                && unsafe { macro_trigger_allowed(state, item) }
        })
        .cloned()
}

unsafe fn macro_for_mouse_trigger(
    state: &AppState,
    message: Uint,
    mouse_data: Dword,
) -> Option<MacroDefinition> {
    if let Some(selected) = state.macro_library.selected()
        && mouse_trigger_matches(selected.trigger, message, mouse_data)
    {
        return unsafe { macro_trigger_allowed(state, selected) }.then(|| selected.clone());
    }
    state
        .macro_library
        .macros
        .iter()
        .find(|item| {
            mouse_trigger_matches(item.trigger, message, mouse_data)
                && unsafe { macro_trigger_allowed(state, item) }
        })
        .cloned()
}

unsafe fn handle_trigger_down(state: &mut AppState, item: MacroDefinition) {
    if state.trigger_down {
        return;
    }
    state.trigger_down = true;
    state.trigger_macro_id = Some(item.id);
    TRIGGER_HELD.store(true, Ordering::Release);
    let destination = if item.mode == MacroMode::Toggle
        && MACRO_PLAYING_ID.load(Ordering::Acquire) == item.id
        && MACRO_PLAYING.load(Ordering::Acquire)
    {
        PlaybackDestination::Foreground
    } else {
        match unsafe { resolve_playback_destination(state, &item) } {
            Ok(destination) => destination,
            Err(message) => {
                state.trigger_down = false;
                state.trigger_macro_id = None;
                TRIGGER_HELD.store(false, Ordering::Release);
                state.macro_status_kind = StatusKind::Error;
                state.macro_status = message;
                unsafe { InvalidateRect(state.window, null(), FALSE) };
                return;
            }
        }
    };
    state.macro_status_kind = StatusKind::Running;
    state.macro_status = match &destination {
        PlaybackDestination::Foreground => format!("Menjalankan {}…", item.name),
        PlaybackDestination::Window(target) => {
            format!("{} berjalan di {} • Alt+Tab aman", item.name, target.title)
        }
    };
    unsafe { InvalidateRect(state.window, null(), FALSE) };
    launch_macro_playback(
        state.window,
        item,
        destination,
        state.settings.max_macro_runtime_seconds,
        state.settings.max_macro_repeats,
    );
}

fn handle_trigger_up(state: &mut AppState) {
    state.trigger_down = false;
    state.trigger_macro_id = None;
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
    let item = unsafe { macro_for_keyboard_trigger(state, key) };
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
    let message = wparam as Uint;
    if !matches!(
        message,
        WM_LBUTTONDOWN
            | WM_LBUTTONUP
            | WM_RBUTTONDOWN
            | WM_RBUTTONUP
            | WM_MBUTTONDOWN
            | WM_MBUTTONUP
            | WM_XBUTTONDOWN
            | WM_XBUTTONUP
            | WM_MOUSEWHEEL
    ) {
        return unsafe { CallNextHookEx(0, code, wparam, lparam) };
    }
    let state_pointer = APP_STATE_POINTER.load(Ordering::Acquire);
    if state_pointer.is_null() {
        return unsafe { CallNextHookEx(0, code, wparam, lparam) };
    }
    let state = unsafe { &mut *state_pointer };
    let button_event = match message {
        WM_LBUTTONDOWN => {
            Some(unsafe { recorded_mouse_event(state, MouseButton::Left, true, data.point) })
        }
        WM_LBUTTONUP => {
            Some(unsafe { recorded_mouse_event(state, MouseButton::Left, false, data.point) })
        }
        WM_RBUTTONDOWN => {
            Some(unsafe { recorded_mouse_event(state, MouseButton::Right, true, data.point) })
        }
        WM_RBUTTONUP => {
            Some(unsafe { recorded_mouse_event(state, MouseButton::Right, false, data.point) })
        }
        WM_MBUTTONDOWN => {
            Some(unsafe { recorded_mouse_event(state, MouseButton::Middle, true, data.point) })
        }
        WM_MBUTTONUP => {
            Some(unsafe { recorded_mouse_event(state, MouseButton::Middle, false, data.point) })
        }
        WM_XBUTTONDOWN if (data.mouse_data >> 16) as u16 == XBUTTON1 => {
            Some(unsafe { recorded_mouse_event(state, MouseButton::X1, true, data.point) })
        }
        WM_XBUTTONUP if (data.mouse_data >> 16) as u16 == XBUTTON1 => {
            Some(unsafe { recorded_mouse_event(state, MouseButton::X1, false, data.point) })
        }
        WM_XBUTTONDOWN => {
            Some(unsafe { recorded_mouse_event(state, MouseButton::X2, true, data.point) })
        }
        WM_XBUTTONUP => {
            Some(unsafe { recorded_mouse_event(state, MouseButton::X2, false, data.point) })
        }
        WM_MOUSEWHEEL => Some(MacroEvent::Wheel((data.mouse_data >> 16) as i16)),
        _ => None,
    };
    if state.recording
        && let Some(event) = button_event
    {
        unsafe { record_macro_event(state, event) };
        return 1;
    }
    let item = unsafe { macro_for_mouse_trigger(state, message, data.mouse_data) };
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

fn macro_has_actions(item: &MacroDefinition) -> bool {
    item.on_press
        .iter()
        .chain(&item.while_holding)
        .chain(&item.on_release)
        .any(|event| !matches!(event, MacroEvent::Delay(_)))
}

unsafe fn reconcile_macro_hooks(state: &mut AppState, instance: Hinstance) -> bool {
    let wants_keyboard = state.recording
        || state.macro_library.macros.iter().any(|item| {
            macro_has_actions(item) && matches!(item.trigger, MacroTrigger::F8 | MacroTrigger::F9)
        });
    let wants_mouse = state.recording
        || state.macro_library.macros.iter().any(|item| {
            macro_has_actions(item)
                && matches!(
                    item.trigger,
                    MacroTrigger::MouseMiddle | MacroTrigger::MouseX1 | MacroTrigger::MouseX2
                )
        });
    unsafe {
        if wants_keyboard && state.keyboard_hook == 0 {
            state.keyboard_hook =
                SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_hook_proc), instance, 0);
        } else if !wants_keyboard && state.keyboard_hook != 0 {
            UnhookWindowsHookEx(state.keyboard_hook);
            state.keyboard_hook = 0;
        }
        if wants_mouse && state.mouse_hook == 0 {
            state.mouse_hook = SetWindowsHookExW(WH_MOUSE_LL, Some(mouse_hook_proc), instance, 0);
        } else if !wants_mouse && state.mouse_hook != 0 {
            UnhookWindowsHookEx(state.mouse_hook);
            state.mouse_hook = 0;
        }
    }
    let ready =
        (!wants_keyboard || state.keyboard_hook != 0) && (!wants_mouse || state.mouse_hook != 0);
    if !ready {
        state.macro_status_kind = StatusKind::Error;
        state.macro_status =
            "Hook global gagal aktif. Jalankan aplikasi pada desktop Windows biasa.".to_owned();
    }
    ready
}

unsafe fn refresh_macro_hooks(state: &mut AppState) -> bool {
    unsafe { reconcile_macro_hooks(state, GetModuleHandleW(null())) }
}

unsafe fn initialize_macro_hooks(state: &mut AppState, instance: Hinstance) {
    unsafe {
        reconcile_macro_hooks(state, instance);
    }
}

unsafe fn confirm_delete_macro(window: Hwnd, name: &str) -> bool {
    #[cfg(test)]
    {
        let _ = (window, name);
        true
    }
    #[cfg(not(test))]
    {
        unsafe {
            MessageBoxW(
                window,
                wide(&format!(
                    "Hapus macro ‘{name}’? Tindakan ini permanen setelah disimpan."
                ))
                .as_ptr(),
                wide("Hapus macro").as_ptr(),
                MB_YESNO | MB_ICONWARNING,
            ) == IDYES
        }
    }
}

unsafe fn confirm_delete_profile(window: Hwnd, name: &str) -> bool {
    #[cfg(test)]
    {
        let _ = (window, name);
        true
    }
    #[cfg(not(test))]
    unsafe {
        MessageBoxW(
            window,
            wide(&format!("Hapus profil ‘{name}’? Macro tidak ikut dihapus.")).as_ptr(),
            wide("Hapus App Profile").as_ptr(),
            MB_YESNO | MB_ICONWARNING,
        ) == IDYES
    }
}

unsafe fn confirm_delete_timer(window: Hwnd, name: &str) -> bool {
    #[cfg(test)]
    {
        let _ = (window, name);
        true
    }
    #[cfg(not(test))]
    unsafe {
        MessageBoxW(
            window,
            wide(&format!(
                "Hapus timer ‘{name}’? Timer yang sedang berjalan tidak dapat dihapus."
            ))
            .as_ptr(),
            wide("Hapus timer").as_ptr(),
            MB_YESNO | MB_ICONWARNING,
        ) == IDYES
    }
}

unsafe fn confirm_import_backup(window: Hwnd) -> bool {
    #[cfg(test)]
    {
        let _ = window;
        true
    }
    #[cfg(not(test))]
    unsafe {
        MessageBoxW(
            window,
            wide("Import akan mengganti macro, profil, timer, dan Settings saat ini. Timer aktif dari backup selalu dibatalkan demi keamanan. Lanjutkan?").as_ptr(),
            wide("Import backup VibeTimer").as_ptr(),
            MB_YESNO | MB_ICONWARNING,
        ) == IDYES
    }
}

unsafe fn handle_click(state: &mut AppState, target: HitTarget) {
    match target {
        HitTarget::None => {}
        HitTarget::TimerTab => unsafe { switch_tab(state, AppTab::Timer) },
        HitTarget::MacroTab => unsafe { switch_tab(state, AppTab::Macro) },
        HitTarget::ProfilesTab => unsafe { switch_tab(state, AppTab::Profiles) },
        HitTarget::SettingsTab => unsafe { switch_tab(state, AppTab::Settings) },
        HitTarget::AddThirtyMinutes => unsafe { add_preset(state, 30 * 60) },
        HitTarget::AddOneHour => unsafe { add_preset(state, 60 * 60) },
        HitTarget::AddThreeHours => unsafe { add_preset(state, 3 * 60 * 60) },
        HitTarget::TimerNew => {
            if !state.running && !unsafe { save_selected_timer(state) } {
                return;
            }
            if state.timer_library.add_timer() {
                state.timer_dirty = true;
                unsafe {
                    sync_selected_timer_to_controls(state);
                    persist_timer_library(state, "Timer baru dibuat.");
                }
            } else {
                state.status_kind = StatusKind::Warning;
                state.status =
                    "Maksimal 6 timer agar semua tetap terlihat dan mudah diawasi.".to_owned();
                unsafe { InvalidateRect(state.window, null(), FALSE) };
            }
        }
        HitTarget::TimerItem(index) => {
            let Some(timer_id) = state.timer_library.timers.get(index).map(|timer| timer.id) else {
                return;
            };
            if timer_id == state.timer_library.selected_id {
                return;
            }
            if !state.running && !unsafe { save_selected_timer(state) } {
                return;
            }
            state.timer_library.selected_id = timer_id;
            unsafe { sync_selected_timer_to_controls(state) };
            state.status_kind = if state.running {
                StatusKind::Running
            } else {
                StatusKind::Ready
            };
            state.status = state
                .timer_library
                .selected()
                .map(|timer| format!("Mengelola {} — {}.", timer.name, timer.phase.label()))
                .unwrap_or_else(|| "Timer dipilih.".to_owned());
            unsafe { InvalidateRect(state.window, null(), FALSE) };
        }
        HitTarget::TimerDuplicate => {
            if !state.running && !unsafe { save_selected_timer(state) } {
                return;
            }
            if state.timer_library.duplicate_selected() {
                state.timer_dirty = true;
                unsafe {
                    sync_selected_timer_to_controls(state);
                    persist_timer_library(state, "Timer disalin sebagai template baru.");
                }
            } else {
                state.status_kind = StatusKind::Warning;
                state.status = "Timer tidak dapat disalin; batas 6 timer tercapai.".to_owned();
                unsafe { InvalidateRect(state.window, null(), FALSE) };
            }
        }
        HitTarget::TimerDelete => {
            let selected = state.timer_library.selected().cloned();
            let Some(timer) = selected else {
                return;
            };
            if timer.is_running() {
                state.status_kind = StatusKind::Warning;
                state.status = "Batalkan timer sebelum menghapusnya.".to_owned();
                unsafe { InvalidateRect(state.window, null(), FALSE) };
                return;
            }
            if unsafe { confirm_delete_timer(state.window, &timer.name) }
                && state.timer_library.delete_selected()
            {
                state.timer_targets.remove(&timer.id);
                state.timer_dirty = true;
                unsafe {
                    sync_selected_timer_to_controls(state);
                    persist_timer_library(state, "Timer dihapus.");
                }
            } else if state.timer_library.timers.len() <= 1 {
                state.status_kind = StatusKind::Warning;
                state.status = "Minimal satu timer harus tetap tersedia.".to_owned();
                unsafe { InvalidateRect(state.window, null(), FALSE) };
            }
        }
        HitTarget::TimerSave => {
            unsafe { save_selected_timer(state) };
        }
        HitTarget::SmartResetClipboard => {
            if state.running {
                return;
            }
            match unsafe { clipboard_text(state.window) } {
                Ok(text) => {
                    unsafe { apply_smart_reset(state, &text) };
                }
                Err(message) => {
                    state.status_kind = StatusKind::Error;
                    state.status = message;
                    unsafe { InvalidateRect(state.window, null(), FALSE) };
                }
            }
        }
        HitTarget::SmartResetApply => {
            if !state.running {
                let text = unsafe { get_window_text(state.smart_reset_edit) };
                unsafe { apply_smart_reset(state, &text) };
            }
        }
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
            if state.macro_library.add_macro().is_none() {
                state.macro_status_kind = StatusKind::Warning;
                state.macro_status =
                    "Maksimal 6 macro agar seluruh assignment tetap terlihat.".to_owned();
                unsafe { InvalidateRect(state.window, null(), FALSE) };
                return;
            }
            state.macro_lane = MacroLane::OnPress;
            state.macro_selected_event = None;
            state.macro_dirty = true;
            state.macro_status_kind = StatusKind::Ready;
            state.macro_status = "Macro baru dibuat. Beri nama lalu rekam timeline.".to_owned();
            unsafe {
                sync_macro_name_edit(state);
                sync_delay_edit(state);
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
                state.macro_selected_event = None;
                state.macro_status_kind = StatusKind::Ready;
                state.macro_status = format!("Mengedit {}.", item.name);
                unsafe {
                    sync_macro_name_edit(state);
                    sync_delay_edit(state);
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
                unsafe {
                    refresh_macro_hooks(state);
                    InvalidateRect(state.window, null(), FALSE);
                };
            }
        }
        HitTarget::MacroLane(lane) => {
            if !state.recording {
                state.macro_lane = lane;
                state.macro_selected_event = None;
                unsafe {
                    refresh_macro_hooks(state);
                    sync_delay_edit(state);
                    InvalidateRect(state.window, null(), FALSE);
                };
            }
        }
        HitTarget::MacroEvent(index) => {
            if !state.recording {
                state.macro_selected_event = Some(index);
                state.macro_status_kind = StatusKind::Ready;
                state.macro_status = if selected_delay(state).is_some() {
                    "Delay dipilih. Edit nilainya atau atur urutan di panel kanan.".to_owned()
                } else {
                    "Langkah dipilih. Gunakan naik, turun, salin, atau hapus.".to_owned()
                };
                unsafe {
                    sync_delay_edit(state);
                    InvalidateRect(state.window, null(), FALSE);
                }
            }
        }
        HitTarget::MacroScopeGlobal => {
            if !state.recording {
                let selected_id = state.macro_library.selected_id;
                if let Some(item) = state.macro_library.selected_mut() {
                    item.target = None;
                }
                state.macro_targets.remove(&selected_id);
                state.macro_dirty = true;
                state.macro_status_kind = StatusKind::Warning;
                state.macro_status =
                    "Output global dipilih; macro akan mengikuti aplikasi aktif.".to_owned();
                unsafe { InvalidateRect(state.window, null(), FALSE) };
            }
        }
        HitTarget::MacroScopeTarget => {
            if !state.recording
                && state
                    .macro_library
                    .selected()
                    .is_some_and(|item| item.target.is_none())
            {
                unsafe { begin_macro_target_capture(state) };
            }
        }
        HitTarget::MacroTargetPick => {
            if !state.recording {
                unsafe { begin_macro_target_capture(state) };
            }
        }
        HitTarget::MacroDelayMinus => {
            if let Some(value) = selected_delay(state) {
                unsafe { set_selected_delay(state, value.saturating_sub(10)) };
            }
        }
        HitTarget::MacroDelayPlus => {
            if let Some(value) = selected_delay(state) {
                unsafe { set_selected_delay(state, value.saturating_add(10).min(60_000)) };
            }
        }
        HitTarget::MacroDelayApply => {
            if selected_delay(state).is_some() {
                unsafe { apply_delay_edit(state) };
            }
        }
        HitTarget::MacroEventUp | HitTarget::MacroEventDown => {
            if !state.recording {
                let Some(index) = state.macro_selected_event else {
                    return;
                };
                let direction = if target == HitTarget::MacroEventUp {
                    -1
                } else {
                    1
                };
                let moved = selected_lane_mut(state)
                    .and_then(|events| move_event(events, index, direction));
                if let Some(next) = moved {
                    state.macro_selected_event = Some(next);
                    state.macro_dirty = true;
                    state.macro_status_kind = StatusKind::Ready;
                    state.macro_status = "Urutan langkah diperbarui.".to_owned();
                } else {
                    state.macro_status_kind = StatusKind::Warning;
                    state.macro_status = "Langkah sudah berada di batas timeline.".to_owned();
                }
                unsafe {
                    sync_delay_edit(state);
                    InvalidateRect(state.window, null(), FALSE);
                }
            }
        }
        HitTarget::MacroEventDuplicate => {
            if !state.recording {
                let Some(index) = state.macro_selected_event else {
                    return;
                };
                let duplicated =
                    selected_lane_mut(state).and_then(|events| duplicate_event(events, index));
                if let Some(next) = duplicated {
                    state.macro_selected_event = Some(next);
                    state.macro_dirty = true;
                    state.macro_status_kind = StatusKind::Ready;
                    state.macro_status = "Langkah disalin tepat setelah pilihan.".to_owned();
                }
                unsafe {
                    sync_delay_edit(state);
                    InvalidateRect(state.window, null(), FALSE);
                }
            }
        }
        HitTarget::MacroEventDelete => {
            if !state.recording {
                let Some(index) = state.macro_selected_event else {
                    return;
                };
                let next = selected_lane_mut(state).and_then(|events| delete_event(events, index));
                state.macro_selected_event = next;
                state.macro_dirty = true;
                state.macro_status_kind = StatusKind::Warning;
                state.macro_status =
                    "Langkah dihapus. Simpan untuk membuatnya permanen.".to_owned();
                unsafe {
                    refresh_macro_hooks(state);
                    sync_delay_edit(state);
                    InvalidateRect(state.window, null(), FALSE);
                }
            }
        }
        HitTarget::MacroInsertDelay => {
            if !state.recording {
                let after = state.macro_selected_event;
                let inserted =
                    selected_lane_mut(state).and_then(|events| insert_delay(events, after, 100));
                if let Some(index) = inserted {
                    state.macro_selected_event = Some(index);
                    state.macro_dirty = true;
                    state.macro_status_kind = StatusKind::Ready;
                    state.macro_status = "Delay 100 ms disisipkan dan siap diedit.".to_owned();
                    unsafe {
                        sync_delay_edit(state);
                        InvalidateRect(state.window, null(), FALSE);
                    }
                }
            }
        }
        HitTarget::MacroDuplicate => {
            if !state.recording && state.macro_library.duplicate_selected().is_some() {
                state.macro_lane = MacroLane::OnPress;
                state.macro_selected_event = None;
                state.macro_dirty = true;
                state.macro_status_kind = StatusKind::Ready;
                state.macro_status = "Macro disalin. Ubah nama lalu simpan.".to_owned();
                unsafe {
                    sync_macro_name_edit(state);
                    sync_delay_edit(state);
                    InvalidateRect(state.window, null(), FALSE);
                }
            }
        }
        HitTarget::MacroDelete => {
            if !state.recording {
                let selected = state
                    .macro_library
                    .selected()
                    .map(|item| (item.id, item.name.clone()));
                if let Some((id, name)) = selected
                    && unsafe { confirm_delete_macro(state.window, &name) }
                    && state.macro_library.delete_selected()
                {
                    state.macro_targets.remove(&id);
                    let macro_ids: Vec<u32> = state
                        .macro_library
                        .macros
                        .iter()
                        .map(|item| item.id)
                        .collect();
                    state.profile_library.remove_missing_macro_links(&macro_ids);
                    let _ = save_profiles(&state.profiles_path, &state.profile_library);
                    state.macro_lane = MacroLane::OnPress;
                    state.macro_selected_event = None;
                    state.macro_dirty = true;
                    state.macro_status_kind = StatusKind::Warning;
                    state.macro_status = format!("{name} dihapus. Simpan untuk permanen.");
                    unsafe {
                        refresh_macro_hooks(state);
                        sync_macro_name_edit(state);
                        sync_delay_edit(state);
                        InvalidateRect(state.window, null(), FALSE);
                    }
                } else if state.macro_library.macros.len() <= 1 {
                    state.macro_status_kind = StatusKind::Warning;
                    state.macro_status = "Minimal satu macro harus tetap tersedia.".to_owned();
                    unsafe { InvalidateRect(state.window, null(), FALSE) };
                }
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
                state.macro_selected_event = None;
                state.macro_dirty = true;
                state.macro_status_kind = StatusKind::Warning;
                state.macro_status =
                    "Lane dibersihkan. Tekan Simpan untuk mempertahankan perubahan.".to_owned();
                unsafe {
                    refresh_macro_hooks(state);
                    sync_delay_edit(state);
                    InvalidateRect(state.window, null(), FALSE);
                };
            }
        }
        HitTarget::MacroSave => {
            if !state.recording {
                unsafe { save_current_macro(state) };
            }
        }
        HitTarget::SettingMinimizeTray => {
            state.settings.minimize_to_tray = !state.settings.minimize_to_tray;
            unsafe { persist_settings(state, "Mode minimize-to-tray diperbarui.") };
        }
        HitTarget::SettingCloseTray => {
            state.settings.close_to_tray = !state.settings.close_to_tray;
            unsafe { persist_settings(state, "Perilaku tombol X diperbarui.") };
        }
        HitTarget::SettingAutoStart => {
            let enabled = !state.settings.auto_start;
            match unsafe { configure_auto_start(enabled) } {
                Ok(()) => {
                    state.settings.auto_start = enabled;
                    unsafe {
                        persist_settings(
                            state,
                            if enabled {
                                "VibeTimer akan mulai di tray bersama Windows."
                            } else {
                                "Auto Start dinonaktifkan."
                            },
                        )
                    };
                }
                Err(message) => {
                    state.settings_status_kind = StatusKind::Error;
                    state.settings_status = message;
                    unsafe { InvalidateRect(state.window, null(), FALSE) };
                }
            }
        }
        HitTarget::SettingEmergencyHotkey(hotkey) => {
            let previous = state.settings.emergency_hotkey;
            state.settings.emergency_hotkey = hotkey;
            match unsafe { register_emergency_hotkey(state) } {
                Ok(()) => {
                    unsafe { persist_settings(state, "Hotkey Emergency Stop diperbarui.") };
                }
                Err(message) => {
                    state.settings.emergency_hotkey = previous;
                    let _ = unsafe { register_emergency_hotkey(state) };
                    state.settings_status_kind = StatusKind::Error;
                    state.settings_status = message;
                    unsafe { InvalidateRect(state.window, null(), FALSE) };
                }
            }
        }
        HitTarget::SettingEmergencyTimers => {
            state.settings.emergency_stops_timers = !state.settings.emergency_stops_timers;
            unsafe { persist_settings(state, "Cakupan Emergency Stop diperbarui.") };
        }
        HitTarget::SettingMaxRuntime(value) => {
            state.settings.max_macro_runtime_seconds = value;
            unsafe { persist_settings(state, "Batas durasi macro diperbarui.") };
        }
        HitTarget::SettingMaxRepeats(value) => {
            state.settings.max_macro_repeats = value;
            unsafe { persist_settings(state, "Batas pengulangan macro diperbarui.") };
        }
        HitTarget::SettingTestEmergency => unsafe { emergency_stop(state, "tombol Settings") },
        HitTarget::ProfileNew => {
            if state.profile_library.add_profile().is_none() {
                state.profile_status_kind = StatusKind::Warning;
                state.profile_status =
                    "Maksimal 6 profil agar seluruh target tetap terlihat.".to_owned();
                unsafe { InvalidateRect(state.window, null(), FALSE) };
                return;
            }
            state.profile_dirty = true;
            state.profile_status_kind = StatusKind::Ready;
            state.profile_status = "Profil baru dibuat. Beri nama dan pilih target.".to_owned();
            unsafe {
                sync_profile_name_edit(state);
                InvalidateRect(state.window, null(), FALSE);
            }
        }
        HitTarget::ProfileItem(index) => {
            if let Some(profile) = state.profile_library.profiles.get(index) {
                state.profile_library.selected_id = profile.id;
                state.profile_status_kind = StatusKind::Ready;
                state.profile_status = format!("Mengedit {}.", profile.name);
                unsafe {
                    sync_profile_name_edit(state);
                    InvalidateRect(state.window, null(), FALSE);
                }
            }
        }
        HitTarget::ProfileDuplicate => {
            if state.profile_library.duplicate_selected().is_some() {
                state.profile_dirty = true;
                state.profile_status_kind = StatusKind::Ready;
                state.profile_status = "Profil disalin. Ubah nama lalu simpan.".to_owned();
                unsafe {
                    sync_profile_name_edit(state);
                    InvalidateRect(state.window, null(), FALSE);
                }
            }
        }
        HitTarget::ProfileDelete => {
            let selected = state
                .profile_library
                .selected()
                .map(|profile| (profile.id, profile.name.clone()));
            if let Some((id, name)) = selected
                && unsafe { confirm_delete_profile(state.window, &name) }
                && state.profile_library.delete_selected()
            {
                state.profile_targets.remove(&id);
                state.profile_dirty = true;
                state.profile_status_kind = StatusKind::Warning;
                state.profile_status = format!("Profil {name} dihapus. Macro tetap tersedia.");
                unsafe {
                    sync_profile_name_edit(state);
                    persist_profiles_and_macros(state, "Profil dihapus dan library diperbarui.");
                }
            } else if state.profile_library.profiles.len() <= 1 {
                state.profile_status_kind = StatusKind::Warning;
                state.profile_status = "Minimal satu profil harus tetap tersedia.".to_owned();
                unsafe { InvalidateRect(state.window, null(), FALSE) };
            }
        }
        HitTarget::ProfileTargetPick => unsafe { begin_profile_target_capture(state) },
        HitTarget::ProfileMacro(index) => {
            let Some(macro_id) = state.macro_library.macros.get(index).map(|item| item.id) else {
                return;
            };
            let Some(target_specification) = state
                .profile_library
                .selected()
                .and_then(|profile| profile.target.clone())
            else {
                state.profile_status_kind = StatusKind::Error;
                state.profile_status = "Pilih target profil sebelum menautkan macro.".to_owned();
                unsafe { InvalidateRect(state.window, null(), FALSE) };
                return;
            };
            let linked = state
                .profile_library
                .selected_mut()
                .is_some_and(|profile| profile.toggle_macro(macro_id));
            if linked
                && let Some(item) = state
                    .macro_library
                    .macros
                    .iter_mut()
                    .find(|item| item.id == macro_id)
            {
                item.target = Some(target_specification);
                state.macro_targets.remove(&macro_id);
            }
            state.profile_dirty = true;
            state.macro_dirty = true;
            unsafe {
                persist_profiles_and_macros(
                    state,
                    if linked {
                        "Macro ditautkan dan targetnya disinkronkan."
                    } else {
                        "Tautan dilepas; target macro yang aman tetap dipertahankan."
                    },
                )
            };
        }
        HitTarget::ProfileUseTimer => {
            let Some(specification) = state
                .profile_library
                .selected()
                .and_then(|profile| profile.target.clone())
            else {
                state.profile_status_kind = StatusKind::Error;
                state.profile_status = "Profil belum memiliki target.".to_owned();
                unsafe { InvalidateRect(state.window, null(), FALSE) };
                return;
            };
            if let Ok(target) = unsafe { find_saved_macro_target(&specification) } {
                state.target = Some(TargetWindow {
                    window: target.root,
                    process_id: target.process_id,
                    title: target.title.clone(),
                    executable: specification.executable.clone(),
                });
                if let Some(timer) = state.timer_library.selected_mut() {
                    timer.target = Some(specification.clone());
                }
                state.timer_dirty = true;
                let _ = save_timers(&state.timers_path, &state.timer_library);
                state
                    .profile_targets
                    .insert(state.profile_library.selected_id, target);
                state.profile_status_kind = StatusKind::Sent;
                state.profile_status = "Target profil dipasang ke Timer.".to_owned();
            } else {
                state.profile_status_kind = StatusKind::Error;
                state.profile_status =
                    "Aplikasi profil belum terbuka atau judul window berubah.".to_owned();
            }
            unsafe { InvalidateRect(state.window, null(), FALSE) };
        }
        HitTarget::ProfileSave => unsafe { save_current_profile(state) },
        HitTarget::BackupExport => {
            if let Some(path) = unsafe { choose_backup_path(state.window, true) } {
                unsafe { export_backup_to_path(state, &path) };
            }
        }
        HitTarget::BackupImport => {
            if let Some(path) = unsafe { choose_backup_path(state.window, false) }
                && unsafe { confirm_import_backup(state.window) }
            {
                unsafe { import_backup_from_path(state, &path) };
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
                if let Err(message) = add_tray_icon(state) {
                    state.settings_status_kind = StatusKind::Error;
                    state.settings_status = message;
                }
                if let Err(message) = register_emergency_hotkey(state) {
                    state.settings_status_kind = StatusKind::Error;
                    state.settings_status = message;
                }
                if state.settings.auto_start
                    && let Err(message) = configure_auto_start(true)
                {
                    state.settings_status_kind = StatusKind::Warning;
                    state.settings_status = message;
                }
                if state.timer_library.running_count() > 0 {
                    SetTimer(state.window, TIMER_COUNTDOWN, 100, null());
                }
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
        WM_SIZE => {
            if wparam == SIZE_MINIMIZED && state.settings.minimize_to_tray {
                unsafe { ShowWindow(window, SW_HIDE) };
            }
            0
        }
        WM_CTLCOLOREDIT | WM_CTLCOLORSTATIC => {
            let dc = wparam as Hdc;
            let panel_editor = lparam == state.macro_name_edit
                || lparam == state.macro_delay_edit
                || lparam == state.profile_name_edit
                || lparam == state.timer_name_edit
                || lparam == state.smart_reset_edit;
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
            if control_id == 107 && notification == 0x0300 && state.tab == AppTab::Profiles {
                state.profile_dirty = true;
                unsafe { InvalidateRect(window, null(), FALSE) };
            }
            if matches!(control_id, 101 | 102 | 103 | 104 | 108)
                && notification == 0x0300
                && state.tab == AppTab::Timer
                && !state.running
            {
                state.timer_dirty = true;
                unsafe { InvalidateRect(window, null(), FALSE) };
            }
            match wparam & 0xFFFF {
                MENU_OPEN => unsafe { restore_from_tray(state) },
                MENU_STOP_ALL => unsafe { emergency_stop(state, "menu tray") },
                MENU_EXIT => {
                    state.exit_requested = true;
                    unsafe { PostMessageW(window, WM_CLOSE, 0, 0) };
                }
                _ => {}
            }
            0
        }
        WM_HOTKEY => {
            if wparam as i32 == EMERGENCY_HOTKEY_ID {
                unsafe { emergency_stop(state, "hotkey global") };
            }
            0
        }
        WM_APP_TRAY => {
            match lparam as Uint {
                WM_LBUTTONUP | WM_LBUTTONDOWN => unsafe { restore_from_tray(state) },
                WM_RBUTTONUP => unsafe { show_tray_menu(state) },
                _ => {}
            }
            0
        }
        WM_APP_MACRO_DONE => {
            if wparam == 1 {
                state.macro_status_kind = StatusKind::Sent;
                state.macro_status = "Macro selesai dijalankan.".to_owned();
            } else if wparam == 2 {
                state.macro_status_kind = StatusKind::Error;
                state.macro_status =
                    "Macro berhenti karena window target ditutup atau berubah.".to_owned();
            } else if wparam == 3 {
                state.macro_status_kind = StatusKind::Warning;
                state.macro_status =
                    "Macro berhenti otomatis setelah mencapai batas aman Settings.".to_owned();
                unsafe {
                    show_tray_notification(
                        state,
                        "Batas macro tercapai",
                        "Macro dihentikan otomatis oleh batas durasi atau pengulangan.",
                    )
                };
            } else if wparam == 4 {
                state.macro_status_kind = StatusKind::Warning;
                state.macro_status = "Macro dihentikan oleh Emergency Stop.".to_owned();
            } else {
                state.macro_status_kind = StatusKind::Error;
                state.macro_status =
                    "Macro kosong atau Windows menolak salah satu input.".to_owned();
            }
            unsafe { InvalidateRect(window, null(), FALSE) };
            0
        }
        WM_TIMER => {
            if wparam == TIMER_COUNTDOWN && state.timer_library.running_count() > 0 {
                unsafe { update_countdown(state) };
            } else if wparam == TIMER_CAPTURE
                && state
                    .capture_deadline
                    .is_some_and(|deadline| Instant::now() >= deadline)
            {
                match state.capture_kind {
                    CaptureKind::Timer => unsafe { finish_target_capture(state) },
                    CaptureKind::Macro => unsafe { finish_macro_target_capture(state) },
                    CaptureKind::Profile => unsafe { finish_profile_target_capture(state) },
                }
            }
            0
        }
        WM_CLOSE => {
            if state.settings.close_to_tray && !state.exit_requested {
                unsafe {
                    ShowWindow(window, SW_HIDE);
                    show_tray_notification(
                        state,
                        "VibeTimer tetap aktif",
                        "Timer dan macro tetap berjalan. Klik ikon tray untuk membuka kembali.",
                    );
                }
                return 0;
            }
            if state.recording {
                unsafe { stop_macro_recording(state) };
            }
            if state.timer_library.running_count() > 0 {
                let response = unsafe {
                    MessageBoxW(
                        window,
                        wide("Satu atau lebih timer masih aktif. Tutup VibeTimer? Timer masa depan akan dilanjutkan saat aplikasi dibuka lagi; timer yang terlewat tidak akan mengirim input.")
                            .as_ptr(),
                        wide("Multi Timer sedang berjalan").as_ptr(),
                        MB_YESNO | MB_ICONWARNING,
                    )
                };
                if response != IDYES {
                    return 0;
                }
            }
            if state.timer_dirty && !state.running && !unsafe { save_selected_timer(state) } {
                unsafe {
                    show_warning(
                        window,
                        "Perubahan timer belum valid atau belum dapat disimpan. Perbaiki dahulu sebelum keluar.",
                    )
                };
                return 0;
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
            if state.profile_dirty {
                let response = unsafe {
                    MessageBoxW(
                        window,
                        wide("Ada perubahan profil yang belum disimpan. Tutup tanpa menyimpan?")
                            .as_ptr(),
                        wide("Profil belum disimpan").as_ptr(),
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

        let mutex_name = wide("Local\\VibeTimer.SingleInstance.v1");
        let mutex = CreateMutexW(null_mut(), TRUE, mutex_name.as_ptr());
        if mutex == 0 {
            return Err("Tidak dapat membuat pengunci single-instance.".to_owned());
        }
        let already_running = GetLastError() == ERROR_ALREADY_EXISTS;
        let _mutex_guard = OwnedKernelHandle(mutex);
        if already_running {
            let class_name = wide("VibeTimerWindowClass");
            let existing = FindWindowW(class_name.as_ptr(), null());
            if existing != 0 {
                ShowWindow(existing, SW_RESTORE);
                SetForegroundWindow(existing);
            }
            return Ok(());
        }

        let instance = GetModuleHandleW(null());
        if instance == 0 {
            return Err("Tidak dapat memperoleh handle aplikasi.".to_owned());
        }

        let window = create_main_window(instance)?;
        let start_in_background = std::env::args_os().any(|argument| argument == "--background");
        ShowWindow(
            window,
            if start_in_background {
                SW_HIDE
            } else {
                SW_SHOWNORMAL
            },
        );
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

    static BACKGROUND_KEY_DOWNS: AtomicUsize = AtomicUsize::new(0);
    static BACKGROUND_MOUSE_DOWNS: AtomicUsize = AtomicUsize::new(0);

    #[test]
    fn macro_safety_limits_are_enforced_and_can_be_disabled() {
        let now = Instant::now();
        assert!(playback_limit_check(now, 99, 0, 100).is_ok());
        assert!(playback_limit_check(now, 100, 0, 100).is_err());
        let old = now
            .checked_sub(Duration::from_secs(2))
            .expect("Instant dapat dikurangi");
        assert!(playback_limit_check(old, 0, 1, 0).is_err());
        assert!(playback_limit_check(old, u32::MAX, 0, 0).is_ok());
    }

    unsafe extern "system" fn background_target_proc(
        window: Hwnd,
        message: Uint,
        wparam: Wparam,
        lparam: Lparam,
    ) -> Lresult {
        match message {
            WM_KEYDOWN => {
                BACKGROUND_KEY_DOWNS.fetch_add(1, Ordering::Relaxed);
                0
            }
            WM_LBUTTONDOWN => {
                BACKGROUND_MOUSE_DOWNS.fetch_add(1, Ordering::Relaxed);
                0
            }
            _ => unsafe { DefWindowProcW(window, message, wparam, lparam) },
        }
    }

    unsafe fn create_background_target(instance: Hinstance, title: &str) -> Hwnd {
        let class_name = wide("VibeTimerBackgroundTargetClass");
        let class = WndClassW {
            style: 0,
            wnd_proc: Some(background_target_proc),
            class_extra: 0,
            window_extra: 0,
            instance,
            icon: 0,
            cursor: unsafe { LoadCursorW(0, IDC_ARROW as *const u16) },
            background: 0,
            menu_name: null(),
            class_name: class_name.as_ptr(),
        };
        unsafe { RegisterClassW(&class) };
        unsafe {
            CreateWindowExW(
                0,
                class_name.as_ptr(),
                wide(title).as_ptr(),
                WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU | WS_VISIBLE,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                420,
                260,
                0,
                0,
                instance,
                null_mut(),
            )
        }
    }

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
            state.macro_library = MacroLibrary::default();
            sync_macro_name_edit(state);
            assert!(refresh_macro_hooks(state));
            assert_eq!(
                state.keyboard_hook, 0,
                "macro kosong tidak boleh memasang keyboard hook"
            );
            assert_eq!(
                state.mouse_hook, 0,
                "macro kosong tidak boleh memasang mouse hook"
            );

            assert_eq!(hit_test(230, 39, state), HitTarget::TimerTab);
            assert_eq!(hit_test(292, 39, state), HitTarget::MacroTab);
            assert_eq!(hit_test(364, 39, state), HitTarget::ProfilesTab);
            assert_eq!(hit_test(451, 39, state), HitTarget::SettingsTab);
            assert_eq!(hit_test(100, 240, state), HitTarget::AddThirtyMinutes);
            assert_eq!(hit_test(210, 240, state), HitTarget::AddOneHour);
            assert_eq!(hit_test(330, 240, state), HitTarget::AddThreeHours);
            assert_eq!(hit_test(410, 330, state), HitTarget::PickTarget);
            assert_eq!(hit_test(145, 442, state), HitTarget::EnterOnly);
            assert_eq!(hit_test(350, 442, state), HitTarget::TextAndEnter);
            assert_eq!(hit_test(260, 575, state), HitTarget::MainAction);
            assert_eq!(hit_test(580, 137, state), HitTarget::TimerNew);
            assert_eq!(hit_test(700, 194, state), HitTarget::TimerItem(0));
            assert_eq!(hit_test(575, 529, state), HitTarget::TimerDuplicate);
            assert_eq!(hit_test(695, 529, state), HitTarget::TimerDelete);
            assert_eq!(hit_test(818, 529, state), HitTarget::TimerSave);
            assert_eq!(hit_test(600, 628, state), HitTarget::SmartResetClipboard);
            assert_eq!(hit_test(790, 628, state), HitTarget::SmartResetApply);

            state.profile_library = ProfileLibrary::default();
            sync_profile_name_edit(state);
            let e2e_profiles_path = PathBuf::from("qa/e2e-profiles.vtp");
            let e2e_profile_macros_path = PathBuf::from("qa/e2e-profile-macros.vtm");
            let e2e_settings_path = PathBuf::from("qa/e2e-settings.vts");
            let e2e_timers_path = PathBuf::from("qa/e2e-timers.vtt");
            let e2e_backup_path = PathBuf::from("qa/e2e-backup.vtb");
            for path in [
                &e2e_profiles_path,
                &e2e_profile_macros_path,
                &e2e_settings_path,
                &e2e_timers_path,
                &e2e_backup_path,
            ] {
                let _ = fs::remove_file(path);
            }
            state.profiles_path = e2e_profiles_path.clone();
            state.macro_path = e2e_profile_macros_path.clone();
            state.settings_path = e2e_settings_path.clone();
            state.timer_library = TimerLibrary::default();
            state.timers_path = e2e_timers_path.clone();
            sync_selected_timer_to_controls(state);
            state.timer_dirty = false;
            switch_tab(state, AppTab::Profiles);
            assert_eq!(hit_test(100, 137, state), HitTarget::ProfileNew);
            assert_eq!(hit_test(84, 568, state), HitTarget::ProfileDuplicate);
            assert_eq!(hit_test(176, 568, state), HitTarget::ProfileDelete);
            assert_eq!(hit_test(410, 262, state), HitTarget::ProfileUseTimer);
            assert_eq!(hit_test(700, 262, state), HitTarget::ProfileTargetPick);
            assert_eq!(hit_test(410, 364, state), HitTarget::ProfileMacro(0));
            assert_eq!(hit_test(360, 570, state), HitTarget::BackupExport);
            assert_eq!(hit_test(540, 570, state), HitTarget::BackupImport);
            assert_eq!(hit_test(740, 570, state), HitTarget::ProfileSave);
            let executable =
                process_executable_name(GetCurrentProcessId()).expect("executable test ditemukan");
            state.profile_library.selected_mut().unwrap().target = Some(MacroTarget {
                executable,
                window_title: "VibeTimer E2E Target".to_owned(),
            });
            handle_click(state, HitTarget::ProfileMacro(0));
            assert!(state.profile_library.selected().unwrap().contains_macro(1));
            assert!(state.macro_library.selected().unwrap().target.is_some());
            handle_click(state, HitTarget::ProfileUseTimer);
            assert_eq!(
                state.target.as_ref().map(|target| target.title.as_str()),
                Some("VibeTimer E2E Target")
            );
            SetWindowTextW(state.profile_name_edit, wide("AI Workspace").as_ptr());
            handle_click(state, HitTarget::ProfileSave);
            assert_eq!(
                state.profile_library.selected().unwrap().name,
                "AI Workspace"
            );
            export_backup_to_path(state, &e2e_backup_path);
            assert_eq!(state.profile_status_kind, StatusKind::Sent);
            state.profile_library.selected_mut().unwrap().name = "Rusak".to_owned();
            import_backup_from_path(state, &e2e_backup_path);
            assert_eq!(
                state.profile_library.selected().unwrap().name,
                "AI Workspace"
            );
            handle_click(state, HitTarget::ProfileDuplicate);
            assert_eq!(state.profile_library.profiles.len(), 2);
            handle_click(state, HitTarget::ProfileDelete);
            assert_eq!(state.profile_library.profiles.len(), 1);
            InvalidateRect(main_window, null(), FALSE);
            UpdateWindow(main_window);
            pump_messages_for(Duration::from_millis(120));
            save_window_bmp(main_window, Path::new("qa/vibetimer-profiles.bmp"))
                .expect("snapshot Profiles dibuat");

            state.macro_library = MacroLibrary::default();
            state.macro_targets.clear();
            sync_macro_name_edit(state);

            switch_tab(state, AppTab::Settings);
            assert_eq!(hit_test(210, 200, state), HitTarget::SettingMinimizeTray);
            assert_eq!(hit_test(210, 260, state), HitTarget::SettingCloseTray);
            assert_eq!(hit_test(210, 320, state), HitTarget::SettingAutoStart);
            assert_eq!(
                hit_test(600, 246, state),
                HitTarget::SettingEmergencyHotkey(EmergencyHotkey::CtrlShiftF12)
            );
            handle_click(state, HitTarget::SettingMinimizeTray);
            assert!(!state.settings.minimize_to_tray);
            handle_click(state, HitTarget::SettingMinimizeTray);
            assert!(state.settings.minimize_to_tray);
            handle_click(state, HitTarget::SettingCloseTray);
            assert!(!state.settings.close_to_tray);
            handle_click(state, HitTarget::SettingCloseTray);
            assert!(state.settings.close_to_tray);
            handle_click(state, HitTarget::SettingAutoStart);
            assert!(state.settings.auto_start);
            assert!(TEST_AUTOSTART_ENABLED.load(Ordering::Acquire));
            handle_click(state, HitTarget::SettingAutoStart);
            assert!(!state.settings.auto_start);
            assert!(!TEST_AUTOSTART_ENABLED.load(Ordering::Acquire));
            handle_click(
                state,
                HitTarget::SettingEmergencyHotkey(EmergencyHotkey::CtrlShiftF12),
            );
            assert_eq!(
                state.settings.emergency_hotkey,
                EmergencyHotkey::CtrlShiftF12
            );
            handle_click(
                state,
                HitTarget::SettingEmergencyHotkey(EmergencyHotkey::CtrlAltF12),
            );
            handle_click(state, HitTarget::SettingEmergencyTimers);
            assert!(!state.settings.emergency_stops_timers);
            handle_click(state, HitTarget::SettingEmergencyTimers);
            handle_click(state, HitTarget::SettingMaxRuntime(5 * 60));
            handle_click(state, HitTarget::SettingMaxRepeats(100));
            assert_eq!(state.settings.max_macro_runtime_seconds, 5 * 60);
            assert_eq!(state.settings.max_macro_repeats, 100);
            handle_click(state, HitTarget::SettingMaxRuntime(30 * 60));
            handle_click(state, HitTarget::SettingMaxRepeats(10_000));
            InvalidateRect(main_window, null(), FALSE);
            UpdateWindow(main_window);
            pump_messages_for(Duration::from_millis(120));
            save_window_bmp(main_window, Path::new("qa/vibetimer-settings.bmp"))
                .expect("snapshot Settings dibuat");
            ShowWindow(main_window, SW_SHOWNORMAL);
            SendMessageW(main_window, WM_SIZE, SIZE_MINIMIZED, 0);
            assert_eq!(IsWindowVisible(main_window), FALSE);
            restore_from_tray(state);
            assert_ne!(IsWindowVisible(main_window), FALSE);
            switch_tab(state, AppTab::Timer);

            SetWindowTextW(
                state.smart_reset_edit,
                wide("Resets in 3 h 27 min").as_ptr(),
            );
            handle_click(state, HitTarget::SmartResetApply);
            assert_eq!(
                read_duration_fields(state),
                DurationFields::new(3, 27, 0),
                "Smart Reset harus menerapkan durasi Claude"
            );
            assert_eq!(state.status_kind, StatusKind::Sent);

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
                executable: "VibeTimer-test.exe".to_owned(),
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
                executable: "invalid.exe".to_owned(),
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

            // Dua timer benar-benar berjalan bersamaan dan masing-masing hanya mengirim sekali.
            state.target = Some(TargetWindow {
                window: target_window,
                process_id: GetCurrentProcessId(),
                title: "VibeTimer E2E Target".to_owned(),
                executable: "VibeTimer-test.exe".to_owned(),
            });
            state.action_mode = ActionMode::EnterOnly;
            set_duration_fields(state, DurationFields::new(0, 0, 1));
            begin_timer(state);
            assert_eq!(state.timer_library.running_count(), 1);
            handle_click(state, HitTarget::TimerNew);
            assert_eq!(state.timer_library.timers.len(), 2);
            state.target = Some(TargetWindow {
                window: target_window,
                process_id: GetCurrentProcessId(),
                title: "VibeTimer E2E Target".to_owned(),
                executable: "VibeTimer-test.exe".to_owned(),
            });
            state.action_mode = ActionMode::EnterOnly;
            set_duration_fields(state, DurationFields::new(0, 0, 2));
            begin_timer(state);
            assert_eq!(state.timer_library.running_count(), 2);
            pump_messages_for(Duration::from_millis(2_450));
            assert_eq!(state.timer_library.running_count(), 0);
            assert!(
                state
                    .timer_library
                    .timers
                    .iter()
                    .all(|timer| timer.phase == TimerPhase::Completed),
                "kedua timer harus selesai satu kali"
            );
            assert!(
                get_window_text(target_edit).len() >= 4,
                "dua timer Enter harus menghasilkan dua baris"
            );
            InvalidateRect(main_window, null(), FALSE);
            UpdateWindow(main_window);
            pump_messages_for(Duration::from_millis(100));
            save_window_bmp(main_window, Path::new("qa/vibetimer-multi-timer.bmp"))
                .expect("snapshot Multi Timer dibuat");
            handle_click(state, HitTarget::TimerDuplicate);
            assert_eq!(state.timer_library.timers.len(), 3);
            handle_click(state, HitTarget::TimerDelete);
            assert_eq!(state.timer_library.timers.len(), 2);

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
            assert_eq!(hit_test(975, 185, state), HitTarget::MacroScopeGlobal);
            assert_eq!(hit_test(1050, 185, state), HitTarget::MacroScopeTarget);
            assert_eq!(hit_test(1010, 235, state), HitTarget::MacroTargetPick);
            assert_eq!(hit_test(965, 410, state), HitTarget::MacroDelayMinus);
            assert_eq!(hit_test(1060, 410, state), HitTarget::MacroDelayPlus);
            assert_eq!(hit_test(1010, 460, state), HitTarget::MacroDelayApply);
            assert_eq!(hit_test(975, 504, state), HitTarget::MacroEventUp);
            assert_eq!(hit_test(1050, 504, state), HitTarget::MacroEventDown);
            assert_eq!(hit_test(690, 570, state), HitTarget::MacroInsertDelay);
            assert_eq!(hit_test(84, 568, state), HitTarget::MacroDuplicate);
            assert_eq!(hit_test(176, 568, state), HitTarget::MacroDelete);

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
            assert_ne!(
                state.keyboard_hook, 0,
                "recording harus memasang keyboard hook"
            );
            assert_ne!(state.mouse_hook, 0, "recording harus memasang mouse hook");
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
            assert_ne!(
                state.keyboard_hook, 0,
                "macro F8 berisi aksi harus mempertahankan keyboard hook"
            );
            assert_eq!(
                state.mouse_hook, 0,
                "macro F8 tidak membutuhkan mouse hook setelah recording"
            );
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
            assert_eq!(
                hit_test(330, 405, state),
                HitTarget::MacroEvent(0),
                "chip delay harus menjadi kontrol yang dapat dipilih"
            );
            handle_click(state, HitTarget::MacroEvent(0));
            assert_eq!(selected_delay(state), Some(25));
            assert_eq!(get_window_text(state.macro_delay_edit), "25");
            SetWindowTextW(state.macro_delay_edit, wide("73").as_ptr());
            handle_click(state, HitTarget::MacroDelayApply);
            assert_eq!(selected_delay(state), Some(73));
            handle_click(state, HitTarget::MacroDelayPlus);
            assert_eq!(selected_delay(state), Some(83));
            handle_click(state, HitTarget::MacroDelayMinus);
            assert_eq!(selected_delay(state), Some(73));
            assert_eq!(
                state
                    .macro_library
                    .selected()
                    .expect("macro delay tersedia")
                    .standard_delay_ms,
                None,
                "edit per langkah harus mematikan override delay lama"
            );
            handle_click(state, HitTarget::MacroEvent(1));
            assert_eq!(state.macro_selected_event, Some(1));
            handle_click(state, HitTarget::MacroEventDuplicate);
            assert_eq!(
                state
                    .macro_library
                    .selected()
                    .expect("macro tersedia")
                    .on_press
                    .len(),
                4
            );
            handle_click(state, HitTarget::MacroEventUp);
            assert_eq!(state.macro_selected_event, Some(1));
            handle_click(state, HitTarget::MacroEventDown);
            assert_eq!(state.macro_selected_event, Some(2));
            handle_click(state, HitTarget::MacroEventDelete);
            assert_eq!(
                state
                    .macro_library
                    .selected()
                    .expect("macro tersedia")
                    .on_press
                    .len(),
                3
            );
            handle_click(state, HitTarget::MacroInsertDelay);
            assert_eq!(selected_delay(state), Some(100));
            handle_click(state, HitTarget::MacroEventDelete);
            assert_eq!(
                state
                    .macro_library
                    .selected()
                    .expect("macro tersedia")
                    .on_press
                    .len(),
                3
            );
            handle_click(state, HitTarget::MacroDuplicate);
            assert_eq!(state.macro_library.macros.len(), 3);
            handle_click(state, HitTarget::MacroDelete);
            assert_eq!(state.macro_library.macros.len(), 2);
            handle_click(state, HitTarget::MacroItem(0));
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
            assert!(refresh_macro_hooks(state));
            assert_eq!(state.keyboard_hook, 0);
            assert_ne!(state.mouse_hook, 0);
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
            assert!(refresh_macro_hooks(state));
            assert_ne!(state.keyboard_hook, 0);
            assert_eq!(state.mouse_hook, 0);
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

            let background_title = "VibeTimer Background Target";
            let background_window = create_background_target(instance, background_title);
            assert_ne!(background_window, 0, "window target background dibuat");
            let other_window = CreateWindowExW(
                0,
                static_class.as_ptr(),
                wide("VibeTimer Alt Tab Workspace").as_ptr(),
                WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU | WS_VISIBLE,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                360,
                220,
                0,
                0,
                instance,
                null_mut(),
            );
            assert_ne!(other_window, 0, "window kerja kedua dibuat");
            UpdateWindow(background_window);
            UpdateWindow(other_window);
            pump_messages_for(Duration::from_millis(80));

            let executable = process_executable_name(GetCurrentProcessId())
                .expect("executable target E2E terbaca");
            let specification = MacroTarget {
                executable,
                window_title: background_title.to_owned(),
            };
            let resolved = find_saved_macro_target(&specification)
                .expect("target tersimpan ditemukan berdasarkan app + title");
            assert_eq!(resolved.root, background_window);
            assert!(validate_macro_playback_target(&resolved).is_ok());

            let selected_id = state.macro_library.selected_id;
            let mut receiver_point = Point { x: 40, y: 55 };
            ClientToScreen(background_window, &mut receiver_point);
            state.macro_targets.insert(
                selected_id,
                MacroPlaybackTarget {
                    root: background_window,
                    receiver: background_window,
                    process_id: GetCurrentProcessId(),
                    title: background_title.to_owned(),
                },
            );
            let item = state
                .macro_library
                .selected_mut()
                .expect("macro background tersedia");
            item.name = "Auto click target".to_owned();
            item.mode = MacroMode::Toggle;
            item.trigger = MacroTrigger::F9;
            item.target = Some(specification.clone());
            item.on_press = vec![
                MacroEvent::Delay(20),
                MacroEvent::MouseDownAt(MouseButton::Left, 40, 55),
                MacroEvent::MouseUpAt(MouseButton::Left, 40, 55),
                MacroEvent::KeyDown(0x41),
                MacroEvent::KeyUp(0x41),
            ];
            item.while_holding.clear();
            item.on_release.clear();
            assert_eq!(
                recorded_mouse_event(state, MouseButton::Left, true, receiver_point),
                MacroEvent::MouseDownAt(MouseButton::Left, 40, 55),
                "recording target harus menyimpan koordinat relatif"
            );

            state.macro_lane = MacroLane::OnPress;
            state.macro_selected_event = Some(0);
            sync_macro_name_edit(state);
            sync_delay_edit(state);
            state.macro_dirty = true;
            handle_click(state, HitTarget::MacroSave);
            let targeted_saved = load_library(&e2e_macro_path).expect("macro target dibaca ulang");
            assert_eq!(
                targeted_saved
                    .selected()
                    .expect("macro target tersimpan")
                    .target,
                Some(specification.clone())
            );
            InvalidateRect(main_window, null(), FALSE);
            UpdateWindow(main_window);
            pump_messages_for(Duration::from_millis(100));
            save_window_bmp(main_window, Path::new("qa/vibetimer-macro-targeted.bmp"))
                .expect("snapshot target + editor delay dibuat");

            BACKGROUND_KEY_DOWNS.store(0, Ordering::Relaxed);
            BACKGROUND_MOUSE_DOWNS.store(0, Ordering::Relaxed);
            SetForegroundWindow(other_window);
            SetActiveWindow(other_window);
            BringWindowToTop(other_window);
            pump_messages_for(Duration::from_millis(80));
            assert_eq!(
                GetActiveWindow(),
                other_window,
                "workspace kedua harus menjadi window aktif sebelum Toggle"
            );
            keyboard_hook_proc(HC_ACTION, WM_KEYDOWN as Wparam, &f9 as *const _ as Lparam);
            keyboard_hook_proc(HC_ACTION, WM_KEYUP as Wparam, &f9 as *const _ as Lparam);
            assert!(
                !MACRO_PLAYING.load(Ordering::Acquire),
                "pemicu target-bound tidak boleh mulai dari aplikasi lain"
            );

            SetForegroundWindow(background_window);
            SetActiveWindow(background_window);
            BringWindowToTop(background_window);
            pump_messages_for(Duration::from_millis(50));
            keyboard_hook_proc(HC_ACTION, WM_KEYDOWN as Wparam, &f9 as *const _ as Lparam);
            keyboard_hook_proc(HC_ACTION, WM_KEYUP as Wparam, &f9 as *const _ as Lparam);
            assert!(MACRO_PLAYING.load(Ordering::Acquire));
            SetForegroundWindow(other_window);
            SetActiveWindow(other_window);
            BringWindowToTop(other_window);
            pump_messages_for(Duration::from_millis(50));
            let foreground_before_toggle = GetForegroundWindow();
            pump_messages_for(Duration::from_millis(180));
            assert!(
                BACKGROUND_KEY_DOWNS.load(Ordering::Relaxed) >= 2,
                "Toggle background harus terus mengirim keyboard ke target"
            );
            assert!(
                BACKGROUND_MOUSE_DOWNS.load(Ordering::Relaxed) >= 2,
                "Toggle background harus terus mengirim klik ke target"
            );
            assert_eq!(
                GetActiveWindow(),
                other_window,
                "playback target tidak boleh merebut window aktif setelah Alt+Tab"
            );
            if foreground_before_toggle != 0 {
                assert_eq!(
                    GetForegroundWindow(),
                    foreground_before_toggle,
                    "playback target tidak boleh mengubah foreground window"
                );
            }
            keyboard_hook_proc(HC_ACTION, WM_KEYDOWN as Wparam, &f9 as *const _ as Lparam);
            keyboard_hook_proc(HC_ACTION, WM_KEYUP as Wparam, &f9 as *const _ as Lparam);
            wait_macro_idle();

            SetForegroundWindow(background_window);
            SetActiveWindow(background_window);
            BringWindowToTop(background_window);
            pump_messages_for(Duration::from_millis(40));
            keyboard_hook_proc(HC_ACTION, WM_KEYDOWN as Wparam, &f9 as *const _ as Lparam);
            keyboard_hook_proc(HC_ACTION, WM_KEYUP as Wparam, &f9 as *const _ as Lparam);
            pump_messages_for(Duration::from_millis(60));
            assert!(MACRO_PLAYING.load(Ordering::Acquire));
            SetActiveWindow(other_window);
            DestroyWindow(background_window);
            wait_macro_idle();
            pump_messages_for(Duration::from_millis(50));
            assert_eq!(
                state.macro_status_kind,
                StatusKind::Error,
                "Toggle harus berhenti aman ketika target ditutup"
            );
            state.macro_targets.remove(&selected_id);
            let target_item = state
                .macro_library
                .selected()
                .expect("macro target masih tersedia")
                .clone();
            assert!(
                resolve_playback_destination(state, &target_item).is_err(),
                "macro harus gagal aman bila app target ditutup"
            );
            handle_click(state, HitTarget::MacroScopeGlobal);
            assert!(
                state
                    .macro_library
                    .selected()
                    .expect("macro global tersedia")
                    .target
                    .is_none()
            );
            DestroyWindow(other_window);

            TEST_INPUT_TARGET.store(0, Ordering::Relaxed);
            let _ = fs::remove_file(&e2e_macro_path);
            let _ = fs::remove_file(e2e_macro_path.with_extension("vtm.tmp"));
            let _ = fs::remove_file(e2e_macro_path.with_extension("vtm.bak"));
            for path in [
                e2e_profiles_path,
                e2e_profile_macros_path,
                e2e_settings_path,
                e2e_timers_path,
                e2e_backup_path,
            ] {
                let _ = fs::remove_file(&path);
                let _ = fs::remove_file(path.with_extension("tmp"));
                let _ = fs::remove_file(path.with_extension("bak"));
            }
            DestroyWindow(target_window);
            DestroyWindow(main_window);
        }
    }
}
