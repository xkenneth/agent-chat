use serde::Deserialize;
use crate::error::Result;

/// JSON structure for SessionStart hook stdin
#[derive(Debug, Deserialize)]
pub struct SessionStartInput {
    pub session_id: String,
    #[allow(dead_code)]
    pub session_type: Option<String>,
}

/// JSON structure for PreToolUse hook stdin
#[derive(Debug, Deserialize)]
pub struct PreToolUseInput {
    #[allow(dead_code)]
    pub tool_name: String,
    pub tool_input: serde_json::Value,
}

/// Read and parse hook JSON from stdin.
pub fn read_session_start() -> Result<SessionStartInput> {
    let mut input = String::new();
    std::io::Read::read_to_string(&mut std::io::stdin(), &mut input)?;
    let parsed: SessionStartInput = serde_json::from_str(&input)?;
    Ok(parsed)
}

/// Read and parse PreToolUse JSON from stdin.
pub fn read_pre_tool_use() -> Result<PreToolUseInput> {
    let mut input = String::new();
    std::io::Read::read_to_string(&mut std::io::stdin(), &mut input)?;
    let parsed: PreToolUseInput = serde_json::from_str(&input)?;
    Ok(parsed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_session_start_json() {
        let json = r#"{"session_id": "abc123", "session_type": "startup"}"#;
        let input: SessionStartInput = serde_json::from_str(json).unwrap();
        assert_eq!(input.session_id, "abc123");
    }

    #[test]
    fn parse_pre_tool_use_edit() {
        let json = r#"{
            "tool_name": "Edit",
            "tool_input": {
                "file_path": "/project/src/main.rs",
                "old_string": "foo",
                "new_string": "bar"
            }
        }"#;
        let input: PreToolUseInput = serde_json::from_str(json).unwrap();
        assert_eq!(input.tool_name, "Edit");
        assert_eq!(
            input.tool_input["file_path"].as_str().unwrap(),
            "/project/src/main.rs"
        );
    }

    #[test]
    fn parse_pre_tool_use_write() {
        let json = r#"{
            "tool_name": "Write",
            "tool_input": {
                "file_path": "/project/src/new_file.rs",
                "content": "fn main() {}"
            }
        }"#;
        let input: PreToolUseInput = serde_json::from_str(json).unwrap();
        assert_eq!(input.tool_name, "Write");
    }

    #[test]
    fn parse_session_start_minimal() {
        let json = r#"{"session_id": "xyz"}"#;
        let input: SessionStartInput = serde_json::from_str(json).unwrap();
        assert_eq!(input.session_id, "xyz");
        assert!(input.session_type.is_none());
    }
}
