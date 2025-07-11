use crate::config::Config as AppConfig;
use anyhow::{anyhow, Result};
use bollard::{
    container::{Config, CreateContainerOptions, RemoveContainerOptions, StartContainerOptions},
    Docker,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tonic::transport::Channel;
use tracing::{info, warn};

pub mod spells {
    tonic::include_proto!("spells");
}

use spells::apprentice_client::ApprenticeClient;
use spells::{ChatHistoryRequest, SpellRequest, StatusRequest};

pub struct Apprentice {
    pub _name: String,
    pub container_id: String,
    pub _port: u16,
    pub client: Option<ApprenticeClient<Channel>>,
}

pub struct Sorcerer {
    docker: Docker,
    apprentices: Arc<Mutex<HashMap<String, Apprentice>>>,
    next_port: Arc<Mutex<u16>>,
    config: AppConfig,
}

impl Sorcerer {
    fn is_valid_apprentice_name(name: &str) -> bool {
        !name.is_empty()
            && name.len() <= 32
            && name
                .chars()
                .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
    }

    async fn connect_to_container_runtime() -> Result<Docker> {
        // Try Podman socket first (rootless)
        if let Ok(socket_path) = std::env::var("XDG_RUNTIME_DIR") {
            let podman_socket = format!("unix://{socket_path}/podman/podman.sock");
            if let Ok(docker) =
                Docker::connect_with_socket(&podman_socket, 120, bollard::API_DEFAULT_VERSION)
            {
                match docker.ping().await {
                    Ok(_) => {
                        info!("Connected to Podman (rootless)");
                        return Ok(docker);
                    }
                    Err(_) => info!("Podman socket found but not responding"),
                }
            }
        }

        // Try system Podman socket
        let system_podman_socket = "unix:///run/podman/podman.sock";
        if let Ok(docker) =
            Docker::connect_with_socket(system_podman_socket, 120, bollard::API_DEFAULT_VERSION)
        {
            match docker.ping().await {
                Ok(_) => {
                    info!("Connected to Podman (system)");
                    return Ok(docker);
                }
                Err(_) => info!("System Podman socket found but not responding"),
            }
        }

        // Fall back to Docker
        match Docker::connect_with_local_defaults() {
            Ok(docker) => match docker.ping().await {
                Ok(_) => {
                    info!("Connected to Docker");
                    Ok(docker)
                }
                Err(e) => Err(anyhow!("Cannot reach Docker daemon. Make sure Docker is running.\n  Error: {}", e)),
            },
            Err(e) => Err(anyhow!("Failed to connect to any container runtime (Podman or Docker).\n  \
                                    Please install and start either Podman or Docker.\n  \
                                    For Podman: sudo pacman -S podman && systemctl --user start podman.socket\n  \
                                    For Docker: sudo pacman -S docker && sudo systemctl start docker\n  \
                                    Error: {}", e)),
        }
    }

    pub async fn new() -> Result<Self> {
        let docker = Self::connect_to_container_runtime().await?;
        let config = AppConfig::default();
        let starting_port = config.starting_port;

        let mut sorcerer = Self {
            docker,
            apprentices: Arc::new(Mutex::new(HashMap::new())),
            next_port: Arc::new(Mutex::new(starting_port)),
            config,
        };

        // Discover existing apprentice containers
        sorcerer.discover_apprentices().await?;

        Ok(sorcerer)
    }

    async fn discover_apprentices(&mut self) -> Result<()> {
        use bollard::container::ListContainersOptions;

        let mut filters = HashMap::new();
        filters.insert("name".to_string(), vec!["apprentice-".to_string()]);

        let options = Some(ListContainersOptions {
            all: true,
            filters,
            ..Default::default()
        });

        let containers = self.docker.list_containers(options).await?;
        let mut apprentices = self.apprentices.lock().await;
        let mut next_port = self.next_port.lock().await;

        for container in containers {
            if let Some(names) = &container.names {
                for name in names {
                    if name.starts_with("/apprentice-") {
                        let apprentice_name = name.strip_prefix("/apprentice-").unwrap();

                        // Get port from container inspect (we'll need to inspect each container)
                        let port = if let Ok(container_info) = self
                            .docker
                            .inspect_container(&container.id.clone().unwrap_or_default(), None)
                            .await
                        {
                            if let Some(config) = container_info.config {
                                if let Some(env) = config.env {
                                    env.iter()
                                        .find(|e| e.starts_with("GRPC_PORT="))
                                        .and_then(|e| e.strip_prefix("GRPC_PORT="))
                                        .and_then(|p| p.parse::<u16>().ok())
                                        .unwrap_or(50051)
                                } else {
                                    50051
                                }
                            } else {
                                50051
                            }
                        } else {
                            50051
                        };

                        // Update next_port to avoid conflicts
                        if port >= *next_port {
                            *next_port = port + 1;
                        }

                        // Try to connect to the apprentice if it's running
                        let mut client = None;
                        if let Some(state) = &container.state {
                            if state == "running" {
                                let addr = format!("http://127.0.0.1:{port}");
                                if let Ok(c) = ApprenticeClient::connect(addr).await {
                                    client = Some(c);
                                }
                            }
                        }

                        apprentices.insert(
                            apprentice_name.to_string(),
                            Apprentice {
                                _name: apprentice_name.to_string(),
                                container_id: container.id.clone().unwrap_or_default(),
                                _port: port,
                                client,
                            },
                        );

                        info!(
                            "Discovered apprentice: {} (port: {})",
                            apprentice_name, port
                        );
                    }
                }
            }
        }

        Ok(())
    }

    pub async fn summon_apprentice(&mut self, name: &str) -> Result<()> {
        // Validate apprentice name
        if !Self::is_valid_apprentice_name(name) {
            return Err(anyhow!(
                "Invalid apprentice name. Names must be 1-32 characters, alphanumeric with hyphens/underscores only"
            ));
        }

        let mut apprentices = self.apprentices.lock().await;

        if apprentices.contains_key(name) {
            return Err(anyhow!("Apprentice {} already exists", name));
        }

        let port = {
            let mut next_port = self.next_port.lock().await;
            let port = *next_port;
            *next_port += 1;
            port
        };

        info!("Summoning apprentice {} on port {}", name, port);

        // Get API key from environment
        let api_key = std::env::var("ANTHROPIC_API_KEY")?;

        // Create container
        let config = Config {
            image: Some(self.config.image_name.clone()),
            env: Some(vec![
                format!("APPRENTICE_NAME={}", name),
                format!("GRPC_PORT={}", port),
                format!("ANTHROPIC_API_KEY={}", api_key),
            ]),
            exposed_ports: Some(HashMap::from([("50051/tcp".to_string(), HashMap::new())])),
            host_config: Some(bollard::models::HostConfig {
                network_mode: Some("host".to_string()),
                ..Default::default()
            }),
            ..Default::default()
        };

        let container = self
            .docker
            .create_container(
                Some(CreateContainerOptions {
                    name: format!("apprentice-{name}"),
                    ..Default::default()
                }),
                config,
            )
            .await?;

        self.docker
            .start_container(&container.id, None::<StartContainerOptions<String>>)
            .await?;

        // Wait for container to be ready
        tokio::time::sleep(tokio::time::Duration::from_secs(
            self.config.container_ready_timeout,
        ))
        .await;

        // Connect to apprentice (using localhost since we're using host networking)
        let addr = format!("http://127.0.0.1:{port}");
        let client = ApprenticeClient::connect(addr.clone()).await?;

        apprentices.insert(
            name.to_string(),
            Apprentice {
                _name: name.to_string(),
                container_id: container.id,
                _port: port,
                client: Some(client),
            },
        );

        info!("Apprentice {} summoned successfully", name);
        Ok(())
    }

    pub async fn cast_spell(&mut self, name: &str, incantation: &str) -> Result<String> {
        let mut apprentices = self.apprentices.lock().await;
        let apprentice = apprentices
            .get_mut(name)
            .ok_or_else(|| anyhow!("Apprentice {} not found", name))?;

        let client = apprentice
            .client
            .as_mut()
            .ok_or_else(|| anyhow!("Apprentice {} is not connected", name))?;

        let request = tonic::Request::new(SpellRequest {
            incantation: incantation.to_string(),
            spell_id: uuid::Uuid::new_v4().to_string(),
        });

        let response = client.cast_spell(request).await?;
        let spell_response = response.into_inner();

        if spell_response.success {
            Ok(spell_response.result)
        } else {
            Err(anyhow!("Tell failed: {}", spell_response.error))
        }
    }

    pub async fn list_apprentices(&self) -> Result<Vec<String>> {
        let apprentices = self.apprentices.lock().await;
        Ok(apprentices
            .iter()
            .filter(|(_, apprentice)| apprentice.client.is_some())
            .map(|(name, _)| name.clone())
            .collect())
    }

    pub async fn banish_apprentice(&mut self, name: &str) -> Result<()> {
        let mut apprentices = self.apprentices.lock().await;
        let apprentice = apprentices
            .remove(name)
            .ok_or_else(|| anyhow!("Apprentice {} not found", name))?;

        // Try to gracefully shut down via gRPC first
        if let Some(mut client) = apprentice.client {
            let _ = client
                .banish(tonic::Request::new(spells::BanishRequest {
                    reason: "Sorcerer's command".to_string(),
                }))
                .await;
        }

        // Stop and remove container
        if let Err(e) = self
            .docker
            .stop_container(&apprentice.container_id, None)
            .await
        {
            warn!("Failed to stop container gracefully: {}", e);
        }

        self.docker
            .remove_container(
                &apprentice.container_id,
                Some(RemoveContainerOptions {
                    force: true,
                    ..Default::default()
                }),
            )
            .await?;

        info!("Apprentice {} has been banished", name);
        Ok(())
    }

    pub async fn get_all_status(&mut self) -> Result<HashMap<String, spells::StatusResponse>> {
        let mut results = HashMap::new();
        let mut apprentices = self.apprentices.lock().await;

        for (name, apprentice) in apprentices.iter_mut() {
            if let Some(client) = &mut apprentice.client {
                match client
                    .get_status(tonic::Request::new(StatusRequest {}))
                    .await
                {
                    Ok(response) => {
                        results.insert(name.clone(), response.into_inner());
                    }
                    Err(e) => {
                        warn!("Failed to get status for {}: {}", name, e);
                    }
                }
            }
        }

        Ok(results)
    }

    pub async fn get_chat_history(&mut self, name: &str, lines: usize) -> Result<Vec<String>> {
        let mut apprentices = self.apprentices.lock().await;
        let apprentice = apprentices
            .get_mut(name)
            .ok_or_else(|| anyhow!("Apprentice {} not found", name))?;

        let client = apprentice
            .client
            .as_mut()
            .ok_or_else(|| anyhow!("Apprentice {} is not connected", name))?;

        let request = tonic::Request::new(ChatHistoryRequest {
            lines: lines as i32,
        });

        let response = client.get_chat_history(request).await?;
        let chat_response = response.into_inner();

        Ok(chat_response.history)
    }
}
