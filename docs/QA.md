# Vibemacro 1.1.0 - QA Evidence

Tanggal verifikasi lokal: 2026-08-12
Platform: Windows x64, Rust 1.97.1, Inno Setup 7.0.2

## Source gate

| Pemeriksaan | Hasil |
|---|---|
| `cargo fmt -- --check` | Lulus |
| `cargo clippy --all-targets -- -D warnings` | Lulus, 0 warning |
| Unit test | 33/33 lulus |
| Windows safety test | 1/1 lulus |
| Windows desktop E2E (`--include-ignored`) | 1/1 lulus |
| Total | 35/35 lulus |
| Release build | Lulus |
| Credential scan working tree + Git history | Lulus, tidak ada pola credential |
| Dependency runtime pihak ketiga | Tidak ada |

E2E updater membuktikan check -> download -> size bound -> SHA-256 -> satu klik
memasuki tahap instalasi. E2E memakai `file:///` hanya melalui policy test-only;
runtime produksi tetap menerima HTTPS saja.

## Rename dan migrasi

- UI, title, EXE, tray, backup dialog, shortcut, installer, dan data directory
  memakai nama Vibemacro.
- Migrasi hanya menyalin delapan nama file data/backup yang diizinkan dari
  `%LOCALAPPDATA%\VibeTimer` ke `%LOCALAPPDATA%\Vibemacro`.
- Test membuktikan file lain tidak ikut disalin, migrasi idempotent, dan folder
  sumber tidak dihapus.
- Header codec lama tetap diterima agar macro/settings/profile/timer 1.0 kompatibel.

## Installer upgrade E2E

Upgrade test memakai AppId QA terpisah agar instalasi VibeTimer pengguna tidak
tersentuh. Fixture lama dan installer Vibemacro memakai pipeline Inno yang sama.

| Pemeriksaan | Hasil |
|---|---|
| Install fixture VibeTimer 1.0 | Lulus |
| Upgrade ke Vibemacro 1.1.0 | Lulus |
| `VibeTimer.exe` lama dihapus | Ya |
| Display name/version | `Vibemacro 1.1.0` |
| Single-instance second launch | Exit 0, satu runtime |
| Data lama disalin byte-identik | Ya |
| Data sumber dipertahankan | Ya |
| Uninstall menghapus program | Ya |
| Uninstall mempertahankan data | Ya |
| Registrasi uninstall QA dibersihkan | Ya |
| Registrasi produksi pengguna tidak berubah | Ya |

AppId produksi tetap `{F677B6B9-347D-4D9F-9444-23A7DA9C6822}` agar installer
1.1 dapat memperbarui VibeTimer 1.0 in-place. Mutex legacy dipertahankan untuk
mencegah runtime lama dan baru berjalan bersamaan.

## Artefak lokal

| File | Ukuran | SHA-256 |
|---|---:|---|
| `target/release/Vibemacro.exe` | 391.168 byte | `D04E96E20516BFB5316D32492C556EC60B83744D4E8E7455AEC5D2745089C44D` |
| `dist/Vibemacro-Setup-1.1.0-x64.exe` | 2.441.416 byte | `8F29322B3BFB761448A213847C348A4486E9B57D4733F9F91E90DA8DB5581A4C` |
| `dist/Vibemacro-1.1.0-portable.exe` | 391.168 byte | `D04E96E20516BFB5316D32492C556EC60B83744D4E8E7455AEC5D2745089C44D` |
| `dist/vibemacro-update.txt` | 210 byte | `08E99FDFB8DCEF0CD946AF5806E1C080C0E32CC041514C0D97FF9DFA938C76F0` |

Manifest menunjuk asset release versi spesifik dan hash installer identik dengan
`SHA256SUMS.txt`.

Artefak lokal di atas dibangun dengan Inno Setup 7.0.2. GitHub membangun ulang
dari commit/tag yang sama memakai runner `windows-2022` dan Inno Setup 6.7.1,
sehingga hash PE publik berbeda dan dicatat terpisah di bawah.

## GitHub Release live

