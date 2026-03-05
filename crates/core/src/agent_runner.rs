use crate::{LlmBundle, PipelineError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentRunnerRequest {
    pub bundle: LlmBundle,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentGeneratedFile {
    pub relative_path: String,
    pub contents: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AgentRunnerOutput {
    pub generated_files: Vec<AgentGeneratedFile>,
}

pub trait AgentRunner {
    fn generate(&self, request: &AgentRunnerRequest) -> Result<AgentRunnerOutput, PipelineError>;
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct MockAgentRunner;

impl AgentRunner for MockAgentRunner {
    fn generate(&self, request: &AgentRunnerRequest) -> Result<AgentRunnerOutput, PipelineError> {
        let target = normalize_target(request.bundle.request.target.as_str());
        let (file_name, header) = if target.contains("swiftui") {
            ("App.swift", "import SwiftUI")
        } else {
            ("App.tsx", "export default function App() {")
        };
        let relative_path = format!("output/generated/{target}/{file_name}");
        let contents = if file_name.ends_with(".swift") {
            format!(
                "{header}\n\n// intent: {}\nstruct App: View {{\n    var body: some View {{ Text(\"Hello from Specloom\") }}\n}}\n",
                request.bundle.request.intent
            )
        } else {
            format!(
                "{header}\n  // intent: {}\n  return <div>Hello from Specloom</div>;\n}}\n",
                request.bundle.request.intent
            )
        };

        Ok(AgentRunnerOutput {
            generated_files: vec![AgentGeneratedFile {
                relative_path,
                contents,
            }],
        })
    }
}

fn normalize_target(raw_target: &str) -> String {
    let mut normalized = String::new();
    for ch in raw_target.trim().chars() {
        if ch.is_ascii_alphanumeric() {
            normalized.push(ch.to_ascii_lowercase());
        } else if ch == '-' || ch == '_' {
            normalized.push(ch);
        } else if ch.is_whitespace() {
            normalized.push('-');
        }
    }
    if normalized.is_empty() {
        "unknown-target".to_string()
    } else {
        normalized
    }
}
