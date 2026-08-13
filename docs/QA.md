# Vibemacro 1.3.0 - QA Evidence

Tanggal verifikasi lokal: 2026-08-13.

## Root cause dan kontrak perbaikan

Scope **Global** memakai `SendInput` tanpa target, sehingga playback mengikuti
foreground window. Ketika pengguna Alt+Tab saat macro klik kiri/kanan berjalan,
seluruh klik dan ketikan berpindah ke aplikasi berikutnya sampai aplikasi itu
tidak dapat dipakai. Vibemacro 1.3 mengunci Global ke root window yang aktif
ketika trigger ditekan:

- Anchor diverifikasi ulang (HWND root + PID) sebelum setiap event dan setiap
  slice delay, memakai jalur focus-lock yang sama dengan mode Game.
- Saat anchor tidak aktif, semua key/button held dilepas satu kali lalu playback
  pause; ketika anchor aktif lagi state held dipulihkan satu kali.
- Anchor yang ditutup atau berganti proses menghentikan macro fail-safe.
- Anchor sengaja tidak mengecualikan window Vibemacro sendiri. Mengembalikan
  "tidak ada anchor" berarti kembali ke follow-focus, yaitu bug yang diperbaiki.

Tidak ada setting baru, perubahan layout, atau perubahan format file macro.

## Batas teknis yang diuji ulang

Permintaan agar macro tetap mengklik game sementara pengguna bekerja di aplikasi
lain tidak dapat dipenuhi dalam satu sesi Windows. Pendekatan desktop Win32
terpisah diuji langsung pada Windows 11 dan gagal:

| Langkah | Hasil |
|---|---|
| `CreateDesktop` + `CreateProcess` dengan `lpDesktop` | Berhasil, probe berjalan pada desktop non-input |
| `EnumDesktopWindows` menemukan window probe | Ya, `visible=True` |
| `SetThreadDesktop` dari thread pemanggil | Berhasil |
| `GetForegroundWindow()` pada desktop itu | `0` |
| `SendInput` keyboard | Mengembalikan `0`, `GetLastError() = 5` |
| `SendInput` mouse | Mengembalikan `0`, `GetLastError() = 5` |
| Event diterima probe | Tidak ada |

Kesimpulan: `SendInput` hanya bekerja pada input desktop yang aktif. Background
sejati hanya lewat pesan window (scope App) atau sesi Windows terpisah (VM/PC
kedua). Tidak ada driver virtual, process injection, atau anti-cheat bypass.

## Gate kode dan test

- `cargo fmt -- --check`: lulus.
- `cargo clippy --all-targets -- -D warnings`: lulus, nol warning.
- 34 test library: lulus.
- 4 test binary/Windows: lulus, termasuk desktop E2E interaktif.
- Total: 38/38.

Test baru pada 1.3 membuktikan scope Global menghasilkan anchor
`ForegroundExclusive` ke window yang aktif, bukan destinasi follow-focus.

Cakupan yang dipertahankan dari 1.2: format macro v3 menyimpan
`ForegroundExclusive`; file v1/v2 dimigrasikan ke App background; WASD Game
memakai scan code; key/button held dideduplikasi, dilepas, dan dilanjutkan tanpa
stacking; toggle Game tidak menambah event ketika target kehilangan foreground;
macro tidak berpindah ke window kedua setelah Alt+Tab; target PID/HWND yang
ditutup atau berubah berhenti fail-safe.

Roblox nyata tidak dibuka atau diotomasi selama QA. Smoke test pengguna pada
game/aplikasi masing-masing tetap diperlukan.

## Visual QA

Snapshot diperiksa pada resolusi asli dengan crop rail "Output macro".

Koreksi terhadap QA 1.2: dokumen 1.2 menyatakan status `Alt+Tab: pause aman`
"tidak terpotong". Pemeriksaan ulang snapshot membuktikan klaim itu **salah** -
teks terpotong di tengah glyph menjadi `Alt+Tab: pause amar` karena rail hanya
selebar 134 px dan label digambar tanpa `DT_END_ELLIPSIS`.

Perbaikan pada 1.3:

- Label mode Game menjadi `Alt+Tab: pause`.
- Label scope Global menjadi `Kunci app pemicu`.
- Label hint scope kini memakai `DT_END_ELLIPSIS`, sehingga overflow di masa
  depan menghasilkan elipsis, bukan potongan glyph.

Snapshot `qa/vibemacro-macro.bmp` dan `qa/vibemacro-game-scope.bmp` diperiksa
ulang setelah perbaikan: ketiga tombol scope terbaca penuh, selection terbaca,
nama target memakai elipsis normal, dan kedua label hint utuh.

## Smoke test binary rilis

`target/release/Vibemacro.exe` dijalankan langsung: `Responding=True`, working
set 26,59 MiB, 11 thread, 448 handle, lalu keluar bersih.

