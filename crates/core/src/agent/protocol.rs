use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum AgentCommand {
    #[serde(rename = "execute")]
    Execute {
        id: String,
        command: String,
        args: Vec<String>,
        workdir: Option<String>,
        timeout_ms: u64,
    },
    #[serde(rename = "read_file")]
    ReadFile {
        id: String,
        path: String,
    },
    #[serde(rename = "write_file")]
    WriteFile {
        id: String,
        path: String,
        content: String,
    },
    #[serde(rename = "list_directory")]
    ListDirectory {
        id: String,
        path: String,
    },
    #[serde(rename = "ping")]
    Ping,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum AgentResponse {
    #[serde(rename = "execute_result")]
    ExecuteResult {
        id: String,
        exit_code: i32,
        stdout: String,
        stderr: String,
        execution_time_ms: u64,
    },
    #[serde(rename = "file_content")]
    FileContent {
        id: String,
        content: String,
    },
    #[serde(rename = "directory_list")]
    DirectoryList {
        id: String,
        entries: Vec<FileEntry>,
    },
    #[serde(rename = "error")]
    Error {
        id: String,
        message: String,
    },
    #[serde(rename = "pong")]
    Pong,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInfo {
    pub id: String,
    pub name: String,
    pub os: String,
    pub arch: String,
    pub status: AgentStatus,
    pub connected_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AgentStatus {
    Connected,
    Busy,
    Disconnected,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_execute_command() {
        let cmd = AgentCommand::Execute {
            id: "test-1".to_string(),
            command: "python3".to_string(),
            args: vec!["-c".to_string(), "print('hello')".to_string()],
            workdir: None,
            timeout_ms: 30000,
        };
        let json = serde_json::to_string(&cmd).unwrap();
        assert!(json.contains("\"type\":\"execute\""));
    }

    #[test]
    fn test_deserialize_pong() {
        let json = r#"{"type":"pong","payload":null}"#;
        let resp: AgentResponse = serde_json::from_str(json).unwrap();
        assert!(matches!(resp, AgentResponse::Pong));
    }
}
