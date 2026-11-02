use serde_json::{json, Value};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Role {
    System,
    User,
    Assistant,
}

impl Role {
    pub fn as_str(self) -> &'static str {
        match self {
            Role::System => "system",
            Role::User => "user",
            Role::Assistant => "assistant",
        }
    }
}

#[derive(Clone)]
pub struct Message {
    pub role: Role,
    pub content: String,
    /// Label kecil di bawah balasan, mis. "Claudia · model".
    pub tag: String,
    /// Nama lampiran yang ditampilkan di bawah pesan user.
    pub files: Vec<String>,
    /// Isi lampiran; ikut dikirim ke model tapi tidak ditampilkan.
    pub context: String,
    /// Alat file yang dipakai model buat balasan ini; hanya ditampilkan, tidak dikirim balik.
    pub tools: Vec<String>,
}

impl Message {
    pub fn new(role: Role, content: impl Into<String>) -> Self {
        Self { role, content: content.into(), tag: String::new(), files: Vec::new(), context: String::new(), tools: Vec::new() }
    }

    pub fn tagged(mut self, tag: impl Into<String>) -> Self {
        self.tag = tag.into();
        self
    }

    pub fn with_files(mut self, files: Vec<String>, context: String) -> Self {
        self.files = files;
        self.context = context;
        self
    }

    /// Yang dikirim ke model: teks pesan ditambah isi lampiran.
    pub fn full(&self) -> String {
        format!("{}{}", self.content, self.context)
    }

    pub fn to_json(&self) -> Value {
        json!({ "role": self.role.as_str(), "content": self.full() })
    }
}
