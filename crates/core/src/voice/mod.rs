//! Voice Module - Speech-to-text and text-to-speech
//!
//! Provides local, offline voice interaction using:
//! - Whisper for speech-to-text
//! - Piper for text-to-speech
//! - cpal for audio capture

pub mod stt;
pub mod tts;
pub mod parser;

pub use stt::SpeechToText;
pub use tts::TextToSpeech;
pub use parser::VoiceCommand;
