mod history;
mod settings;
mod tags;

pub use history::{
    ClipboardHistoryInput, ClipboardHistoryPage, ClipboardHistoryQuery, HistoryStats, HistoryStore,
};
pub use settings::{AppSettings, AppTheme, SettingsState, SettingsStore};
pub use tags::{ClipboardTag, ClipboardTagInput, TagStore};

pub type StorageResult<T = ()> = Result<T, String>;
