//! Voice Command Parser - Parse natural language commands

use serde::{Deserialize, Serialize};

/// Parsed voice command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceCommand {
    pub action: CommandAction,
    pub target: Option<String>,
    pub args: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CommandAction {
    OpenFile,
    RunTests,
    GenerateCode,
    SearchGraph,
    ExecuteCode,
    CreateFile,
    ListFiles,
    Unknown,
}

/// Parse voice input into commands
pub struct VoiceCommandParser;

impl VoiceCommandParser {
    pub fn new() -> Self {
        Self
    }

    /// Parse text into a voice command
    pub fn parse(&self, text: &str) -> VoiceCommand {
        let lower = text.to_lowercase();

        if lower.contains("abre") || lower.contains("open") {
            let target = self.extract_target(&lower, &["abre", "open"]);
            return VoiceCommand {
                action: CommandAction::OpenFile,
                target,
                args: vec![],
            };
        }

        if lower.contains("ejecuta") || lower.contains("run") || lower.contains("test") {
            return VoiceCommand {
                action: CommandAction::RunTests,
                target: None,
                args: vec![text.to_string()],
            };
        }

        if lower.contains("genera") || lower.contains("create") || lower.contains("generar") {
            let target = self.extract_target(&lower, &["genera", "create", "generar", "para"]);
            return VoiceCommand {
                action: CommandAction::GenerateCode,
                target,
                args: vec![text.to_string()],
            };
        }

        if lower.contains("busca") || lower.contains("search") || lower.contains("buscar") {
            let target = self.extract_target(&lower, &["busca", "search", "buscar", "en el grafo"]);
            return VoiceCommand {
                action: CommandAction::SearchGraph,
                target,
                args: vec![],
            };
        }

        if lower.contains("ejecutar código") || lower.contains("run code") {
            return VoiceCommand {
                action: CommandAction::ExecuteCode,
                target: None,
                args: vec![text.to_string()],
            };
        }

        if lower.contains("crea archivo") || lower.contains("new file") {
            let target = self.extract_target(&lower, &["crea archivo", "new file", "crear"]);
            return VoiceCommand {
                action: CommandAction::CreateFile,
                target,
                args: vec![],
            };
        }

        if lower.contains("lista") || lower.contains("list") || lower.contains("archivos") {
            return VoiceCommand {
                action: CommandAction::ListFiles,
                target: None,
                args: vec![],
            };
        }

        VoiceCommand {
            action: CommandAction::Unknown,
            target: None,
            args: vec![text.to_string()],
        }
    }

    fn extract_target(&self, text: &str, keywords: &[&str]) -> Option<String> {
        for keyword in keywords {
            if let Some(pos) = text.find(keyword) {
                let after = &text[pos + keyword.len()..];
                let trimmed = after.trim();
                // Skip common filler words
                let trimmed = trimmed
                    .trim_start_matches("archivo")
                    .trim_start_matches("el")
                    .trim_start_matches("la")
                    .trim_start_matches("un")
                    .trim_start_matches("una")
                    .trim();
                if !trimmed.is_empty() {
                    return Some(trimmed.to_string());
                }
            }
        }
        None
    }
}

impl Default for VoiceCommandParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_open() {
        let parser = VoiceCommandParser::new();
        let cmd = parser.parse("abre archivo main.rs");
        assert_eq!(cmd.action, CommandAction::OpenFile);
        assert_eq!(cmd.target, Some("main.rs".to_string()));
    }

    #[test]
    fn test_parse_generate() {
        let parser = VoiceCommandParser::new();
        let cmd = parser.parse("genera código para una función de排序");
        assert_eq!(cmd.action, CommandAction::GenerateCode);
    }

    #[test]
    fn test_parse_search() {
        let parser = VoiceCommandParser::new();
        let cmd = parser.parse("busca en el grafo módulos de autenticación");
        assert_eq!(cmd.action, CommandAction::SearchGraph);
    }

    #[test]
    fn test_parse_unknown() {
        let parser = VoiceCommandParser::new();
        let cmd = parser.parse("hola mundo");
        assert_eq!(cmd.action, CommandAction::Unknown);
    }
}
