# VibeTimer — QA Evidence

Tanggal verifikasi: 2026-08-08  
Platform build: Windows x64, Rust 1.97.1

## Gerbang otomatis

| Pemeriksaan | Hasil |
|---|---|
| `cargo fmt -- --check` | Lulus |
| `cargo clippy --all-targets -- -D warnings` | Lulus, 0 warning |
| `cargo test -- --test-threads=1` | Lulus, 5 test |
| `cargo build --release` | Lulus |
| Ukuran `VibeTimer.exe` | 133.120 byte (±130 KB) |
| Working set setelah startup | 12,7 MB |
| Private memory setelah startup | 1,94 MB |

## Cakupan test

Empat unit test memeriksa validasi waktu, rollover preset, batas maksimum, dan
format countdown. Satu integration test Windows membuat UI VibeTimer dan sebuah
jendela target dengan edit control sungguhan, menjalankan timer satu detik,
membentuk batch Unicode `lanjutkan` + Enter, lalu memverifikasi target menerima
teks dan baris baru.

Integration test juga menangkap tiga artefak lokal di folder `qa/`:

- `vibetimer-idle.bmp`
- `vibetimer-running.bmp`
- `e2e-target.bmp`

Folder tersebut sengaja diabaikan Git karena merupakan hasil test yang dapat
dibuat ulang.

## Batas verifikasi sandbox

Desktop otomasi Codex pada sesi ini tidak memiliki izin foreground/input global.
Windows mengembalikan penolakan saat test memanggil `SendInput` dari desktop
noninteraktif. Karena itu, binary test memiliki fallback **khusus `cfg(test)`**
yang meneruskan batch keyboard yang sama langsung ke edit control target.

Executable release tidak memiliki fallback tersebut: ia memakai `SendInput`
Windows asli dan gagal-aman bila foreground atau input ditolak. Satu smoke test
di desktop interaktif pengguna masih diperlukan sebelum menyebut injeksi global
ke Codex/Claude terverifikasi langsung.

## Smoke test desktop interaktif

1. Buka Notepad dan VibeTimer.
2. Set timer ke `00:00:05`.
3. Pilih target; saat VibeTimer mengecil, klik area ketik Notepad.
4. Pastikan mode **Teks + Enter** dan isi `lanjutkan`.
5. Mulai timer dan jangan menyentuh mouse sampai timer nol.
6. Lulus bila Notepad menjadi foreground, berisi `lanjutkan`, dan caret berada
   pada baris baru.

