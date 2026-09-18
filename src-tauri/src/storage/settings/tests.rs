use super::{
    default_quick_tool_ids, default_quick_tool_shortcuts, migrate_legacy_default_quick_tools,
    migrate_legacy_quick_tool_shortcuts, AppSettings, QuickToolId,
};

#[test]
fn default_main_shortcut_matches_the_current_platform() {
    assert_eq!(
        AppSettings::default().main_shortcut,
        crate::platform::DEFAULT_MAIN_SHORTCUT
    );
}

#[test]
fn settings_without_quick_tools_receive_the_default_order() {
    let mut value = serde_json::to_value(AppSettings::default()).unwrap();
    value.as_object_mut().unwrap().remove("quickToolIds");
    let settings = serde_json::from_value::<AppSettings>(value).unwrap();
    assert_eq!(settings.quick_tool_ids, default_quick_tool_ids());
}

#[test]
fn settings_without_quick_tool_shortcuts_receive_command_defaults() {
    let mut value = serde_json::to_value(AppSettings::default()).unwrap();
    value.as_object_mut().unwrap().remove("quickToolShortcuts");
    let settings = serde_json::from_value::<AppSettings>(value).unwrap();
    assert_eq!(
        settings.quick_tool_shortcuts,
        vec![
            "Command+1",
            "Command+2",
            "Command+3",
            "Command+4",
            "Command+5"
        ]
    );
}

#[test]
fn quick_tool_ids_use_frontend_names() {
    assert_eq!(
        serde_json::to_value(QuickToolId::TextDiff).unwrap(),
        "text-diff"
    );
    assert_eq!(serde_json::to_value(QuickToolId::Base64).unwrap(), "base64");
    assert_eq!(
        serde_json::to_value(QuickToolId::SequentialPaste).unwrap(),
        "sequential-paste"
    );
}

#[test]
fn legacy_alt_quick_tool_shortcuts_migrate_to_command_defaults() {
    let mut settings = AppSettings::default();
    settings.quick_tool_shortcuts = (1..=5).map(|index| format!("Alt+{index}")).collect();
    assert!(migrate_legacy_quick_tool_shortcuts(&mut settings));
    assert_eq!(
        settings.quick_tool_shortcuts,
        default_quick_tool_shortcuts()
    );
}

#[test]
fn custom_quick_tool_shortcuts_are_not_migrated() {
    let mut settings = AppSettings::default();
    settings.quick_tool_shortcuts[0] = "Alt+Q".to_owned();
    let original = settings.quick_tool_shortcuts.clone();
    assert!(!migrate_legacy_quick_tool_shortcuts(&mut settings));
    assert_eq!(settings.quick_tool_shortcuts, original);
}

#[test]
fn previous_default_tools_migrate_to_sequential_paste() {
    let mut settings = AppSettings::default();
    settings.quick_tool_ids = vec![
        QuickToolId::TextDiff,
        QuickToolId::Json,
        QuickToolId::Url,
        QuickToolId::Base64,
        QuickToolId::Date,
    ];
    assert!(migrate_legacy_default_quick_tools(&mut settings));
    assert_eq!(settings.quick_tool_ids, default_quick_tool_ids());
}
