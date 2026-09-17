use super::settings_validation::validate_settings;
use super::StorageResult;
use crate::platform;
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf, sync::RwLock};
use tauri::{AppHandle, Manager};

const SETTINGS_FILE_NAME: &str = "settings.json";
const DEFAULT_PREVIOUS_FILTER_SHORTCUT: &str = "Ctrl+Q";
const DEFAULT_NEXT_FILTER_SHORTCUT: &str = "Ctrl+E";
const DEFAULT_PREVIOUS_CARD_SHORTCUT: &str = "Ctrl+A";
const DEFAULT_NEXT_CARD_SHORTCUT: &str = "Ctrl+D";
const DEFAULT_HISTORY_RETENTION_DAYS: u32 = 30;
const DEFAULT_MAX_HISTORY_ENTRIES: u32 = 200;
pub(super) const MAX_QUICK_TOOLS: usize = 5;
const DEFAULT_QUICK_TOOL_SHORTCUTS: [&str; MAX_QUICK_TOOLS] = [
    "Command+1",
    "Command+2",
    "Command+3",
    "Command+4",
    "Command+5",
];
const LEGACY_QUICK_TOOL_SHORTCUTS: [&str; MAX_QUICK_TOOLS] =
    ["Alt+1", "Alt+2", "Alt+3", "Alt+4", "Alt+5"];

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AppTheme {
    #[default]
    SoftGlow,
    Classic,
    Dark,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum QuickToolId {
    TextDiff,
    Json,
    Xml,
    Html,
    Url,
    Base64,
    Date,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    #[serde(default)]
    pub theme: AppTheme,
    pub main_shortcut: String,
    #[serde(default = "default_previous_filter_shortcut")]
    pub previous_filter_shortcut: String,
    #[serde(default = "default_next_filter_shortcut")]
    pub next_filter_shortcut: String,
    #[serde(default = "default_previous_card_shortcut")]
    pub previous_card_shortcut: String,
    #[serde(default = "default_next_card_shortcut")]
    pub next_card_shortcut: String,
    #[serde(default = "default_quick_tool_ids")]
    pub quick_tool_ids: Vec<QuickToolId>,
    #[serde(default = "default_quick_tool_shortcuts")]
    pub quick_tool_shortcuts: Vec<String>,
    #[serde(default)]
    pub show_format_filters: bool,
    pub history_retention_days: u32,
    pub max_history_entries: u32,
    pub record_text: bool,
    pub record_images: bool,
    pub record_files: bool,
    pub auto_paste: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            theme: AppTheme::default(),
            main_shortcut: platform::DEFAULT_MAIN_SHORTCUT.to_owned(),
            previous_filter_shortcut: default_previous_filter_shortcut(),
            next_filter_shortcut: default_next_filter_shortcut(),
            previous_card_shortcut: default_previous_card_shortcut(),
            next_card_shortcut: default_next_card_shortcut(),
            quick_tool_ids: default_quick_tool_ids(),
            quick_tool_shortcuts: default_quick_tool_shortcuts(),
            show_format_filters: false,
            history_retention_days: DEFAULT_HISTORY_RETENTION_DAYS,
            max_history_entries: DEFAULT_MAX_HISTORY_ENTRIES,
            record_text: true,
            record_images: true,
            record_files: true,
            auto_paste: true,
        }
    }
}

impl AppSettings {
    pub fn validate(&self) -> StorageResult {
        validate_settings(self)
    }
}

fn default_previous_filter_shortcut() -> String {
    DEFAULT_PREVIOUS_FILTER_SHORTCUT.to_owned()
}

fn default_next_filter_shortcut() -> String {
    DEFAULT_NEXT_FILTER_SHORTCUT.to_owned()
}

fn default_previous_card_shortcut() -> String {
    DEFAULT_PREVIOUS_CARD_SHORTCUT.to_owned()
}

fn default_next_card_shortcut() -> String {
    DEFAULT_NEXT_CARD_SHORTCUT.to_owned()
}

fn default_quick_tool_ids() -> Vec<QuickToolId> {
    vec![
        QuickToolId::TextDiff,
        QuickToolId::Json,
        QuickToolId::Url,
        QuickToolId::Base64,
        QuickToolId::Date,
    ]
}

fn default_quick_tool_shortcuts() -> Vec<String> {
    DEFAULT_QUICK_TOOL_SHORTCUTS
        .iter()
        .map(|shortcut| (*shortcut).to_owned())
        .collect()
}

fn migrate_legacy_quick_tool_shortcuts(settings: &mut AppSettings) -> bool {
    let is_legacy = settings.quick_tool_shortcuts.len() == MAX_QUICK_TOOLS
        && settings
            .quick_tool_shortcuts
            .iter()
            .zip(LEGACY_QUICK_TOOL_SHORTCUTS)
            .all(|(current, legacy)| current.trim().eq_ignore_ascii_case(legacy));
    if !is_legacy {
        return false;
    }
    settings.quick_tool_shortcuts = default_quick_tool_shortcuts();
    true
}

