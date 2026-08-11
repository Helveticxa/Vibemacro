# VibeTimer 1.0.0

Rilis stabil pertama VibeTimer menyatukan timer AI-limit, Macro Studio,
Profiles, backup portable, keselamatan runtime, dan packaging Windows per-user.

## Roadmap yang selesai

### v0.4

- Timeline editor lengkap.
- Emergency Stop global dengan pilihan cakupan timer.
- Tray Mode, close/minimize-to-tray, dan Auto Start.
- Batas durasi serta pengulangan macro.

### v0.5

- App Profiles berdasarkan executable + judul window.
- Sinkronisasi target profile ke macro dan Timer.
- Import/export backup `.vtb` dengan CRC dan rollback aman.

### v0.6

- Smart Reset untuk durasi, jam, dan nama hari.
- Maksimal enam timer konkuren yang persisten.
- Recovery setelah restart tanpa silent dispatch untuk timer terlewat.

### v1.0

- Single-instance guard.
- Penyimpanan atomik dan recovery backup yang diperkuat.
- Hook global dinamis untuk idle CPU yang lebih rendah.
- Optional updater: HTTPS-only, manifest strict, version comparison, SHA-256,
  dan blok instalasi ketika state belum aman.
- Ikon VibeTimer final.
- Installer Inno Setup 7 per-user, dark, upgradeable, dan data-preserving.
- Dokumentasi, audit, E2E installer, dan Microsoft Defender scan final.

## Artefak

- Portable: `target/release/VibeTimer.exe`
  - 388.608 byte
  - SHA-256 `E746F8C06D1411BE900669D78FFB3FAA42A2AB8D6644310941707E45BA1613BC`
- Installer: `dist/VibeTimer-Setup-1.0.0-x64.exe`
  - 2.440.989 byte
  - SHA-256 `8E2AFDC6B11CDB840F95CBD12E4E3975C159760189160976F01917268FE8DBBF`

Kedua artefak belum ditandatangani. Windows dapat menampilkan Unknown Publisher
sampai tersedia sertifikat code-signing.

