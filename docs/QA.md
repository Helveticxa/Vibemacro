# Vibemacro 1.2.0 - QA Evidence

Tanggal verifikasi lokal: 2026-08-12.

## Root cause dan kontrak perbaikan

Mode Window lama memakai `PostMessageW`. Jalur itu dapat mengisi chat/control
Win32, tetapi game yang membaca Raw Input tidak wajib menganggapnya sebagai
keyboard/mouse perangkat. Vibemacro 1.2 memisahkan dua target mode:

- **App**: pesan background untuk aplikasi Win32 yang mendukungnya.
- **Game**: `SendInput` scan-code/mouse hanya ketika exact target instance menjadi
  foreground. Alt+Tab me-release input held dan mem-pause playback.

Tidak ada driver virtual, process injection, atau anti-cheat bypass. Windows
tidak menyediakan virtual mouse per-window melalui `SendInput`; karena itu input
Game tidak berjalan di Roblox background dan tidak dialihkan ke app lain.

## Gate kode dan test

- `cargo fmt -- --check`: lulus.
- `cargo clippy --all-targets -- -D warnings`: lulus, nol warning.
- 34 test library: lulus.
- 4 test binary/Windows: lulus, termasuk desktop E2E interaktif.
- Total: 38/38.

Test baru membuktikan:

- format macro v3 menyimpan `ForegroundExclusive`;
- file v2 dimigrasikan ke mode App background;
- WASD Game memakai scan code, bukan hanya virtual-key message;
- key/button held dideduplikasi, dilepas, dan dapat dilanjutkan tanpa stacking;
- toggle Game tidak menambah event ketika target kehilangan foreground;
- macro tidak berpindah ke window kedua setelah Alt+Tab;
- target PID/HWND yang ditutup atau berubah berhenti fail-safe.

Actual Roblox account tidak dibuka atau diotomasi selama QA. Roblox/game dapat
memiliki kebijakan dan filter input sendiri; smoke test user tetap diperlukan.

## Visual QA

Snapshot `qa/vibemacro-game-scope.bmp` diperiksa pada resolusi asli. Scope
Global/App/Game terbaca penuh, selection Game jelas, status
`Alt+Tab: pause aman` tidak terpotong, dan timeline W + left click tetap rapi.

## Installer upgrade E2E

Fixture memakai QA AppId terpisah; instalasi produksi user tidak disentuh.

| Pemeriksaan | Hasil |
|---|---|
| Install fixture VibeTimer 1.0 | Lulus |
| Upgrade ke Vibemacro 1.2.0 | Lulus |
| EXE lama terhapus | Ya |
| Display name/version | `Vibemacro 1.2.0` |
| Single-instance | Exit code 0 |
| Data legacy tersalin tepat | Ya |
| Data legacy dipertahankan | Ya |
| Uninstall menghapus app directory | Ya |
| Uninstall mempertahankan user data | Ya |
| Registrasi uninstall QA dibersihkan | Ya |
| Registrasi produksi pengguna berubah | Tidak |

## Artefak lokal

| File | Ukuran | SHA-256 |
|---|---:|---|
| `Vibemacro-Setup-1.2.0-x64.exe` | 2.444.655 byte | `851BE7340C7B092D92745821D607934210D8CF1BF252DD7862841F1E5EA3AB77` |
| `Vibemacro-1.2.0-portable.exe` | 396.800 byte | `7385B6C3F26FD87B1AF66F73C7F91E1BA5BD3BBA6989443B211D7B174D450C0A` |
| `vibemacro-update.txt` | 210 byte | `27D222B225D92CC7EAECCAA468D94EC768E438B76D1997225682C3F2854AD0FB` |
| `SHA256SUMS.txt` | 278 byte | `9C0E236215BE6E0F14060B744DCA9F5A5E94AB2052281842C908B156AB89EF50` |

Hash publik GitHub dapat berbeda karena workflow membangun ulang dari commit/tag
yang sama pada Windows runner. Setelah release, hash asset live harus dicatat di
bagian berikut dan dicocokkan dengan manifest/checksums yang diunduh anonim.

## Security dan performance lokal

- `tools/security-scan.ps1 -IncludeHistory`: lulus; kandidat file dan histori
  Git dipindai tanpa mencetak nilai secret.
- Microsoft Defender custom scan seluruh `dist/`: exit 0, `found no threats`.
- Authenticode: installer dan portable `NotSigned`; SmartScreen masih mungkin.
- Profil idle 8 detik setelah startup update check pada data QA terisolasi:
  CPU 0,000% satu core, handle 404 -> 404, thread 11 -> 11, working set
  21,59 MiB, private memory 3,66 MiB, responding.

Hasil scan bersifat scoped pada artefak dan pola saat ini, bukan jaminan universal
bahwa software tidak mungkin memiliki kerentanan.

## GitHub Release live

Belum diisi. Bagian ini hanya boleh diperbarui setelah CI, tag `v1.2.0`, Release,
manifest latest, hash download anonim, checksum, dan Defender asset live lulus.
