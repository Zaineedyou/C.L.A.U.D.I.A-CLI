# C.L.A.U.D.I.A-CLI

CLI chat AI dengan TUI mewah (Rust + ratatui), tata letak ala OpenCode.
Semua provider bicara dialek OpenAI.

    export GROQ_API_KEY=xxx      # atau OPENAI_API_KEY, ANTHROPIC_API_KEY, GEMINI_API_KEY, ...
    cargo run --release          # binary-nya bernama `claudia`

Tombol: Ctrl+P provider · Ctrl+O model · Enter kirim · Esc batal · Ctrl+L bersih · Ctrl+C keluar
Perintah: /provider [nama] · /model [nama] · /key <kunci> · /clear · /help · /exit

Env opsional: CLAUDIA_PROVIDER, CLAUDIA_MODEL, CLAUDIA_NAME, CLAUDIA_SYSTEM,
CLAUDIA_BASE_URL + CLAUDIA_API_KEY (provider "custom"). Pilihan terakhir disimpan di
~/.config/claudia/state (API key tidak pernah disimpan).
