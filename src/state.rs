use crate::config::ExtendLayout;
use hyprland::data::Monitor;
use iced::Event;
use iced_layershell::to_layer_message;

#[derive(Debug, Clone)]
pub enum State {
    Loading,
    Loaded {
        monitors: Vec<Monitor>,
        selected_index: usize,
    },
    MonitorSettings {
        monitors: Vec<Monitor>,
        settings: MonitorSettingsData,
        selected_field: usize,
    },
    Error {
        message: String,
    },
}

#[to_layer_message]
#[derive(Debug, Clone)]
pub enum Message {
    LoadMonitors,
    MonitorsLoaded(Result<Vec<Monitor>, String>),
    SetMode(DisplayMode),
    OpenExtendSettings,
    UpdatePrimaryResolution(String),
    UpdatePrimaryRotation(String),
    UpdateSecondaryResolution(String),
    UpdateSecondaryRotation(String),
    UpdateLayout(ExtendLayout),
    UpdatePrimaryMonitor(String),
    ApplyExtendSettings,
    BackToMain,
    Cancel,
    ResetToDefaults,
    IcedEvent(Event),
    // Navigation messages
    NavigateUp,
    NavigateDown,
    NavigateLeft,
    NavigateRight,
    SelectCurrent,
}

#[derive(Debug, Clone)]
pub enum DisplayMode {
    Mirror,
    Extend,
    MainScreenOnly,
    SecondScreenOnly,
}

#[derive(Debug, Clone)]
pub struct MonitorSettingsData {
    pub primary_monitor: String,
    pub primary_resolution: String,
    pub primary_rotation: String,
    pub secondary_resolution: String,
    pub secondary_rotation: String,
    pub layout: ExtendLayout,
    pub primary_available_resolutions: Vec<String>,
    pub secondary_available_resolutions: Vec<String>,
    pub available_monitors: Vec<String>,
}

impl Default for MonitorSettingsData {
    fn default() -> Self {
        Self {
            primary_monitor: "".to_string(),
            primary_resolution: "auto".to_string(),
            primary_rotation: "normal".to_string(),
            secondary_resolution: "1920x1080".to_string(),
            secondary_rotation: "normal".to_string(),
            layout: ExtendLayout::LeftToRight,
            primary_available_resolutions: vec!["auto".to_string()],
            secondary_available_resolutions: vec!["1920x1080".to_string()],
            available_monitors: vec![],
        }
    }
}
