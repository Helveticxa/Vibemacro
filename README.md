# VibeTimer

VibeTimer adalah utilitas Windows ringan untuk melanjutkan sesi AI setelah batas
usage/credit kembali tersedia. Pengguna memilih jendela target, mengatur waktu
tunggu, lalu VibeTimer mengirim **Enter saja** atau **teks + Enter** tepat ketika
countdown mencapai nol.

## Kenapa Rust + Win32 native

- Tidak memakai Electron, WebView2, Tauri, atau runtime tambahan.
- Tidak memiliki dependency crate eksternal.
- Satu executable native dengan penggunaan RAM rendah.
- Teks dikirim sebagai Unicode melalui `SendInput`; clipboard pengguna tidak disentuh.

## Alur penggunaan

1. Isi jam, menit, dan detik sesuai waktu reset yang ditampilkan AI.
2. Tekan **Pilih target**. Setelah dialog petunjuk ditutup, VibeTimer mengecil.
3. Dalam tiga detik, klik jendela AI dan letakkan kursor di kolom inputnya.
4. Pilih **Hanya Enter** atau **Teks + Enter**. Teks awalnya adalah `lanjutkan`.
5. Tekan **Mulai timer**. VibeTimer melakukan aksi satu kali saat timer nol.

Target diverifikasi kembali berdasarkan window handle dan process ID sebelum
pengiriman. Jika target ditutup atau berganti proses, VibeTimer memilih gagal
aman dan tidak mengetik ke jendela lain.

## Build dan test

```powershell
cargo test
cargo build --release
```

Executable hasil build berada di `target/release/VibeTimer.exe`.

## Batasan MVP

- Windows dapat memblokir input lintas level privilege. Jika target dijalankan
  sebagai Administrator, jalankan VibeTimer dengan level yang sama.
- Jendela target harus tetap terbuka. Letakkan fokus terakhir pada kolom input.
- Timer adalah aksi satu kali; tidak ada retry otomatis agar perintah tidak
  terkirim berulang tanpa sepengetahuan pengguna.
- MVP belum membaca waktu reset secara otomatis dari layar karena format dan
  aksesibilitas tiap aplikasi AI berbeda. Input manual lebih dapat diprediksi.
