const MAX_CAPTURE_SECONDS: u64 = 7 * 24 * 60 * 60;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClockContext {
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
    /// 0 = Minggu, 1 = Senin, ... 6 = Sabtu.
    pub weekday: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CaptureKind {
    RelativeDuration,
    ClockTime,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResetCapture {
    pub seconds: u64,
    pub kind: CaptureKind,
    pub summary: String,
}

pub fn parse_reset_text(text: &str, now: ClockContext) -> Result<ResetCapture, &'static str> {
    let normalized = text.to_lowercase();
    if let Some(seconds) = parse_duration(&normalized)? {
        return Ok(ResetCapture {
            seconds,
            kind: CaptureKind::RelativeDuration,
            summary: format!("Durasi reset terbaca: {}", format_duration_words(seconds)),
        });
    }
    if let Some(seconds) = parse_clock_time(&normalized, now)? {
        return Ok(ResetCapture {
            seconds,
            kind: CaptureKind::ClockTime,
            summary: format!("Jam reset terbaca: {} lagi", format_duration_words(seconds)),
        });
    }
    Err("Teks reset tidak dikenali. Contoh: 'Resets in 3 h 27 min' atau 'Resets at 4:59 AM'.")
}

fn parse_duration(text: &str) -> Result<Option<u64>, &'static str> {
    let bytes = text.as_bytes();
    let mut index = 0;
    let mut total = 0u64;
    let mut matched = false;
    while index < bytes.len() {
        if !bytes[index].is_ascii_digit() {
            index += 1;
            continue;
        }
        let number_start = index;
        while index < bytes.len() && bytes[index].is_ascii_digit() {
            index += 1;
        }
        let value = text[number_start..index]
            .parse::<u64>()
            .map_err(|_| "Angka durasi terlalu besar.")?;
        while index < bytes.len() && bytes[index].is_ascii_whitespace() {
            index += 1;
        }
        let unit_start = index;
        while index < bytes.len() && bytes[index].is_ascii_alphabetic() {
            index += 1;
        }
        let unit = &text[unit_start..index];
        let multiplier = match unit {
            "d" | "day" | "days" | "hari" => Some(24 * 60 * 60),
            "h" | "hr" | "hrs" | "hour" | "hours" | "jam" => Some(60 * 60),
            "m" | "min" | "mins" | "minute" | "minutes" | "menit" => Some(60),
            "s" | "sec" | "secs" | "second" | "seconds" | "detik" => Some(1),
            _ => None,
        };
        if let Some(multiplier) = multiplier {
            matched = true;
            total = total
                .checked_add(
                    value
                        .checked_mul(multiplier)
                        .ok_or("Durasi reset terlalu besar.")?,
                )
                .ok_or("Durasi reset terlalu besar.")?;
        }
    }
    if !matched {
        return Ok(None);
    }
    if total == 0 || total > MAX_CAPTURE_SECONDS {
        return Err("Durasi reset harus lebih dari nol dan maksimal 7 hari.");
    }
    Ok(Some(total))
}

