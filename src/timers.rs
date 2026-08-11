use crate::macro_engine::MacroTarget;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const MAGIC: &[u8; 4] = b"VTT1";
const VERSION: u16 = 1;
const MAX_TIMERS: usize = 6;
const MAX_STRING_BYTES: usize = 16 * 1024;
const MAX_DURATION_SECONDS: u64 = 7 * 24 * 60 * 60;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TimerAction {
    EnterOnly,
    TextAndEnter,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TimerPhase {
    Idle,
    Running,
    Dispatching,
    Completed,
    Failed,
    Missed,
    Cancelled,
}

impl TimerPhase {
    pub fn label(self) -> &'static str {
        match self {
            Self::Idle => "Siap",
            Self::Running => "Berjalan",
            Self::Dispatching => "Mengirim",
            Self::Completed => "Selesai",
            Self::Failed => "Gagal",
            Self::Missed => "Terlewat",
            Self::Cancelled => "Dibatalkan",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TimerDefinition {
    pub id: u32,
    pub name: String,
    pub duration_seconds: u64,
    pub remaining_seconds: u64,
    pub deadline_unix_ms: u64,
    pub phase: TimerPhase,
    pub action: TimerAction,
    pub prompt: String,
    pub target: Option<MacroTarget>,
}

impl TimerDefinition {
    fn fresh(id: u32, name: String) -> Self {
        Self {
            id,
            name,
            duration_seconds: 30 * 60,
            remaining_seconds: 30 * 60,
            deadline_unix_ms: 0,
            phase: TimerPhase::Idle,
            action: TimerAction::TextAndEnter,
            prompt: "lanjutkan".to_owned(),
            target: None,
        }
    }

    pub fn is_running(&self) -> bool {
        self.phase == TimerPhase::Running
    }

    pub fn start(
        &mut self,
        now_unix_ms: u64,
        duration_seconds: u64,
        action: TimerAction,
        prompt: String,
        target: MacroTarget,
    ) -> Result<(), &'static str> {
        if duration_seconds == 0 || duration_seconds > MAX_DURATION_SECONDS {
            return Err("Durasi timer harus antara 1 detik dan 7 hari.");
        }
        let duration_ms = duration_seconds
            .checked_mul(1_000)
            .ok_or("Durasi timer terlalu besar.")?;
        let deadline = now_unix_ms
            .checked_add(duration_ms)
            .ok_or("Deadline timer terlalu besar.")?;
        self.duration_seconds = duration_seconds;
        self.remaining_seconds = duration_seconds;
        self.deadline_unix_ms = deadline;
        self.phase = TimerPhase::Running;
        self.action = action;
        self.prompt = prompt;
        self.target = Some(target);
        Ok(())
    }

    pub fn cancel(&mut self) {
        self.deadline_unix_ms = 0;
        self.phase = TimerPhase::Cancelled;
    }

    pub fn refresh_remaining(&mut self, now_unix_ms: u64) {
        if !self.is_running() {
            return;
        }
        let milliseconds = self.deadline_unix_ms.saturating_sub(now_unix_ms);
        self.remaining_seconds =
            milliseconds / 1_000 + u64::from(!milliseconds.is_multiple_of(1_000));
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TimerLibrary {
    pub selected_id: u32,
    pub next_id: u32,
    pub timers: Vec<TimerDefinition>,
}

impl Default for TimerLibrary {
    fn default() -> Self {
        let timer = TimerDefinition::fresh(1, "Timer utama".to_owned());
        Self {
            selected_id: timer.id,
            next_id: 2,
            timers: vec![timer],
        }
    }
}

impl TimerLibrary {
    pub fn selected(&self) -> Option<&TimerDefinition> {
        self.timers
            .iter()
            .find(|timer| timer.id == self.selected_id)
    }

    pub fn selected_mut(&mut self) -> Option<&mut TimerDefinition> {
        self.timers
            .iter_mut()
            .find(|timer| timer.id == self.selected_id)
    }

    pub fn running_count(&self) -> usize {
        self.timers
            .iter()
            .filter(|timer| timer.is_running())
            .count()
    }

    pub fn add_timer(&mut self) -> bool {
        if self.timers.len() >= MAX_TIMERS {
            return false;
        }
        let id = self.next_id.max(1);
        self.next_id = id.saturating_add(1).max(1);
        let timer = TimerDefinition::fresh(id, format!("Timer {:02}", self.timers.len() + 1));
        self.selected_id = id;
        self.timers.push(timer);
        true
    }

    pub fn duplicate_selected(&mut self) -> bool {
        if self.timers.len() >= MAX_TIMERS {
            return false;
        }
        let Some(mut duplicate) = self.selected().cloned() else {
            return false;
        };
        let id = self.next_id.max(1);
        self.next_id = id.saturating_add(1).max(1);
        duplicate.id = id;
        duplicate.name = format!("{} copy", duplicate.name);
        duplicate.phase = TimerPhase::Idle;
        duplicate.deadline_unix_ms = 0;
        duplicate.remaining_seconds = duplicate.duration_seconds;
        self.selected_id = id;
        self.timers.push(duplicate);
        true
    }

    pub fn delete_selected(&mut self) -> bool {
        if self.timers.len() <= 1 || self.selected().is_some_and(TimerDefinition::is_running) {
            return false;
        }
        let selected_id = self.selected_id;
        self.timers.retain(|timer| timer.id != selected_id);
        self.selected_id = self.timers.first().map_or(0, |timer| timer.id);
        true
    }

    pub fn refresh_due(&mut self, now_unix_ms: u64) -> Vec<u32> {
        let mut due = Vec::new();
        for timer in &mut self.timers {
            if !timer.is_running() {
                continue;
            }
            if timer.deadline_unix_ms <= now_unix_ms {
                timer.remaining_seconds = 0;
                timer.deadline_unix_ms = 0;
                timer.phase = TimerPhase::Dispatching;
                due.push(timer.id);
            } else {
                timer.refresh_remaining(now_unix_ms);
            }
        }
        due
    }

    pub fn recover_after_restart(&mut self, now_unix_ms: u64) -> usize {
        let mut missed = 0;
        for timer in &mut self.timers {
            if timer.phase == TimerPhase::Dispatching {
                timer.remaining_seconds = 0;
                timer.deadline_unix_ms = 0;
                timer.phase = TimerPhase::Missed;
                missed += 1;
                continue;
            }
            if !timer.is_running() {
                continue;
            }
            if timer.deadline_unix_ms <= now_unix_ms {
                timer.remaining_seconds = 0;
                timer.deadline_unix_ms = 0;
                timer.phase = TimerPhase::Missed;
                missed += 1;
            } else {
                timer.refresh_remaining(now_unix_ms);
            }
        }
        missed
    }

    pub fn mark_result(&mut self, id: u32, succeeded: bool) {
        if let Some(timer) = self.timers.iter_mut().find(|timer| timer.id == id) {
            timer.phase = if succeeded {
                TimerPhase::Completed
            } else {
                TimerPhase::Failed
            };
            timer.deadline_unix_ms = 0;
            timer.remaining_seconds = 0;
        }
    }

    pub fn cancel_all(&mut self) -> usize {
        let mut cancelled = 0;
        for timer in &mut self.timers {
            if timer.is_running() || timer.phase == TimerPhase::Dispatching {
                timer.cancel();
                cancelled += 1;
            }
        }
        cancelled
    }
}

pub fn now_unix_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .min(u128::from(u64::MAX)) as u64
}

pub fn default_timers_path() -> PathBuf {
    let base = std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);
    base.join("VibeTimer").join("timers.vtt")
}

pub fn encode_timers(library: &TimerLibrary) -> io::Result<Vec<u8>> {
    if library.timers.is_empty() || library.timers.len() > MAX_TIMERS {
        return Err(invalid_data("Jumlah timer tidak valid."));
    }
    let mut output = Vec::new();
    output.extend_from_slice(MAGIC);
    put_u16(&mut output, VERSION);
    put_u32(&mut output, library.selected_id);
    put_u32(&mut output, library.next_id);
    put_u16(&mut output, library.timers.len() as u16);
    for timer in &library.timers {
        put_u32(&mut output, timer.id);
        put_string(&mut output, &timer.name)?;
        put_u64(&mut output, timer.duration_seconds);
        put_u64(&mut output, timer.remaining_seconds);
        put_u64(&mut output, timer.deadline_unix_ms);
        output.push(phase_to_byte(timer.phase));
        output.push(match timer.action {
            TimerAction::EnterOnly => 0,
            TimerAction::TextAndEnter => 1,
        });
        put_string(&mut output, &timer.prompt)?;
        output.push(u8::from(timer.target.is_some()));
        if let Some(target) = &timer.target {
            put_string(&mut output, &target.executable)?;
            put_string(&mut output, &target.window_title)?;
        }
    }
    Ok(output)
}

pub fn decode_timers(bytes: &[u8]) -> io::Result<TimerLibrary> {
    let mut reader = Reader::new(bytes);
    if reader.take(4)? != MAGIC {
        return Err(invalid_data("Magic file timer tidak cocok."));
    }
    if reader.u16()? != VERSION {
        return Err(invalid_data("Versi file timer belum didukung."));
    }
    let selected_id = reader.u32()?;
    let next_id = reader.u32()?;
    let count = reader.u16()? as usize;
    if count == 0 || count > MAX_TIMERS {
        return Err(invalid_data("Jumlah timer tidak valid."));
    }
    let mut timers = Vec::with_capacity(count);
    for _ in 0..count {
        let id = reader.u32()?;
        let name = reader.string()?;
        let duration_seconds = reader.u64()?;
        let remaining_seconds = reader.u64()?;
        let deadline_unix_ms = reader.u64()?;
        if duration_seconds == 0 || duration_seconds > MAX_DURATION_SECONDS {
            return Err(invalid_data("Durasi timer di luar batas."));
        }
        let phase = byte_to_phase(reader.byte()?)?;
        let action = match reader.byte()? {
            0 => TimerAction::EnterOnly,
            1 => TimerAction::TextAndEnter,
            _ => return Err(invalid_data("Aksi timer tidak valid.")),
        };
        let prompt = reader.string()?;
        let target = match reader.byte()? {
            0 => None,
            1 => Some(MacroTarget {
                executable: reader.string()?,
                window_title: reader.string()?,
            }),
            _ => return Err(invalid_data("Flag target timer tidak valid.")),
        };
        timers.push(TimerDefinition {
            id,
            name,
            duration_seconds,
            remaining_seconds: remaining_seconds.min(duration_seconds),
            deadline_unix_ms,
            phase,
            action,
            prompt,
            target,
        });
    }
    if !reader.is_finished() {
        return Err(invalid_data("File timer memiliki data tambahan."));
    }
    if !timers.iter().any(|timer| timer.id == selected_id) {
        return Err(invalid_data("Timer terpilih tidak ditemukan."));
    }
    let mut ids: Vec<u32> = timers.iter().map(|timer| timer.id).collect();
    ids.sort_unstable();
    ids.dedup();
    if ids.len() != timers.len() {
        return Err(invalid_data("ID timer duplikat."));
    }
    Ok(TimerLibrary {
        selected_id,
        next_id: next_id.max(ids.last().copied().unwrap_or(0).saturating_add(1)),
        timers,
    })
}

pub fn load_timers(path: &Path) -> io::Result<TimerLibrary> {
    if !path.exists() {
        return Ok(TimerLibrary::default());
    }
    decode_timers(&fs::read(path)?)
}

pub fn save_timers(path: &Path, library: &TimerLibrary) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let temporary = path.with_extension("vtt.tmp");
    let backup = path.with_extension("vtt.bak");
    fs::write(&temporary, encode_timers(library)?)?;
    if path.exists() {
        let _ = fs::remove_file(&backup);
        fs::rename(path, &backup)?;
    }
    if let Err(error) = fs::rename(&temporary, path) {
        if backup.exists() {
            let _ = fs::rename(&backup, path);
        }
        return Err(error);
    }
    let _ = fs::remove_file(&backup);
    Ok(())
}

fn phase_to_byte(phase: TimerPhase) -> u8 {
    match phase {
        TimerPhase::Idle => 0,
        TimerPhase::Running => 1,
        TimerPhase::Dispatching => 2,
        TimerPhase::Completed => 3,
        TimerPhase::Failed => 4,
        TimerPhase::Missed => 5,
        TimerPhase::Cancelled => 6,
    }
}

fn byte_to_phase(value: u8) -> io::Result<TimerPhase> {
    match value {
        0 => Ok(TimerPhase::Idle),
        1 => Ok(TimerPhase::Running),
        2 => Ok(TimerPhase::Dispatching),
        3 => Ok(TimerPhase::Completed),
        4 => Ok(TimerPhase::Failed),
        5 => Ok(TimerPhase::Missed),
        6 => Ok(TimerPhase::Cancelled),
        _ => Err(invalid_data("Status timer tidak valid.")),
    }
}

fn invalid_data(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

fn put_u16(output: &mut Vec<u8>, value: u16) {
    output.extend_from_slice(&value.to_le_bytes());
}

fn put_u32(output: &mut Vec<u8>, value: u32) {
    output.extend_from_slice(&value.to_le_bytes());
}

fn put_u64(output: &mut Vec<u8>, value: u64) {
    output.extend_from_slice(&value.to_le_bytes());
}

fn put_string(output: &mut Vec<u8>, value: &str) -> io::Result<()> {
    if value.len() > MAX_STRING_BYTES {
        return Err(invalid_data("Teks timer terlalu panjang."));
    }
    put_u32(output, value.len() as u32);
    output.extend_from_slice(value.as_bytes());
    Ok(())
}

struct Reader<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> Reader<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }

    fn take(&mut self, length: usize) -> io::Result<&'a [u8]> {
        let end = self
            .offset
            .checked_add(length)
            .ok_or_else(|| invalid_data("Ukuran file timer overflow."))?;
        let slice = self
            .bytes
            .get(self.offset..end)
            .ok_or_else(|| invalid_data("File timer terpotong."))?;
        self.offset = end;
        Ok(slice)
    }

    fn byte(&mut self) -> io::Result<u8> {
        Ok(self.take(1)?[0])
    }

    fn u16(&mut self) -> io::Result<u16> {
        Ok(u16::from_le_bytes(self.take(2)?.try_into().unwrap()))
    }

    fn u32(&mut self) -> io::Result<u32> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }

    fn u64(&mut self) -> io::Result<u64> {
        Ok(u64::from_le_bytes(self.take(8)?.try_into().unwrap()))
    }

    fn string(&mut self) -> io::Result<String> {
        let length = self.u32()? as usize;
        if length > MAX_STRING_BYTES {
            return Err(invalid_data("Teks timer terlalu panjang."));
        }
        String::from_utf8(self.take(length)?.to_vec())
            .map_err(|_| invalid_data("Teks timer bukan UTF-8 valid."))
    }

    fn is_finished(&self) -> bool {
        self.offset == self.bytes.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn target() -> MacroTarget {
        MacroTarget {
            executable: "editor.exe".to_owned(),
            window_title: "AI Editor".to_owned(),
        }
    }

    #[test]
    fn concurrent_timers_refresh_and_dispatch_once() {
        let mut library = TimerLibrary::default();
        library
            .selected_mut()
            .unwrap()
            .start(1_000, 2, TimerAction::EnterOnly, String::new(), target())
            .unwrap();
        assert!(library.add_timer());
        library
            .selected_mut()
            .unwrap()
            .start(
                1_000,
                5,
                TimerAction::TextAndEnter,
                "lanjut".to_owned(),
                target(),
            )
            .unwrap();
        assert_eq!(library.running_count(), 2);
        assert!(library.refresh_due(2_999).is_empty());
        assert_eq!(library.timers[0].remaining_seconds, 1);
        assert_eq!(library.refresh_due(3_000), vec![1]);
        assert!(library.refresh_due(3_000).is_empty());
        library.mark_result(1, true);
        assert_eq!(library.timers[0].phase, TimerPhase::Completed);
        assert_eq!(library.running_count(), 1);
    }

    #[test]
    fn restart_recovers_future_and_marks_past_timer_missed() {
        let mut library = TimerLibrary::default();
        library
            .selected_mut()
            .unwrap()
            .start(10_000, 2, TimerAction::EnterOnly, String::new(), target())
            .unwrap();
        assert!(library.add_timer());
        library
            .selected_mut()
            .unwrap()
            .start(10_000, 10, TimerAction::EnterOnly, String::new(), target())
            .unwrap();
        assert!(library.add_timer());
        library.selected_mut().unwrap().phase = TimerPhase::Dispatching;
        assert_eq!(library.recover_after_restart(13_000), 2);
        assert_eq!(library.timers[0].phase, TimerPhase::Missed);
        assert_eq!(library.timers[1].phase, TimerPhase::Running);
        assert_eq!(library.timers[1].remaining_seconds, 7);
        assert_eq!(library.timers[2].phase, TimerPhase::Missed);
    }

    #[test]
    fn library_round_trips_every_field_and_rejects_damage() {
        let mut library = TimerLibrary::default();
        let timer = library.selected_mut().unwrap();
        timer.name = "Claude reset".to_owned();
        timer
            .start(
                5_000,
                4_321,
                TimerAction::TextAndEnter,
                "lanjutkan".to_owned(),
                target(),
            )
            .unwrap();
        let encoded = encode_timers(&library).unwrap();
        assert_eq!(decode_timers(&encoded).unwrap(), library);
        assert!(decode_timers(&encoded[..encoded.len() - 1]).is_err());
        let mut damaged = encoded;
        damaged[0] = b'X';
        assert!(decode_timers(&damaged).is_err());
    }

    #[test]
    fn add_duplicate_delete_and_atomic_save_are_safe() {
        let mut library = TimerLibrary::default();
        assert!(library.add_timer());
        assert!(library.duplicate_selected());
        assert!(library.delete_selected());
        library.selected_id = library.timers[0].id;
        library.timers[0].phase = TimerPhase::Running;
        assert!(!library.delete_selected());
        let path =
            std::env::temp_dir().join(format!("vibe-timer-{}-timers.vtt", std::process::id()));
        let _ = fs::remove_file(&path);
        save_timers(&path, &library).unwrap();
        assert_eq!(load_timers(&path).unwrap(), library);
        let _ = fs::remove_file(&path);
    }
}
