use crate::claude::ClaudeClient;
use crate::commands::{parse_commands, CommandBatch, CommandExecutor, CommandResult};
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::Mutex;
use tonic::{Request, Response, Status};
use tracing::{error, info, warn};

pub mod spells {
    tonic::include_proto!("spells");
}

use spells::apprentice_server::Apprentice;
use spells::{
    ChatHistoryRequest, ChatHistoryResponse, KillRequest, KillResponse, SpellRequest,
    SpellResponse, StatusRequest, StatusResponse,
};

#[derive(Debug, Clone)]
pub struct ApprenticeState {
    name: String,
    state: String,
    spells_cast: i32,
    last_spell_time: Option<String>,
    chat_history: Vec<String>,
    agent_mode: bool,
    system_prompt: Option<String>,
}

pub struct ApprenticeServer {
    state: Arc<Mutex<ApprenticeState>>,
    claude_client: Arc<ClaudeClient>,
    command_executor: Arc<Mutex<CommandExecutor>>,
}

impl ApprenticeServer {
    pub fn new(name: String) -> Self {
        // Check if agent mode is enabled
        let agent_mode = std::env::var("AGENT_MODE")
            .unwrap_or_else(|_| "false".to_string())
            .parse::<bool>()
            .unwrap_or(false);

        // Load system prompt if provided
        let system_prompt = if agent_mode {
            std::env::var("SYSTEM_PROMPT_PATH")
                .ok()
                .and_then(|path| std::fs::read_to_string(path).ok())
                .or_else(|| {
                    // Use default agent prompt
                    Some(include_str!("../prompts/agent_template.md").to_string())
                })
        } else {
            None
        };

        let state = Arc::new(Mutex::new(ApprenticeState {
            name: name.clone(),
            state: "idle".to_string(),
            spells_cast: 0,
            last_spell_time: None,
            chat_history: Vec::new(),
            agent_mode,
            system_prompt,
        }));

        let claude_client = Arc::new(ClaudeClient::new());
        let command_executor = Arc::new(Mutex::new(CommandExecutor::new()));

        Self {
            state,
            claude_client,
            command_executor,
        }
    }

    async fn handle_agent_response(&self, response: &str) -> Result<String> {
        // Try to parse as command batch
        match parse_commands(response) {
            Ok(command_batch) => {
                let mut results = Vec::new();
                let mut executor = self.command_executor.lock().await;

                for command in command_batch.commands {
                    info!("Executing command: {:?}", command);
                    let result = executor.execute(command).await;

                    match &result {
                        CommandResult::Success(msg) => results.push(format!("✓ {}", msg)),
                        CommandResult::Error(msg) => results.push(format!("✗ {}", msg)),
                        CommandResult::FileList(files) => {
                            results.push(format!("Found {} files", files.len()));
                            for file in files.iter().take(10) {
                                results.push(format!(
                                    "  {} {}",
                                    if file.is_dir { "📁" } else { "📄" },
                                    file.path
                                ));
                            }
                            if files.len() > 10 {
                                results.push(format!("  ... and {} more", files.len() - 10));
                            }
                        }
                        CommandResult::SearchResults(matches) => {
                            results.push(format!("Found {} matches", matches.len()));
                            for m in matches.iter().take(5) {
                                results.push(format!("  {}:{} - {}", m.file, m.line, m.content));
                            }
                            if matches.len() > 5 {
                                results.push(format!("  ... and {} more", matches.len() - 5));
                            }
                        }
                        CommandResult::Value(val) => results.push(format!("Parsed: {:?}", val)),
                        CommandResult::None => {}
                    }
                }

                Ok(results.join("\n"))
            }
            Err(_) => {
                // If not valid JSON commands, treat as regular response
                warn!("Response was not valid command JSON, treating as regular text");
                Ok(response.to_string())
            }
        }
    }
}

#[tonic::async_trait]
impl Apprentice for ApprenticeServer {
    async fn cast_spell(
        &self,
        request: Request<SpellRequest>,
    ) -> Result<Response<SpellResponse>, Status> {
        let spell = request.into_inner();
        info!("Casting spell {}: {}", spell.spell_id, spell.incantation);

        {
            let mut state = self.state.lock().await;
            state.state = "casting".to_string();
        }

        // Get the current conversation history and agent mode before sending the message
        let (conversation_history, agent_mode, system_prompt) = {
            let state = self.state.lock().await;
            (
                state.chat_history.clone(),
                state.agent_mode,
                state.system_prompt.clone(),
            )
        };

        let result = match self
            .claude_client
            .send_message_with_system(
                &spell.incantation,
                &conversation_history,
                system_prompt.as_deref(),
            )
            .await
        {
            Ok(response) => {
                let final_response = if agent_mode {
                    // Parse and execute commands
                    match self.handle_agent_response(&response).await {
                        Ok(execution_result) => execution_result,
                        Err(e) => format!("Error executing commands: {}", e),
                    }
                } else {
                    response.clone()
                };

                let mut state = self.state.lock().await;
                state.state = "idle".to_string();
                state.spells_cast += 1;
                state.last_spell_time = Some(chrono::Utc::now().to_rfc3339());

                // Add to chat history
                let apprentice_name = state.name.clone();
                state
                    .chat_history
                    .push(format!("Sorcerer: {}", spell.incantation));
                state
                    .chat_history
                    .push(format!("{}: {}", apprentice_name, response));

                // Keep only last 50 exchanges (100 lines)
                if state.chat_history.len() > 100 {
                    let len = state.chat_history.len();
                    state.chat_history.drain(0..len - 100);
                }

                SpellResponse {
                    spell_id: spell.spell_id,
                    result: final_response,
                    success: true,
                    error: String::new(),
                }
            }
            Err(e) => {
                error!("Spell casting failed: {}", e);
                let mut state = self.state.lock().await;
                state.state = "error".to_string();

                SpellResponse {
                    spell_id: spell.spell_id,
                    result: String::new(),
                    success: false,
                    error: e.to_string(),
                }
            }
        };

        Ok(Response::new(result))
    }

    async fn get_status(
        &self,
        _request: Request<StatusRequest>,
    ) -> Result<Response<StatusResponse>, Status> {
        let state = self.state.lock().await;

        Ok(Response::new(StatusResponse {
            apprentice_name: state.name.clone(),
            state: state.state.clone(),
            last_spell_time: state.last_spell_time.clone().unwrap_or_default(),
        }))
    }

    async fn get_chat_history(
        &self,
        request: Request<ChatHistoryRequest>,
    ) -> Result<Response<ChatHistoryResponse>, Status> {
        let lines = request.into_inner().lines as usize;
        let state = self.state.lock().await;

        // Get the last n lines
        let history = if lines == 0 {
            state.chat_history.clone()
        } else {
            let start = if state.chat_history.len() > lines {
                state.chat_history.len() - lines
            } else {
                0
            };
            state.chat_history[start..].to_vec()
        };

        Ok(Response::new(ChatHistoryResponse { history }))
    }

    async fn kill(&self, request: Request<KillRequest>) -> Result<Response<KillResponse>, Status> {
        let reason = request.into_inner().reason;
        info!("Apprentice being killed: {}", reason);

        tokio::spawn(async {
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            std::process::exit(0);
        });

        Ok(Response::new(KillResponse {
            success: true,
            message: format!("Fading away into the ether... ({})", reason),
        }))
    }
}
