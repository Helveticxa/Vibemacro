# VibeTimer 1.0 — QA Evidence

Tanggal verifikasi: 12 Agustus 2026
Platform: Windows x64, Rust 1.97.1, Inno Setup 7.0.2

## Status gerbang final

| Pemeriksaan | Hasil |
|---|---|
| `cargo fmt -- --check` | Lulus |
| `cargo clippy --all-targets -- -D warnings` | Lulus, 0 warning |
| `cargo test --all-targets -- --test-threads=1` | Lulus, 31 unit + 2 Windows E2E |
| `cargo build --release` | Lulus |
| Dependency runtime eksternal | Tidak ada |
| Install → launch → uninstall | Lulus |
| Microsoft Defender custom scan | Tidak ada threat pada EXE dan installer |
| Authenticode artefak VibeTimer | `NotSigned` |

## Artefak final

| Artefak | Ukuran | SHA-256 |
|---|---:|---|
| `target/release/VibeTimer.exe` | 388.608 byte | `E746F8C06D1411BE900669D78FFB3FAA42A2AB8D6644310941707E45BA1613BC` |
| `dist/VibeTimer-Setup-1.0.0-x64.exe` | 2.440.989 byte | `8E2AFDC6B11CDB840F95CBD12E4E3975C159760189160976F01917268FE8DBBF` |

## Cakupan otomatis

Unit test mencakup validasi waktu, Smart Reset, version comparison, parser
manifest HTTPS, SHA-256 known vectors, serialisasi dan migrasi state, CRC backup,
penyimpanan atomik, recovery backup, timeline editing, batas library, batas
delay, restart recovery timer, serta dua timer konkuren yang dispatch tepat satu
kali per timer.

Windows E2E membuat UI dan target edit control sungguhan, lalu memverifikasi:

- render tab Timer, Macro, Profiles, dan Settings;
- Timer mode Enter serta Teks + Enter, cancel tanpa input, target PID salah;
- Smart Reset dan Multi Timer;
- macro recorder keyboard, mouse, wheel, dan `Esc`;
- empat mode macro, lima pemicu, tiga lane, dan timeline editor;
- playback Window Target tetap berjalan saat Alt+Tab tanpa mengubah foreground;
- target yang ditutup menghentikan loop tanpa fallback ke window lain;
- Tray Mode, Auto Start test double, dan Emergency Stop;
- App Profiles, backup export/import, serta rollback state;
- hook dinamis: tidak aktif untuk macro kosong, keduanya aktif saat recording,
  lalu hanya hook yang diperlukan tetap terpasang;
- updater lengkap dengan feed file lokal khusus `cfg(test)`: check → download →
  SHA-256 → installer ready. Jalur production tetap HTTPS-only.

## E2E installer

Installer dibangun per-user dengan AppId stabil dan diuji pada root QA
terisolasi:

1. install silent selesai dengan exit code 0;
2. hash executable terpasang identik dengan executable release;
3. aplikasi terpasang dapat berjalan dan responsive;
4. instance kedua keluar dengan code 0 dan jumlah proses tetap satu;
5. uninstaller selesai dengan exit code 0;
6. direktori program terhapus;
7. marker data pengguna tetap tersedia sesudah uninstall.

Wizard installer juga dibuka secara interaktif dan diperiksa pada resolusi asli.
Hasilnya full dark, ikon VibeTimer benar, teks dan destination terbaca, serta
tombol Back/Install/Cancel tidak bertabrakan.

## Audit keamanan dan reliabilitas

- Named mutex mencegah beberapa instance mengirim input bersamaan.
- Clipboard reader dibatasi oleh ukuran alokasi dan terminator UTF-16.
- File state memakai temporary file, `sync_all`, backup, dan rollback best-effort.
- Import memvalidasi semua section sebelum mengganti state aktif.
- ID, nama, target, jumlah object, dan delay divalidasi saat decode/encode.
- Installer update tidak dapat dijalankan sebelum SHA-256 cocok.
- Feed dan URL installer production wajib HTTPS.
- Update ditolak ketika timer/macro/recording aktif atau perubahan belum disimpan.
- Uninstaller hanya menghapus Run value jika command-nya tepat menunjuk instalasi
  VibeTimer yang sedang dihapus.
- `%LOCALAPPDATA%\VibeTimer` tidak menjadi target uninstall.

Microsoft Defender memindai `VibeTimer.exe` dan installer final dengan custom
scan tanpa remediation. Keduanya selesai exit code 0 dan melaporkan tidak ada
threat. Hasil ini terbatas pada engine/signature Defender saat pengujian dan
bukan jaminan universal bahwa software bebas dari seluruh kemungkinan risiko.

## Performa profil kosong terisolasi

Pengukuran executable v1.0 final selama delapan detik, dengan data QA kosong dan
update startup nonaktif:

| Metrik | Hasil |
|---|---:|
| CPU | 0,000% dari satu core |
| Thread | 4 → 4 |
| Handle | 146 → 146 |
| Working set | 10,41 MiB |
| Private memory | 1,86 MiB |
| Status | Responsive |

## Risiko tersisa

- EXE dan installer belum ditandatangani. SmartScreen dapat memperingatkan
  Unknown Publisher sampai tersedia sertifikat code-signing dan reputasi rilis.
- Background macro bergantung pada dukungan window message aplikasi target.
- Raw Input, DirectInput, anti-cheat, dan target elevated dapat menolak input.
- Feed update online belum diaktifkan karena proyek belum memiliki URL rilis
  HTTPS. Engine sudah ada, default off, dan tidak mengklaim update online aktif.
