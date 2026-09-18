mod history;
mod settings;
mod settings_validation;
mod tags;

pub use history::{
    ClipboardFormat, ClipboardHistoryInput, ClipboardHistoryPage, ClipboardHistoryQuery,
    HistoryStats, HistoryStore,
};
pub use settings::{AppSettings, AppTheme, QuickToolId, SettingsState, SettingsStore};
pub use tags::{ClipboardTag, ClipboardTagInput, TagStore};

pub type StorageResult<T = ()> = Result<T, String>;
