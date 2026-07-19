//! Speech-to-Text using Whisper

use std::path::PathBuf;

/// Speech-to-text engine
pub struct SpeechToText {
    model_path: Option<PathBuf>,
    language: String,
}

impl SpeechToText {
    pub fn new() -> Self {
        Self {
            model_path: None,
            language: "es".to_string(), // Default to Spanish
        }
    }

    /// Load whisper model
    pub fn load_model(&mut self, path: PathBuf) -> Result<(), String> {
        if !path.exists() {
            return Err(format!("Model file not found: {:?}", path));
        }
        self.model_path = Some(path);
        tracing::info!("Whisper model loaded");
        Ok(())
    }

    /// Set language
    pub fn set_language(&mut self, lang: &str) {
        self.language = lang.to_string();
    }

    /// Transcribe audio data (PCM f32 samples)
    pub fn transcribe(&self, audio: &[f32], sample_rate: u32) -> Result<String, String> {
        if self.model_path.is_none() {
            return Err("No model loaded".to_string());
        }

        // Stub implementation - in production, use whisper.cpp via whisper-rs
        let duration_ms = (audio.len() as f64 / sample_rate as f64 * 1000.0) as u64;
        tracing::debug!(
            "Transcribing {}ms of audio (language={})",
            duration_ms,
            self.language
        );

        // Return placeholder
        Ok("[Voice transcription placeholder]".to_string())
    }

    /// Check if model is loaded
    pub fn is_loaded(&self) -> bool {
        self.model_path.is_some()
    }
}

impl Default for SpeechToText {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stt_new() {
        let stt = SpeechToText::new();
        assert!(!stt.is_loaded());
        assert_eq!(stt.language, "es");
    }

    #[test]
    fn test_stt_no_model() {
        let stt = SpeechToText::new();
        let result = stt.transcribe(&[0.0; 16000], 16000);
        assert!(result.is_err());
    }
}
