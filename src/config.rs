use std::env;

pub struct Config {
    pub image_name: String,
    pub starting_port: u16,
    pub container_ready_timeout: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            image_name: env::var("SORCERER_IMAGE")
                .unwrap_or_else(|_| "sorcerer-agent:latest".to_string()),
            starting_port: env::var("SORCERER_STARTING_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(50100),
            container_ready_timeout: env::var("SORCERER_CONTAINER_TIMEOUT")
                .ok()
                .and_then(|t| t.parse().ok())
                .unwrap_or(2),
        }
    }
}
