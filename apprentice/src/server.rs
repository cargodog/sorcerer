use crate::claude::ClaudeClient;
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::Mutex;
use tonic::{Request, Response, Status};
use tracing::{error, info};

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
}

pub struct ApprenticeServer {
    state: Arc<Mutex<ApprenticeState>>,
    claude_client: Arc<ClaudeClient>,
}

impl ApprenticeServer {
    pub fn new(name: String) -> Self {
        let state = Arc::new(Mutex::new(ApprenticeState {
            name: name.clone(),
            state: "idle".to_string(),
            spells_cast: 0,
            last_spell_time: None,
            chat_history: Vec::new(),
        }));

        let claude_client = Arc::new(ClaudeClient::new());

        Self {
            state,
            claude_client,
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

        // Get the current conversation history before sending the message
        let conversation_history = {
            let state = self.state.lock().await;
            state.chat_history.clone()
        };

        let result = match self
            .claude_client
            .send_message(&spell.incantation, &conversation_history)
            .await
        {
            Ok(response) => {
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
                    result: response.clone(),
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
