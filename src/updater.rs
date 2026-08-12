//! Parser manifest dan verifikasi artefak update tanpa dependency eksternal.

use std::fs;
use std::io;
use std::path::Path;

const MANIFEST_HEADER: &str = "VIBEMACRO-UPDATE-1";
const MAX_MANIFEST_BYTES: usize = 8 * 1024;
pub const DEFAULT_FEED_URL: &str =
    "https://github.com/Helveticxa/Vibemacro/releases/latest/download/vibemacro-update.txt";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateManifest {
    pub version: String,
    pub installer_url: String,
    pub sha256: [u8; 32],
}

pub fn configured_feed_url() -> Option<String> {
    std::env::var("VIBEMACRO_UPDATE_FEED_URL")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .or_else(|| {
            std::env::var("VIBETIMER_UPDATE_FEED_URL")
                .ok()
                .filter(|value| !value.trim().is_empty())
        })
        .or_else(|| option_env!("VIBEMACRO_UPDATE_FEED_URL").map(str::to_owned))
        .or_else(|| Some(DEFAULT_FEED_URL.to_owned()))
}

pub fn parse_manifest(bytes: &[u8]) -> Result<UpdateManifest, &'static str> {
    parse_manifest_with_policy(bytes, false)
}

#[doc(hidden)]
pub fn parse_test_manifest(bytes: &[u8]) -> Result<UpdateManifest, &'static str> {
    parse_manifest_with_policy(bytes, true)
}

fn parse_manifest_with_policy(
    bytes: &[u8],
    allow_local_file: bool,
) -> Result<UpdateManifest, &'static str> {
    if bytes.len() > MAX_MANIFEST_BYTES {
        return Err("Manifest update terlalu besar.");
    }
    let text = std::str::from_utf8(bytes).map_err(|_| "Manifest update bukan UTF-8.")?;
    let mut lines = text.lines();
    if lines.next().map(str::trim) != Some(MANIFEST_HEADER) {
        return Err("Header manifest update tidak valid.");
    }
    let mut version = None;
    let mut installer_url = None;
    let mut sha256 = None;
    for line in lines {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let (key, value) = line
            .split_once('=')
            .ok_or("Baris manifest update tidak valid.")?;
        let value = value.trim();
        if value.is_empty() {
            return Err("Nilai manifest update tidak boleh kosong.");
        }
        match key.trim() {
            "version" if version.is_none() => version = Some(value.to_owned()),
            "installer" if installer_url.is_none() => installer_url = Some(value.to_owned()),
            "sha256" if sha256.is_none() => sha256 = Some(parse_sha256(value)?),
            "version" | "installer" | "sha256" => {
                return Err("Kunci manifest update duplikat.");
            }
            _ => return Err("Kunci manifest update tidak dikenal."),
        }
    }
    let version = version.ok_or("Versi update tidak tersedia.")?;
    parse_version(&version)?;
    let installer_url = installer_url.ok_or("URL installer tidak tersedia.")?;
    let secure = installer_url.starts_with("https://")
        || (allow_local_file && installer_url.starts_with("file:///"));
    if !secure || installer_url.chars().any(char::is_whitespace) || installer_url.contains('#') {
        return Err("URL installer wajib HTTPS yang valid.");
    }
    Ok(UpdateManifest {
        version,
        installer_url,
        sha256: sha256.ok_or("SHA-256 installer tidak tersedia.")?,
    })
}

pub fn validate_feed_url(url: &str) -> Result<(), &'static str> {
    validate_feed_url_with_policy(url, false)
}

#[doc(hidden)]
pub fn validate_test_feed_url(url: &str) -> Result<(), &'static str> {
    validate_feed_url_with_policy(url, true)
}

fn validate_feed_url_with_policy(url: &str, allow_local_file: bool) -> Result<(), &'static str> {
    let secure = url.starts_with("https://") || (allow_local_file && url.starts_with("file:///"));
    if !secure || url.chars().any(char::is_whitespace) || url.contains('#') {
        return Err("Feed update wajib memakai URL HTTPS yang valid.");
    }
    Ok(())
}

pub fn is_newer_version(candidate: &str, current: &str) -> Result<bool, &'static str> {
    Ok(parse_version(candidate)? > parse_version(current)?)
}

fn parse_version(value: &str) -> Result<(u32, u32, u32), &'static str> {
    let value = value.strip_prefix('v').unwrap_or(value);
    let mut parts = value.split('.');
    let major = parse_version_part(parts.next())?;
    let minor = parse_version_part(parts.next())?;
    let patch = parse_version_part(parts.next())?;
    if parts.next().is_some() {
        return Err("Format versi update tidak valid.");
    }
    Ok((major, minor, patch))
}

fn parse_version_part(value: Option<&str>) -> Result<u32, &'static str> {
    let value = value.ok_or("Format versi update tidak valid.")?;
    if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err("Format versi update tidak valid.");
    }
    value
        .parse::<u32>()
        .map_err(|_| "Komponen versi update terlalu besar.")
}

