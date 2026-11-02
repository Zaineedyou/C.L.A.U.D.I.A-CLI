# C.L.A.U.D.I.A-CLI

CLI chat AI dengan TUI mewah (Rust + ratatui), tata letak ala OpenCode dengan tampilan pastel yang lembut: tab sesi di atas, sidebar,
prompt berpanel, dropdown perintah, dan animasi ala Crush. Semua provider bicara dialek OpenAI.

    cargo run --release          # binary-nya bernama `claudia`

API key: ketik `/connect`, pilih provider, lalu tempel key di dialog (Enter simpan). Key disimpan di
~/.config/claudia/keys (izin 600, teks biasa) dan dipakai lagi di sesi berikutnya. `/disconnect` menghapus
key provider yang aktif. Alternatif: export GROQ_API_KEY=xxx (atau OPENAI_API_KEY, ANTHROPIC_API_KEY,
GEMINI_API_KEY, VENICE_API_KEY, HF_TOKEN, ...); key dari environment tidak pernah ditulis ke disk.

Perintah: ketik `/` buat dropdown (↑↓ pilih, Tab lengkapi, Enter jalankan, Esc tutup). Ctrl+P membuka palet
perintah yang sama. /connect · /provider · /model · /system · /new · /close · /clear · /tab <n> · /key <kunci> · /disconnect · /help · /exit
Tombol: Ctrl+O model · Enter kirim · Esc batal · Ctrl+L bersih · Ctrl+C keluar
Tab sesi: klik/sentuh tab atau "+" · Ctrl+N sesi baru · Ctrl+W tutup · Alt+panah / Alt+1..9 pindah tab · roda/geser menggulir

System prompt: `/system` membuka editor (Ctrl+S simpan, Ctrl+R kembali ke bawaan, Ctrl+U kosongkan, Esc batal).
`/system <teks>` mengganti langsung, `/system reset` mengembalikan bawaan. Hasil edit disimpan di
~/.config/claudia/system dan dipakai mulai pesan berikutnya.

Env opsional: CLAUDIA_PROVIDER, CLAUDIA_MODEL, CLAUDIA_NAME, CLAUDIA_SYSTEM (dikalahkan hasil edit),
CLAUDIA_BASE_URL + CLAUDIA_API_KEY (provider "custom"). Pilihan terakhir disimpan di ~/.config/claudia/state.

Hugging Face Space: pilih provider "HF Space" lewat /connect (isi HF_TOKEN), lalu ketik id Space di /model
(mis. `owner/nama-space`) atau set env HF_SPACE. Dipanggil lewat API Gradio (`/gradio_api/call/chat`), nilai
parameter tambahan diambil dari skema Space. Kalau endpoint-nya bukan /chat, set HF_SPACE_API; kalau
parameter tambahan perlu diisi sendiri, set HF_SPACE_ARGS (JSON list, mis. `["Kamu asisten ramah", 512, 0.7, 0.95]`).

File di perangkat: Claudia tidak membaca file sendiri. Baru ada yang dibaca kalau lu menyuruh: `/files` (atau Ctrl+F)
membuka browser file (Enter buka folder / lampirkan file, Tab lampirkan dan tetap terbuka, ← naik, Ctrl+A lampirkan
folder ini, Ctrl+T file tersembunyi, Ctrl+G home, Ctrl+R root, ketik buat filter), `/attach <path>` melampirkan langsung,
dan menulis `@path` di pesan melampirkan file/folder itu. Lampiran tampil di atas prompt (Ctrl+X atau `/detach` melepas)
dan isinya dikirim bareng pesan berikutnya. Hanya teks (maks 100 KB per file, 300 KB per pesan); file biner ditolak.

Akses file untuk model: model punya alat baca (read-only) ke seluruh file di perangkat (list_dir, read_file, find_files, grep),
tapi alat itu cuma ditawarkan pada giliran yang pesan lu jelas meminta soal file/folder (mis. "baca file ~/a.txt", "lihat isi
folder download", atau menyebut path), dan system prompt memerintahkannya tidak menjelajah sendiri. Tiap pemakaian alat tampil
sebagai baris `◈ baca ...` di atas balasan. `/akses auto` (bawaan), `/akses on` (selalu ditawarkan), `/akses off` (mati);
env CLAUDIA_FILES mengatur nilai awal. Model tidak bisa menulis atau menghapus file. Hasil baca tidak disimpan di riwayat,
jadi pertanyaan lanjutan soal file yang sama mungkin membacanya lagi. Provider harus mendukung function calling (Groq, OpenAI,
OpenRouter, dll.); kalau tidak, otomatis lanjut tanpa baca file.
