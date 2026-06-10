#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tracing::{error, info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub connections: Vec<ConnectionProfile>,
    #[serde(default)]
    pub keybindings: HashMap<String, String>,
}

const KEYCHAIN_MARKER: &str = "<keychain>";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionProfile {
    pub name: String,
    pub db_type: String,
    #[serde(default)]
    pub host: String,
    #[serde(default)]
    pub port: u16,
    #[serde(default)]
    pub database: String,
    #[serde(default)]
    pub user: String,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub password: Option<String>,
}

impl ConnectionProfile {
    pub fn keychain_service(&self) -> String {
        format!("twisterDBA/{}", self.name)
    }

    pub fn password_is_keychain(&self) -> bool {
        self.password.as_deref() == Some(KEYCHAIN_MARKER)
    }

    pub fn store_password(&self, password: &str) -> Result<(), String> {
        let entry = keyring::Entry::new(&self.keychain_service(), &self.user).map_err(|e| {
            warn!("Keychain unavailable: {e}");
            format!("Keychain error: {e}")
        })?;
        entry.set_password(password).map_err(|e| {
            warn!("Failed to store password in keychain: {e}");
            format!("Failed to store password: {e}")
        })?;
        info!("Password stored in keychain for profile '{}'", self.name);
        Ok(())
    }

    pub fn get_password(&self) -> Result<String, String> {
        let entry = keyring::Entry::new(&self.keychain_service(), &self.user).map_err(|e| {
            warn!("Keychain unavailable: {e}");
            format!("Keychain error: {e}")
        })?;
        entry.get_password().map_err(|e| {
            warn!("Failed to retrieve password from keychain for '{}': {e}", self.name);
            format!("Failed to retrieve password: {e}")
        })
    }

    pub fn delete_password(&self) -> Result<(), String> {
        let entry = keyring::Entry::new(&self.keychain_service(), &self.user).map_err(|e| {
            warn!("Keychain unavailable: {e}");
            format!("Keychain error: {e}")
        })?;
        entry.delete_credential().map_err(|e| {
            warn!("Failed to delete password from keychain for '{}': {e}", self.name);
            format!("Failed to delete password: {e}")
        })?;
        info!("Password deleted from keychain for profile '{}'", self.name);
        Ok(())
    }
}

impl Config {
    pub fn load() -> Self {
        let config_path = Self::config_path();

        if !config_path.exists() {
            info!("No config file found, creating default at {:?}", config_path);
            let default = Self::default();
            if let Err(e) = default.save() {
                error!("Failed to write default config: {e}");
            }
            return default;
        }

        match fs::read_to_string(&config_path) {
            Ok(content) => match toml::from_str(&content) {
                Ok(config) => {
                    info!("Config loaded from {:?}", config_path);
                    config
                },
                Err(e) => {
                    error!("Failed to parse config: {e}");
                    Self::default()
                },
            },
            Err(e) => {
                error!("Failed to read config file: {e}");
                Self::default()
            },
        }
    }

    pub fn default() -> Self {
        Self {
            connections: vec![ConnectionProfile {
                name: "local-postgres".to_string(),
                db_type: "postgres".to_string(),
                host: "localhost".to_string(),
                port: 5432,
                database: String::new(),
                user: String::new(),
                password: None,
            }],
            keybindings: {
                let mut map = HashMap::new();
                map.insert("execute_query".to_string(), "Ctrl+E".to_string());
                map.insert("cancel_query".to_string(), "Ctrl+C".to_string());
                map
            },
        }
    }

    pub fn save(&self) -> Result<(), String> {
        let config_path = Self::config_path();

        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("Failed to create config dir: {e}"))?;
        }

        let sanitized = Self {
            connections: self
                .connections
                .iter()
                .map(|p| {
                    let mut p = p.clone();
                    if p.password.as_deref() != Some(KEYCHAIN_MARKER) {
                        p.password = None;
                    }
                    p
                })
                .collect(),
            keybindings: self.keybindings.clone(),
        };

        let toml_str =
            toml::to_string_pretty(&sanitized).map_err(|e| format!("Failed to serialize: {e}"))?;

        let content = format!(
            "# twisterDBA configuration\n# See https://github.com/... for docs\n\n{toml_str}"
        );

        fs::write(&config_path, content).map_err(|e| format!("Failed to write config: {e}"))?;

        info!("Config saved to {:?}", config_path);
        Ok(())
    }

    pub fn add_profile(&mut self, profile: ConnectionProfile) {
        if self.connections.iter().any(|p| p.name == profile.name) {
            return;
        }
        self.connections.push(profile);
        if let Err(e) = self.save() {
            error!("Failed to save config after adding profile: {e}");
        }
    }

    pub fn upsert_profile(&mut self, profile: ConnectionProfile) {
        if let Some(existing) = self.connections.iter_mut().find(|p| p.name == profile.name) {
            *existing = profile;
        } else {
            self.connections.push(profile);
        }
        if let Err(e) = self.save() {
            error!("Failed to save config after upserting profile: {e}");
        }
    }

    pub fn mark_profile_keychain(&mut self, profile_name: &str) {
        if let Some(profile) = self.connections.iter_mut().find(|p| p.name == profile_name) {
            profile.password = Some(KEYCHAIN_MARKER.to_string());
            if let Err(e) = self.save() {
                error!("Failed to save config after marking keychain: {e}");
            }
        }
    }

    pub fn clear_profile_keychain(&mut self, profile_name: &str) {
        if let Some(profile) = self.connections.iter_mut().find(|p| p.name == profile_name) {
            profile.password = None;
            if let Err(e) = self.save() {
                error!("Failed to save config after clearing keychain: {e}");
            }
        }
    }

    fn config_path() -> PathBuf {
        let base = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        base.join("twisterDBA").join("config.toml")
    }
}