fn parse_sha256(value: &str) -> Result<[u8; 32], &'static str> {
    if value.len() != 64 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err("SHA-256 installer tidak valid.");
    }
    let mut digest = [0u8; 32];
    for (index, slot) in digest.iter_mut().enumerate() {
        *slot = u8::from_str_radix(&value[index * 2..index * 2 + 2], 16)
            .map_err(|_| "SHA-256 installer tidak valid.")?;
    }
    Ok(digest)
}

pub fn sha256_file(path: &Path) -> io::Result<[u8; 32]> {
    fs::read(path).map(|bytes| sha256(&bytes))
}

pub fn sha256(bytes: &[u8]) -> [u8; 32] {
    const INITIAL: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
        0x5be0cd19,
    ];
    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
        0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
        0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
        0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
        0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
        0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
        0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
        0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
        0xc67178f2,
    ];
    let bit_len = (bytes.len() as u64).wrapping_mul(8);
    let mut padded = Vec::with_capacity((bytes.len() + 72) & !63);
    padded.extend_from_slice(bytes);
    padded.push(0x80);
    while padded.len() % 64 != 56 {
        padded.push(0);
    }
    padded.extend_from_slice(&bit_len.to_be_bytes());

    let mut state = INITIAL;
    for chunk in padded.chunks_exact(64) {
        let mut schedule = [0u32; 64];
        for (index, word) in schedule.iter_mut().take(16).enumerate() {
            let offset = index * 4;
            *word = u32::from_be_bytes([
                chunk[offset],
                chunk[offset + 1],
                chunk[offset + 2],
                chunk[offset + 3],
            ]);
        }
        for index in 16..64 {
            let s0 = schedule[index - 15].rotate_right(7)
                ^ schedule[index - 15].rotate_right(18)
                ^ (schedule[index - 15] >> 3);
            let s1 = schedule[index - 2].rotate_right(17)
                ^ schedule[index - 2].rotate_right(19)
                ^ (schedule[index - 2] >> 10);
            schedule[index] = schedule[index - 16]
                .wrapping_add(s0)
                .wrapping_add(schedule[index - 7])
                .wrapping_add(s1);
        }
        let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut h] = state;
        for index in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let choice = (e & f) ^ ((!e) & g);
            let temp1 = h
                .wrapping_add(s1)
                .wrapping_add(choice)
                .wrapping_add(K[index])
                .wrapping_add(schedule[index]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let majority = (a & b) ^ (a & c) ^ (b & c);
            let temp2 = s0.wrapping_add(majority);
            h = g;
            g = f;
            f = e;
            e = d.wrapping_add(temp1);
            d = c;
            c = b;
            b = a;
            a = temp1.wrapping_add(temp2);
        }
        for (slot, value) in state.iter_mut().zip([a, b, c, d, e, f, g, h]) {
            *slot = slot.wrapping_add(value);
        }
    }
    let mut digest = [0u8; 32];
    for (chunk, word) in digest.chunks_exact_mut(4).zip(state) {
        chunk.copy_from_slice(&word.to_be_bytes());
    }
    digest
}

pub fn digest_hex(digest: &[u8; 32]) -> String {
    let mut value = String::with_capacity(64);
    for byte in digest {
        use std::fmt::Write;
        let _ = write!(value, "{byte:02X}");
    }
    value
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha256_matches_known_vectors() {
        assert_eq!(
            digest_hex(&sha256(b"")),
            "E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855"
        );
        assert_eq!(
            digest_hex(&sha256(b"abc")),
            "BA7816BF8F01CFEA414140DE5DAE2223B00361A396177A9CB410FF61F20015AD"
        );
    }

    #[test]
    fn manifest_is_strict_and_https_only() {
        let valid = b"VIBEMACRO-UPDATE-1\nversion=1.2.3\ninstaller=https://example.com/VibemacroSetup.exe\nsha256=BA7816BF8F01CFEA414140DE5DAE2223B00361A396177A9CB410FF61F20015AD\n";
        let parsed = parse_manifest(valid).expect("manifest valid");
        assert_eq!(parsed.version, "1.2.3");
        assert!(parse_manifest(&valid[..20]).is_err());
        let insecure = std::str::from_utf8(valid)
            .unwrap()
            .replace("https://", "http://");
        assert!(parse_manifest(insecure.as_bytes()).is_err());
    }

    #[test]
    fn version_comparison_is_numeric_and_strict() {
        assert_eq!(is_newer_version("1.0.1", "1.0.0"), Ok(true));
        assert_eq!(is_newer_version("1.10.0", "1.9.9"), Ok(true));
        assert_eq!(is_newer_version("1.0.0", "1.0.0"), Ok(false));
        assert!(is_newer_version("1.0-beta", "1.0.0").is_err());
    }

    #[test]
    fn default_feed_is_the_public_github_release_asset() {
        assert_eq!(
            DEFAULT_FEED_URL,
            "https://github.com/Helveticxa/Vibemacro/releases/latest/download/vibemacro-update.txt"
        );
        assert!(validate_feed_url(DEFAULT_FEED_URL).is_ok());
    }
}
