mod history;
mod settings;
mod tags;

pub use history::{ClipboardHistoryEntry, ClipboardHistoryInput, HistoryStats, HistoryStore};
pub use settings::{AppSettings, SettingsState, SettingsStore};
pub use tags::{ClipboardTag, ClipboardTagInput, TagStore};

pub type StorageResult<T = ()> = Result<T, String>;
