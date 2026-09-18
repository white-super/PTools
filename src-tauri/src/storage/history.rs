use super::{AppSettings, StorageResult};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};
use tauri::{AppHandle, Manager};

mod query;
#[cfg(test)]
mod tests;

pub use query::{ClipboardHistoryPage, ClipboardHistoryQuery};

pub(super) const HISTORY_DATABASE_FILE_NAME: &str = "history.sqlite";
const SECONDS_PER_DAY: i64 = 86_400;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ClipboardFormat {
    Text,
    Image,
    File,
}

impl ClipboardFormat {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Text => "text",
            Self::Image => "image",
            Self::File => "file",
        }
    }

    fn from_str(value: &str) -> StorageResult<Self> {
        match value {
            "text" => Ok(Self::Text),
            "image" => Ok(Self::Image),
            "file" => Ok(Self::File),
            _ => Err(format!("unsupported clipboard format: {value}")),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ClipboardHistoryInput {
    pub format: ClipboardFormat,
    pub content: String,
    #[serde(default)]
    pub file_paths: Vec<String>,
}

impl ClipboardHistoryInput {
    pub(crate) fn validate(&self) -> StorageResult {
        if self.content.is_empty() {
            return Err("剪贴板内容不能为空".to_owned());
        }
        if matches!(self.format, ClipboardFormat::File) && self.file_paths.is_empty() {
            return Err("文件历史记录必须包含文件路径".to_owned());
        }
        Ok(())
    }

    fn is_enabled(&self, settings: &AppSettings) -> bool {
        match self.format {
            ClipboardFormat::Text => settings.record_text,
            ClipboardFormat::Image => settings.record_images,
            ClipboardFormat::File => settings.record_files,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClipboardHistoryEntry {
    pub id: i64,
    pub format: ClipboardFormat,
    pub content: String,
    pub file_paths: Vec<String>,
    pub updated_at: i64,
    pub tag_ids: Vec<i64>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryStats {
    pub count: i64,
    pub storage_bytes: u64,
}

pub struct HistoryStore {
    path: PathBuf,
}

impl HistoryStore {
    pub fn new(app_handle: &AppHandle) -> StorageResult<Self> {
        let directory = app_handle
            .path()
            .app_data_dir()
            .map_err(|error| format!("failed to resolve history directory: {error}"))?;
        fs::create_dir_all(&directory)
            .map_err(|error| format!("failed to create history directory: {error}"))?;
        let store = Self {
            path: directory.join(HISTORY_DATABASE_FILE_NAME),
        };
        store.initialize()?;
        Ok(store)
    }

    pub fn query(
        &self,
        settings: &AppSettings,
        query: &ClipboardHistoryQuery,
    ) -> StorageResult<ClipboardHistoryPage> {
        let connection = self.open_connection()?;
        self.cleanup(&connection, settings)?;
        query::query_history(&connection, query)
    }

    pub fn cleanup_history(&self, settings: &AppSettings) -> StorageResult {
        let connection = self.open_connection()?;
        self.cleanup(&connection, settings)
    }

    pub fn record(&self, entry: ClipboardHistoryInput, settings: &AppSettings) -> StorageResult {
        entry.validate()?;
        if !entry.is_enabled(settings) {
            return Ok(());
        }

        let now = current_timestamp()?;
        let file_paths = serde_json::to_string(&entry.file_paths)
            .map_err(|error| format!("failed to serialize file paths: {error}"))?;
        let mut connection = self.open_connection()?;
        let transaction = connection
            .transaction()
            .map_err(|error| format!("failed to create history transaction: {error}"))?;
        transaction
            .execute(
                "DELETE FROM clipboard_history WHERE format = ?1 AND content = ?2 AND file_paths = ?3",
                params![entry.format.as_str(), entry.content, file_paths],
            )
            .map_err(|error| format!("failed to deduplicate history entry: {error}"))?;
        transaction
            .execute(
                "INSERT INTO clipboard_history (format, content, file_paths, updated_at) VALUES (?1, ?2, ?3, ?4)",
                params![entry.format.as_str(), entry.content, file_paths, now],
            )
            .map_err(|error| format!("failed to store clipboard history: {error}"))?;
        self.cleanup(&transaction, settings)?;
        transaction
            .commit()
            .map_err(|error| format!("failed to commit clipboard history: {error}"))?;
        Ok(())
    }

    pub fn delete(&self, id: i64, settings: &AppSettings) -> StorageResult {
        let mut connection = self.open_connection()?;
        let transaction = connection
            .transaction()
            .map_err(|error| format!("failed to create history transaction: {error}"))?;
        let deleted = transaction
            .execute("DELETE FROM clipboard_history WHERE id = ?1", params![id])
            .map_err(|error| format!("failed to delete clipboard history entry: {error}"))?;
        if deleted == 0 {
            return Err(format!("clipboard history entry not found: {id}"));
        }
        transaction
            .commit()
            .map_err(|error| format!("failed to commit clipboard history deletion: {error}"))?;
        self.cleanup_history(settings)
    }

    pub fn clear(&self) -> StorageResult {
        let connection = self.open_connection()?;
        connection
            .execute("DELETE FROM clipboard_history", [])
            .map_err(|error| format!("failed to clear clipboard history: {error}"))?;
        connection
            .execute_batch("VACUUM")
            .map_err(|error| format!("failed to reclaim history storage: {error}"))
    }

    pub fn stats(&self, settings: &AppSettings) -> StorageResult<HistoryStats> {
        let connection = self.open_connection()?;
        self.cleanup(&connection, settings)?;
        let count = connection
            .query_row("SELECT COUNT(*) FROM clipboard_history", [], |row| {
                row.get::<_, i64>(0)
            })
            .map_err(|error| format!("failed to count clipboard history: {error}"))?;
        let storage_bytes = fs::metadata(&self.path)
            .map_err(|error| format!("failed to read history storage size: {error}"))?
            .len();
        Ok(HistoryStats {
            count,
            storage_bytes,
        })
    }

    fn initialize(&self) -> StorageResult {
        let connection = self.open_connection()?;
        connection
            .execute_batch(
                "
                CREATE TABLE IF NOT EXISTS clipboard_history (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    format TEXT NOT NULL CHECK(format IN ('text', 'image', 'file')),
                    content TEXT NOT NULL,
                    file_paths TEXT NOT NULL,
                    updated_at INTEGER NOT NULL
                );
                CREATE INDEX IF NOT EXISTS clipboard_history_updated_at
                ON clipboard_history(updated_at DESC, id DESC);
                ",
            )
            .map_err(|error| format!("failed to initialize history storage: {error}"))
    }

    fn open_connection(&self) -> StorageResult<Connection> {
        let connection = Connection::open(&self.path)
            .map_err(|error| format!("failed to open history storage: {error}"))?;
        connection
            .execute_batch("PRAGMA foreign_keys = ON")
            .map_err(|error| format!("failed to enable history foreign keys: {error}"))?;
        Ok(connection)
    }

    fn cleanup(&self, connection: &Connection, settings: &AppSettings) -> StorageResult {
        self.remove_expired_entries(connection, settings)?;
        self.trim_to_max_entries(connection, settings)
    }

    fn remove_expired_entries(
        &self,
        connection: &Connection,
        settings: &AppSettings,
    ) -> StorageResult {
        if settings.history_retention_days == 0 {
            return Ok(());
        }
        let cutoff =
            current_timestamp()? - i64::from(settings.history_retention_days) * SECONDS_PER_DAY;
        connection
            .execute(
                "DELETE FROM clipboard_history
                 WHERE updated_at < ?1
                   AND NOT EXISTS (
                       SELECT 1
                       FROM clipboard_history_tag_links
                       WHERE history_id = clipboard_history.id
                   )",
                params![cutoff],
            )
            .map_err(|error| format!("failed to remove expired history: {error}"))?;
        Ok(())
    }

    fn trim_to_max_entries(
        &self,
        connection: &Connection,
        settings: &AppSettings,
    ) -> StorageResult {
        connection
            .execute(
                "DELETE FROM clipboard_history
                 WHERE NOT EXISTS (
                     SELECT 1
                     FROM clipboard_history_tag_links
                     WHERE history_id = clipboard_history.id
                 )
                   AND id NOT IN (
                       SELECT history.id
                       FROM clipboard_history AS history
                       WHERE NOT EXISTS (
                           SELECT 1
                           FROM clipboard_history_tag_links
                           WHERE history_id = history.id
                       )
                       ORDER BY history.updated_at DESC, history.id DESC
                       LIMIT ?1
                   )",
                params![settings.max_history_entries],
            )
            .map_err(|error| format!("failed to trim clipboard history: {error}"))?;
        Ok(())
    }
}

fn current_timestamp() -> StorageResult<i64> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| format!("failed to read system time: {error}"))
        .map(|duration| duration.as_secs() as i64)
}
