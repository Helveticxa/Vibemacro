# Contributing to Vibemacro

Terima kasih ingin membantu. Gunakan issue untuk bug/feature proposal dan pull
request kecil yang fokus pada satu perubahan.

## Development gate

```powershell
cargo fmt -- --check
cargo clippy --all-targets -- -D warnings
cargo test --all-targets -- --test-threads=1
.\tools\security-scan.ps1 -IncludeHistory
```

Untuk perubahan installer, jalankan `tools\build-release.ps1` dan uji install,
upgrade, serta uninstall pada Windows x64.

## Aturan keselamatan

- Jangan commit API key, token, password, certificate, atau private key.
- Jangan menambah bypass anti-cheat, privilege boundary, atau stealth behavior.
- Pertahankan input routing fail-safe: target hilang tidak boleh fallback ke
  foreground.
- Jangan menambah telemetry tanpa proposal dan persetujuan eksplisit.
- Pertahankan kompatibilitas data VibeTimer/Vibemacro yang sudah ada.

Dengan mengirim kontribusi, Anda menyetujui kontribusi tersebut dilisensikan
di bawah MIT License repository ini.