pub struct SettingsStore {
    path: PathBuf,
}

impl SettingsStore {
    pub fn new(app_handle: &AppHandle) -> StorageResult<Self> {
        let directory = app_handle
            .path()
            .app_config_dir()
            .map_err(|error| format!("failed to resolve settings directory: {error}"))?;
        fs::create_dir_all(&directory)
            .map_err(|error| format!("failed to create settings directory: {error}"))?;
        Ok(Self {
            path: directory.join(SETTINGS_FILE_NAME),
        })
    }

    pub fn load(&self) -> StorageResult<AppSettings> {
        if !self.path.exists() {
            let settings = AppSettings::default();
            self.save(&settings)?;
            return Ok(settings);
        }

        let contents = fs::read_to_string(&self.path)
            .map_err(|error| format!("failed to read settings file: {error}"))?;
        let mut settings = serde_json::from_str::<AppSettings>(&contents)
            .map_err(|error| format!("failed to parse settings file: {error}"))?;
        let migrated = migrate_legacy_quick_tool_shortcuts(&mut settings);
        settings.validate()?;
        if migrated {
            self.save(&settings)?;
        }
        Ok(settings)
    }

    pub fn save(&self, settings: &AppSettings) -> StorageResult {
        settings.validate()?;
        let contents = serde_json::to_string_pretty(settings)
            .map_err(|error| format!("failed to serialize settings: {error}"))?;
        fs::write(&self.path, contents)
            .map_err(|error| format!("failed to save settings file: {error}"))
    }
}

pub struct SettingsState {
    settings: RwLock<AppSettings>,
}

impl SettingsState {
    pub fn new(settings: AppSettings) -> Self {
        Self {
            settings: RwLock::new(settings),
        }
    }

    pub fn get(&self) -> StorageResult<AppSettings> {
        self.settings
            .read()
            .map_err(|_| "failed to read application settings".to_owned())
            .map(|settings| settings.clone())
    }

    pub fn replace(&self, settings: AppSettings) -> StorageResult {
        let mut current_settings = self
            .settings
            .write()
            .map_err(|_| "failed to update application settings".to_owned())?;
        *current_settings = settings;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{
        default_quick_tool_shortcuts, migrate_legacy_quick_tool_shortcuts, AppSettings, QuickToolId,
    };

    #[test]
    fn default_main_shortcut_matches_the_current_platform() {
        assert_eq!(
            AppSettings::default().main_shortcut,
            crate::platform::DEFAULT_MAIN_SHORTCUT
        );
    }

    #[test]
    fn settings_without_quick_tools_receive_the_default_order() {
        let mut value = serde_json::to_value(AppSettings::default()).unwrap();
        value.as_object_mut().unwrap().remove("quickToolIds");
        let settings = serde_json::from_value::<AppSettings>(value).unwrap();
        assert_eq!(
            settings.quick_tool_ids,
            vec![
                QuickToolId::TextDiff,
                QuickToolId::Json,
                QuickToolId::Url,
                QuickToolId::Base64,
                QuickToolId::Date,
            ]
        );
    }

    #[test]
    fn settings_without_quick_tool_shortcuts_receive_command_defaults() {
        let mut value = serde_json::to_value(AppSettings::default()).unwrap();
        value.as_object_mut().unwrap().remove("quickToolShortcuts");
        let settings = serde_json::from_value::<AppSettings>(value).unwrap();
        assert_eq!(
            settings.quick_tool_shortcuts,
            vec![
                "Command+1",
                "Command+2",
                "Command+3",
                "Command+4",
                "Command+5"
            ]
        );
    }

    #[test]
    fn quick_tool_ids_use_frontend_names() {
        assert_eq!(
            serde_json::to_value(QuickToolId::TextDiff).unwrap(),
            "text-diff"
        );
        assert_eq!(serde_json::to_value(QuickToolId::Base64).unwrap(), "base64");
    }

    #[test]
    fn legacy_alt_quick_tool_shortcuts_migrate_to_command_defaults() {
        let mut settings = AppSettings::default();
        settings.quick_tool_shortcuts = (1..=5).map(|index| format!("Alt+{index}")).collect();
        assert!(migrate_legacy_quick_tool_shortcuts(&mut settings));
        assert_eq!(
            settings.quick_tool_shortcuts,
            default_quick_tool_shortcuts()
        );
    }

    #[test]
    fn custom_quick_tool_shortcuts_are_not_migrated() {
        let mut settings = AppSettings::default();
        settings.quick_tool_shortcuts[0] = "Alt+Q".to_owned();
        let original = settings.quick_tool_shortcuts.clone();
        assert!(!migrate_legacy_quick_tool_shortcuts(&mut settings));
        assert_eq!(settings.quick_tool_shortcuts, original);
    }
}
