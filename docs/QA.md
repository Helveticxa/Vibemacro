# VibeTimer — QA Evidence

Tanggal verifikasi: 2026-08-09
Platform build: Windows x64, Rust 1.97.1

## Gerbang otomatis

| Pemeriksaan | Hasil |
|---|---|
| `cargo fmt -- --check` | Lulus |
| `cargo clippy --all-targets -- -D warnings` | Lulus, 0 warning |
| `cargo test` | Lulus, 9 test (8 unit + 1 integration E2E) |
| `cargo build --release` | Lulus |
| Ukuran `VibeTimer.exe` | 196.608 byte (192 KB) |
| Working set setelah startup | 7,27 MB |
| Private memory setelah startup | 1,22 MB |
| Handle setelah startup | 87 |

## Cakupan test

Delapan unit test memeriksa validasi waktu serta model, serialisasi, validasi,
default aman, dan penyimpanan atomik macro. Satu integration test Windows
membuat UI VibeTimer dan jendela target dengan edit control sungguhan. Test itu
menjalankan timer satu detik, memverifikasi `lanjutkan` + Enter, membuka tab
Macro, merekam key down/up melalui callback hook, menghentikan recording dengan
Esc, lalu memicu macro No Repeat melalui F8 dan Mouse 4 serta memverifikasi Enter
diterima target. Controller Repeat While Holding, Toggle, dan ketiga lane
Sequence juga dijalankan dengan assertion jumlah input playback.

Integration test juga menangkap empat artefak lokal di folder `qa/`:

- `vibetimer-idle.bmp`
- `vibetimer-running.bmp`
- `e2e-target.bmp`
- `vibetimer-macro.bmp`

Folder tersebut sengaja diabaikan Git karena merupakan hasil test yang dapat
dibuat ulang.

## Batas verifikasi sandbox

Desktop test noninteraktif dapat menolak foreground/input global. Karena itu,
binary test memiliki fallback **khusus `cfg(test)`** yang meneruskan input
keyboard yang sama langsung ke edit control target bila `SendInput` ditolak.

Executable release tidak memiliki fallback tersebut: ia memakai `SendInput`
Windows asli dan gagal-aman bila input ditolak. Pengguna telah mengonfirmasi
timer + auto Enter bekerja pada desktop interaktif pada 2026-08-09. Callback
recorder, pemicu, playback, dan render Macro Studio sudah tercakup test E2E;
smoke test fisik Mouse 4/5 tetap perlu dilakukan pada mouse pengguna karena
desktop test tidak dapat menekan tombol fisik tersebut.

## Smoke test timer desktop interaktif

1. Buka Notepad dan VibeTimer.
2. Set timer ke `00:00:05`.
3. Pilih target; saat VibeTimer mengecil, klik area ketik Notepad.
4. Pastikan mode **Teks + Enter** dan isi `lanjutkan`.
5. Mulai timer dan jangan menyentuh mouse sampai timer nol.
6. Lulus bila Notepad menjadi foreground, berisi `lanjutkan`, dan caret berada
   pada baris baru.

## Smoke test macro fisik

1. Buka Notepad dan VibeTimer, lalu buka tab **Macro**.
2. Buat macro No Repeat, pilih `Mouse 4` atau `Mouse 5`, lalu rekam teks pendek
   beserta Enter. Tekan `Esc`, lalu **Simpan**.
3. Fokuskan Notepad dan tekan tombol mouse yang dipilih.
4. Lulus bila macro berjalan sekali, urutan tombol benar, dan VibeTimer berubah
   ke status **Terkirim**.
