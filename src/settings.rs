//! Pengaturan runtime VibeTimer yang dapat diuji tanpa Win32.

use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

const MAGIC: &[u8; 4] = b"VTS1";
const VERSION: u16 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmergencyHotkey {
    CtrlAltF12,
    CtrlShiftF12,
    CtrlAltPause,
}

impl EmergencyHotkey {
    pub const ALL: [Self; 3] = [Self::CtrlAltF12, Self::CtrlShiftF12, Self::CtrlAltPause];

    pub const fn label(self) -> &'static str {
        match self {
            Self::CtrlAltF12 => "Ctrl + Alt + F12",
            Self::CtrlShiftF12 => "Ctrl + Shift + F12",
            Self::CtrlAltPause => "Ctrl + Alt + Pause",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppSettings {
    pub minimize_to_tray: bool,
    pub close_to_tray: bool,
    pub auto_start: bool,
    pub emergency_hotkey: EmergencyHotkey,
    pub emergency_stops_timers: bool,
    pub max_macro_runtime_seconds: u32,
    pub max_macro_repeats: u32,
    pub update_checks_enabled: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            minimize_to_tray: true,
            close_to_tray: true,
            auto_start: false,
            emergency_hotkey: EmergencyHotkey::CtrlAltF12,
            emergency_stops_timers: true,
            max_macro_runtime_seconds: 30 * 60,
            max_macro_repeats: 10_000,
            update_checks_enabled: false,
        }
    }
}

impl AppSettings {
    pub fn normalize(&mut self) {
        self.max_macro_runtime_seconds = self.max_macro_runtime_seconds.min(24 * 60 * 60);
        self.max_macro_repeats = self.max_macro_repeats.min(1_000_000);
    }
}

pub fn default_settings_path() -> PathBuf {
    data_directory().join("settings.vts")
}

pub fn data_directory() -> PathBuf {
    std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir)
        .join("VibeTimer")
}

pub fn load_settings(path: &Path) -> io::Result<AppSettings> {
    match read_with_backup_recovery(path)? {
        Some(bytes) => decode_settings(&bytes)
            .map_err(|message| io::Error::new(io::ErrorKind::InvalidData, message)),
        None => Ok(AppSettings::default()),
    }
}

pub fn save_settings(path: &Path, settings: &AppSettings) -> io::Result<()> {
    save_atomic(path, &encode_settings(settings), "vts.tmp")
}

pub fn save_atomic(path: &Path, bytes: &[u8], temporary_extension: &str) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let temporary = path.with_extension(temporary_extension);
    let mut file = fs::File::create(&temporary)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    drop(file);
    let backup = path.with_extension("bak");
    if path.exists() {
        let _ = fs::remove_file(&backup);
        fs::rename(path, &backup)?;
        if let Err(error) = fs::rename(&temporary, path) {
            let _ = fs::rename(&backup, path);
            return Err(error);
        }
        let _ = fs::remove_file(backup);
    } else {
        let _ = fs::remove_file(&backup);
        fs::rename(temporary, path)?;
    }
    Ok(())
}

pub fn read_with_backup_recovery(path: &Path) -> io::Result<Option<Vec<u8>>> {
    match fs::read(path) {
        Ok(bytes) => Ok(Some(bytes)),
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            let backup = path.with_extension("bak");
            match fs::read(&backup) {
                Ok(bytes) => {
                    // Best effort: kembalikan primary agar proses save berikutnya normal.
                    let _ = fs::rename(&backup, path);
                    Ok(Some(bytes))
                }
                Err(backup_error) if backup_error.kind() == io::ErrorKind::NotFound => Ok(None),
                Err(backup_error) => Err(backup_error),
            }
        }
        Err(error) => Err(error),
    }
}

pub fn encode_settings(settings: &AppSettings) -> Vec<u8> {
    let mut out = Vec::with_capacity(24);
    out.extend_from_slice(MAGIC);
    out.extend_from_slice(&VERSION.to_le_bytes());
    out.push(u8::from(settings.minimize_to_tray));
    out.push(u8::from(settings.close_to_tray));
    out.push(u8::from(settings.auto_start));
    out.push(hotkey_to_byte(settings.emergency_hotkey));
    out.push(u8::from(settings.emergency_stops_timers));
    out.extend_from_slice(&settings.max_macro_runtime_seconds.to_le_bytes());
    out.extend_from_slice(&settings.max_macro_repeats.to_le_bytes());
    out.push(u8::from(settings.update_checks_enabled));
    out
}

