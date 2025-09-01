use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DisplayConfig {
    pub extend_configurations: HashMap<String, ExtendConfiguration>,
    pub last_used_extend_config: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtendConfiguration {
    pub name: String,
    pub primary_monitor: String,
    pub primary_resolution: String,
    pub primary_rotation: String,
    pub secondary_monitor: String,
    pub secondary_resolution: String,
    pub secondary_rotation: String,
    pub layout: ExtendLayout,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_used: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ExtendLayout {
    LeftToRight,
    RightToLeft,
    TopToBottom,
    BottomToTop,
}

impl std::fmt::Display for ExtendLayout {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExtendLayout::LeftToRight => write!(f, "Left to Right"),
            ExtendLayout::RightToLeft => write!(f, "Right to Left"),
            ExtendLayout::TopToBottom => write!(f, "Top to Bottom"),
            ExtendLayout::BottomToTop => write!(f, "Bottom to Top"),
        }
    }
}

pub struct ConfigManager {
    config_path: PathBuf,
    config: DisplayConfig,
}

impl Default for ConfigManager {
    fn default() -> Self {
        Self {
            config_path: PathBuf::from("config.json"),
            config: DisplayConfig::default(),
        }
    }
}

impl ConfigManager {
    pub fn new() -> anyhow::Result<Self> {
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("hyprland-display-switcher");

        fs::create_dir_all(&config_dir)?;

        let config_path = config_dir.join("config.json");

        let config = if config_path.exists() {
            let content = fs::read_to_string(&config_path)?;
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            DisplayConfig::default()
        };

        Ok(Self {
            config_path,
            config,
        })
    }

    pub fn save_extend_configuration(&mut self, config: ExtendConfiguration) -> anyhow::Result<()> {
        let key = self.generate_config_key(&config);

        let mut updated_config = config.clone();
        updated_config.last_used = chrono::Utc::now();

        self.config
            .extend_configurations
            .insert(key.clone(), updated_config);
        self.config.last_used_extend_config = Some(key);

        self.save_to_disk()
    }

    pub fn get_last_extend_configuration(&self) -> Option<&ExtendConfiguration> {
        self.config
            .last_used_extend_config
            .as_ref()
            .and_then(|key| self.config.extend_configurations.get(key))
    }

    pub fn get_extend_configurations(&self) -> &HashMap<String, ExtendConfiguration> {
        &self.config.extend_configurations
    }

    pub fn get_extend_configuration_by_monitors(
        &self,
        primary: &str,
        secondary: &str,
    ) -> Option<&ExtendConfiguration> {
        self.config.extend_configurations.values().find(|config| {
            (config.primary_monitor == primary && config.secondary_monitor == secondary)
                || (config.primary_monitor == secondary && config.secondary_monitor == primary)
        })
    }

    pub fn get_extend_configuration_for_monitors(
        &self,
        available_monitors: &[String],
    ) -> Option<&ExtendConfiguration> {
        // Find the most recent configuration that uses any of the available monitors
        let mut matching_configs: Vec<_> = self
            .config
            .extend_configurations
            .values()
            .filter(|config| {
                available_monitors.contains(&config.primary_monitor)
                    && available_monitors.contains(&config.secondary_monitor)
            })
            .collect();

        // Sort by last_used date (most recent first)
        matching_configs.sort_by(|a, b| b.last_used.cmp(&a.last_used));
        matching_configs.first().copied()
    }

    fn generate_config_key(&self, config: &ExtendConfiguration) -> String {
        format!("{}_{}", config.primary_monitor, config.secondary_monitor)
    }

    fn save_to_disk(&self) -> anyhow::Result<()> {
        let content = serde_json::to_string_pretty(&self.config)?;
        fs::write(&self.config_path, content)?;
        Ok(())
    }

    pub fn get_preferred_primary_monitor(&self, available_monitors: &[String]) -> Option<String> {
        if let Some(last_config) = self.get_extend_configuration_for_monitors(available_monitors) {
            if available_monitors.contains(&last_config.primary_monitor) {
                return Some(last_config.primary_monitor.clone());
            }
        }

        None
    }

    pub fn create_config_from_settings(
        primary_monitor: String,
        secondary_monitor: String,
        primary_resolution: String,
        primary_rotation: String,
        secondary_resolution: String,
        secondary_rotation: String,
        layout: ExtendLayout,
    ) -> ExtendConfiguration {
        let now = chrono::Utc::now();
        ExtendConfiguration {
            name: format!("{primary_monitor} + {secondary_monitor}"),
            primary_monitor,
            primary_resolution,
            primary_rotation,
            secondary_monitor,
            secondary_resolution,
            secondary_rotation,
            layout,
            created_at: now,
            last_used: now,
        }
    }
}
