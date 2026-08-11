//! Model dan penyimpanan macro yang tidak bergantung pada Win32.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

const MAGIC: &[u8; 4] = b"VTM1";
const VERSION: u16 = 2;
const MAX_ITEMS: usize = 10_000;
const MAX_NAME_BYTES: usize = 160;
const MAX_TARGET_BYTES: usize = 520;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MacroMode {
    NoRepeat,
    RepeatWhileHolding,
    Toggle,
    Sequence,
}

impl MacroMode {
    pub const ALL: [Self; 4] = [
        Self::NoRepeat,
        Self::RepeatWhileHolding,
        Self::Toggle,
        Self::Sequence,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            Self::NoRepeat => "No Repeat",
            Self::RepeatWhileHolding => "While Holding",
            Self::Toggle => "Toggle",
            Self::Sequence => "Sequence",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MacroTrigger {
    F8,
    F9,
    MouseMiddle,
    MouseX1,
    MouseX2,
}

impl MacroTrigger {
    pub const ALL: [Self; 5] = [
        Self::F8,
        Self::F9,
        Self::MouseMiddle,
        Self::MouseX1,
        Self::MouseX2,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            Self::F8 => "F8",
            Self::F9 => "F9",
            Self::MouseMiddle => "Middle",
            Self::MouseX1 => "Mouse 4",
            Self::MouseX2 => "Mouse 5",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    X1,
    X2,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MacroEvent {
    Delay(u32),
    KeyDown(u16),
    KeyUp(u16),
    MouseDown(MouseButton),
    MouseUp(MouseButton),
    MouseDownAt(MouseButton, i32, i32),
    MouseUpAt(MouseButton, i32, i32),
    Wheel(i16),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MacroTarget {
    pub executable: String,
    pub window_title: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MacroDefinition {
    pub id: u32,
    pub name: String,
    pub mode: MacroMode,
    pub trigger: MacroTrigger,
    pub standard_delay_ms: Option<u32>,
    pub show_key_releases: bool,
    pub target: Option<MacroTarget>,
    pub on_press: Vec<MacroEvent>,
    pub while_holding: Vec<MacroEvent>,
    pub on_release: Vec<MacroEvent>,
}

impl MacroDefinition {
    pub fn new(id: u32, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            mode: MacroMode::NoRepeat,
            trigger: MacroTrigger::F8,
            standard_delay_ms: None,
            show_key_releases: true,
            target: None,
            on_press: Vec::new(),
            while_holding: Vec::new(),
            on_release: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MacroLibrary {
    pub selected_id: u32,
    pub next_id: u32,
    pub macros: Vec<MacroDefinition>,
}

impl Default for MacroLibrary {
    fn default() -> Self {
        let first = MacroDefinition::new(1, "Macro pertama");
        Self {
            selected_id: 1,
            next_id: 2,
            macros: vec![first],
        }
    }
}

impl MacroLibrary {
    pub fn selected(&self) -> Option<&MacroDefinition> {
        self.macros.iter().find(|item| item.id == self.selected_id)
    }

    pub fn selected_mut(&mut self) -> Option<&mut MacroDefinition> {
        let selected_id = self.selected_id;
        self.macros.iter_mut().find(|item| item.id == selected_id)
    }

    pub fn add_macro(&mut self) -> u32 {
        let id = self.next_id.max(1);
        self.next_id = id.saturating_add(1);
        let name = format!("Macro {}", self.macros.len() + 1);
        self.macros.push(MacroDefinition::new(id, name));
        self.selected_id = id;
        id
    }
}

pub fn default_data_path() -> PathBuf {
    let base = std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);
    base.join("VibeTimer").join("macros.vtm")
}

pub fn load_library(path: &Path) -> io::Result<MacroLibrary> {
    match fs::read(path) {
        Ok(bytes) => decode_library(&bytes)
            .map_err(|message| io::Error::new(io::ErrorKind::InvalidData, message)),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(MacroLibrary::default()),
        Err(error) => Err(error),
    }
}

pub fn save_library(path: &Path, library: &MacroLibrary) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let temporary = path.with_extension("vtm.tmp");
    fs::write(&temporary, encode_library(library)?)?;
    if path.exists() {
        let backup = path.with_extension("vtm.bak");
        let _ = fs::remove_file(&backup);
        fs::rename(path, &backup)?;
        if let Err(error) = fs::rename(&temporary, path) {
            let _ = fs::rename(&backup, path);
            return Err(error);
        }
        let _ = fs::remove_file(backup);
    } else {
        fs::rename(temporary, path)?;
    }
    Ok(())
}

pub fn encode_library(library: &MacroLibrary) -> io::Result<Vec<u8>> {
    if library.macros.len() > MAX_ITEMS {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Terlalu banyak macro.",
        ));
    }
    let mut out = Vec::new();
    out.extend_from_slice(MAGIC);
    push_u16(&mut out, VERSION);
    push_u32(&mut out, library.selected_id);
    push_u32(&mut out, library.next_id);
    push_u32(&mut out, library.macros.len() as u32);
    for item in &library.macros {
        push_u32(&mut out, item.id);
        let name = item.name.as_bytes();
        if name.len() > MAX_NAME_BYTES {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Nama macro terlalu panjang.",
            ));
        }
        push_u16(&mut out, name.len() as u16);
        out.extend_from_slice(name);
        out.push(mode_to_byte(item.mode));
        out.push(trigger_to_byte(item.trigger));
        push_u32(&mut out, item.standard_delay_ms.unwrap_or(u32::MAX));
        out.push(u8::from(item.show_key_releases));
        match &item.target {
            Some(target) => {
                out.push(1);
                push_string(&mut out, &target.executable, MAX_TARGET_BYTES)?;
                push_string(&mut out, &target.window_title, MAX_TARGET_BYTES)?;
            }
            None => out.push(0),
        }
        encode_events(&mut out, &item.on_press)?;
        encode_events(&mut out, &item.while_holding)?;
        encode_events(&mut out, &item.on_release)?;
    }
    Ok(out)
}

pub fn decode_library(bytes: &[u8]) -> Result<MacroLibrary, &'static str> {
    let mut reader = Reader::new(bytes);
    if reader.take(4)? != MAGIC {
        return Err("File macro bukan format VibeTimer.");
    }
    let version = reader.u16()?;
    if !matches!(version, 1 | VERSION) {
        return Err("Versi file macro belum didukung.");
    }
    let selected_id = reader.u32()?;
    let next_id = reader.u32()?;
    let count = reader.u32()? as usize;
    if count > MAX_ITEMS {
        return Err("Jumlah macro tidak valid.");
    }
    let mut macros = Vec::with_capacity(count);
    for _ in 0..count {
        let id = reader.u32()?;
        let name_len = reader.u16()? as usize;
        if name_len > MAX_NAME_BYTES {
            return Err("Nama macro terlalu panjang.");
        }
        let name = std::str::from_utf8(reader.take(name_len)?)
            .map_err(|_| "Nama macro bukan UTF-8 yang valid.")?
            .to_owned();
        let mode = byte_to_mode(reader.u8()?)?;
        let trigger = byte_to_trigger(reader.u8()?)?;
        let standard = reader.u32()?;
        let show_key_releases = match reader.u8()? {
            0 => false,
            1 => true,
            _ => return Err("Nilai pengaturan macro tidak valid."),
        };
        let target = if version >= 2 {
            match reader.u8()? {
                0 => None,
                1 => Some(MacroTarget {
                    executable: reader.string(MAX_TARGET_BYTES)?,
                    window_title: reader.string(MAX_TARGET_BYTES)?,
                }),
                _ => return Err("Nilai target macro tidak valid."),
            }
        } else {
            None
        };
        macros.push(MacroDefinition {
            id,
            name,
            mode,
            trigger,
            standard_delay_ms: (standard != u32::MAX).then_some(standard),
            show_key_releases,
            target,
            on_press: decode_events(&mut reader, version)?,
            while_holding: decode_events(&mut reader, version)?,
            on_release: decode_events(&mut reader, version)?,
        });
    }
    if !reader.is_empty() {
        return Err("File macro memiliki data tambahan yang tidak dikenal.");
    }
    if macros.is_empty() {
        return Ok(MacroLibrary::default());
    }
    let selected_id = if macros.iter().any(|item| item.id == selected_id) {
        selected_id
    } else {
        macros[0].id
    };
    Ok(MacroLibrary {
        selected_id,
        next_id: next_id.max(
            macros
                .iter()
                .map(|item| item.id)
                .max()
                .unwrap_or(0)
                .saturating_add(1),
        ),
        macros,
    })
}

fn encode_events(out: &mut Vec<u8>, events: &[MacroEvent]) -> io::Result<()> {
    if events.len() > MAX_ITEMS {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Terlalu banyak langkah macro.",
        ));
    }
    push_u32(out, events.len() as u32);
    for event in events {
        match *event {
            MacroEvent::Delay(value) => {
                out.push(0);
                push_u32(out, value);
            }
            MacroEvent::KeyDown(value) => {
                out.push(1);
                push_u32(out, value as u32);
            }
            MacroEvent::KeyUp(value) => {
                out.push(2);
                push_u32(out, value as u32);
            }
            MacroEvent::MouseDown(value) => {
                out.push(3);
                push_u32(out, mouse_to_byte(value) as u32);
            }
            MacroEvent::MouseUp(value) => {
                out.push(4);
                push_u32(out, mouse_to_byte(value) as u32);
            }
            MacroEvent::Wheel(value) => {
                out.push(5);
                push_u32(out, value as i32 as u32);
            }
            MacroEvent::MouseDownAt(button, x, y) => {
                out.push(6);
                push_u32(out, mouse_to_byte(button) as u32);
                push_u32(out, x as u32);
                push_u32(out, y as u32);
            }
            MacroEvent::MouseUpAt(button, x, y) => {
                out.push(7);
                push_u32(out, mouse_to_byte(button) as u32);
                push_u32(out, x as u32);
                push_u32(out, y as u32);
            }
        }
    }
    Ok(())
}

