# Vibemacro

[![Windows](https://img.shields.io/badge/Windows-10%2F11-111111?logo=windows)](https://github.com/Helveticxa/Vibemacro/releases/latest)
[![Release](https://img.shields.io/github/v/release/Helveticxa/Vibemacro?display_name=tag&color=baff29)](https://github.com/Helveticxa/Vibemacro/releases/latest)
[![License: MIT](https://img.shields.io/badge/license-MIT-baff29.svg)](LICENSE)

Vibemacro adalah utility Windows-native yang menggabungkan timer untuk limit AI
dengan Macro Studio keyboard/mouse yang ringan dan vendor-neutral. Aplikasi ini
dibangun memakai Rust + raw Win32/GDI: tanpa Electron, WebView, driver vendor,
akun, telemetry, atau API key.

> Sebelumnya bernama **VibeTimer**. Vibemacro 1.1 memigrasikan macro, timer,
> profile, dan settings lama secara otomatis tanpa menghapus folder sumber.

## Download dan instalasi

1. Buka [GitHub Releases terbaru](https://github.com/Helveticxa/Vibemacro/releases/latest).
2. Unduh `Vibemacro-Setup-<versi>-x64.exe`.
3. Jalankan installer. Instalasi bersifat **per-user**, jadi normalnya tidak
   meminta hak Administrator.
4. Shortcut dibuat di Start Menu. Shortcut Desktop bersifat opsional.

Windows SmartScreen dapat menampilkan **Unknown publisher** karena binary publik
saat ini belum memiliki sertifikat Authenticode. Selalu unduh dari repository
ini dan cocokkan SHA-256 dengan `SHA256SUMS.txt` pada release yang sama:

```powershell
Get-FileHash .\Vibemacro-Setup-1.2.0-x64.exe -Algorithm SHA256
```

Versi portable `Vibemacro-<versi>-portable.exe` juga tersedia, tetapi installer
direkomendasikan agar upgrade, shortcut, dan uninstall berjalan konsisten.

## Fitur

- Hingga enam timer AI-limit persisten dengan **Enter** atau **Teks + Enter**.
- Smart Reset untuk teks seperti `Resets in 3 h 27 min`, jam reset, nama hari,
  serta variasi bahasa Indonesia/Inggris.
- Macro recorder keyboard, klik, dan wheel dengan **No Repeat**,
  **While Holding**, **Toggle**, dan **Sequence**.
- Timeline editor untuk delay `0-60000 ms`, reorder, duplicate, insert, delete.
- Trigger `F8`, `F9`, `Middle`, `Mouse 4`, dan `Mouse 5`.
- Tiga scope output: **Global**, **App** background, dan **Game** focus-lock.
- Mode Game memakai keyboard scan code dan mouse input Windows agar lebih cocok
  untuk game yang mengabaikan pesan window biasa.
- Exact-instance lock, auto-pause saat Alt+Tab, serta release/resume tombol dan
  klik yang sedang ditahan agar input tidak menumpuk ke aplikasi lain.
- App Profiles dan backup portable `.vtb`.
- Tray Mode, Auto Start, serta Emergency Stop global.
- Batas durasi/repeat agar loop tidak berjalan tanpa kendali.
- Auto-update dari GitHub Releases dengan verifikasi SHA-256 sebelum installer
  boleh dijalankan.

## Menggunakan timer

1. Pilih atau buat timer.
2. Tempel teks reset ke Smart Reset lalu tekan **Terapkan**, atau isi manual.
3. Tekan **Pilih target**, kemudian klik kolom input AI dalam tiga detik.
4. Pilih **Hanya Enter** atau **Teks + Enter**.
5. Tekan **Mulai timer**.

Setiap timer hanya mengirim satu aksi. Timer yang terlewat saat aplikasi tutup
menjadi **Missed** dan tidak pernah mengetik diam-diam ketika Vibemacro dibuka.

## Menggunakan Macro Studio

1. Buat macro, pilih perilaku dan trigger.
2. Pilih scope **Global**, **App**, atau **Game**.
3. Tekan **Rekam input**, jalankan urutan input, lalu tekan `Esc`.
4. Edit delay/urutan event dan tekan **Simpan**.

Gunakan **App** untuk aplikasi desktop Win32 yang menerima pesan keyboard/mouse
di background. Gunakan **Game** untuk Roblox atau game yang membaca input lewat
jalur perangkat: target harus menjadi foreground ketika menerima input. Saat
Alt+Tab, Vibemacro melepas tombol/klik yang masih down dan mem-pause playback;
ketika kembali ke instance yang sama playback dilanjutkan. Input tidak dialihkan
ke window lain. Jika ada beberapa instance dengan executable dan judul identik,
Vibemacro gagal aman dan meminta target dipilih ulang.

Mode Game mengikuti fokus internal game seperti keyboard fisik. Jika chat Roblox
sedang aktif, WASD tetap akan masuk ke chat; tutup chat/klik viewport gameplay
sebelum memicu macro. Vibemacro tidak membaca atau memodifikasi state internal
game untuk membedakan chat dari kontrol karakter.

Windows tidak menyediakan mouse virtual per-window melalui `SendInput`; cursor
dan input stream tetap resource desktop bersama. Karena itu mode Game tidak
berpura-pura menjalankan input di Roblox background. Vibemacro juga tidak
memasang driver virtual, menginjeksi proses game, atau melewati anti-cheat.
Periksa aturan game/experience sebelum menggunakan automation.

Emergency Stop default adalah `Ctrl + Alt + F12`. Jangan mengetik password atau
data sensitif ketika recorder sedang aktif.

## Auto-update GitHub

Pemeriksaan update aktif secara default saat startup dan dapat dimatikan dari
Settings. Vibemacro membaca asset tetap berikut:

```text
https://github.com/Helveticxa/Vibemacro/releases/latest/download/vibemacro-update.txt
```

Jika versi baru tersedia, klik tombol update sekali. Vibemacro mengunduh
installer, memverifikasi ukuran dan SHA-256, lalu menampilkan konfirmasi instalasi.
Installer tidak akan diluncurkan ketika timer, recording, macro, atau perubahan
yang belum disimpan masih aktif.

Manifest custom dipakai karena aplikasi ini native tanpa runtime updater pihak
ketiga. GitHub Actions membuat manifest dan release assets otomatis dari tag
`v<major>.<minor>.<patch>` setelah semua test lulus.

## Data dan upgrade dari VibeTimer

Data aktif berada di:

```text
%LOCALAPPDATA%\Vibemacro
```

Saat pertama dijalankan, hanya file data yang dikenal yang disalin dari
`%LOCALAPPDATA%\VibeTimer`. Folder lama tidak dihapus. Installer memakai AppId
yang sama agar VibeTimer 1.0 dapat di-upgrade in-place. Uninstall juga tidak
menghapus data pengguna.

## Build dari source

Persyaratan:

- Windows 10/11 x64
- Rust stable x64 MSVC
- Inno Setup 6.7+ atau 7 untuk membuat installer

```powershell
git clone https://github.com/Helveticxa/Vibemacro.git
cd Vibemacro
cargo fmt -- --check
cargo clippy --all-targets -- -D warnings
cargo test --all-targets -- --include-ignored --test-threads=1
.\tools\security-scan.ps1 -IncludeHistory
.\tools\build-release.ps1
```

Artefak lokal dibuat di `dist/`. Tidak ada dependency crate/runtime eksternal.
Test desktop bertanda `ignored` agar runner GitHub headless tidak memberikan
false failure; `--include-ignored` di atas wajib dipakai pada Windows interaktif.

## Security dan privacy

- Vibemacro tidak membutuhkan API key dan tidak menyimpan credential.
- Tidak ada telemetry; koneksi keluar hanya untuk pemeriksaan/download update
  GitHub ketika fitur tersebut aktif.
- Workflow memakai `GITHUB_TOKEN` sementara milik GitHub Actions dengan
  permission minimum, bukan token pribadi yang disimpan di repository.
- Manifest dan installer dibatasi ukurannya; URL wajib HTTPS; hash wajib cocok.
- Lihat [SECURITY.md](SECURITY.md) untuk pelaporan kerentanan.

## Lisensi

[MIT License](LICENSE): bebas dipakai, dipelajari, dimodifikasi, didistribusikan,
dan dijual kembali selama notice lisensi/copyright dipertahankan. Software
disediakan tanpa garansi.

Kontribusi dipersilakan melalui [CONTRIBUTING.md](CONTRIBUTING.md).
