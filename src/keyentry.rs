use crate::input::Input;

/// Dialog isi API key untuk satu provider (`provider` = indeks di `provider::ALL`).
pub struct KeyEntry {
    pub provider: usize,
    pub input: Input,
    pub error: Option<String>,
    /// Umur popup dalam tick, untuk animasi buka.
    pub age: u8,
}

impl KeyEntry {
    pub fn new(provider: usize) -> Self {
        Self { provider, input: Input::default(), error: None, age: 0 }
    }
}