fn decode_events(reader: &mut Reader<'_>, version: u16) -> Result<Vec<MacroEvent>, &'static str> {
    let count = reader.u32()? as usize;
    if count > MAX_ITEMS {
        return Err("Jumlah langkah macro tidak valid.");
    }
    let mut events = Vec::with_capacity(count);
    for _ in 0..count {
        let kind = reader.u8()?;
        let value = reader.u32()?;
        let event = match kind {
            0 => MacroEvent::Delay(value),
            1 if value <= u16::MAX as u32 => MacroEvent::KeyDown(value as u16),
            2 if value <= u16::MAX as u32 => MacroEvent::KeyUp(value as u16),
            3 if value <= u8::MAX as u32 => MacroEvent::MouseDown(byte_to_mouse(value as u8)?),
            4 if value <= u8::MAX as u32 => MacroEvent::MouseUp(byte_to_mouse(value as u8)?),
            5 if (i16::MIN as i32..=i16::MAX as i32).contains(&(value as i32)) => {
                MacroEvent::Wheel(value as i32 as i16)
            }
            6 if version >= 2 && value <= u8::MAX as u32 => MacroEvent::MouseDownAt(
                byte_to_mouse(value as u8)?,
                reader.u32()? as i32,
                reader.u32()? as i32,
            ),
            7 if version >= 2 && value <= u8::MAX as u32 => MacroEvent::MouseUpAt(
                byte_to_mouse(value as u8)?,
                reader.u32()? as i32,
                reader.u32()? as i32,
            ),
            _ => return Err("Jenis langkah macro tidak valid."),
        };
        events.push(event);
    }
    Ok(events)
}

fn mode_to_byte(value: MacroMode) -> u8 {
    match value {
        MacroMode::NoRepeat => 0,
        MacroMode::RepeatWhileHolding => 1,
        MacroMode::Toggle => 2,
        MacroMode::Sequence => 3,
    }
}

fn byte_to_mode(value: u8) -> Result<MacroMode, &'static str> {
    match value {
        0 => Ok(MacroMode::NoRepeat),
        1 => Ok(MacroMode::RepeatWhileHolding),
        2 => Ok(MacroMode::Toggle),
        3 => Ok(MacroMode::Sequence),
        _ => Err("Jenis macro tidak valid."),
    }
}

