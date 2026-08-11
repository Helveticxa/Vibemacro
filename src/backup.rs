//! Container backup portabel dengan checksum per bagian.

use std::fs;
use std::io;
use std::path::Path;

use crate::macro_engine::{MacroLibrary, decode_library, encode_library};
use crate::profiles::{ProfileLibrary, decode_profiles, encode_profiles};
use crate::settings::{AppSettings, decode_settings, encode_settings, save_atomic};
use crate::timers::{TimerLibrary, decode_timers, encode_timers};

const MAGIC: &[u8; 4] = b"VTB1";
const VERSION: u16 = 1;
const MAX_SECTION_BYTES: usize = 16 * 1024 * 1024;
const MAX_SECTIONS: usize = 32;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackupBundle {
    pub macros: MacroLibrary,
    pub profiles: ProfileLibrary,
    pub settings: AppSettings,
    pub timers: TimerLibrary,
}

impl BackupBundle {
    pub fn new(macros: MacroLibrary, profiles: ProfileLibrary, settings: AppSettings) -> Self {
        Self {
            macros,
            profiles,
            settings,
            timers: TimerLibrary::default(),
        }
    }

    pub fn with_timers(
        macros: MacroLibrary,
        profiles: ProfileLibrary,
        settings: AppSettings,
        timers: TimerLibrary,
    ) -> Self {
        Self {
            macros,
            profiles,
            settings,
            timers,
        }
    }
}

pub fn save_backup(path: &Path, bundle: &BackupBundle) -> io::Result<()> {
    save_atomic(path, &encode_backup(bundle)?, "vtb.tmp")
}

pub fn load_backup(path: &Path) -> io::Result<BackupBundle> {
    let bytes = fs::read(path)?;
    decode_backup(&bytes).map_err(|message| io::Error::new(io::ErrorKind::InvalidData, message))
}

pub fn encode_backup(bundle: &BackupBundle) -> io::Result<Vec<u8>> {
    let sections = [
        (*b"MACR", encode_library(&bundle.macros)?),
        (*b"PROF", encode_profiles(&bundle.profiles)?),
        (*b"SETT", encode_settings(&bundle.settings)),
        (*b"TIMR", encode_timers(&bundle.timers)?),
    ];
    let mut out = Vec::new();
    out.extend_from_slice(MAGIC);
    out.extend_from_slice(&VERSION.to_le_bytes());
    out.extend_from_slice(&(sections.len() as u16).to_le_bytes());
    for (tag, data) in sections {
        if data.len() > MAX_SECTION_BYTES {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Bagian backup terlalu besar.",
            ));
        }
        out.extend_from_slice(&tag);
        out.extend_from_slice(&(data.len() as u32).to_le_bytes());
        out.extend_from_slice(&crc32(&data).to_le_bytes());
        out.extend_from_slice(&data);
    }
    Ok(out)
}

pub fn decode_backup(bytes: &[u8]) -> Result<BackupBundle, &'static str> {
    let mut reader = Reader::new(bytes);
    if reader.take(4)? != MAGIC {
        return Err("File bukan backup VibeTimer.");
    }
    if reader.u16()? != VERSION {
        return Err("Versi backup belum didukung.");
    }
    let count = reader.u16()? as usize;
    if count == 0 || count > MAX_SECTIONS {
        return Err("Jumlah bagian backup tidak valid.");
    }
    let mut macros = None;
    let mut profiles = None;
    let mut settings = None;
    let mut timers = None;
    for _ in 0..count {
        let tag: [u8; 4] = reader
            .take(4)?
            .try_into()
            .map_err(|_| "Tag backup rusak.")?;
        let length = reader.u32()? as usize;
        if length > MAX_SECTION_BYTES {
            return Err("Bagian backup terlalu besar.");
        }
        let expected_crc = reader.u32()?;
        let data = reader.take(length)?;
        if crc32(data) != expected_crc {
            return Err("Checksum backup tidak cocok.");
        }
        match &tag {
            b"MACR" if macros.is_none() => macros = Some(decode_library(data)?),
            b"PROF" if profiles.is_none() => profiles = Some(decode_profiles(data)?),
            b"SETT" if settings.is_none() => settings = Some(decode_settings(data)?),
            b"TIMR" if timers.is_none() => {
                timers = Some(decode_timers(data).map_err(|_| "Bagian timer backup rusak.")?)
            }
            _ => {}
        }
    }
    if !reader.is_empty() {
        return Err("Backup memiliki data tambahan yang tidak dikenal.");
    }
    Ok(BackupBundle {
        macros: macros.ok_or("Backup tidak memiliki library macro.")?,
        profiles: profiles.unwrap_or_default(),
        settings: settings.unwrap_or_default(),
        timers: timers.unwrap_or_default(),
    })
}

pub fn crc32(bytes: &[u8]) -> u32 {
    let mut crc = u32::MAX;
    for byte in bytes {
        crc ^= *byte as u32;
        for _ in 0..8 {
            let mask = 0u32.wrapping_sub(crc & 1);
            crc = (crc >> 1) ^ (0xEDB8_8320 & mask);
        }
    }
    !crc
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
        let end = self.position.checked_add(count).ok_or("Backup rusak.")?;
        let result = self
            .bytes
            .get(self.position..end)
            .ok_or("Backup terpotong.")?;
        self.position = end;
        Ok(result)
    }

    fn u16(&mut self) -> Result<u16, &'static str> {
        Ok(u16::from_le_bytes(
            self.take(2)?.try_into().map_err(|_| "Backup rusak.")?,
        ))
    }

    fn u32(&mut self) -> Result<u32, &'static str> {
        Ok(u32::from_le_bytes(
            self.take(4)?.try_into().map_err(|_| "Backup rusak.")?,
        ))
    }

    fn is_empty(&self) -> bool {
        self.position == self.bytes.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::macro_engine::{MacroEvent, MacroTarget};

    fn sample_bundle() -> BackupBundle {
        let mut macros = MacroLibrary::default();
        macros.selected_mut().unwrap().on_press = vec![MacroEvent::Delay(42)];
        let mut profiles = ProfileLibrary::default();
        profiles.selected_mut().unwrap().target = Some(MacroTarget {
            executable: "app.exe".to_owned(),
            window_title: "Target".to_owned(),
        });
        profiles.selected_mut().unwrap().macro_ids = vec![1];
        BackupBundle::new(macros, profiles, AppSettings::default())
    }

    #[test]
    fn backup_round_trip_all_sections() {
        let bundle = sample_bundle();
        assert_eq!(decode_backup(&encode_backup(&bundle).unwrap()), Ok(bundle));
    }

    #[test]
    fn backup_rejects_checksum_damage_and_truncation() {
        let mut bytes = encode_backup(&sample_bundle()).unwrap();
        let last = bytes.len() - 1;
        bytes[last] ^= 0x80;
        assert_eq!(decode_backup(&bytes), Err("Checksum backup tidak cocok."));
        assert!(decode_backup(&bytes[..12]).is_err());
    }

    #[test]
    fn backup_saves_atomically() {
        let directory =
            std::env::temp_dir().join(format!("vibetimer-backup-test-{}", std::process::id()));
        let path = directory.join("backup.vtb");
        let bundle = sample_bundle();
        save_backup(&path, &bundle).unwrap();
        assert_eq!(load_backup(&path).unwrap(), bundle);
        fs::remove_file(path).unwrap();
        fs::remove_dir(directory).unwrap();
    }
}
