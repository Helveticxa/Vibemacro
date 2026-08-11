# VibeTimer 1.0

VibeTimer adalah utilitas Windows native yang melanjutkan sesi AI setelah usage
limit reset dan menyediakan Macro Studio ringan untuk keyboard/mouse HID umum.
UI memakai dark graphite dengan aksen acid-lime; aplikasi dibangun dengan Rust
dan Win32 tanpa Electron, WebView, driver vendor, atau dependency runtime pihak
ketiga.

## Fitur utama

- Timer satu kali dengan aksi **Hanya Enter** atau **Teks + Enter**.
- Smart Reset untuk teks seperti `Resets in 3 h 27 min`, jam 12/24-hour,
  nama hari, dan variasi bahasa Indonesia.
- Maksimal enam timer konkuren yang persisten dan masing-masing hanya dispatch
  satu kali.
- Pemilihan target berdasarkan window handle, process ID, executable, dan judul.
- Macro recorder untuk keyboard, klik, dan wheel dengan mode **No Repeat**,
  **While Holding**, **Toggle**, dan **Sequence**.
- Timeline editor: pilih event, ubah delay `0—60000 ms`, geser, duplikat,
  hapus, dan sisipkan delay.
- Pemicu `F8`, `F9`, `Middle`, `Mouse 4`, atau `Mouse 5`.
- Window-bound background macro yang tetap berjalan pada target saat Alt+Tab,
  tanpa mengambil fokus window aktif.
- App Profiles untuk menyatukan target aplikasi dengan kumpulan macro.
- Backup portable `.vtb` untuk settings, profiles, macros, dan timers.
- Tray Mode, close/minimize-to-tray, dan Auto Start per-user.
- Emergency Stop global yang dapat ikut membatalkan semua timer.
- Batas durasi dan pengulangan macro agar loop tidak berjalan tanpa kendali.
- Optional auto-update dengan manifest HTTPS dan verifikasi SHA-256 sebelum
  installer dapat dijalankan.

## Instalasi

Jalankan `dist/VibeTimer-Setup-1.0.0-x64.exe`. Installer bersifat per-user,
tidak meminta Administrator, membuat shortcut Start Menu, menyediakan shortcut
Desktop opsional, dan mendukung upgrade in-place melalui AppId yang tetap.

Data pribadi tidak dihapus saat uninstall dan berada di:

```text
%LOCALAPPDATA%\VibeTimer
```

Auto Start dapat diaktifkan dari tab **Settings** setelah instalasi.

## Alur Timer dan Smart Reset

1. Pilih salah satu timer pada rail kanan atau buat timer baru.
2. Tempel teks reset pada Smart Reset, lalu tekan **Terapkan**, atau isi waktu
   secara manual.
3. Tekan **Pilih target** lalu klik kolom input aplikasi AI dalam tiga detik.
4. Pilih **Hanya Enter** atau **Teks + Enter** dan isi prompt jika diperlukan.
5. Tekan **Mulai timer**. Aksi hanya dikirim satu kali ketika timer mencapai nol.

Jika aplikasi ditutup, timer masa depan dipulihkan saat startup. Timer yang
sudah terlewat ditandai **Missed** dan tidak mengirim input secara diam-diam.
Tombol clipboard Smart Reset hanya membaca ketika diminta dan tidak pernah
menimpa isi clipboard.

## Alur Macro Studio

1. Pilih atau buat macro, kemudian tentukan mode dan pemicu.
2. Pilih scope **Global** atau **Window**. Untuk Window, tekan **Pilih window**
   dan klik target yang akan menerima macro.
3. Untuk Sequence, pilih lane **Saat ditekan**, **Saat ditahan**, atau
   **Saat dilepas**.
4. Tekan **Rekam input**, lakukan rangkaian input, lalu tekan `Esc`.
5. Edit chip delay, pindahkan/duplikat/hapus event bila perlu, lalu **Simpan**.

Saat recording, input ditahan agar tidak ikut masuk ke aplikasi lain. Hindari
mengetik password atau data sensitif selama recorder aktif. Untuk Toggle yang
window-bound, pemicu awal hanya diterima ketika target berada di depan; setelah
loop berjalan, pemicu yang sama dapat menghentikannya dari aplikasi lain.

## Profiles dan backup

Satu App Profile menyimpan executable + judul target dan tautan ke beberapa
macro. Target profil dapat disinkronkan ke macro terkait atau dipakai oleh Timer.
Gunakan **Export backup** sebelum migrasi mesin dan **Import backup** untuk
memulihkan state tervalidasi. Timer aktif dari backup selalu diimpor dalam
keadaan berhenti agar tidak mengirim input tanpa persetujuan.

## Settings dan keselamatan

- **Minimize ke tray** dan **Tombol X tetap di tray** menjaga timer/macro aktif.
- **Mulai bersama Windows** memakai registry Current User.
- Emergency Stop default: `Ctrl + Alt + F12`.
- Emergency Stop dapat membatalkan semua timer sekaligus.
- Toggle memiliki batas durasi dan jumlah pengulangan yang dapat dimatikan
  secara eksplisit.
- Global hook hanya dipasang jika macro berisi aksi yang memerlukannya, dan
  dilepas kembali ketika tidak digunakan.
- Hanya satu instance VibeTimer yang dapat berjalan.

## Update opsional

Pemeriksaan update default-nya **nonaktif**. Build publik perlu menyediakan URL
feed HTTPS saat kompilasi:

```powershell
$env:VIBETIMER_UPDATE_FEED_URL = 'https://example.com/vibetimer-update.txt'
powershell -ExecutionPolicy Bypass -File .\tools\build-release.ps1
```

Manifest rilis dibuat setelah installer tersedia:

```powershell
powershell -ExecutionPolicy Bypass -File .\tools\write-update-manifest.ps1 `
  -InstallerUrl 'https://example.com/VibeTimer-Setup-1.0.0-x64.exe'
```

Tanpa feed, UI menjelaskan bahwa update online belum dikonfigurasi dan upgrade
manual melalui installer tetap bekerja. VibeTimer menolak HTTP, manifest rusak,
versi tidak valid, installer kosong/terlalu besar, dan SHA-256 yang berbeda.

## Build dan test

Persyaratan: Rust stable x64 MSVC dan Inno Setup 7 untuk installer.

```powershell
cargo fmt -- --check
cargo clippy --all-targets -- -D warnings
cargo test --all-targets -- --test-threads=1
cargo build --release
powershell -ExecutionPolicy Bypass -File .\tools\build-release.ps1
```

Artefak:

- `target/release/VibeTimer.exe` — executable portable.
- `dist/VibeTimer-Setup-1.0.0-x64.exe` — installer per-user.

## Batasan

- Binary saat ini belum memiliki sertifikat code-signing; Windows SmartScreen
  dapat menampilkan peringatan Unknown Publisher.
- Windows dapat memblokir input menuju aplikasi dengan privilege lebih tinggi.
- Beberapa game Raw Input/DirectInput atau anti-cheat mengabaikan background
  window message. VibeTimer tidak mencoba melewati proteksi tersebut.
- Window target harus tetap terbuka. Jika executable atau judul berubah, pilih
  target lagi.
- Kompatibilitas macro dengan aplikasi/game spesifik tetap perlu smoke test
  langsung karena cara tiap aplikasi memproses input berbeda.
