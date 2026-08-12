use super::{AppSettings, StorageResult};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};
use tauri::{AppHandle, Manager};

pub(super) const HISTORY_DATABASE_FILE_NAME: &str = "history.sqlite";
const SECONDS_PER_DAY: i64 = 86_400;

#[derive(Clone, Debug, Deserialize, Serialize)]
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

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClipboardHistoryInput {
    pub format: ClipboardFormat,
    pub content: String,
    #[serde(default)]
    pub file_paths: Vec<String>,
}

impl ClipboardHistoryInput {
    fn validate(&self) -> StorageResult {
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

struct StoredHistoryEntry {
    id: i64,
    format: String,
    content: String,
    file_paths: String,
    updated_at: i64,
    tag_ids: String,
}

impl TryFrom<StoredHistoryEntry> for ClipboardHistoryEntry {
    type Error = String;

    fn try_from(entry: StoredHistoryEntry) -> Result<Self, Self::Error> {
        let file_paths = serde_json::from_str(&entry.file_paths)
            .map_err(|error| format!("failed to parse stored file paths: {error}"))?;
        let tag_ids = serde_json::from_str(&entry.tag_ids)
            .map_err(|error| format!("failed to parse stored tag ids: {error}"))?;
        Ok(Self {
            id: entry.id,
            format: ClipboardFormat::from_str(&entry.format)?,
            content: entry.content,
            file_paths,
            updated_at: entry.updated_at,
            tag_ids,
        })
    }
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

    pub fn list(&self, settings: &AppSettings) -> StorageResult<Vec<ClipboardHistoryEntry>> {
        let connection = self.open_connection()?;
        self.cleanup(&connection, settings)?;
        self.read_entries(&connection)
    }

    pub fn record(
        &self,
        entry: ClipboardHistoryInput,
        settings: &AppSettings,
    ) -> StorageResult<Vec<ClipboardHistoryEntry>> {
        entry.validate()?;
        if !entry.is_enabled(settings) {
            return self.list(settings);
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
        self.list(settings)
    }

    pub fn delete(
        &self,
        id: i64,
        settings: &AppSettings,
    ) -> StorageResult<Vec<ClipboardHistoryEntry>> {
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
        self.list(settings)
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
        let entries = self.list(settings)?;
        let storage_bytes = fs::metadata(&self.path)
            .map_err(|error| format!("failed to read history storage size: {error}"))?
            .len();
        Ok(HistoryStats {
            count: entries.len() as i64,
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
                "DELETE FROM clipboard_history WHERE updated_at < ?1",
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
                "DELETE FROM clipboard_history WHERE id NOT IN (SELECT id FROM clipboard_history ORDER BY updated_at DESC, id DESC LIMIT ?1)",
                params![settings.max_history_entries],
            )
            .map_err(|error| format!("failed to trim clipboard history: {error}"))?;
        Ok(())
    }

    fn read_entries(&self, connection: &Connection) -> StorageResult<Vec<ClipboardHistoryEntry>> {
        let mut statement = connection
            .prepare(
                "SELECT history.id, history.format, history.content, history.file_paths, history.updated_at, COALESCE((SELECT json_group_array(tag_id) FROM clipboard_history_tag_links WHERE history_id = history.id), '[]') FROM clipboard_history AS history ORDER BY history.updated_at DESC, history.id DESC",
            )
            .map_err(|error| format!("failed to query clipboard history: {error}"))?;
        let rows = statement
            .query_map([], |row| {
                Ok(StoredHistoryEntry {
                    id: row.get(0)?,
                    format: row.get(1)?,
                    content: row.get(2)?,
                    file_paths: row.get(3)?,
                    updated_at: row.get(4)?,
                    tag_ids: row.get(5)?,
                })
            })
            .map_err(|error| format!("failed to read clipboard history: {error}"))?;
        let stored_entries = rows
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| format!("failed to decode clipboard history: {error}"))?;
        stored_entries
            .into_iter()
            .map(ClipboardHistoryEntry::try_from)
            .collect()
    }
}

fn current_timestamp() -> StorageResult<i64> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| format!("failed to read system time: {error}"))
        .map(|duration| duration.as_secs() as i64)
}
