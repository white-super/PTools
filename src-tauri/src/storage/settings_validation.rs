use super::{
    settings::{AppSettings, QuickToolId, MAX_QUICK_TOOLS},
    StorageResult,
};

const SEARCH_SHORTCUT: &str = "Command+F";
const SUPPORTED_HISTORY_RETENTION_DAYS: [u32; 4] = [0, 7, 30, 90];

pub(super) fn validate_settings(settings: &AppSettings) -> StorageResult {
    let navigation_shortcuts = [
        settings.main_shortcut.trim(),
        settings.previous_filter_shortcut.trim(),
        settings.next_filter_shortcut.trim(),
        settings.previous_card_shortcut.trim(),
        settings.next_card_shortcut.trim(),
        settings.sequential_paste_shortcut.trim(),
    ];
    validate_navigation_shortcuts(&navigation_shortcuts)?;
    validate_quick_tool_shortcuts(&navigation_shortcuts, &settings.quick_tool_shortcuts)?;
    validate_quick_tool_ids(&settings.quick_tool_ids)?;
    validate_history_settings(settings)
}

fn validate_navigation_shortcuts(shortcuts: &[&str]) -> StorageResult {
    if shortcuts[0].is_empty() {
        return Err("唤醒快捷键不能为空".to_owned());
    }
    if shortcuts[1..].iter().any(|shortcut| shortcut.is_empty()) {
        return Err("标签、卡片切换和顺序粘贴快捷键不能为空".to_owned());
    }
    if has_duplicate_shortcuts(shortcuts) {
        return Err("唤醒、标签切换、卡片切换和顺序粘贴快捷键不能重复".to_owned());
    }
    if shortcuts_equal_to_search(shortcuts) {
        return Err("唤醒、标签、卡片切换和顺序粘贴不能使用搜索快捷键 Command+F".to_owned());
    }
    Ok(())
}

fn validate_quick_tool_shortcuts(
    navigation_shortcuts: &[&str],
    quick_tool_shortcuts: &[String],
) -> StorageResult {
    if quick_tool_shortcuts.len() != MAX_QUICK_TOOLS {
        return Err("快捷工具需要配置 5 个快捷键".to_owned());
    }
    let shortcuts = quick_tool_shortcuts
        .iter()
        .map(|shortcut| shortcut.trim())
        .collect::<Vec<_>>();
    if shortcuts.iter().any(|shortcut| shortcut.is_empty()) {
        return Err("快捷工具快捷键不能为空".to_owned());
    }
    if has_duplicate_shortcuts(&shortcuts) {
        return Err("快捷工具快捷键不能重复".to_owned());
    }
    if shortcuts_equal_to_search(&shortcuts) {
        return Err("快捷工具快捷键不能使用搜索快捷键 Command+F".to_owned());
    }
    if shortcuts.iter().any(|shortcut| {
        navigation_shortcuts
            .iter()
            .any(|navigation| shortcuts_equal(shortcut, navigation))
    }) {
        return Err("快捷工具快捷键不能与唤醒、标签、卡片切换或顺序粘贴快捷键重复".to_owned());
    }
    Ok(())
}

fn validate_quick_tool_ids(tool_ids: &[QuickToolId]) -> StorageResult {
    if tool_ids.len() > MAX_QUICK_TOOLS {
        return Err("快捷工具最多只能添加 5 个".to_owned());
    }
    if tool_ids
        .iter()
        .enumerate()
        .any(|(index, tool_id)| tool_ids[..index].iter().any(|other| other == tool_id))
    {
        return Err("快捷工具不能重复添加".to_owned());
    }
    Ok(())
}

fn validate_history_settings(settings: &AppSettings) -> StorageResult {
    if settings.max_history_entries == 0 {
        return Err("最大历史记录数必须大于 0".to_owned());
    }
    if !SUPPORTED_HISTORY_RETENTION_DAYS.contains(&settings.history_retention_days) {
        return Err("历史保留时间仅支持 7 天、30 天、90 天或永久保留".to_owned());
    }
    if !settings.record_text && !settings.record_images && !settings.record_files {
        return Err("请至少选择一种要记录的剪贴板类型".to_owned());
    }
    Ok(())
}

fn has_duplicate_shortcuts(shortcuts: &[&str]) -> bool {
    shortcuts.iter().enumerate().any(|(index, shortcut)| {
        shortcuts[..index]
            .iter()
            .any(|other| shortcuts_equal(shortcut, other))
    })
}

fn shortcuts_equal_to_search(shortcuts: &[&str]) -> bool {
    shortcuts
        .iter()
        .any(|shortcut| shortcuts_equal(shortcut, SEARCH_SHORTCUT))
}

fn shortcuts_equal(left: &str, right: &str) -> bool {
    normalize_shortcut(left) == normalize_shortcut(right)
}

fn normalize_shortcut(shortcut: &str) -> String {
    shortcut
        .split('+')
        .map(|part| {
            let normalized = part.trim().to_ascii_lowercase();
            if normalized == "option" {
                "alt".to_owned()
            } else {
                normalized
            }
        })
        .collect::<Vec<_>>()
        .join("+")
}

#[cfg(test)]
mod tests {
    use super::validate_settings;
    use crate::storage::AppSettings;

    #[test]
    fn option_and_alt_shortcuts_conflict() {
        let mut settings = AppSettings::default();
        settings.main_shortcut = "Alt+1".to_owned();
        settings.quick_tool_shortcuts[0] = "Option+1".to_owned();
        assert!(validate_settings(&settings).is_err());
    }

    #[test]
    fn custom_quick_tool_shortcuts_are_valid() {
        let mut settings = AppSettings::default();
        settings.quick_tool_shortcuts = (1..=5).map(|index| format!("Shift+Alt+{index}")).collect();
        assert!(validate_settings(&settings).is_ok());
    }

    #[test]
    fn sequential_paste_shortcut_must_be_present_and_unique() {
        let mut settings = AppSettings::default();
        settings.sequential_paste_shortcut.clear();
        assert!(validate_settings(&settings).is_err());

        settings.sequential_paste_shortcut = settings.quick_tool_shortcuts[0].clone();
        assert!(validate_settings(&settings).is_err());
    }
}
