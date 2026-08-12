//! Logika murni Vibemacro yang dapat diuji tanpa Windows UI.

pub mod backup;
pub mod macro_engine;
pub mod profiles;
pub mod settings;
pub mod smart_reset;
pub mod timers;
pub mod updater;

pub const MAX_HOURS: u32 = 168;
pub const MAX_TOTAL_SECONDS: u64 = 7 * 24 * 3_600;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DurationFields {
    pub hours: u32,
    pub minutes: u32,
    pub seconds: u32,
}

impl DurationFields {
    pub const fn new(hours: u32, minutes: u32, seconds: u32) -> Self {
        Self {
            hours,
            minutes,
            seconds,
        }
    }

    pub fn validate(self) -> Result<u64, &'static str> {
        if self.minutes > 59 || self.seconds > 59 {
            return Err("Menit dan detik harus antara 00 sampai 59.");
        }

        let total = self.hours as u64 * 3_600 + self.minutes as u64 * 60 + self.seconds as u64;
        if self.hours > MAX_HOURS || total > MAX_TOTAL_SECONDS {
            return Err("Durasi maksimal 7 hari.");
        }
        if total == 0 {
            return Err("Atur waktu lebih dari 00:00:00.");
        }
        Ok(total)
    }

    pub fn from_total_seconds(total: u64) -> Self {
        let clamped = total.min(MAX_TOTAL_SECONDS);
        Self {
            hours: (clamped / 3_600) as u32,
            minutes: ((clamped % 3_600) / 60) as u32,
            seconds: (clamped % 60) as u32,
        }
    }

    pub fn add_seconds(self, delta: u64) -> Self {
        let current = self.hours as u64 * 3_600
            + self.minutes.min(59) as u64 * 60
            + self.seconds.min(59) as u64;
        Self::from_total_seconds(current.saturating_add(delta))
    }
}

pub fn format_duration(total_seconds: u64) -> String {
    let value = DurationFields::from_total_seconds(total_seconds);
    format!(
        "{:02}:{:02}:{:02}",
        value.hours, value.minutes, value.seconds
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_a_normal_reset_window() {
        assert_eq!(DurationFields::new(3, 27, 10).validate(), Ok(12_430));
    }

    #[test]
    fn rejects_zero_and_invalid_minute_fields() {
        assert_eq!(
            DurationFields::new(0, 0, 0).validate(),
            Err("Atur waktu lebih dari 00:00:00.")
        );
        assert_eq!(
            DurationFields::new(1, 60, 0).validate(),
            Err("Menit dan detik harus antara 00 sampai 59.")
        );
    }

    #[test]
    fn presets_roll_over_and_clamp() {
        assert_eq!(
            DurationFields::new(0, 45, 30).add_seconds(30 * 60),
            DurationFields::new(1, 15, 30)
        );
        assert_eq!(
            DurationFields::new(168, 0, 0).add_seconds(60),
            DurationFields::new(168, 0, 0)
        );
    }

    #[test]
    fn formats_countdown_with_leading_zeroes() {
        assert_eq!(format_duration(12_420), "03:27:00");
        assert_eq!(format_duration(5), "00:00:05");
    }
}
