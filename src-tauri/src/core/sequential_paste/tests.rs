use super::{
    SequentialPasteDirection, SequentialPasteError, SequentialPasteMode, SequentialPasteState,
};
use crate::storage::{ClipboardFormat, ClipboardHistoryInput};

fn text(content: &str) -> ClipboardHistoryInput {
    ClipboardHistoryInput {
        format: ClipboardFormat::Text,
        content: content.to_owned(),
        file_paths: Vec::new(),
    }
}

#[test]
fn capture_keeps_duplicates_and_supports_both_directions() {
    let state = SequentialPasteState::default();
    state.enable();
    assert!(state.capture(text("one")));
    assert!(state.capture(text("one")));
    assert!(state.capture(text("three")));
    state.set_mode(SequentialPasteMode::Paste).unwrap();
    assert_eq!(state.next_item().unwrap().content, "one");
    state
        .set_direction(SequentialPasteDirection::Reverse)
        .unwrap();
    assert_eq!(state.next_item().unwrap().content, "three");
}

#[test]
fn completed_items_leave_the_queue_only_after_confirmation() {
    let state = SequentialPasteState::default();
    state.enable();
    state.capture(text("one"));
    state.capture(text("two"));
    state.set_mode(SequentialPasteMode::Paste).unwrap();
    let item = state.next_item().unwrap();
    assert!(!state.complete_item(item.id).unwrap());
    assert_eq!(state.next_item().unwrap().content, "two");
}

#[test]
fn internal_writes_are_consumed_once_without_entering_capture() {
    let state = SequentialPasteState::default();
    state.enable();
    let entry = text("internal");
    state.expect_internal_write(entry.clone());
    assert!(state.consume_internal_write(&entry));
    assert!(!state.consume_internal_write(&entry));
    assert!(state.snapshot().items.is_empty());
}

#[test]
fn closing_discards_unconsumed_internal_writes() {
    let state = SequentialPasteState::default();
    state.enable();
    let entry = text("internal");
    state.expect_internal_write(entry.clone());
    state.disable_and_clear();
    state.enable();
    assert!(!state.consume_internal_write(&entry));
}

#[test]
fn global_errors_omit_the_item_identifier() {
    let error = SequentialPasteError {
        item_id: None,
        message: "shortcut failed".to_owned(),
    };
    let value = serde_json::to_value(error).unwrap();
    assert!(value.get("itemId").is_none());
}

#[test]
fn capture_queue_can_be_reordered_and_removed() {
    let state = SequentialPasteState::default();
    state.enable();
    state.capture(text("one"));
    state.capture(text("two"));
    let ids = state
        .snapshot()
        .items
        .iter()
        .map(|item| item.id)
        .collect::<Vec<_>>();
    state.reorder(&[ids[1], ids[0]]).unwrap();
    assert_eq!(state.snapshot().items[0].content, "two");
    state.remove(ids[1]).unwrap();
    assert_eq!(state.snapshot().items[0].content, "one");
}

#[test]
fn invalid_reorder_keeps_the_original_queue() {
    let state = SequentialPasteState::default();
    state.enable();
    state.capture(text("one"));
    state.capture(text("two"));
    let original = state.snapshot().items;
    assert!(state.reorder(&[original[0].id, u64::MAX]).is_err());
    assert_eq!(state.snapshot().items, original);
}