## Artefak lokal

| File | Ukuran | SHA-256 |
|---|---:|---|
| `Vibemacro-Setup-1.3.0-x64.exe` | 2.445.234 byte | `0165F0388D487A43801768EEDE78DE225106D29DD169C294134D4E4E766ED119` |
| `Vibemacro-1.3.0-portable.exe` | 396.800 byte | `45D9DDEAE209FCB8B118DAD3706A38B8439B1F429C0DC7D7932DECE14CE8508D` |
| `vibemacro-update.txt` | 210 byte | `FDE63D6C649EC2C7FB83E374957EB25F3A7BA67619244A42BE349F813FC359BB` |
| `SHA256SUMS.txt` | 278 byte | `BF8069795841D725EC3CDEEED2B2BEDE6153C57B7E1B108AA348B6486DDD25A9` |

Hash publik GitHub dapat berbeda karena workflow membangun ulang dari commit/tag
yang sama pada Windows runner. Hash asset live dicatat di bagian berikut setelah
release dan dicocokkan dengan manifest/checksums yang diunduh anonim.

## Security dan performance lokal

- `tools/security-scan.ps1 -IncludeHistory`: lulus; 33 tracked file dan histori
  Git dipindai tanpa mencetak nilai secret.
- Microsoft Defender custom scan seluruh `dist/`: exit 0, `found no threats`.
- Authenticode: installer dan portable `NotSigned`; SmartScreen masih mungkin
  memperingatkan.

Hasil scan bersifat scoped pada artefak dan pola saat ini, bukan jaminan
universal bahwa software tidak mungkin memiliki kerentanan.

## Catatan workflow release

Job publish sebelumnya memakai path catatan rilis tetap `docs/RELEASE-v1.2.md`.
Tag baru akan diam-diam mempublikasikan catatan rilis lama. Pada 1.3 path itu
dihitung dari tag (`docs/RELEASE-v<major>.<minor>.md`) dan job gagal eksplisit
bila file catatan tidak ada.

## GitHub Release live

- CI commit `b78df32`: sukses pada run `31689241205`.
- Workflow Release tag `v1.3.0`: sukses pada run `31689255595`.
- Release `Vibemacro 1.3.0`: draft `false`, prerelease `false`, ditandai latest.
- Catatan rilis yang terbit berasal dari `docs/RELEASE-v1.3.md`, membuktikan
  perbaikan path dinamis pada job publish bekerja.

| Asset publik | Ukuran | SHA-256 GitHub |
|---|---:|---|
| `Vibemacro-Setup-1.3.0-x64.exe` | 2.453.420 byte | `9C3F96D224C1E1E42031B6278092EE08841CF43D1EA51DEE5F52FDACB5744961` |
| `Vibemacro-1.3.0-portable.exe` | 396.800 byte | `BC917290F067A634B2CC36DEE05C926222C80DA37B0F15A260CC743E9F1D766E` |
| `vibemacro-update.txt` | 210 byte | `96485C25587D3EBD8CF28ADA9CD33A2C3AF9C043954690A2ACD67F5A7A56AB07` |
| `SHA256SUMS.txt` | 278 byte | `960857BD0CD0199BE053FCC2DEA6A6A5036C1440FD5E84F667022FA93E762E7C` |

Hash lokal dan hash publik berbeda karena workflow membangun ulang installer
dari tag yang sama pada Windows runner. Yang mengikat adalah manifest: manifest
publik menunjuk hash installer publik.

### Verifikasi jalur auto-update end-to-end

Dilakukan anonim, persis seperti yang dilakukan aplikasi pengguna:

1. `GET /releases/latest/download/vibemacro-update.txt` mengembalikan
   `version=1.3.0` dan URL installer versi-spesifik untuk `v1.3.0`.
2. Installer diunduh dari URL di dalam manifest itu, bukan dari URL lain.
3. SHA-256 hasil unduhan cocok persis dengan `sha256=` pada manifest.
4. Ketiga entri `SHA256SUMS.txt` cocok dengan hash aktual asset yang diunduh.
5. Microsoft Defender custom scan pada seluruh asset hasil unduhan: exit 0,
   `found no threats`.
6. Installer publik tetap `NotSigned`; SmartScreen masih dapat memperingatkan.

### Riwayat 1.2.0

- CI commit `7949911`: sukses pada run `31560478389`.
- Workflow Release tag `v1.2.0`: sukses pada run `31560582389`.
- Installer live 2.452.805 byte, SHA-256
  `B5175FF02A0AFA753ACC2A84C5F367B2A51129C1A664F9D3E231B78A40D0AAE0`.
- Portable live 396.800 byte, SHA-256
  `14663760B124E1C088E3FB84617F4B3428BDB2EC8801523644BB39122397F692`.
