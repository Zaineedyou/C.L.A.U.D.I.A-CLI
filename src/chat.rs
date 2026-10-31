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
}

impl Message {
    pub fn new(role: Role, content: impl Into<String>) -> Self {
        Self { role, content: content.into(), tag: String::new() }
    }

    pub fn tagged(mut self, tag: impl Into<String>) -> Self {
        self.tag = tag.into();
        self
    }

    pub fn to_json(&self) -> Value {
        json!({ "role": self.role.as_str(), "content": self.content })
    }
}
