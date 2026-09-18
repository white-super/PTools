use crate::storage::{ClipboardFormat, ClipboardHistoryInput};
use parking_lot::{Mutex, MutexGuard};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, VecDeque},
    sync::atomic::{AtomicU64, Ordering},
};

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum SequentialPasteMode {
    #[default]
    Capture,
    Paste,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum SequentialPasteDirection {
    #[default]
    Forward,
    Reverse,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SequentialPasteItem {
    pub id: u64,
    pub format: ClipboardFormat,
    pub content: String,
    pub file_paths: Vec<String>,
}

impl SequentialPasteItem {
    fn from_input(id: u64, input: ClipboardHistoryInput) -> Self {
        Self {
            id,
            format: input.format,
            content: input.content,
            file_paths: input.file_paths,
        }
    }

    pub fn clipboard_input(&self) -> ClipboardHistoryInput {
        ClipboardHistoryInput {
            format: self.format.clone(),
            content: self.content.clone(),
            file_paths: self.file_paths.clone(),
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SequentialPasteError {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub item_id: Option<u64>,
    pub message: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SequentialPasteSnapshot {
    pub enabled: bool,
    pub mode: SequentialPasteMode,
    pub direction: SequentialPasteDirection,
    pub items: Vec<SequentialPasteItem>,
    pub error: Option<SequentialPasteError>,
}

#[derive(Clone)]
struct ExpectedClipboardWrite {
    token: u64,
    input: ClipboardHistoryInput,
}

#[derive(Default)]
struct SequentialPasteInner {
    enabled: bool,
    mode: SequentialPasteMode,
    direction: SequentialPasteDirection,
    items: VecDeque<SequentialPasteItem>,
    expected_writes: VecDeque<ExpectedClipboardWrite>,
    error: Option<SequentialPasteError>,
}

pub struct SequentialPasteState {
    inner: Mutex<SequentialPasteInner>,
    operation: Mutex<()>,
    next_item_id: AtomicU64,
    next_write_token: AtomicU64,
}

impl Default for SequentialPasteState {
    fn default() -> Self {
        Self {
            inner: Mutex::default(),
            operation: Mutex::default(),
            next_item_id: AtomicU64::new(1),
            next_write_token: AtomicU64::new(1),
        }
    }
}

impl SequentialPasteState {
    pub fn lock_operation(&self) -> MutexGuard<'_, ()> {
        self.operation.lock()
    }

    pub fn snapshot(&self) -> SequentialPasteSnapshot {
        let inner = self.inner.lock();
        SequentialPasteSnapshot {
            enabled: inner.enabled,
            mode: inner.mode,
            direction: inner.direction,
            items: inner.items.iter().cloned().collect(),
            error: inner.error.clone(),
        }
    }

    pub fn enable(&self) {
        let mut inner = self.inner.lock();
        inner.enabled = true;
        inner.error = None;
    }

    pub fn disable_and_clear(&self) {
        let mut inner = self.inner.lock();
        inner.enabled = false;
        inner.mode = SequentialPasteMode::Capture;
        inner.direction = SequentialPasteDirection::Forward;
        inner.items.clear();
        inner.expected_writes.clear();
        inner.error = None;
    }

    pub fn set_mode(&self, mode: SequentialPasteMode) -> Result<(), String> {
        let mut inner = self.inner.lock();
        ensure_enabled(&inner)?;
        if mode == SequentialPasteMode::Paste && inner.items.is_empty() {
            return Err("队列为空，请先采集需要顺序粘贴的内容".to_owned());
        }
        inner.mode = mode;
        inner.error = None;
        Ok(())
    }

    pub fn set_direction(&self, direction: SequentialPasteDirection) -> Result<(), String> {
        let mut inner = self.inner.lock();
        ensure_enabled(&inner)?;
        inner.direction = direction;
        inner.error = None;
        Ok(())
    }

    pub fn capture(&self, input: ClipboardHistoryInput) -> bool {
        let mut inner = self.inner.lock();
        if !inner.enabled || inner.mode != SequentialPasteMode::Capture {
            return false;
        }
        let id = self.next_item_id.fetch_add(1, Ordering::Relaxed);
        inner
            .items
            .push_back(SequentialPasteItem::from_input(id, input));
        inner.error = None;
        true
    }

    pub fn next_item(&self) -> Result<SequentialPasteItem, String> {
        let inner = self.inner.lock();
        ensure_enabled(&inner)?;
        if inner.mode != SequentialPasteMode::Paste {
            return Err("请先切换到粘贴模式".to_owned());
        }
        let item = match inner.direction {
            SequentialPasteDirection::Forward => inner.items.front(),
            SequentialPasteDirection::Reverse => inner.items.back(),
        };
        item.cloned()
            .ok_or_else(|| "顺序粘贴队列已经完成".to_owned())
    }

    pub fn complete_item(&self, item_id: u64) -> Result<bool, String> {
        let mut inner = self.inner.lock();
        let next = match inner.direction {
            SequentialPasteDirection::Forward => inner.items.front(),
            SequentialPasteDirection::Reverse => inner.items.back(),
        };
        if next.map(|item| item.id) != Some(item_id) {
            return Err("顺序粘贴队列在执行期间发生了变化".to_owned());
        }
        match inner.direction {
            SequentialPasteDirection::Forward => inner.items.pop_front(),
            SequentialPasteDirection::Reverse => inner.items.pop_back(),
        };
        inner.error = None;
        Ok(inner.items.is_empty())
    }

    pub fn reorder(&self, item_ids: &[u64]) -> Result<(), String> {
        let mut inner = self.inner.lock();
        ensure_capture_mode(&inner)?;
        if item_ids.len() != inner.items.len() {
            return Err("拖拽后的队列内容不完整".to_owned());
        }
        let mut items = inner
            .items
            .iter()
            .cloned()
            .map(|item| (item.id, item))
            .collect::<HashMap<_, _>>();
        let ordered = item_ids
            .iter()
            .map(|id| {
                items
                    .remove(id)
                    .ok_or_else(|| "拖拽后的队列包含未知内容".to_owned())
            })
            .collect::<Result<VecDeque<_>, _>>()?;
        if !items.is_empty() {
            return Err("拖拽后的队列缺少原有内容".to_owned());
        }
        inner.items = ordered;
        Ok(())
    }

    pub fn remove(&self, item_id: u64) -> Result<(), String> {
        let mut inner = self.inner.lock();
        ensure_capture_mode(&inner)?;
        let index = inner
            .items
            .iter()
            .position(|item| item.id == item_id)
            .ok_or_else(|| "顺序粘贴队列项不存在".to_owned())?;
        inner.items.remove(index);
        inner.error = None;
        Ok(())
    }

    pub fn clear(&self) -> Result<(), String> {
        let mut inner = self.inner.lock();
        ensure_capture_mode(&inner)?;
        inner.items.clear();
        inner.error = None;
        Ok(())
    }

    pub fn set_error(&self, item_id: Option<u64>, message: String) {
        self.inner.lock().error = Some(SequentialPasteError { item_id, message });
    }

    pub fn expect_internal_write(&self, input: ClipboardHistoryInput) -> u64 {
        let token = self.next_write_token.fetch_add(1, Ordering::Relaxed);
        self.inner
            .lock()
            .expected_writes
            .push_back(ExpectedClipboardWrite { token, input });
        token
    }

    pub fn cancel_internal_write(&self, token: u64) {
        self.inner
            .lock()
            .expected_writes
            .retain(|expected| expected.token != token);
    }

    pub fn consume_internal_write(&self, input: &ClipboardHistoryInput) -> bool {
        let mut inner = self.inner.lock();
        let Some(index) = inner
            .expected_writes
            .iter()
            .position(|expected| expected.input == *input)
        else {
            return false;
        };
        inner.expected_writes.remove(index);
        true
    }
}

fn ensure_enabled(inner: &SequentialPasteInner) -> Result<(), String> {
    inner
        .enabled
        .then_some(())
        .ok_or_else(|| "顺序粘贴队列尚未打开".to_owned())
}

fn ensure_capture_mode(inner: &SequentialPasteInner) -> Result<(), String> {
    ensure_enabled(inner)?;
    (inner.mode == SequentialPasteMode::Capture)
        .then_some(())
        .ok_or_else(|| "请切换到采集模式后修改队列".to_owned())
}

#[cfg(test)]
mod tests;
