use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "cmd")]
pub enum Command {
    // File Operations
    Read {
        path: String,
    },
    Write {
        path: String,
        content: String,
    },
    Edit {
        path: String,
        pattern: String,
        replacement: String,
    },
    Delete {
        path: String,
    },

    // System Operations
    Exec {
        command: String,
        args: Vec<String>,
    },
    List {
        path: String,
        pattern: Option<String>,
    },
    Search {
        pattern: String,
        path: Option<String>,
        file_type: Option<String>,
    },

    // Planning & Context
    Think {
        reasoning: String,
    },
    Plan {
        tasks: Vec<String>,
    },
    UpdatePlan {
        plan_id: String,
        task_id: String,
        status: TaskStatus,
    },
    Remember {
        key: String,
        value: String,
    },
    Recall {
        key: String,
    },

    // External Resources
    WebFetch {
        url: String,
        extract: Option<String>,
    },
    Parse {
        content: String,
        format: DataFormat,
    },

    // Reporting
    Status {
        message: String,
        level: StatusLevel,
    },
    Report {
        title: String,
        sections: Vec<Section>,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CommandBatch {
    pub commands: Vec<Command>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum TaskStatus {
    #[serde(rename = "pending")]
    Pending,
    #[serde(rename = "in_progress")]
    InProgress,
    #[serde(rename = "completed")]
    Completed,
    #[serde(rename = "failed")]
    Failed,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum DataFormat {
    #[serde(rename = "json")]
    Json,
    #[serde(rename = "yaml")]
    Yaml,
    #[serde(rename = "toml")]
    Toml,
    #[serde(rename = "xml")]
    Xml,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum StatusLevel {
    #[serde(rename = "info")]
    Info,
    #[serde(rename = "warning")]
    Warning,
    #[serde(rename = "error")]
    Error,
    #[serde(rename = "success")]
    Success,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Section {
    pub title: String,
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileInfo {
    pub path: String,
    pub is_dir: bool,
    pub size: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchMatch {
    pub file: String,
    pub line: usize,
    pub content: String,
}

#[derive(Debug, Serialize)]
pub enum CommandResult {
    Success(String),
    Error(String),
    FileList(Vec<FileInfo>),
    SearchResults(Vec<SearchMatch>),
    Value(serde_json::Value),
    None,
}

pub struct CommandExecutor {
    memory: HashMap<String, String>,
}

impl CommandExecutor {
    pub fn new() -> Self {
        Self {
            memory: HashMap::new(),
        }
    }

    pub async fn execute(&mut self, command: Command) -> CommandResult {
        match command {
            Command::Read { path } => self.execute_read(path).await,
            Command::Write { path, content } => self.execute_write(path, content).await,
            Command::Edit {
                path,
                pattern,
                replacement,
            } => self.execute_edit(path, pattern, replacement).await,
            Command::Delete { path } => self.execute_delete(path).await,
            Command::Exec { command, args } => self.execute_exec(command, args).await,
            Command::List { path, pattern } => self.execute_list(path, pattern).await,
            Command::Search {
                pattern,
                path,
                file_type,
            } => self.execute_search(pattern, path, file_type).await,
            Command::Think { reasoning } => self.execute_think(reasoning).await,
            Command::Plan { tasks } => self.execute_plan(tasks).await,
            Command::UpdatePlan {
                plan_id,
                task_id,
                status,
            } => self.execute_update_plan(plan_id, task_id, status).await,
            Command::Remember { key, value } => self.execute_remember(key, value).await,
            Command::Recall { key } => self.execute_recall(key).await,
            Command::WebFetch { url, extract } => self.execute_web_fetch(url, extract).await,
            Command::Parse { content, format } => self.execute_parse(content, format).await,
            Command::Status { message, level } => self.execute_status(message, level).await,
            Command::Report { title, sections } => self.execute_report(title, sections).await,
        }
    }

    async fn execute_read(&self, path: String) -> CommandResult {
        match tokio::fs::read_to_string(&path).await {
            Ok(content) => CommandResult::Success(content),
            Err(e) => CommandResult::Error(format!("Failed to read {}: {}", path, e)),
        }
    }

    async fn execute_write(&self, path: String, content: String) -> CommandResult {
        match tokio::fs::write(&path, content).await {
            Ok(_) => CommandResult::Success(format!("Successfully wrote to {}", path)),
            Err(e) => CommandResult::Error(format!("Failed to write to {}: {}", path, e)),
        }
    }

    async fn execute_edit(
        &self,
        path: String,
        pattern: String,
        replacement: String,
    ) -> CommandResult {
        match tokio::fs::read_to_string(&path).await {
            Ok(content) => {
                let new_content = content.replace(&pattern, &replacement);
                match tokio::fs::write(&path, new_content).await {
                    Ok(_) => CommandResult::Success(format!("Successfully edited {}", path)),
                    Err(e) => CommandResult::Error(format!("Failed to write to {}: {}", path, e)),
                }
            }
            Err(e) => CommandResult::Error(format!("Failed to read {}: {}", path, e)),
        }
    }

    async fn execute_delete(&self, path: String) -> CommandResult {
        match tokio::fs::remove_file(&path).await {
            Ok(_) => CommandResult::Success(format!("Successfully deleted {}", path)),
            Err(e) => CommandResult::Error(format!("Failed to delete {}: {}", path, e)),
        }
    }

    async fn execute_exec(&self, command: String, args: Vec<String>) -> CommandResult {
        use tokio::process::Command;

        match Command::new(&command).args(&args).output().await {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);

                if output.status.success() {
                    CommandResult::Success(stdout.to_string())
                } else {
                    CommandResult::Error(format!("Command failed: {}", stderr))
                }
            }
            Err(e) => CommandResult::Error(format!("Failed to execute command: {}", e)),
        }
    }

    async fn execute_list(&self, path: String, pattern: Option<String>) -> CommandResult {
        use tokio::fs;

        let mut files = Vec::new();

        match fs::read_dir(&path).await {
            Ok(mut entries) => {
                while let Ok(Some(entry)) = entries.next_entry().await {
                    let path = entry.path();
                    let path_str = path.to_string_lossy().to_string();

                    // Apply pattern filter if provided
                    if let Some(ref pat) = pattern {
                        if !path_str.contains(pat)
                            && !path
                                .file_name()
                                .map(|n| n.to_string_lossy().contains(pat))
                                .unwrap_or(false)
                        {
                            continue;
                        }
                    }

                    if let Ok(metadata) = entry.metadata().await {
                        files.push(FileInfo {
                            path: path_str,
                            is_dir: metadata.is_dir(),
                            size: metadata.len(),
                        });
                    }
                }
                CommandResult::FileList(files)
            }
            Err(e) => CommandResult::Error(format!("Failed to list directory {}: {}", path, e)),
        }
    }

    async fn execute_search(
        &self,
        pattern: String,
        path: Option<String>,
        file_type: Option<String>,
    ) -> CommandResult {
        use tokio::process::Command;

        let mut cmd = Command::new("rg");
        cmd.arg(&pattern);

        if let Some(p) = path {
            cmd.arg(&p);
        }

        if let Some(ft) = file_type {
            cmd.arg("-t").arg(&ft);
        }

        cmd.arg("--json");

        match cmd.output().await {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let mut matches = Vec::new();

                for line in stdout.lines() {
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(line) {
                        if json["type"] == "match" {
                            if let (Some(path), Some(line_number), Some(lines)) = (
                                json["data"]["path"]["text"].as_str(),
                                json["data"]["line_number"].as_u64(),
                                json["data"]["lines"]["text"].as_str(),
                            ) {
                                matches.push(SearchMatch {
                                    file: path.to_string(),
                                    line: line_number as usize,
                                    content: lines.trim().to_string(),
                                });
                            }
                        }
                    }
                }

                CommandResult::SearchResults(matches)
            }
            Err(e) => CommandResult::Error(format!("Search failed: {}", e)),
        }
    }

    async fn execute_think(&self, reasoning: String) -> CommandResult {
        tracing::info!("Thinking: {}", reasoning);
        CommandResult::None
    }

    async fn execute_plan(&self, tasks: Vec<String>) -> CommandResult {
        let plan_id = uuid::Uuid::new_v4().to_string();
        tracing::info!("Created plan {}: {:?}", plan_id, tasks);
        CommandResult::Success(plan_id)
    }

    async fn execute_update_plan(
        &self,
        plan_id: String,
        task_id: String,
        status: TaskStatus,
    ) -> CommandResult {
        tracing::info!("Updated plan {} task {} to {:?}", plan_id, task_id, status);
        CommandResult::None
    }

    async fn execute_remember(&mut self, key: String, value: String) -> CommandResult {
        self.memory.insert(key.clone(), value);
        CommandResult::Success(format!("Remembered: {}", key))
    }

    async fn execute_recall(&self, key: String) -> CommandResult {
        match self.memory.get(&key) {
            Some(value) => CommandResult::Success(value.clone()),
            None => CommandResult::Error(format!("No memory found for key: {}", key)),
        }
    }

    async fn execute_web_fetch(&self, url: String, extract: Option<String>) -> CommandResult {
        use reqwest;

        match reqwest::get(&url).await {
            Ok(response) => match response.text().await {
                Ok(text) => {
                    if let Some(extraction) = extract {
                        CommandResult::Success(format!("Fetched from {}: {}", url, extraction))
                    } else {
                        CommandResult::Success(text)
                    }
                }
                Err(e) => CommandResult::Error(format!("Failed to read response: {}", e)),
            },
            Err(e) => CommandResult::Error(format!("Failed to fetch {}: {}", url, e)),
        }
    }

    async fn execute_parse(&self, content: String, format: DataFormat) -> CommandResult {
        match format {
            DataFormat::Json => match serde_json::from_str::<serde_json::Value>(&content) {
                Ok(value) => CommandResult::Value(value),
                Err(e) => CommandResult::Error(format!("Failed to parse JSON: {}", e)),
            },
            DataFormat::Yaml => match serde_yaml::from_str::<serde_json::Value>(&content) {
                Ok(value) => CommandResult::Value(value),
                Err(e) => CommandResult::Error(format!("Failed to parse YAML: {}", e)),
            },
            DataFormat::Toml => match toml::from_str::<toml::Value>(&content) {
                Ok(value) => CommandResult::Success(format!("{:?}", value)),
                Err(e) => CommandResult::Error(format!("Failed to parse TOML: {}", e)),
            },
            DataFormat::Xml => CommandResult::Error("XML parsing not implemented".to_string()),
        }
    }

    async fn execute_status(&self, message: String, level: StatusLevel) -> CommandResult {
        match level {
            StatusLevel::Info => tracing::info!("{}", message),
            StatusLevel::Warning => tracing::warn!("{}", message),
            StatusLevel::Error => tracing::error!("{}", message),
            StatusLevel::Success => tracing::info!("✓ {}", message),
        }
        CommandResult::None
    }

    async fn execute_report(&self, title: String, sections: Vec<Section>) -> CommandResult {
        let mut report = format!("# {}\n\n", title);

        for section in sections {
            report.push_str(&format!("## {}\n\n{}\n\n", section.title, section.content));
        }

        tracing::info!("Report generated:\n{}", report);
        CommandResult::Success(report)
    }
}

pub fn parse_commands(response: &str) -> Result<CommandBatch, serde_json::Error> {
    serde_json::from_str(response)
}
