//! App Profiles: satu target aplikasi dengan kumpulan macro yang terkait.

use std::io;
use std::path::{Path, PathBuf};

use crate::macro_engine::MacroTarget;
use crate::settings::{data_directory, read_with_backup_recovery, save_atomic};

#[cfg(test)]
use std::fs;

const MAGIC: &[u8; 4] = b"VTP1";
const VERSION: u16 = 1;
const MAX_PROFILES: usize = 6;
const MAX_NAME_BYTES: usize = 160;
const MAX_TARGET_BYTES: usize = 520;
const MAX_LINKS: usize = 10_000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppProfile {
    pub id: u32,
    pub name: String,
    pub target: Option<MacroTarget>,
    pub macro_ids: Vec<u32>,
}

impl AppProfile {
    pub fn new(id: u32, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            target: None,
            macro_ids: Vec::new(),
        }
    }

    pub fn contains_macro(&self, macro_id: u32) -> bool {
        self.macro_ids.contains(&macro_id)
    }

    pub fn toggle_macro(&mut self, macro_id: u32) -> bool {
        if let Some(index) = self.macro_ids.iter().position(|value| *value == macro_id) {
            self.macro_ids.remove(index);
            false
        } else if self.macro_ids.len() < MAX_LINKS {
            self.macro_ids.push(macro_id);
            true
        } else {
            false
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfileLibrary {
    pub selected_id: u32,
    pub next_id: u32,
    pub profiles: Vec<AppProfile>,
}

impl Default for ProfileLibrary {
    fn default() -> Self {
        Self {
            selected_id: 1,
            next_id: 2,
            profiles: vec![AppProfile::new(1, "Profil utama")],
        }
    }
}

impl ProfileLibrary {
    pub fn selected(&self) -> Option<&AppProfile> {
        self.profiles
            .iter()
            .find(|profile| profile.id == self.selected_id)
    }

    pub fn selected_mut(&mut self) -> Option<&mut AppProfile> {
        let selected = self.selected_id;
        self.profiles
            .iter_mut()
            .find(|profile| profile.id == selected)
    }

    pub fn add_profile(&mut self) -> Option<u32> {
        if self.profiles.len() >= MAX_PROFILES {
            return None;
        }
        let id = self.next_id.max(1);
        self.next_id = id.saturating_add(1);
        self.profiles.push(AppProfile::new(
            id,
            format!("Profil {}", self.profiles.len() + 1),
        ));
        self.selected_id = id;
        Some(id)
    }

    pub fn duplicate_selected(&mut self) -> Option<u32> {
        if self.profiles.len() >= MAX_PROFILES {
            return None;
        }
        let mut duplicate = self.selected()?.clone();
        let id = self.next_id.max(1);
        self.next_id = id.saturating_add(1);
        duplicate.id = id;
        duplicate.name = format!("{} salinan", duplicate.name);
        self.profiles.push(duplicate);
        self.selected_id = id;
        Some(id)
    }

    pub fn delete_selected(&mut self) -> bool {
        if self.profiles.len() <= 1 {
            return false;
        }
        let Some(index) = self
            .profiles
            .iter()
            .position(|profile| profile.id == self.selected_id)
        else {
            return false;
        };
        self.profiles.remove(index);
        self.selected_id = self.profiles[index.min(self.profiles.len() - 1)].id;
        true
    }

    pub fn remove_missing_macro_links(&mut self, available_ids: &[u32]) {
        for profile in &mut self.profiles {
            profile
                .macro_ids
                .retain(|macro_id| available_ids.contains(macro_id));
            profile.macro_ids.sort_unstable();
            profile.macro_ids.dedup();
        }
    }
}

pub fn default_profiles_path() -> PathBuf {
    data_directory().join("profiles.vtp")
}

pub fn load_profiles(path: &Path) -> io::Result<ProfileLibrary> {
    match read_with_backup_recovery(path)? {
        Some(bytes) => decode_profiles(&bytes)
            .map_err(|message| io::Error::new(io::ErrorKind::InvalidData, message)),
        None => Ok(ProfileLibrary::default()),
    }
}

pub fn save_profiles(path: &Path, profiles: &ProfileLibrary) -> io::Result<()> {
    save_atomic(path, &encode_profiles(profiles)?, "vtp.tmp")
}

pub fn encode_profiles(library: &ProfileLibrary) -> io::Result<Vec<u8>> {
    if library.profiles.len() > MAX_PROFILES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Terlalu banyak profil.",
        ));
    }
    let mut ids: Vec<u32> = library.profiles.iter().map(|profile| profile.id).collect();
    ids.sort_unstable();
    let original_count = ids.len();
    ids.dedup();
    if ids.first() == Some(&0)
        || ids.last() == Some(&u32::MAX)
        || ids.len() != original_count
        || !ids.contains(&library.selected_id)
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "ID profil tidak unik atau berada di luar batas aman.",
        ));
    }
    let mut out = Vec::new();
    out.extend_from_slice(MAGIC);
    push_u16(&mut out, VERSION);
    push_u32(&mut out, library.selected_id);
    push_u32(&mut out, library.next_id);
    push_u32(&mut out, library.profiles.len() as u32);
    for profile in &library.profiles {
        if profile.id == 0 || profile.id == u32::MAX || profile.name.trim().is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "ID atau nama profil tidak valid.",
            ));
        }
        push_u32(&mut out, profile.id);
        push_string(&mut out, &profile.name, MAX_NAME_BYTES)?;
        match &profile.target {
            Some(target) => {
                if target.executable.trim().is_empty() || target.window_title.trim().is_empty() {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "Target profil tidak lengkap.",
                    ));
                }
                out.push(1);
                push_string(&mut out, &target.executable, MAX_TARGET_BYTES)?;
                push_string(&mut out, &target.window_title, MAX_TARGET_BYTES)?;
            }
            None => out.push(0),
        }
        if profile.macro_ids.len() > MAX_LINKS {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Terlalu banyak tautan macro.",
            ));
        }
        push_u32(&mut out, profile.macro_ids.len() as u32);
        for macro_id in &profile.macro_ids {
            push_u32(&mut out, *macro_id);
        }
    }
    Ok(out)
}

