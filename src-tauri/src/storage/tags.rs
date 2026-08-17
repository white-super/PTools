use super::{history::HISTORY_DATABASE_FILE_NAME, StorageResult};
use rusqlite::{params, Connection, Transaction};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};
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
        ensure_history_exists(&transaction, history_id)?;
        let next_tag_ids = tag_ids.iter().copied().collect::<HashSet<_>>();
        ensure_tags_exist(&transaction, &next_tag_ids)?;
        let current_tag_ids = get_history_tag_ids(&transaction, history_id)?;
        for tag_id in current_tag_ids.difference(&next_tag_ids) {
            transaction
                .execute(
                    "DELETE FROM clipboard_history_tag_links WHERE history_id = ?1 AND tag_id = ?2",
                    params![history_id, tag_id],
                )
                .map_err(|error| format!("failed to remove clipboard tag assignment: {error}"))?;
        }
        let assigned_at = current_timestamp_millis()?;
        for tag_id in next_tag_ids.difference(&current_tag_ids) {
            transaction
                .execute(
                    "INSERT INTO clipboard_history_tag_links (history_id, tag_id, assigned_at) VALUES (?1, ?2, ?3)",
                    params![history_id, tag_id, assigned_at],
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
                    assigned_at INTEGER NOT NULL,
                    PRIMARY KEY(history_id, tag_id),
                    FOREIGN KEY(history_id) REFERENCES clipboard_history(id) ON DELETE CASCADE,
                    FOREIGN KEY(tag_id) REFERENCES clipboard_tags(id) ON DELETE CASCADE
                );
                CREATE INDEX IF NOT EXISTS clipboard_history_tag_links_tag_id
                ON clipboard_history_tag_links(tag_id);
                ",
            )
            .map_err(|error| format!("failed to initialize clipboard tag storage: {error}"))?;
        ensure_tag_link_timestamps(&connection)
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

fn ensure_history_exists(transaction: &Transaction<'_>, history_id: i64) -> StorageResult {
    let exists = transaction
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM clipboard_history WHERE id = ?1)",
            params![history_id],
            |row| row.get::<_, bool>(0),
        )
        .map_err(|error| format!("failed to verify clipboard history entry: {error}"))?;
    if !exists {
        return Err(format!("clipboard history entry not found: {history_id}"));
    }
    Ok(())
}

fn ensure_tags_exist(transaction: &Transaction<'_>, tag_ids: &HashSet<i64>) -> StorageResult {
    for tag_id in tag_ids {
        let exists = transaction
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM clipboard_tags WHERE id = ?1)",
                params![tag_id],
                |row| row.get::<_, bool>(0),
            )
            .map_err(|error| format!("failed to verify clipboard tag: {error}"))?;
        if !exists {
            return Err(format!("clipboard tag not found: {tag_id}"));
        }
    }
    Ok(())
}

fn get_history_tag_ids(
    transaction: &Transaction<'_>,
    history_id: i64,
) -> StorageResult<HashSet<i64>> {
    let mut statement = transaction
        .prepare("SELECT tag_id FROM clipboard_history_tag_links WHERE history_id = ?1")
        .map_err(|error| format!("failed to query clipboard tag assignments: {error}"))?;
    let rows = statement
        .query_map(params![history_id], |row| row.get(0))
        .map_err(|error| format!("failed to read clipboard tag assignments: {error}"))?;
    rows.collect::<Result<HashSet<_>, _>>()
        .map_err(|error| format!("failed to decode clipboard tag assignments: {error}"))
}

fn ensure_tag_link_timestamps(connection: &Connection) -> StorageResult {
    let has_assigned_at = connection
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM pragma_table_info('clipboard_history_tag_links') WHERE name = 'assigned_at')",
            [],
            |row| row.get::<_, bool>(0),
        )
        .map_err(|error| format!("failed to inspect clipboard tag storage: {error}"))?;
    if !has_assigned_at {
        connection
            .execute_batch(
                "ALTER TABLE clipboard_history_tag_links ADD COLUMN assigned_at INTEGER NOT NULL DEFAULT 0",
            )
            .map_err(|error| format!("failed to migrate clipboard tag storage: {error}"))?;
    }
    connection
        .execute(
            "UPDATE clipboard_history_tag_links SET assigned_at = COALESCE((SELECT updated_at * 1000 FROM clipboard_history WHERE id = history_id), 0) WHERE assigned_at = 0",
            [],
        )
        .map_err(|error| format!("failed to initialize clipboard tag timestamps: {error}"))?;
    connection
        .execute_batch(
            "CREATE INDEX IF NOT EXISTS clipboard_history_tag_links_assigned_at ON clipboard_history_tag_links(tag_id, assigned_at DESC, history_id DESC)",
        )
        .map_err(|error| format!("failed to index clipboard tag timestamps: {error}"))
}

fn current_timestamp_millis() -> StorageResult<i64> {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| format!("failed to read system time: {error}"))?
        .as_millis();
    i64::try_from(millis).map_err(|error| format!("system timestamp is out of range: {error}"))
}