pub fn decode_settings(bytes: &[u8]) -> Result<AppSettings, &'static str> {
    if bytes.len() != 20 || bytes.get(..4) != Some(MAGIC) {
        return Err("File pengaturan bukan format VibeTimer.");
    }
    let version = u16::from_le_bytes(bytes[4..6].try_into().map_err(|_| "File terpotong.")?);
    if version != VERSION {
        return Err("Versi pengaturan belum didukung.");
    }
    let boolean = |value: u8| match value {
        0 => Ok(false),
        1 => Ok(true),
        _ => Err("Nilai pengaturan tidak valid."),
    };
    let mut settings = AppSettings {
        minimize_to_tray: boolean(bytes[6])?,
        close_to_tray: boolean(bytes[7])?,
        auto_start: boolean(bytes[8])?,
        emergency_hotkey: byte_to_hotkey(bytes[9])?,
        emergency_stops_timers: boolean(bytes[10])?,
        max_macro_runtime_seconds: u32::from_le_bytes(
            bytes[11..15].try_into().map_err(|_| "File terpotong.")?,
        ),
        max_macro_repeats: u32::from_le_bytes(
            bytes[15..19].try_into().map_err(|_| "File terpotong.")?,
        ),
        update_checks_enabled: boolean(bytes[19])?,
    };
    settings.normalize();
    Ok(settings)
}

fn hotkey_to_byte(value: EmergencyHotkey) -> u8 {
    match value {
        EmergencyHotkey::CtrlAltF12 => 0,
        EmergencyHotkey::CtrlShiftF12 => 1,
        EmergencyHotkey::CtrlAltPause => 2,
    }
}

fn byte_to_hotkey(value: u8) -> Result<EmergencyHotkey, &'static str> {
    match value {
        0 => Ok(EmergencyHotkey::CtrlAltF12),
        1 => Ok(EmergencyHotkey::CtrlShiftF12),
        2 => Ok(EmergencyHotkey::CtrlAltPause),
        _ => Err("Hotkey darurat tidak valid."),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_round_trip_every_field() {
        let settings = AppSettings {
            minimize_to_tray: false,
            close_to_tray: true,
            auto_start: true,
            emergency_hotkey: EmergencyHotkey::CtrlAltPause,
            emergency_stops_timers: false,
            max_macro_runtime_seconds: 75,
            max_macro_repeats: 123,
            update_checks_enabled: true,
        };
        assert_eq!(decode_settings(&encode_settings(&settings)), Ok(settings));
    }

    #[test]
    fn settings_reject_corruption_and_clamp_limits() {
        assert!(decode_settings(b"NOPE").is_err());
        let mut bytes = encode_settings(&AppSettings::default());
        bytes[11..15].copy_from_slice(&u32::MAX.to_le_bytes());
        bytes[15..19].copy_from_slice(&u32::MAX.to_le_bytes());
        let decoded = decode_settings(&bytes).expect("pengaturan valid");
        assert_eq!(decoded.max_macro_runtime_seconds, 86_400);
        assert_eq!(decoded.max_macro_repeats, 1_000_000);
    }

    #[test]
    fn settings_save_atomically() {
        let directory =
            std::env::temp_dir().join(format!("vibetimer-settings-test-{}", std::process::id()));
        let path = directory.join("settings.vts");
        let settings = AppSettings::default();
        save_settings(&path, &settings).expect("pengaturan disimpan");
        assert_eq!(load_settings(&path).expect("pengaturan dimuat"), settings);
        let backup = path.with_extension("bak");
        fs::rename(&path, &backup).expect("simulasi crash dibuat");
        assert_eq!(load_settings(&path).expect("backup dipulihkan"), settings);
        assert!(path.exists(), "primary dikembalikan dari backup");
        fs::remove_file(path).expect("file dibersihkan");
        fs::remove_dir(directory).expect("folder dibersihkan");
    }
}