fn trigger_to_byte(value: MacroTrigger) -> u8 {
    match value {
        MacroTrigger::F8 => 0,
        MacroTrigger::F9 => 1,
        MacroTrigger::MouseMiddle => 2,
        MacroTrigger::MouseX1 => 3,
        MacroTrigger::MouseX2 => 4,
    }
}

fn byte_to_trigger(value: u8) -> Result<MacroTrigger, &'static str> {
    match value {
        0 => Ok(MacroTrigger::F8),
        1 => Ok(MacroTrigger::F9),
        2 => Ok(MacroTrigger::MouseMiddle),
        3 => Ok(MacroTrigger::MouseX1),
        4 => Ok(MacroTrigger::MouseX2),
        _ => Err("Tombol pemicu tidak valid."),
    }
}

fn mouse_to_byte(value: MouseButton) -> u8 {
    match value {
        MouseButton::Left => 0,
        MouseButton::Right => 1,
        MouseButton::Middle => 2,
        MouseButton::X1 => 3,
        MouseButton::X2 => 4,
    }
}

fn byte_to_mouse(value: u8) -> Result<MouseButton, &'static str> {
    match value {
        0 => Ok(MouseButton::Left),
        1 => Ok(MouseButton::Right),
        2 => Ok(MouseButton::Middle),
        3 => Ok(MouseButton::X1),
        4 => Ok(MouseButton::X2),
        _ => Err("Tombol mouse tidak valid."),
    }
}