pub fn decode_profiles(bytes: &[u8]) -> Result<ProfileLibrary, &'static str> {
    let mut reader = Reader::new(bytes);
    if reader.take(4)? != MAGIC {
        return Err("File profil bukan format VibeTimer.");
    }
    if reader.u16()? != VERSION {
        return Err("Versi profil belum didukung.");
    }
    let selected_id = reader.u32()?;
    let next_id = reader.u32()?;
    let count = reader.u32()? as usize;
    if count > MAX_PROFILES {
        return Err("Jumlah profil tidak valid.");
    }
    let mut profiles = Vec::with_capacity(count);
    for _ in 0..count {
        let id = reader.u32()?;
        let name = reader.string(MAX_NAME_BYTES)?;
        if name.trim().is_empty() {
            return Err("Nama profil tidak boleh kosong.");
        }
        let target = match reader.u8()? {
            0 => None,
            1 => Some(MacroTarget {
                executable: reader.string(MAX_TARGET_BYTES)?,
                window_title: reader.string(MAX_TARGET_BYTES)?,
            }),
            _ => return Err("Nilai target profil tidak valid."),
        };
        if target.as_ref().is_some_and(|target| {
            target.executable.trim().is_empty() || target.window_title.trim().is_empty()
        }) {
            return Err("Target profil tidak lengkap.");
        }
        let link_count = reader.u32()? as usize;
        if link_count > MAX_LINKS {
            return Err("Jumlah tautan macro tidak valid.");
        }
        let mut macro_ids = Vec::with_capacity(link_count);
        for _ in 0..link_count {
            macro_ids.push(reader.u32()?);
        }
        macro_ids.sort_unstable();
        macro_ids.dedup();
        profiles.push(AppProfile {
            id,
            name,
            target,
            macro_ids,
        });
    }
    if !reader.is_empty() {
        return Err("File profil memiliki data tambahan yang tidak dikenal.");
    }
    if profiles.is_empty() {
        return Ok(ProfileLibrary::default());
    }
    let mut ids: Vec<u32> = profiles.iter().map(|profile| profile.id).collect();
    ids.sort_unstable();
    if ids.first() == Some(&0) || ids.last() == Some(&u32::MAX) {
        return Err("ID profil berada di luar batas aman.");
    }
    let original_count = ids.len();
    ids.dedup();
    if ids.len() != original_count {
        return Err("ID profil duplikat.");
    }
    let selected_id = if profiles.iter().any(|profile| profile.id == selected_id) {
        selected_id
    } else {
        profiles[0].id
    };
    let minimum_next = profiles
        .iter()
        .map(|profile| profile.id)
        .max()
        .unwrap_or(0)
        .saturating_add(1);
    Ok(ProfileLibrary {
        selected_id,
        next_id: next_id.max(minimum_next),
        profiles,
    })
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
            "Teks profil terlalu panjang.",
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
            .ok_or("File profil rusak.")?;
        let result = self
            .bytes
            .get(self.position..end)
            .ok_or("File profil terpotong.")?;
        self.position = end;
        Ok(result)
    }

    fn u8(&mut self) -> Result<u8, &'static str> {
        Ok(self.take(1)?[0])
    }

    fn u16(&mut self) -> Result<u16, &'static str> {
        Ok(u16::from_le_bytes(
            self.take(2)?.try_into().map_err(|_| "File rusak.")?,
        ))
    }

    fn u32(&mut self) -> Result<u32, &'static str> {
        Ok(u32::from_le_bytes(
            self.take(4)?.try_into().map_err(|_| "File rusak.")?,
        ))
    }

    fn string(&mut self, maximum: usize) -> Result<String, &'static str> {
        let length = self.u16()? as usize;
        if length > maximum {
            return Err("Teks profil terlalu panjang.");
        }
        std::str::from_utf8(self.take(length)?)
            .map(str::to_owned)
            .map_err(|_| "Teks profil bukan UTF-8 yang valid.")
    }

    fn is_empty(&self) -> bool {
        self.position == self.bytes.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profile_library_round_trip_and_deduplicates_links() {
        let mut library = ProfileLibrary::default();
        let profile = library.selected_mut().expect("profil tersedia");
        profile.name = "Claude Code".to_owned();
        profile.target = Some(MacroTarget {
            executable: "claude.exe".to_owned(),
            window_title: "Claude".to_owned(),
        });
        profile.macro_ids = vec![9, 2, 9];
        let expected_target = profile.target.clone();
        let decoded = decode_profiles(&encode_profiles(&library).unwrap()).unwrap();
        assert_eq!(decoded.selected().unwrap().macro_ids, vec![2, 9]);
        assert_eq!(decoded.selected().unwrap().target, expected_target);
    }

    #[test]
    fn profiles_add_duplicate_toggle_delete_and_clean_links() {
        let mut library = ProfileLibrary::default();
        let profile = library.selected_mut().unwrap();
        assert!(profile.toggle_macro(7));
        assert!(!profile.toggle_macro(7));
        assert!(profile.toggle_macro(7));
        let duplicate = library.duplicate_selected().unwrap();
        assert_eq!(library.selected_id, duplicate);
        library.remove_missing_macro_links(&[8]);
        assert!(library.selected().unwrap().macro_ids.is_empty());
        assert!(library.delete_selected());
        assert!(!library.delete_selected());
        assert_eq!(library.add_profile(), Some(3));
        while library.profiles.len() < MAX_PROFILES {
            assert!(library.add_profile().is_some());
        }
        assert!(library.add_profile().is_none());
        assert!(library.duplicate_selected().is_none());
    }

    #[test]
    fn profiles_save_atomically() {
        let directory =
            std::env::temp_dir().join(format!("vibetimer-profile-test-{}", std::process::id()));
        let path = directory.join("profiles.vtp");
        let profiles = ProfileLibrary::default();
        save_profiles(&path, &profiles).unwrap();
        assert_eq!(load_profiles(&path).unwrap(), profiles);
        fs::remove_file(path).unwrap();
        fs::remove_dir(directory).unwrap();
    }
}
