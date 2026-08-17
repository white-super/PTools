use super::{ClipboardFormat, ClipboardHistoryEntry};
use crate::storage::StorageResult;
use rusqlite::{params_from_iter, types::Value, Connection};
use serde::{Deserialize, Serialize};

const MAX_HISTORY_PAGE_SIZE: u32 = 20;

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClipboardHistoryQuery {
    #[serde(default)]
    pub search: String,
    #[serde(default)]
    pub format: Option<ClipboardFormat>,
    #[serde(default)]
    pub tag_id: Option<i64>,
    #[serde(default)]
    pub offset: u32,
    #[serde(default = "default_history_page_size")]
    pub limit: u32,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClipboardHistoryPage {
    pub entries: Vec<ClipboardHistoryEntry>,
    pub has_more: bool,
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

pub(super) fn query_history(
    connection: &Connection,
    query: &ClipboardHistoryQuery,
) -> StorageResult<ClipboardHistoryPage> {
    let (sql, values, limit) = build_query(query);
    let mut statement = connection
        .prepare(&sql)
        .map_err(|error| format!("failed to query clipboard history: {error}"))?;
    let rows = statement
        .query_map(params_from_iter(values.iter()), decode_stored_entry)
        .map_err(|error| format!("failed to read clipboard history: {error}"))?;
    let stored_entries = rows
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("failed to decode clipboard history: {error}"))?;
    let has_more = stored_entries.len() > limit;
    let entries = stored_entries
        .into_iter()
        .take(limit)
        .map(ClipboardHistoryEntry::try_from)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(ClipboardHistoryPage { entries, has_more })
}

fn build_query(query: &ClipboardHistoryQuery) -> (String, Vec<Value>, usize) {
    let mut sql = String::from(
        "SELECT history.id, history.format, history.content, history.file_paths, history.updated_at, COALESCE((SELECT json_group_array(tag_id) FROM clipboard_history_tag_links WHERE history_id = history.id), '[]') FROM clipboard_history AS history",
    );
    let mut values = Vec::new();
    if let Some(tag_id) = query.tag_id {
        sql.push_str(
            " INNER JOIN clipboard_history_tag_links AS selected_tag_link ON selected_tag_link.history_id = history.id AND selected_tag_link.tag_id = ?",
        );
        values.push(Value::Integer(tag_id));
    }
    sql.push_str(" WHERE 1 = 1");
    append_search_filter(&mut sql, &mut values, &query.search);
    if let Some(format) = &query.format {
        sql.push_str(" AND history.format = ?");
        values.push(Value::Text(format.as_str().to_owned()));
    }
    let limit = query.limit.clamp(1, MAX_HISTORY_PAGE_SIZE) as usize;
    if query.tag_id.is_some() {
        sql.push_str(" ORDER BY selected_tag_link.assigned_at DESC, history.updated_at DESC, history.id DESC");
    } else {
        sql.push_str(" ORDER BY history.updated_at DESC, history.id DESC");
    }
    sql.push_str(" LIMIT ? OFFSET ?");
    values.push(Value::Integer(limit as i64 + 1));
    values.push(Value::Integer(i64::from(query.offset)));
    (sql, values, limit)
}

fn decode_stored_entry(row: &rusqlite::Row<'_>) -> rusqlite::Result<StoredHistoryEntry> {
    Ok(StoredHistoryEntry {
        id: row.get(0)?,
        format: row.get(1)?,
        content: row.get(2)?,
        file_paths: row.get(3)?,
        updated_at: row.get(4)?,
        tag_ids: row.get(5)?,
    })
}

fn append_search_filter(sql: &mut String, values: &mut Vec<Value>, search: &str) {
    let trimmed_search = search.trim();
    if trimmed_search.is_empty() {
        return;
    }
    let escaped_search = escape_like_pattern(trimmed_search);
    let pattern = Value::Text(format!("%{escaped_search}%"));
    sql.push_str(
        " AND history.format != 'image' AND (history.content LIKE ? ESCAPE '\\' OR history.file_paths LIKE ? ESCAPE '\\')",
    );
    values.push(pattern.clone());
    values.push(pattern);
}

fn escape_like_pattern(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_")
}

fn default_history_page_size() -> u32 {
    MAX_HISTORY_PAGE_SIZE
}
