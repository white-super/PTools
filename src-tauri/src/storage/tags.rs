use super::{history::HISTORY_DATABASE_FILE_NAME, StorageResult};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, fs, path::PathBuf};
use tauri::{AppHandle, Manager};

const HEX_COLOR_LENGTH: usize = 7;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClipboardTag {
    pub id: i64,
    pub name: String,
    pub color: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClipboardTagInput {
    pub name: String,
    pub color: String,
}

impl ClipboardTagInput {
    fn validate(&self) -> StorageResult {
        if self.name.trim().is_empty() {
            return Err("标签名称不能为空".to_owned());
        }
        if self.color.len() != HEX_COLOR_LENGTH
            || !self.color.starts_with('#')
            || !self.color[1..]
                .chars()
                .all(|character| character.is_ascii_hexdigit())
        {
            return Err("标签颜色必须是十六进制颜色值".to_owned());
        }
        Ok(())
    }
}

pub struct TagStore {
    path: PathBuf,
}

impl TagStore {
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

    pub fn list(&self) -> StorageResult<Vec<ClipboardTag>> {
        let connection = self.open_connection()?;
        let mut statement = connection
            .prepare("SELECT id, name, color FROM clipboard_tags ORDER BY id ASC")
            .map_err(|error| format!("failed to query clipboard tags: {error}"))?;
        let rows = statement
            .query_map([], |row| {
                Ok(ClipboardTag {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    color: row.get(2)?,
                })
            })
            .map_err(|error| format!("failed to read clipboard tags: {error}"))?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|error| format!("failed to decode clipboard tags: {error}"))
    }

    pub fn create(&self, input: ClipboardTagInput) -> StorageResult<Vec<ClipboardTag>> {
        input.validate()?;
        let connection = self.open_connection()?;
        connection
            .execute(
                "INSERT INTO clipboard_tags (name, color) VALUES (?1, ?2)",
                params![input.name.trim(), input.color],
            )
            .map_err(|error| format!("failed to create clipboard tag: {error}"))?;
        self.list()
    }

    pub fn update(&self, id: i64, input: ClipboardTagInput) -> StorageResult<Vec<ClipboardTag>> {
        input.validate()?;
        let connection = self.open_connection()?;
        let changed = connection
            .execute(
                "UPDATE clipboard_tags SET name = ?1, color = ?2 WHERE id = ?3",
                params![input.name.trim(), input.color, id],
            )
            .map_err(|error| format!("failed to update clipboard tag: {error}"))?;
        if changed == 0 {
            return Err(format!("clipboard tag not found: {id}"));
        }
        self.list()
    }

    pub fn delete(&self, id: i64) -> StorageResult<Vec<ClipboardTag>> {
        let mut connection = self.open_connection()?;
        let transaction = connection
            .transaction()
            .map_err(|error| format!("failed to create tag transaction: {error}"))?;
        transaction
            .execute(
                "DELETE FROM clipboard_history_tag_links WHERE tag_id = ?1",
                params![id],
            )
            .map_err(|error| format!("failed to unlink clipboard tag: {error}"))?;
        let deleted = transaction
            .execute("DELETE FROM clipboard_tags WHERE id = ?1", params![id])
            .map_err(|error| format!("failed to delete clipboard tag: {error}"))?;
        if deleted == 0 {
            return Err(format!("clipboard tag not found: {id}"));
        }
        transaction
            .commit()
            .map_err(|error| format!("failed to commit tag deletion: {error}"))?;
        self.list()
    }

    pub fn set_history_tags(&self, history_id: i64, tag_ids: &[i64]) -> StorageResult {
        let mut connection = self.open_connection()?;
        let transaction = connection
            .transaction()
            .map_err(|error| format!("failed to create tag transaction: {error}"))?;
        let history_exists = transaction
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM clipboard_history WHERE id = ?1)",
                params![history_id],
                |row| row.get::<_, bool>(0),
            )
            .map_err(|error| format!("failed to verify clipboard history entry: {error}"))?;
        if !history_exists {
            return Err(format!("clipboard history entry not found: {history_id}"));
        }
        let unique_tag_ids = tag_ids.iter().copied().collect::<HashSet<_>>();
        for tag_id in &unique_tag_ids {
            let tag_exists = transaction
                .query_row(
                    "SELECT EXISTS(SELECT 1 FROM clipboard_tags WHERE id = ?1)",
                    params![tag_id],
                    |row| row.get::<_, bool>(0),
                )
                .map_err(|error| format!("failed to verify clipboard tag: {error}"))?;
            if !tag_exists {
                return Err(format!("clipboard tag not found: {tag_id}"));
            }
        }
        transaction
            .execute(
                "DELETE FROM clipboard_history_tag_links WHERE history_id = ?1",
                params![history_id],
            )
            .map_err(|error| format!("failed to clear clipboard tags: {error}"))?;
        for tag_id in unique_tag_ids {
            transaction
                .execute(
                    "INSERT INTO clipboard_history_tag_links (history_id, tag_id) VALUES (?1, ?2)",
                    params![history_id, tag_id],
                )
                .map_err(|error| format!("failed to assign clipboard tag: {error}"))?;
        }
        transaction
            .commit()
            .map_err(|error| format!("failed to commit clipboard tags: {error}"))
    }

    fn initialize(&self) -> StorageResult {
        let connection = self.open_connection()?;
        connection
            .execute_batch(
                "
                CREATE TABLE IF NOT EXISTS clipboard_tags (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    name TEXT NOT NULL COLLATE NOCASE UNIQUE,
                    color TEXT NOT NULL
                );
                CREATE TABLE IF NOT EXISTS clipboard_history_tag_links (
                    history_id INTEGER NOT NULL,
                    tag_id INTEGER NOT NULL,
                    PRIMARY KEY(history_id, tag_id),
                    FOREIGN KEY(history_id) REFERENCES clipboard_history(id) ON DELETE CASCADE,
                    FOREIGN KEY(tag_id) REFERENCES clipboard_tags(id) ON DELETE CASCADE
                );
                CREATE INDEX IF NOT EXISTS clipboard_history_tag_links_tag_id
                ON clipboard_history_tag_links(tag_id);
                ",
            )
            .map_err(|error| format!("failed to initialize clipboard tag storage: {error}"))
    }

    fn open_connection(&self) -> StorageResult<Connection> {
        let connection = Connection::open(&self.path)
            .map_err(|error| format!("failed to open history storage: {error}"))?;
        connection
            .execute_batch("PRAGMA foreign_keys = ON")
            .map_err(|error| format!("failed to enable tag foreign keys: {error}"))?;
        Ok(connection)
    }
}