fn push_u16(out: &mut Vec<u8>, value: u16) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn push_u32(out: &mut Vec<u8>, value: u32) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn push_string(out: &mut Vec<u8>, value: &str, maximum: usize) -> io::Result<()> {
    let bytes = value.as_bytes();
    if bytes.len() > maximum || bytes.len() > u16::MAX as usize {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Teks target macro terlalu panjang.",
        ));
    }
    push_u16(out, bytes.len() as u16);
    out.extend_from_slice(bytes);
    Ok(())
}

struct Reader<'a> {
    bytes: &'a [u8],
    position: usize,
}

impl<'a> Reader<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, position: 0 }
    }

    fn take(&mut self, count: usize) -> Result<&'a [u8], &'static str> {
        let end = self
            .position
            .checked_add(count)
            .ok_or("File macro rusak.")?;
        let value = self
            .bytes
            .get(self.position..end)
            .ok_or("File macro terpotong.")?;
        self.position = end;
        Ok(value)
    }

    fn u8(&mut self) -> Result<u8, &'static str> {
        Ok(self.take(1)?[0])
    }

    fn u16(&mut self) -> Result<u16, &'static str> {
        let bytes: [u8; 2] = self.take(2)?.try_into().map_err(|_| "File macro rusak.")?;
        Ok(u16::from_le_bytes(bytes))
    }

    fn u32(&mut self) -> Result<u32, &'static str> {
        let bytes: [u8; 4] = self.take(4)?.try_into().map_err(|_| "File macro rusak.")?;
        Ok(u32::from_le_bytes(bytes))
    }

    fn string(&mut self, maximum: usize) -> Result<String, &'static str> {
        let length = self.u16()? as usize;
        if length > maximum {
            return Err("Teks target macro terlalu panjang.");
        }
        std::str::from_utf8(self.take(length)?)
            .map(str::to_owned)
            .map_err(|_| "Teks target macro bukan UTF-8 yang valid.")
    }

    fn is_empty(&self) -> bool {
        self.position == self.bytes.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_every_macro_field_and_event() {
        let mut library = MacroLibrary::default();
        let item = library.selected_mut().unwrap();
        item.name = "Build cepat".to_owned();
        item.mode = MacroMode::Sequence;
        item.trigger = MacroTrigger::MouseX2;
        item.standard_delay_ms = Some(45);
        item.show_key_releases = false;
        item.target = Some(MacroTarget {
            executable: "game.exe".to_owned(),
            window_title: "Game Window".to_owned(),
        });
        item.on_press = vec![
            MacroEvent::KeyDown(0x41),
            MacroEvent::Delay(137),
            MacroEvent::KeyUp(0x41),
            MacroEvent::MouseDown(MouseButton::X1),
            MacroEvent::MouseUp(MouseButton::X1),
            MacroEvent::MouseDownAt(MouseButton::Left, 320, 240),
            MacroEvent::MouseUpAt(MouseButton::Left, 320, 240),
            MacroEvent::Wheel(-120),
        ];
        item.while_holding = vec![MacroEvent::Delay(20)];
        item.on_release = vec![MacroEvent::KeyDown(0x0D), MacroEvent::KeyUp(0x0D)];

        let encoded = encode_library(&library).unwrap();
        assert_eq!(decode_library(&encoded).unwrap(), library);
    }

    #[test]
    fn rejects_wrong_magic_and_truncated_data() {
        assert_eq!(
            decode_library(b"NOPE"),
            Err("File macro bukan format VibeTimer.")
        );
        assert_eq!(decode_library(b"VTM1\x01"), Err("File macro terpotong."));
    }

    #[test]
    fn migrates_v1_library_without_a_window_target() {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(MAGIC);
        push_u16(&mut bytes, 1);
        push_u32(&mut bytes, 1);
        push_u32(&mut bytes, 2);
        push_u32(&mut bytes, 1);
        push_u32(&mut bytes, 1);
        push_string(&mut bytes, "Macro lama", MAX_NAME_BYTES).unwrap();
        bytes.push(mode_to_byte(MacroMode::NoRepeat));
        bytes.push(trigger_to_byte(MacroTrigger::F8));
        push_u32(&mut bytes, u32::MAX);
        bytes.push(1);
        encode_events(&mut bytes, &[MacroEvent::Delay(125)]).unwrap();
        encode_events(&mut bytes, &[]).unwrap();
        encode_events(&mut bytes, &[]).unwrap();

        let migrated = decode_library(&bytes).expect("format V1 dimigrasikan");
        let item = migrated.selected().expect("macro lama tersedia");
        assert_eq!(item.name, "Macro lama");
        assert_eq!(item.target, None);
        assert_eq!(item.on_press, vec![MacroEvent::Delay(125)]);
    }

    #[test]
    fn new_library_has_safe_no_repeat_macro() {
        let library = MacroLibrary::default();
        let item = library.selected().unwrap();
        assert_eq!(item.mode, MacroMode::NoRepeat);
        assert_eq!(item.trigger, MacroTrigger::F8);
        assert!(item.on_press.is_empty());
    }

    #[test]
    fn saves_and_loads_library_atomically_on_disk() {
        let directory =
            std::env::temp_dir().join(format!("vibetimer-macro-test-{}", std::process::id()));
        let path = directory.join("macros.vtm");
        let mut library = MacroLibrary::default();
        library.selected_mut().unwrap().on_press = vec![
            MacroEvent::KeyDown(0x41),
            MacroEvent::Delay(12),
            MacroEvent::KeyUp(0x41),
        ];

        save_library(&path, &library).expect("library disimpan");
        assert_eq!(load_library(&path).expect("library dibaca"), library);

        fs::remove_file(path).expect("file test dibersihkan");
        fs::remove_dir(directory).expect("folder test dibersihkan");
    }
}
