use std::collections::HashMap;
use std::fs;

pub struct AutocompleteEngine {
    words: HashMap<String, String>, // lowercase_key -> original_word
}

impl AutocompleteEngine {
    pub fn new() -> Self {
        Self {
            words: HashMap::new(),
        }
    }

    pub fn load_from_file(&mut self, file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let content = fs::read_to_string(file_path)?;
        self.words.clear();
        
        // Parse comma-separated words
        for word in content.split(',') {
            let trimmed = word.trim();
            if !trimmed.is_empty() {
                // Store with lowercase key but preserve original casing
                self.words.insert(trimmed.to_lowercase(), trimmed.to_string());
            }
        }
        
        Ok(())
    }

    pub fn get_suggestions(&self, input: &str, max_suggestions: usize) -> Vec<String> {
        if input.is_empty() {
            return Vec::new();
        }

        let input_lower = input.to_lowercase();
        let mut suggestions: Vec<String> = self.words
            .iter()
            .filter(|(key, _)| key.starts_with(&input_lower))
            .map(|(_, original)| original.clone()) // Return original casing
            .collect();

        // Sort by length (shorter words first, then alphabetically by lowercase)
        suggestions.sort_by(|a, b| {
            a.len().cmp(&b.len()).then_with(|| a.to_lowercase().cmp(&b.to_lowercase()))
        });

        suggestions.truncate(max_suggestions);
        suggestions
    }

    pub fn add_word(&mut self, word: &str) {
        if !word.trim().is_empty() {
            let trimmed = word.trim();
            self.words.insert(trimmed.to_lowercase(), trimmed.to_string());
        }
    }

    pub fn save_to_file(&self, file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut words_vec: Vec<String> = self.words.values().cloned().collect();
        words_vec.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));
        let content = words_vec.join(", ");
        fs::write(file_path, content)?;
        Ok(())
    }

    pub fn has_words(&self) -> bool {
        !self.words.is_empty()
    }
}

impl Default for AutocompleteEngine {
    fn default() -> Self {
        Self::new()
    }
}

// Clipboard functionality
pub fn copy_to_clipboard(text: &str) -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(not(windows))]
    {
        use std::process::{Command, Stdio};
        
        // Try xclip first, then xsel as fallback
        let mut cmd = Command::new("xclip")
            .arg("-selection")
            .arg("clipboard")
            .stdin(Stdio::piped())
            .spawn();

        if cmd.is_err() {
            cmd = Command::new("xsel")
                .arg("--clipboard")
                .arg("--input")
                .stdin(Stdio::piped())
                .spawn();
        }

        if let Ok(mut child) = cmd {
            if let Some(stdin) = child.stdin.as_mut() {
                use std::io::Write;
                stdin.write_all(text.as_bytes())?;
            }
            child.wait()?;
        } else {
            return Err("Neither xclip nor xsel found. Please install one of them for clipboard functionality.".into());
        }
    }

    #[cfg(windows)]
    {
        use std::process::Command;
        Command::new("cmd")
            .args(["/C", "echo", text, "|", "clip"])
            .output()?;
    }

    Ok(())
}
