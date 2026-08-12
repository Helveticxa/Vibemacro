# Vibemacro 1.1.0

VibeTimer kini bernama **Vibemacro**. Release ini mempertahankan fitur, data,
dan jalur upgrade 1.0 sambil menambahkan update publik langsung dari GitHub.

## Perubahan utama

- Seluruh identitas aplikasi, EXE, installer, shortcut, tray, dan UI menjadi
  Vibemacro.
- Data lama dari `%LOCALAPPDATA%\VibeTimer` dimigrasikan secara allowlist ke
  `%LOCALAPPDATA%\Vibemacro` tanpa menghapus sumber.
- AppId dan mutex installer dipertahankan untuk upgrade in-place yang aman.
- Auto-update default aktif memakai GitHub Releases, HTTPS, bounded download,
  dan verifikasi SHA-256.
- Satu klik update mengunduh, memverifikasi, lalu membuka konfirmasi installer.
- GitHub Actions membangun release Windows dari tag hanya setelah formatting,
  clippy, test, dan credential scan lulus.
- README publik, SECURITY, CONTRIBUTING, dan MIT license dipoles untuk release.

## Asset

- `Vibemacro-Setup-1.1.0-x64.exe` - installer per-user yang direkomendasikan.
- `Vibemacro-1.1.0-portable.exe` - executable portable.
- `vibemacro-update.txt` - feed updater native.
- `SHA256SUMS.txt` - checksum seluruh asset penting.

## Catatan keamanan

- Tidak ada API key atau personal access token di source/release.
- Workflow memakai `GITHUB_TOKEN` sementara dengan permission minimum.
- Binary belum Authenticode signed; Windows dapat menampilkan Unknown Publisher.
- Macro global tetap dapat menangkap input saat recording; hindari data sensitif.
