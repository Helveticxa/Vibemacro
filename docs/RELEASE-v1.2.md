# Vibemacro 1.2.0

Release ini memperbaiki klik dan WASD pada game yang mengabaikan pesan Win32
background, sekaligus mencegah macro berpindah ke aplikasi lain ketika Alt+Tab.

## Game focus-lock

- Scope Macro Studio kini terdiri dari **Global**, **App**, dan **Game**.
- App mempertahankan mode background `PostMessageW` untuk software Win32 biasa.
- Game memakai `SendInput` dengan keyboard scan code dan mouse event hanya ketika
  exact target instance sedang foreground.
- Alt+Tab otomatis melepas semua key/button yang masih down dan mem-pause macro.
- Ketika instance yang sama aktif lagi, state held input dipulihkan dan timeline
  dilanjutkan tanpa memindahkan klik/WASD ke aplikasi lain.
- Jika target ditutup, berganti PID, atau beberapa instance tidak dapat dibedakan,
  macro berhenti/fail-safe dan meminta pengguna memilih ulang target.
- Format macro v3 menyimpan output mode; file v1/v2 tetap dapat dibaca dan
  dimigrasikan ke mode App background yang kompatibel.

E2E Windows terisolasi membuktikan scan-code WASD dan klik berhenti ketika
window QA kehilangan foreground, tidak dialihkan ke window kedua, lalu dapat
dihentikan melalui trigger global. Pengujian ini tidak membuka atau mengotomasi
akun Roblox milik pengguna. Jika chat dalam game sedang fokus, input akan tetap
masuk ke chat sebagaimana keyboard fisik; fokuskan viewport gameplay dahulu.

## Batas teknis dan keamanan

Roblox dan game lain dapat memakai Raw Input yang berbeda dari pesan window
tradisional. Windows `SendInput` memasukkan event ke input stream desktop, bukan
membuat mouse virtual terpisah untuk setiap window. Karena itu mode Game harus
foreground dan berhenti mengirim saat Alt+Tab. Release ini tidak memasang driver,
tidak menginjeksi proses game, dan tidak mencoba melewati anti-cheat.

Pengguna tetap bertanggung jawab mematuhi aturan game dan experience. Binary
publik belum Authenticode-signed, sehingga SmartScreen dapat memperingatkan.

## Asset

- `Vibemacro-Setup-1.2.0-x64.exe`
- `Vibemacro-1.2.0-portable.exe`
- `vibemacro-update.txt`
- `SHA256SUMS.txt`