- Repository: `Helveticxa/Vibemacro`, public, default branch `main`, MIT.
- CI commit `51b7d2e`: success.
- Release workflow `v1.1.0`: success pada seluruh langkah.
- Release: `Vibemacro 1.1.0`, draft `false`, prerelease `false`.
- Endpoint tetap `/releases/latest/download/vibemacro-update.txt` dapat diunduh
  tanpa autentikasi dan mengarah ke installer versi 1.1.0.

| Asset publik | Ukuran | SHA-256 GitHub |
|---|---:|---|
| `Vibemacro-Setup-1.1.0-x64.exe` | 2.449.657 byte | `7801248F24A24777A6F3B2DD952A02B42B0B9138FD5846B71CF2B49D63D2BE67` |
| `Vibemacro-1.1.0-portable.exe` | 391.168 byte | `572B60C56BB19127F94A6B1E8465FE65D613301A2C7497F533A7D860B62F31E5` |
| `vibemacro-update.txt` | 210 byte | `92B7D14156A4B539EA26F57BAF8C430CC3EA84DA37A1FF6BB3D5CB41E2D9BD10` |
| `SHA256SUMS.txt` | 278 byte | `D9C803D0B7D505FAF0B8F13C83411ED2D32463AD8A718EEEA4C139FA4BB7E269` |

Installer diunduh ulang melalui URL pada manifest. Hash aktual sama persis
dengan hash manifest. Isi `SHA256SUMS.txt` juga cocok untuk installer, portable,
dan manifest. Defender scan pada installer dan portable hasil download live:
exit 0, `found no threats`. Salinan QA unduhan kemudian dihapus.

## Malware dan secret scope

- Microsoft Defender custom scan pada installer dan portable EXE: exit 0,
  `found no threats`.
- `tools/security-scan.ps1 -IncludeHistory` memindai private-key marker,
  GitHub/PAT, AWS, Google, Slack, generic assigned secret, working tree, dan
  seluruh patch Git tanpa mencetak nilai yang mungkin sensitif.
- Workflow tidak menyimpan API key/PAT. Release memakai `GITHUB_TOKEN` sementara
  dengan permission hanya `contents: write`; CI memakai `contents: read`.

Hasil ini adalah bukti scoped pada artefak dan pola yang diperiksa, bukan klaim
universal bahwa software tidak mungkin memiliki kerentanan.

## Visual dan performance

- Main Timer dan Settings snapshot diinspeksi pada resolusi asli: brand
  Vibemacro, dark graphite, acid-lime, versi 1.1.0, dan label GitHub Releases
  tampil bersih tanpa panel putih/ungu atau overlap.
- Profil idle terisolasi selama 8 detik:
  - CPU: 0,000% dari satu core
  - handles: 146 -> 146
  - threads: 4 -> 4
  - working set: 10,47 MiB
  - private memory: 1,86 MiB
  - responding: true

## GitHub release gate

Workflow release dipicu hanya oleh tag `v*.*.*`, memverifikasi tag sama dengan
Cargo version, menjalankan credential scan dan seluruh build/test gate, lalu
membuat GitHub Release dengan installer, portable EXE, manifest tetap, dan
checksums. Asset live dan endpoint `/releases/latest/download/` sudah diverifikasi
setelah tag 1.1.0 dipublikasikan.

GitHub-hosted runner bersifat headless, sehingga test `SendInput`/focus desktop
ditandai `ignored` pada CI namun wajib dan sudah dijalankan lokal dengan
`--include-ignored`. CI tetap mengompilasi test tersebut dan menjalankan seluruh
unit test serta safety-limit test noninteraktif.

## Risiko tersisa

- EXE dan installer berstatus `NotSigned`; SmartScreen dapat memberi peringatan.
- Code signing Authenticode tetap direkomendasikan sebelum distribusi luas.
- Mouse 4/5 fisik dan kompatibilitas game tertentu memerlukan smoke test user.
- Raw Input/DirectInput/anti-cheat dapat mengabaikan background window message;
  Vibemacro tidak menyediakan bypass.
