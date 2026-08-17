use super::{current_timestamp, HistoryStore, SECONDS_PER_DAY};
use crate::storage::AppSettings;
use rusqlite::{params, Connection};
use std::path::PathBuf;

fn test_store() -> HistoryStore {
    HistoryStore {
        path: PathBuf::new(),
    }
}

fn test_connection() -> Connection {
    let connection = Connection::open_in_memory().expect("failed to open test database");
    connection
        .execute_batch(
            "
            CREATE TABLE clipboard_history (
                id INTEGER PRIMARY KEY,
                format TEXT NOT NULL,
                content TEXT NOT NULL,
                file_paths TEXT NOT NULL,
                updated_at INTEGER NOT NULL
            );
            CREATE TABLE clipboard_history_tag_links (
                history_id INTEGER NOT NULL,
                tag_id INTEGER NOT NULL,
                PRIMARY KEY(history_id, tag_id)
            );
            ",
        )
        .expect("failed to initialize test database");
    connection
}

fn insert_history_entry(connection: &Connection, id: i64, updated_at: i64) {
    connection
        .execute(
            "INSERT INTO clipboard_history (id, format, content, file_paths, updated_at)
             VALUES (?1, 'text', ?2, '[]', ?3)",
            params![id, format!("entry-{id}"), updated_at],
        )
        .expect("failed to insert test history entry");
}

fn tag_history_entry(connection: &Connection, history_id: i64) {
    connection
        .execute(
            "INSERT INTO clipboard_history_tag_links (history_id, tag_id) VALUES (?1, 1)",
            params![history_id],
        )
        .expect("failed to tag test history entry");
}

fn remaining_history_ids(connection: &Connection) -> Vec<i64> {
    let mut statement = connection
        .prepare("SELECT id FROM clipboard_history ORDER BY id ASC")
        .expect("failed to query test history entries");
    statement
        .query_map([], |row| row.get(0))
        .expect("failed to read test history entries")
        .collect::<Result<Vec<_>, _>>()
        .expect("failed to decode test history entries")
}

#[test]
fn expired_cleanup_keeps_tagged_entries() {
    let store = test_store();
    let connection = test_connection();
    let now = current_timestamp().expect("failed to read test timestamp");
    let old_timestamp = now - 8 * SECONDS_PER_DAY;
    let recent_timestamp = now - SECONDS_PER_DAY;
    insert_history_entry(&connection, 1, old_timestamp);
    insert_history_entry(&connection, 2, old_timestamp);
    insert_history_entry(&connection, 3, recent_timestamp);
    tag_history_entry(&connection, 2);

    let settings = AppSettings {
        history_retention_days: 7,
        ..AppSettings::default()
    };
    store
        .remove_expired_entries(&connection, &settings)
        .expect("expired cleanup failed");

    assert_eq!(remaining_history_ids(&connection), vec![2, 3]);
}

#[test]
fn max_entries_cleanup_excludes_tagged_entries_from_limit() {
    let store = test_store();
    let connection = test_connection();
    insert_history_entry(&connection, 1, 100);
    insert_history_entry(&connection, 2, 200);
    insert_history_entry(&connection, 3, 300);
    insert_history_entry(&connection, 4, 1);
    tag_history_entry(&connection, 4);

    let settings = AppSettings {
        max_history_entries: 2,
        ..AppSettings::default()
    };
    store
        .trim_to_max_entries(&connection, &settings)
        .expect("max entries cleanup failed");

    assert_eq!(remaining_history_ids(&connection), vec![2, 3, 4]);
}
