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
| Ukuran `VibeTimer.exe` | 197.632 byte (193 KB) |
| Working set setelah startup | 10,08 MiB |
| Private memory setelah startup | 1,82 MiB |
| Handle setelah startup | 115 |
| Status proses release | Responding / sehat |

## Cakupan test

Delapan unit test memeriksa validasi waktu serta model, serialisasi, validasi,
default aman, dan penyimpanan atomik macro. Satu integration test Windows
membuat UI VibeTimer dan jendela target dengan edit control sungguhan. Test itu
menjalankan timer satu detik dalam mode **Teks + Enter** dan **Hanya Enter**,
memverifikasi pembatalan tidak mengirim input, serta menolak PID target yang
tidak cocok. Ketiga preset waktu dan seluruh hit-area Timer juga diuji.

Pada Macro Studio, E2E membuat dan memilih macro, mengubah empat mode, lima
pemicu, dan tiga lane, membersihkan lane, merekam keyboard + klik + wheel,
menghentikan recording dengan Esc, serta melakukan save/load file nyata.
Playback **No Repeat** dijalankan melalui F8, Middle, Mouse 4, dan Mouse 5;
F9 menjalankan **While Holding** serta **Toggle**; sedangkan **Sequence**
memverifikasi On Press, While Holding, dan On Release.

## Matriks fungsi UI

| Fungsi terlihat | Verifikasi |
|---|---|
| Input jam/menit/detik + `+30 mnt`, `+1 jam`, `+3 jam` | Unit + E2E nilai akhir 04:30:00 |
| Pilih dan validasi target | Window handle + PID diuji; target salah ditolak |
| Hanya Enter / Teks + Enter | Kedua aksi diterima edit control Windows nyata |
| Mulai / batalkan timer | Countdown selesai sekali; cancel mengirim 0 input |
| Tab Timer / Macro | Hit-area dan resize native dijalankan dalam E2E |
| Macro baru / pilih macro / edit nama | State pilihan dan nama tersimpan diverifikasi |
| 4 mode macro | No Repeat, While Holding, Toggle, Sequence dijalankan |
| 5 pemicu global | F8, F9, Middle, Mouse 4, Mouse 5 dipetakan dan dipicu |
| 3 lane timeline | Pemilihan, clear, recording, dan Sequence diverifikasi |
| Rekam / Esc untuk selesai | Key down/up, mouse down/up, wheel terekam |
| Simpan | File ditulis atomik lalu dimuat ulang dan dibandingkan |

Integration test juga menangkap lima artefak lokal di folder `qa/`:

- `vibetimer-idle.bmp`
- `vibetimer-running.bmp`
- `e2e-target.bmp`
- `vibetimer-macro.bmp`
- `vibetimer-macro-empty.bmp`

Folder tersebut sengaja diabaikan Git karena merupakan hasil test yang dapat
dibuat ulang.

## QA visual 0.2.2

Snapshot idle, running, Macro berisi event, dan Macro empty-state dirender lewat
`PrintWindow` dari aplikasi native yang sama, lalu diinspeksi pada resolusi asli.
Redesign menggunakan satu bahasa visual dark graphite: background hitam hangat,
panel gelap berlapis tipis, acid-lime sebagai aksen interaksi, Segoe UI Variable
Display, tab berbasis teks, serta pengurangan border dan nested-card. Tidak ada
panel putih/ivory atau hover yang berubah putih. Tidak ada image asset yang
dibakar ke UI; seluruh visual tetap GDI programatik.

SHA-256 release 0.2.2:
`EA6A020EF692B290F3DEDDDB8CD112A41AD226F06E1996BAFC28AD27A4916737`.

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
