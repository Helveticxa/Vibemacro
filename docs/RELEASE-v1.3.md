# Vibemacro 1.3.0

Release ini memperbaiki masalah paling merusak dari scope **Global**: macro klik
kiri/kanan yang masih berjalan ketika pengguna Alt+Tab akan mengikuti fokus dan
membanjiri aplikasi berikutnya sampai aplikasi itu tidak dapat diketik sama
sekali.

## Anchor scope Global

- Scope **Global** tidak lagi mengikuti fokus. Root window yang aktif ketika
  trigger ditekan menjadi anchor untuk seluruh sesi playback.
- Selama anchor tidak aktif, playback mem-pause: semua key dan button yang masih
  ditahan dilepas satu kali, lalu dipulihkan satu kali saat anchor aktif kembali.
  Tidak ada input yang menumpuk dan tidak ada input yang bocor ke aplikasi lain.
- Anchor memakai jalur focus-lock yang sama dengan mode Game, termasuk verifikasi
  HWND root + PID sebelum setiap event dan setiap slice delay.
- Jika aplikasi anchor ditutup atau berganti proses, macro berhenti fail-safe dan
  tidak dialihkan ke window mana pun.
- Tidak ada setting baru, tidak ada perubahan layout, dan format file macro tidak
  berubah. Macro lama berjalan apa adanya.

Label panel Output macro untuk Global berubah dari "Mengikuti app aktif" menjadi
"Kunci app saat trigger".

## Kompatibilitas klik background

Klik pada scope **App** kini mengirim `WM_MOUSEMOVE` ke receiver sebelum
button-down. Sebagian kontrol Win32 hanya memproses klik setelah state hover
diperbarui, sehingga perubahan ini menaikkan jumlah aplikasi yang benar-benar
menerima klik background. Kegagalan mouse move sengaja tidak fatal.

## Batas teknis yang diverifikasi ulang

Permintaan agar macro tetap mengklik game sementara pengguna bekerja di aplikasi
lain **tidak dapat dipenuhi dalam satu sesi Windows**. Satu sesi hanya memiliki
satu input desktop dan satu foreground window, dan `SendInput` selalu masuk ke
input desktop yang sedang aktif.

Pendekatan desktop Win32 terpisah diuji langsung pada Windows 11 dan gagal:
proses probe berhasil dijalankan pada desktop non-input dan window-nya terlihat
lewat `EnumDesktopWindows`, tetapi `GetForegroundWindow()` pada desktop itu
bernilai `0` dan `SendInput` keyboard maupun mouse mengembalikan `0` dengan
`GetLastError() = 5` (`ERROR_ACCESS_DENIED`). Tidak ada satu pun event yang
diterima probe.

Karena itu hanya ada dua jalur yang jujur:

- Aplikasi yang menerima pesan window memakai scope **App**, dan itu memang
  berjalan di background sambil pengguna Alt+Tab.
- Game yang membaca Raw Input, termasuk Roblox, memerlukan foreground. Bila
  komputer harus tetap dipakai untuk hal lain, jalankan game pada sesi Windows
  terpisah (VM atau PC kedua) yang memiliki input desktop sendiri.

Release ini tidak memasang driver, tidak menginjeksi proses game, dan tidak
mencoba melewati anti-cheat. Pengguna tetap bertanggung jawab mematuhi aturan
game dan experience. Binary publik belum Authenticode-signed, sehingga
SmartScreen dapat memperingatkan.

## Asset

- `Vibemacro-Setup-1.3.0-x64.exe`
- `Vibemacro-1.3.0-portable.exe`
- `vibemacro-update.txt`
- `SHA256SUMS.txt`
