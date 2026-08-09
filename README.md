# VibeTimer

VibeTimer adalah utilitas Windows ringan untuk melanjutkan sesi AI setelah batas
usage/credit kembali tersedia. Pengguna memilih jendela target, mengatur waktu
tunggu, lalu VibeTimer mengirim **Enter saja** atau **teks + Enter** tepat ketika
countdown mencapai nol.

Versi 0.2 menambahkan **Macro Studio**: recorder keyboard/mouse global yang
memberi fungsi macro bergaya Logitech G HUB pada mouse HID umum, tanpa
memerlukan perangkat atau driver Logitech. UI dan branding tetap milik
VibeTimer; tidak ada aset atau logo Logitech yang disalin.

Versi 0.2.1 merombak visual menjadi studio utility yang lebih editorial: shell
hitam hangat, kanvas ivory, aksen acid-lime, navigasi tanpa pill berlebihan, dan
hierarki yang berbeda jelas antara Timer dan Macro Editor.

## Kenapa Rust + Win32 native

- Tidak memakai Electron, WebView2, Tauri, atau runtime tambahan.
- Tidak memiliki dependency crate eksternal.
- Satu executable native dengan penggunaan RAM rendah.
- Teks dikirim sebagai Unicode melalui `SendInput`; clipboard pengguna tidak disentuh.
- Recorder dan pemicu memakai low-level hook Win32; playback tetap melalui
  `SendInput` native.

## Alur penggunaan

1. Isi jam, menit, dan detik sesuai waktu reset yang ditampilkan AI.
2. Tekan **Pilih target**. Setelah dialog petunjuk ditutup, VibeTimer mengecil.
3. Dalam tiga detik, klik jendela AI dan letakkan kursor di kolom inputnya.
4. Pilih **Hanya Enter** atau **Teks + Enter**. Teks awalnya adalah `lanjutkan`.
5. Tekan **Mulai timer**. VibeTimer melakukan aksi satu kali saat timer nol.

Target diverifikasi kembali berdasarkan window handle dan process ID sebelum
pengiriman. Jika target ditutup atau berganti proses, VibeTimer memilih gagal
aman dan tidak mengetik ke jendela lain.

## Alur Macro Studio

1. Buka tab **Macro**, lalu pilih **Buat macro baru** atau macro yang sudah ada.
2. Pilih tipe: **No Repeat**, **While Holding**, **Toggle**, atau **Sequence**.
3. Pasang pemicu global: `F8`, `F9`, `Middle`, `Mouse 4`, atau `Mouse 5`.
4. Untuk Sequence, pilih lane **On Press**, **While Holding**, atau
   **On Release** sebelum merekam.
5. Tekan **Rekam input**. Lakukan kombinasi keyboard/klik/scroll yang diinginkan,
   lalu tekan `Esc` untuk selesai. Delay nyata serta key/button down dan up ikut
   direkam.
6. Tekan **Simpan**. Macro tersimpan di
   `%LOCALAPPDATA%\VibeTimer\macros.vtm` dan pemicunya aktif selama VibeTimer
   berjalan.

Saat recording, input keyboard dan mouse ditahan agar tidak ikut terkirim ke
aplikasi lain. Hindari mengetik password atau data sensitif selama recorder
aktif. Jika dua macro memakai pemicu yang sama, macro yang sedang dipilih
mendapat prioritas; gunakan pemicu berbeda agar semua assignment dapat dipakai.

## Build dan test

```powershell
cargo test
cargo build --release
```

Executable hasil build berada di `target/release/VibeTimer.exe`.

## Batasan

- Windows dapat memblokir input lintas level privilege. Jika target dijalankan
  sebagai Administrator, jalankan VibeTimer dengan level yang sama.
- Jendela target harus tetap terbuka. Letakkan fokus terakhir pada kolom input.
- Timer adalah aksi satu kali; tidak ada retry otomatis agar perintah tidak
  terkirim berulang tanpa sepengetahuan pengguna.
- MVP belum membaca waktu reset secara otomatis dari layar karena format dan
  aksesibilitas tiap aplikasi AI berbeda. Input manual lebih dapat diprediksi.
- Pemicu macro bekerja pada aplikasi dengan level privilege yang sama. Aplikasi
  Administrator dapat menolak input dari VibeTimer non-Administrator.
- VibeTimer tidak dirancang untuk melewati anti-cheat, proteksi game, atau
  pembatasan keamanan aplikasi lain.