fn parse_clock_time(text: &str, now: ClockContext) -> Result<Option<u64>, &'static str> {
    let Some(colon) = text.find(':') else {
        return Ok(None);
    };
    let bytes = text.as_bytes();
    let mut hour_start = colon;
    while hour_start > 0 && bytes[hour_start - 1].is_ascii_digit() {
        hour_start -= 1;
    }
    let mut minute_end = colon + 1;
    while minute_end < bytes.len() && bytes[minute_end].is_ascii_digit() {
        minute_end += 1;
    }
    if hour_start == colon || minute_end == colon + 1 {
        return Ok(None);
    }
    let mut hour = text[hour_start..colon]
        .parse::<u8>()
        .map_err(|_| "Jam reset tidak valid.")?;
    let minute = text[colon + 1..minute_end]
        .parse::<u8>()
        .map_err(|_| "Menit reset tidak valid.")?;
    if minute > 59 {
        return Err("Menit reset harus 00 sampai 59.");
    }
    let suffix = text[minute_end..].trim_start();
    let has_am = suffix.starts_with("am") || suffix.starts_with("a.m.");
    let has_pm = suffix.starts_with("pm") || suffix.starts_with("p.m.");
    if has_am || has_pm {
        if !(1..=12).contains(&hour) {
            return Err("Jam AM/PM harus 1 sampai 12.");
        }
        if hour == 12 {
            hour = 0;
        }
        if has_pm {
            hour += 12;
        }
    } else if hour > 23 {
        return Err("Jam reset harus 00 sampai 23.");
    }

    let target_weekday = parse_weekday(text);
    let now_seconds =
        u64::from(now.hour) * 3_600 + u64::from(now.minute) * 60 + u64::from(now.second);
    let target_seconds = u64::from(hour) * 3_600 + u64::from(minute) * 60;
    let days_ahead = if let Some(target_day) = target_weekday {
        let mut distance = (7 + i16::from(target_day) - i16::from(now.weekday)) % 7;
        if distance == 0 && target_seconds <= now_seconds {
            distance = 7;
        }
        distance as u64
    } else if target_seconds <= now_seconds {
        1
    } else {
        0
    };
    let seconds = days_ahead * 24 * 3_600 + target_seconds;
    let seconds = seconds.saturating_sub(now_seconds);
    if seconds == 0 || seconds > MAX_CAPTURE_SECONDS {
        return Err("Jam reset berada di luar rentang 7 hari.");
    }
    Ok(Some(seconds))
}

fn parse_weekday(text: &str) -> Option<u8> {
    let words = text.split(|character: char| !character.is_ascii_alphabetic());
    for word in words {
        let day = match word {
            "sun" | "sunday" | "minggu" => 0,
            "mon" | "monday" | "senin" => 1,
            "tue" | "tues" | "tuesday" | "selasa" => 2,
            "wed" | "wednesday" | "rabu" => 3,
            "thu" | "thur" | "thurs" | "thursday" | "kamis" => 4,
            "fri" | "friday" | "jumat" => 5,
            "sat" | "saturday" | "sabtu" => 6,
            _ => continue,
        };
        return Some(day);
    }
    None
}

fn format_duration_words(seconds: u64) -> String {
    let days = seconds / 86_400;
    let hours = seconds % 86_400 / 3_600;
    let minutes = seconds % 3_600 / 60;
    let seconds = seconds % 60;
    let mut parts = Vec::new();
    if days > 0 {
        parts.push(format!("{days} hari"));
    }
    if hours > 0 {
        parts.push(format!("{hours} jam"));
    }
    if minutes > 0 {
        parts.push(format!("{minutes} menit"));
    }
    if seconds > 0 && parts.len() < 2 {
        parts.push(format!("{seconds} detik"));
    }
    parts.join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn monday_noon() -> ClockContext {
        ClockContext {
            hour: 12,
            minute: 0,
            second: 0,
            weekday: 1,
        }
    }

    #[test]
    fn parses_claude_and_indonesian_relative_durations() {
        assert_eq!(
            parse_reset_text("Resets in 3 h 27 min", monday_noon())
                .unwrap()
                .seconds,
            12_420
        );
        assert_eq!(
            parse_reset_text("Try again after 2hours 15minutes", monday_noon())
                .unwrap()
                .seconds,
            8_100
        );
        assert_eq!(
            parse_reset_text("Reset dalam 1 jam 8 menit 4 detik", monday_noon())
                .unwrap()
                .seconds,
            4_084
        );
    }

    #[test]
    fn parses_clock_today_tomorrow_and_weekday() {
        assert_eq!(
            parse_reset_text("Resets at 4:59 PM", monday_noon())
                .unwrap()
                .seconds,
            17_940
        );
        assert_eq!(
            parse_reset_text("Resets at 4:59 AM", monday_noon())
                .unwrap()
                .seconds,
            61_140
        );
        assert_eq!(
            parse_reset_text("Weekly resets Fri 4:59 AM", monday_noon())
                .unwrap()
                .seconds,
            320_340
        );
    }

    #[test]
    fn rejects_noise_zero_and_out_of_range_values() {
        assert!(parse_reset_text("usage 53%", monday_noon()).is_err());
        assert!(parse_reset_text("resets in 0 min", monday_noon()).is_err());
        assert!(parse_reset_text("resets in 8 days", monday_noon()).is_err());
        assert!(parse_reset_text("resets at 29:99", monday_noon()).is_err());
    }
}
