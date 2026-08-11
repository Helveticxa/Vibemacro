# Audit pra-rilis VibeTimer v0.6

Tanggal audit: 12 Agustus 2026

## Kesimpulan

Gate pra-installer lulus. Build release tidak memakai dependency pihak ketiga,
seluruh test otomatis lulus, state penting disimpan secara atomik, hanya satu
instance aplikasi yang dapat berjalan, dan global hook hanya aktif ketika ada
macro yang benar-benar membutuhkannya.

## Perbaikan yang diterapkan

- Menambahkan named mutex per-user untuk mencegah dua instance mengirim aksi
  secara bersamaan. Instance kedua memulihkan window instance pertama lalu
  keluar dengan aman.
- Membatasi pembacaan clipboard Unicode berdasarkan ukuran alokasi global dan
  menolak data tanpa terminator agar pembacaan tidak melewati buffer.
- Menguatkan penyimpanan `settings`, `macros`, `profiles`, dan `timers` dengan
  temporary file, `sync_all`, backup, pemulihan backup, dan rollback best-effort
  untuk operasi multi-file.
- Memvalidasi ID, nama, target, jumlah item yang dapat ditampilkan, dan delay
  maksimum 60.000 ms ketika menyimpan maupun mengimpor data.
- Membatasi library menjadi enam macro, enam profile, dan enam timer agar semua
  objek selalu dapat diakses dari UI.
- Mengoptimalkan hook global. Keyboard hook hanya terpasang untuk macro F8/F9
  berisi aksi, mouse hook hanya terpasang untuk trigger mouse berisi aksi, dan
  keduanya sementara aktif saat recording.
- Memfilter event mouse yang tidak relevan sebelum mengakses state aplikasi.
- Mengubah recovery timer setelah restart: timer masa depan dilanjutkan,
  sedangkan timer yang sudah lewat atau tertinggal saat dispatch ditandai
  `Missed` dan tidak pernah mengirim input secara diam-diam.
- Import backup memvalidasi semua section sebelum mengganti state. Timer aktif
  dari mesin lain selalu diimpor dalam keadaan berhenti.

## Bukti verifikasi

- `cargo fmt -- --check`: lulus.
- `cargo clippy --all-targets -- -D warnings`: lulus, 0 warning.
- `cargo test --all-targets -- --test-threads=1`: 28 unit test dan 2 Windows
  E2E lulus.
- E2E mencakup render empat tab, timeline editing, recording keyboard/mouse,
  semua mode macro, window-bound background playback, emergency stop,
  penyimpanan/import, Smart Reset, dan dua timer konkuren yang masing-masing
  hanya dispatch satu kali.
- `cargo build --release`: lulus.
- Ukuran `VibeTimer.exe`: 315.392 byte.
- SHA-256: `3B92B0B7C443A76B1B22882EA1A5ABEF8276202D18F4C88070CA70C97F668DBA`.
- Runtime idle dengan profil data kosong terisolasi selama delapan detik:
  CPU 0,000% dari satu core, 137 handle stabil, 4 thread stabil, working set
  10,07 MiB, private memory 1,66 MiB, dan proses tetap responsive.
- Uji single-instance aktual: instance kedua keluar dengan code 0 dan jumlah
  proses tetap satu.
- Dependency runtime Rust eksternal: tidak ada (`cargo tree` hanya berisi
  package VibeTimer).

## Risiko yang diketahui sebelum v1.0

- Binary belum ditandatangani (`Authenticode: NotSigned`). Windows SmartScreen
  dapat memperingatkan pengguna sampai tersedia sertifikat code-signing dan
  reputasi penerbit.
- Input sintetis Windows dapat diblokir oleh aplikasi yang berjalan dengan hak
  lebih tinggi, anti-cheat, atau aplikasi yang hanya menerima DirectInput/raw
  input. VibeTimer tidak mencoba melewati proteksi tersebut.
- Background macro memakai pesan window terarah. Dukungan akhir tetap bergantung
  pada cara aplikasi target memproses keyboard/mouse message.
- Auto-update online belum dapat diaktifkan tanpa URL feed HTTPS dan artefak
  rilis yang dipercaya. v1.0 harus tetap default-off dan tidak boleh mengklaim
  update tersedia sebelum infrastruktur rilis dikonfigurasi.

