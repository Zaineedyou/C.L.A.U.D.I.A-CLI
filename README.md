# C.L.A.U.D.I.A-CLI

CLI chat AI dengan TUI mewah (Rust + ratatui), tata letak ala OpenCode: tab sesi di atas, sidebar,
prompt berpanel, dan animasi ala Crush. Semua provider bicara dialek OpenAI.

    export GROQ_API_KEY=xxx      # atau OPENAI_API_KEY, ANTHROPIC_API_KEY, GEMINI_API_KEY, VENICE_API_KEY, HF_TOKEN, ...
    cargo run --release          # binary-nya bernama `claudia`

Tombol: Ctrl+P provider · Ctrl+O model · Enter kirim · Esc batal · Ctrl+L bersih · Ctrl+C keluar
Tab sesi: Ctrl+N sesi baru · Ctrl+W tutup · Alt+panah / Alt+1..9 pindah tab
Perintah: /provider [nama] · /model [nama] · /key <kunci> · /system · /new · /close · /tab <n> · /clear · /help · /exit

System prompt: `/system` membuka editor (Ctrl+S simpan, Ctrl+R kembali ke bawaan, Ctrl+U kosongkan, Esc batal).
`/system <teks>` mengganti langsung, `/system reset` mengembalikan bawaan. Hasil edit disimpan di
~/.config/claudia/system dan dipakai mulai pesan berikutnya.

Env opsional: CLAUDIA_PROVIDER, CLAUDIA_MODEL, CLAUDIA_NAME, CLAUDIA_SYSTEM (dikalahkan hasil edit),
CLAUDIA_BASE_URL + CLAUDIA_API_KEY (provider "custom"). Pilihan terakhir disimpan di
~/.config/claudia/state (API key tidak pernah disimpan).
