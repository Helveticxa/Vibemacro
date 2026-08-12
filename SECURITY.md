# Security Policy

## Versi yang didukung

Hanya release stabil terbaru yang menerima security fix. Unduh binary melalui
halaman [Releases](https://github.com/Helveticxa/Vibemacro/releases/latest) dan
verifikasi hash dengan `SHA256SUMS.txt`.

## Melaporkan kerentanan

Jangan memasukkan credential, data pribadi, atau exploit yang siap disalahgunakan
ke issue publik. Gunakan halaman **Security > Report a vulnerability** repository:

https://github.com/Helveticxa/Vibemacro/security/advisories/new

Sertakan versi, versi Windows, langkah reproduksi minimal, dampak, dan bukti yang
sudah disamarkan. Jika private reporting belum tersedia, buat issue singkat tanpa
detail sensitif agar maintainer dapat membuka jalur komunikasi privat.

## Model keamanan updater

- Feed dan installer produksi wajib HTTPS.
- Manifest dibatasi 8 KiB dan installer 256 MiB.
- Installer harus cocok dengan SHA-256 di manifest sebelum dapat dijalankan.
- Update tidak berjalan silent dan tidak dapat dipasang ketika automation aktif.
- Workflow release hanya mendapat `contents: write` dan memakai `GITHUB_TOKEN`
  sementara; repository tidak membutuhkan API key atau personal access token.

Binary belum ditandatangani Authenticode. SHA-256 membuktikan kecocokan dengan
asset release, tetapi bukan pengganti code signing dan reputasi SmartScreen.
