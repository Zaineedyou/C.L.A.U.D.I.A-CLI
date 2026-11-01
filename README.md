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
Tab sesi: Ctrl+N sesi baru · Ctrl+W tutup · Alt+panah / Alt+1..9 pindah tab

System prompt: `/system` membuka editor (Ctrl+S simpan, Ctrl+R kembali ke bawaan, Ctrl+U kosongkan, Esc batal).
`/system <teks>` mengganti langsung, `/system reset` mengembalikan bawaan. Hasil edit disimpan di
~/.config/claudia/system dan dipakai mulai pesan berikutnya.

Env opsional: CLAUDIA_PROVIDER, CLAUDIA_MODEL, CLAUDIA_NAME, CLAUDIA_SYSTEM (dikalahkan hasil edit),
CLAUDIA_BASE_URL + CLAUDIA_API_KEY (provider "custom"). Pilihan terakhir disimpan di ~/.config/claudia/state.
