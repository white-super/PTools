use super::StorageResult;
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf, sync::RwLock};
use tauri::{AppHandle, Manager};

const SETTINGS_FILE_NAME: &str = "settings.json";
const DEFAULT_MAIN_SHORTCUT: &str = "Ctrl+V";
const DEFAULT_PREVIOUS_FILTER_SHORTCUT: &str = "Ctrl+Q";
const DEFAULT_NEXT_FILTER_SHORTCUT: &str = "Ctrl+E";
const DEFAULT_PREVIOUS_CARD_SHORTCUT: &str = "Ctrl+A";
const DEFAULT_NEXT_CARD_SHORTCUT: &str = "Ctrl+D";
const SEARCH_SHORTCUT: &str = "Command+F";
const DEFAULT_HISTORY_RETENTION_DAYS: u32 = 30;
const DEFAULT_MAX_HISTORY_ENTRIES: u32 = 200;
const SUPPORTED_HISTORY_RETENTION_DAYS: [u32; 4] = [0, 7, 30, 90];

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AppTheme {
    #[default]
    SoftGlow,
    Classic,
    Dark,
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
            main_shortcut: DEFAULT_MAIN_SHORTCUT.to_owned(),
            previous_filter_shortcut: default_previous_filter_shortcut(),
            next_filter_shortcut: default_next_filter_shortcut(),
            previous_card_shortcut: default_previous_card_shortcut(),
            next_card_shortcut: default_next_card_shortcut(),
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
        let main_shortcut = self.main_shortcut.trim();
        let previous_filter_shortcut = self.previous_filter_shortcut.trim();
        let next_filter_shortcut = self.next_filter_shortcut.trim();
        let previous_card_shortcut = self.previous_card_shortcut.trim();
        let next_card_shortcut = self.next_card_shortcut.trim();
        if main_shortcut.is_empty() {
            return Err("唤醒快捷键不能为空".to_owned());
        }
        if previous_filter_shortcut.is_empty()
            || next_filter_shortcut.is_empty()
            || previous_card_shortcut.is_empty()
            || next_card_shortcut.is_empty()
        {
            return Err("标签和卡片切换快捷键不能为空".to_owned());
        }
        let shortcuts = [
            main_shortcut,
            previous_filter_shortcut,
            next_filter_shortcut,
            previous_card_shortcut,
            next_card_shortcut,
        ];
        if has_duplicate_shortcuts(&shortcuts) {
            return Err("唤醒、标签切换和卡片切换快捷键不能重复".to_owned());
        }
        if shortcuts
            .iter()
            .any(|shortcut| shortcut.eq_ignore_ascii_case(SEARCH_SHORTCUT))
        {
            return Err("标签和卡片切换快捷键不能使用搜索快捷键 Command+F".to_owned());
        }
        if self.max_history_entries == 0 {
            return Err("最大历史记录数必须大于 0".to_owned());
        }
        if !SUPPORTED_HISTORY_RETENTION_DAYS.contains(&self.history_retention_days) {
            return Err("历史保留时间仅支持 7 天、30 天、90 天或永久保留".to_owned());
        }
        if !self.record_text && !self.record_images && !self.record_files {
            return Err("请至少选择一种要记录的剪贴板类型".to_owned());
        }
        Ok(())
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

fn has_duplicate_shortcuts(shortcuts: &[&str]) -> bool {
    shortcuts.iter().enumerate().any(|(index, shortcut)| {
        shortcuts[..index]
            .iter()
            .any(|other| other.eq_ignore_ascii_case(shortcut))
    })
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
        let settings = serde_json::from_str::<AppSettings>(&contents)
            .map_err(|error| format!("failed to parse settings file: {error}"))?;
        settings.validate()?;
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
