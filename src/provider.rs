//! Provider yang bicara dialek OpenAI (`/chat/completions` dan `/models`).

pub struct Provider {
    pub id: &'static str,
    pub name: &'static str,
    pub base_url: &'static str,
    /// Nama env var API key. Kosong = tidak butuh key (provider lokal).
    pub key_env: &'static str,
    pub default_model: &'static str,
    /// Dipakai selama daftar model asli dari API belum (atau tidak bisa) diambil.
    pub fallback_models: &'static [&'static str],
}

impl Provider {
    pub fn is_custom(&self) -> bool {
        self.id == "custom"
    }

    /// Yang harus diset user supaya provider ini siap dipakai.
    pub fn requirement(&self) -> &'static str {
        if self.is_custom() {
            "CLAUDIA_BASE_URL"
        } else {
            self.key_env
        }
    }
}

const fn p(
    id: &'static str,
    name: &'static str,
    base_url: &'static str,
    key_env: &'static str,
    default_model: &'static str,
    fallback_models: &'static [&'static str],
) -> Provider {
    Provider { id, name, base_url, key_env, default_model, fallback_models }
}

pub static ALL: &[Provider] = &[
    p("groq", "Groq", "https://api.groq.com/openai/v1", "GROQ_API_KEY", "llama-3.3-70b-versatile",
      &["llama-3.3-70b-versatile", "llama-3.1-8b-instant", "openai/gpt-oss-120b", "openai/gpt-oss-20b"]),
    p("openai", "OpenAI", "https://api.openai.com/v1", "OPENAI_API_KEY", "gpt-4o-mini",
      &["gpt-4o-mini", "gpt-4o", "gpt-4.1", "gpt-4.1-mini", "o4-mini"]),
    p("anthropic", "Anthropic", "https://api.anthropic.com/v1", "ANTHROPIC_API_KEY", "claude-sonnet-5-5",
      &["claude-opus-5-5", "claude-sonnet-5-5", "claude-haiku-5-5"]),
    p("gemini", "Google Gemini", "https://generativelanguage.googleapis.com/v1beta/openai", "GEMINI_API_KEY", "gemini-2.5-flash",
      &["gemini-2.5-flash", "gemini-2.5-pro", "gemini-2.0-flash"]),
    p("openrouter", "OpenRouter", "https://openrouter.ai/api/v1", "OPENROUTER_API_KEY", "meta-llama/llama-3.3-70b-instruct",
      &["meta-llama/llama-3.3-70b-instruct", "deepseek/deepseek-chat", "google/gemini-2.5-flash", "qwen/qwen-2.5-72b-instruct"]),
    p("deepseek", "DeepSeek", "https://api.deepseek.com/v1", "DEEPSEEK_API_KEY", "deepseek-chat",
      &["deepseek-chat", "deepseek-reasoner"]),
    p("mistral", "Mistral", "https://api.mistral.ai/v1", "MISTRAL_API_KEY", "mistral-small-latest",
      &["mistral-small-latest", "mistral-large-latest", "open-mistral-nemo"]),
    p("xai", "xAI (Grok)", "https://api.x.ai/v1", "XAI_API_KEY", "grok-3-mini",
      &["grok-3-mini", "grok-3", "grok-4"]),
    p("together", "Together AI", "https://api.together.xyz/v1", "TOGETHER_API_KEY", "meta-llama/Llama-3.3-70B-Instruct-Turbo",
      &["meta-llama/Llama-3.3-70B-Instruct-Turbo", "Qwen/Qwen2.5-72B-Instruct-Turbo", "deepseek-ai/DeepSeek-V3"]),
    p("fireworks", "Fireworks", "https://api.fireworks.ai/inference/v1", "FIREWORKS_API_KEY", "accounts/fireworks/models/llama-v3p3-70b-instruct",
      &["accounts/fireworks/models/llama-v3p3-70b-instruct", "accounts/fireworks/models/deepseek-v3"]),
    p("cerebras", "Cerebras", "https://api.cerebras.ai/v1", "CEREBRAS_API_KEY", "llama-3.3-70b",
      &["llama-3.3-70b", "llama3.1-8b"]),
    p("sambanova", "SambaNova", "https://api.sambanova.ai/v1", "SAMBANOVA_API_KEY", "Meta-Llama-3.3-70B-Instruct",
      &["Meta-Llama-3.3-70B-Instruct", "DeepSeek-V3-0324"]),
    p("nvidia", "NVIDIA NIM", "https://integrate.api.nvidia.com/v1", "NVIDIA_API_KEY", "meta/llama-3.3-70b-instruct",
      &["meta/llama-3.3-70b-instruct", "nvidia/llama-3.1-nemotron-70b-instruct"]),
    p("deepinfra", "DeepInfra", "https://api.deepinfra.com/v1/openai", "DEEPINFRA_API_KEY", "meta-llama/Llama-3.3-70B-Instruct",
      &["meta-llama/Llama-3.3-70B-Instruct", "deepseek-ai/DeepSeek-V3"]),
    p("hyperbolic", "Hyperbolic", "https://api.hyperbolic.xyz/v1", "HYPERBOLIC_API_KEY", "meta-llama/Llama-3.3-70B-Instruct",
      &["meta-llama/Llama-3.3-70B-Instruct", "deepseek-ai/DeepSeek-V3"]),
    p("perplexity", "Perplexity", "https://api.perplexity.ai", "PERPLEXITY_API_KEY", "sonar",
      &["sonar", "sonar-pro", "sonar-reasoning", "sonar-reasoning-pro", "sonar-deep-research"]),
    p("moonshot", "Moonshot (Kimi)", "https://api.moonshot.ai/v1", "MOONSHOT_API_KEY", "kimi-k2-0905-preview",
      &["kimi-k2-0905-preview", "moonshot-v1-8k"]),
    p("zai", "Z.ai (GLM)", "https://api.z.ai/api/paas/v4", "ZAI_API_KEY", "glm-4.5",
      &["glm-4.5", "glm-4.5-air"]),
    p("siliconflow", "SiliconFlow", "https://api.siliconflow.com/v1", "SILICONFLOW_API_KEY", "Qwen/Qwen2.5-72B-Instruct",
      &["Qwen/Qwen2.5-72B-Instruct", "deepseek-ai/DeepSeek-V3"]),
    p("ollama", "Ollama (lokal)", "http://localhost:11434/v1", "", "llama3.2",
      &["llama3.2", "qwen2.5", "gemma3"]),
    p("lmstudio", "LM Studio (lokal)", "http://localhost:1234/v1", "", "local-model", &[]),
    p("custom", "Custom (CLAUDIA_BASE_URL)", "", "CLAUDIA_API_KEY", "gpt-4o-mini", &[]),
];

/// Cari indeks provider dari id atau nama (tanpa peduli huruf besar/kecil).
pub fn find(query: &str) -> Option<usize> {
    let q = query.trim().to_lowercase();
    ALL.iter()
        .position(|p| p.id == q || p.name.to_lowercase() == q)
}
