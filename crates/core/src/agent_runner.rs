use crate::{LlmBundle, PipelineError};
use rig::{client::CompletionClient, completion::Prompt, providers::anthropic};

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnthropicRunnerConfig {
    pub api_key: String,
    pub model: String,
    pub api_base_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnthropicAgentRunner {
    config: AnthropicRunnerConfig,
}

impl AnthropicAgentRunner {
    pub fn new(config: AnthropicRunnerConfig) -> Result<Self, PipelineError> {
        if config.api_key.trim().is_empty() {
            return Err(PipelineError::AgentRunner(
                "anthropic provider missing required value(s): ANTHROPIC_API_KEY (or --api-key). Provide the missing value(s) and retry.".to_string(),
            ));
        }
        if config.model.trim().is_empty() {
            return Err(PipelineError::AgentRunner(
                "anthropic provider missing required value(s): --model. Provide the missing value(s) and retry.".to_string(),
            ));
        }
        Ok(Self { config })
    }
}

impl AgentRunner for AnthropicAgentRunner {
    fn generate(&self, request: &AgentRunnerRequest) -> Result<AgentRunnerOutput, PipelineError> {
        let target = normalize_target(request.bundle.request.target.as_str());
        let (file_name, language_hint) = if target.contains("swiftui") {
            ("App.swift", "SwiftUI")
        } else {
            ("App.tsx", "React TSX")
        };
        let relative_path = format!("output/generated/{target}/{file_name}");

        let prompt = build_anthropic_prompt(&request.bundle, file_name, language_hint);

        let client = if let Some(api_base_url) = self.config.api_base_url.as_ref() {
            anthropic::Client::builder(self.config.api_key.as_str())
                .base_url(api_base_url.as_str())
                .build()
                .map_err(|err| {
                    PipelineError::AgentRunner(format!("anthropic client build failed: {err}"))
                })?
        } else {
            anthropic::Client::new(self.config.api_key.as_str())
        };
        let agent = client
            .agent(self.config.model.as_str())
            .preamble(
                "You generate UI files for Specloom. Return code only without markdown fences.",
            )
            .build();

        let runtime = tokio::runtime::Runtime::new().map_err(|err| {
            PipelineError::AgentRunner(format!("anthropic runtime initialization failed: {err}"))
        })?;
        let response = runtime
            .block_on(agent.prompt(prompt.as_str()).into_future())
            .map_err(|err| {
                PipelineError::AgentRunner(format!("anthropic generation failed: {err}"))
            })?;
        let contents = strip_markdown_fences(response.as_str());

        if contents.trim().is_empty() {
            return Err(PipelineError::AgentRunner(
                "anthropic generation returned empty output".to_string(),
            ));
        }

        Ok(AgentRunnerOutput {
            generated_files: vec![AgentGeneratedFile {
                relative_path,
                contents,
            }],
        })
    }
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

fn build_anthropic_prompt(bundle: &LlmBundle, file_name: &str, language_hint: &str) -> String {
    let bundle_json = serde_json::to_string_pretty(bundle).unwrap_or_else(|_| "{}".to_string());
    format!(
        "Generate one {language_hint} UI file based on the provided bundle JSON.\nTarget file: {file_name}\nIntent: {intent}\n\nRules:\n- Return source code only, no markdown fences.\n- Preserve the target technology and naming conventions.\n- Use semantic structure with clear hierarchy.\n\nBundle JSON:\n{bundle_json}",
        intent = bundle.request.intent
    )
}

fn strip_markdown_fences(value: &str) -> String {
    let trimmed = value.trim();
    if !trimmed.starts_with("```") {
        return trimmed.to_string();
    }

    let mut lines = trimmed.lines();
    let _ = lines.next();
    let mut remaining = lines.collect::<Vec<_>>();
    if remaining.last().map(|line| line.trim()) == Some("```") {
        let _ = remaining.pop();
    }
    remaining.join("\n").trim().to_string()
}
