mod common;

use skill_manager::models::config::{AppConfig, HarnessConfig, SourceConfig, TargetConfig};
use skill_manager::services::config_store::{load_config_from_path, save_config_to_path};
use skill_manager::services::i18n::{t, tr, Language};
use std::path::PathBuf;

#[test]
fn test_language_from_code_and_labels() {
    assert_eq!(Language::from_code("en"), Language::English);
    assert_eq!(Language::from_code("de"), Language::Deutsch);
    assert_eq!(Language::from_code("DE"), Language::Deutsch);
    assert_eq!(Language::from_code("deutsch"), Language::Deutsch);
    assert_eq!(Language::from_code("german"), Language::Deutsch);
    assert_eq!(Language::from_code("unknown"), Language::English);

    assert_eq!(Language::English.label(), "English");
    assert_eq!(Language::Deutsch.label(), "Deutsch");

    assert_eq!(Language::English.code(), "en");
    assert_eq!(Language::Deutsch.code(), "de");
}

#[test]
fn test_ui_labels_and_headings_switching() {
    // Top-level menu and tabs
    assert_eq!(t(Language::English, "menu.language"), "Language");
    assert_eq!(t(Language::Deutsch, "menu.language"), "Sprache");

    assert_eq!(t(Language::English, "tabs.sources"), "1. Sources");
    assert_eq!(t(Language::Deutsch, "tabs.sources"), "1. Quellen");

    assert_eq!(t(Language::English, "tabs.targets"), "2. Targets");
    assert_eq!(t(Language::Deutsch, "tabs.targets"), "2. Zielumgebungen");

    assert_eq!(t(Language::English, "tabs.harnesses"), "3. Harnesses");
    assert_eq!(t(Language::Deutsch, "tabs.harnesses"), "3. Harnesses");

    // Common labels
    assert_eq!(t(Language::English, "common.active"), "Active");
    assert_eq!(t(Language::Deutsch, "common.active"), "Aktiv");
    assert_eq!(t(Language::English, "common.missing"), "Missing");
    assert_eq!(t(Language::Deutsch, "common.missing"), "Fehlt");
    assert_eq!(t(Language::English, "common.pinned"), "Pinned");
    assert_eq!(t(Language::Deutsch, "common.pinned"), "Fixiert");
    assert_eq!(t(Language::English, "common.cancel"), "Cancel");
    assert_eq!(t(Language::Deutsch, "common.cancel"), "Abbrechen");
    assert_eq!(t(Language::English, "common.save_changes"), "Save Changes");
    assert_eq!(t(Language::Deutsch, "common.save_changes"), "Änderungen speichern");

    // Sources headings and buttons
    assert_eq!(t(Language::English, "sources.title"), "Source Repositories");
    assert_eq!(t(Language::Deutsch, "sources.title"), "Quell-Repositories");
    assert_eq!(t(Language::English, "sources.btn_add"), "+ Add");
    assert_eq!(t(Language::Deutsch, "sources.btn_add"), "+ Hinzufügen");
    assert_eq!(t(Language::English, "sources.btn_edit"), "Edit");
    assert_eq!(t(Language::Deutsch, "sources.btn_edit"), "Bearbeiten");
    assert_eq!(t(Language::English, "sources.btn_remove"), "Remove");
    assert_eq!(t(Language::Deutsch, "sources.btn_remove"), "Entfernen");
    assert_eq!(t(Language::English, "sources.empty"), "No sources added yet.");
    assert_eq!(t(Language::Deutsch, "sources.empty"), "Noch keine Quellen hinzugefügt.");

    // Targets headings and buttons
    assert_eq!(t(Language::English, "targets.title"), "Target Environments");
    assert_eq!(t(Language::Deutsch, "targets.title"), "Zielumgebungen");
    assert_eq!(t(Language::English, "targets.btn_add"), "+ Add");
    assert_eq!(t(Language::Deutsch, "targets.btn_add"), "+ Hinzufügen");
    assert_eq!(t(Language::English, "targets.btn_remove_skill"), "Remove Skill");
    assert_eq!(t(Language::Deutsch, "targets.btn_remove_skill"), "Skill entfernen");

    // Harnesses headings and buttons
    assert_eq!(t(Language::English, "harnesses.title"), "Configured Harnesses");
    assert_eq!(t(Language::Deutsch, "harnesses.title"), "Konfigurierte Harnesses");
    assert_eq!(t(Language::English, "harnesses.lbl_display_name"), "Harness Display Name");
    assert_eq!(t(Language::Deutsch, "harnesses.lbl_display_name"), "Harness-Anzeigename");

    // Copy modal headings and buttons
    assert_eq!(t(Language::English, "copy_modal.title"), "Copy Skills to Targets");
    assert_eq!(t(Language::Deutsch, "copy_modal.title"), "Skills in Zielumgebungen kopieren");
    assert_eq!(t(Language::English, "copy_modal.btn_deploy"), "Deploy Skills");
    assert_eq!(t(Language::Deutsch, "copy_modal.btn_deploy"), "Skills bereitstellen");
}

#[test]
fn test_placeholder_substitutions() {
    // Sources placeholder substitution
    let en_skills_heading = tr(
        Language::English,
        "sources.skills_heading",
        &[("name", "TYPO3 Source"), ("count", "5")],
    );
    assert_eq!(en_skills_heading, "Skills in TYPO3 Source (5)");

    let de_skills_heading = tr(
        Language::Deutsch,
        "sources.skills_heading",
        &[("name", "TYPO3 Source"), ("count", "5")],
    );
    assert_eq!(de_skills_heading, "Skills in TYPO3 Source (5)");

    // Copy skills button count placeholder
    assert_eq!(
        tr(Language::English, "sources.btn_copy_skills", &[("count", "3")]),
        "Copy Skills (3)"
    );
    assert_eq!(
        tr(Language::Deutsch, "sources.btn_copy_skills", &[("count", "3")]),
        "Skills kopieren (3)"
    );

    // Targets installed skills heading
    assert_eq!(
        tr(
            Language::English,
            "targets.skills_heading",
            &[("name", "Project Alpha"), ("count", "12")]
        ),
        "Installed Skills in Project Alpha (12)"
    );
    assert_eq!(
        tr(
            Language::Deutsch,
            "targets.skills_heading",
            &[("name", "Project Alpha"), ("count", "12")]
        ),
        "Installierte Skills in Project Alpha (12)"
    );

    // Removal status message with skill name placeholder
    assert_eq!(
        tr(
            Language::English,
            "targets.msg_remove_success",
            &[("name", "audit-tool")]
        ),
        "Successfully removed skill 'audit-tool' from all harnesses."
    );
    assert_eq!(
        tr(
            Language::Deutsch,
            "targets.msg_remove_success",
            &[("name", "audit-tool")]
        ),
        "Skill 'audit-tool' erfolgreich aus allen Harnesses entfernt."
    );

    // Copy modal result counts
    assert_eq!(
        tr(Language::English, "copy_modal.msg_copied", &[("count", "4")]),
        "Copied 4 skill destinations."
    );
    assert_eq!(
        tr(Language::Deutsch, "copy_modal.msg_copied", &[("count", "4")]),
        "4 Skill-Ziele kopiert."
    );
}

#[test]
fn test_language_persistence_in_config() {
    let mock_dir = common::create_mock_config_dir();
    let config_path = mock_dir.path().join("config.json");

    let mut config = AppConfig::default_seeded();
    assert_eq!(config.language, Language::English);

    // Switch to Deutsch and persist
    config.language = Language::Deutsch;
    save_config_to_path(&config, &config_path).expect("Saving config should succeed");

    let reloaded = load_config_from_path(&config_path).expect("Loading config should succeed");
    assert_eq!(reloaded.language, Language::Deutsch);
}

#[test]
fn test_user_defined_names_remain_independent_of_app_language() {
    let mock_dir = common::create_mock_config_dir();
    let config_path = mock_dir.path().join("config.json");

    let mut config = AppConfig::default_seeded();
    config.language = Language::Deutsch;

    // The user enters custom names in their own language / choice
    config.sources.push(SourceConfig {
        id: "source-1".to_string(),
        name: "Meine benutzerdefinierte Quelle".to_string(),
        path: PathBuf::from("/sources/custom"),
    });

    config.targets.push(TargetConfig {
        id: "target-1".to_string(),
        name: "Kundenprojekt Hauptseite".to_string(),
        path: PathBuf::from("/projects/client"),
        is_global: false,
        is_pinned: false,
    });

    config.harnesses.push(HarnessConfig {
        id: "harness-1".to_string(),
        name: "Custom Harness Environment".to_string(),
        relative_path: PathBuf::from(".custom/skills"),
        global_path: None,
        is_pinned: false,
    });

    save_config_to_path(&config, &config_path).expect("Save should succeed");

    // Load with English
    let mut loaded = load_config_from_path(&config_path).expect("Load should succeed");
    assert_eq!(loaded.sources[0].name, "Meine benutzerdefinierte Quelle");
    assert_eq!(loaded.targets[1].name, "Kundenprojekt Hauptseite");
    let custom_harness_idx = loaded.harnesses.len() - 1;
    assert_eq!(loaded.harnesses[custom_harness_idx].name, "Custom Harness Environment");

    // Switch UI language to English - user text fields remain unaffected
    loaded.language = Language::English;
    save_config_to_path(&loaded, &config_path).expect("Save should succeed");

    let reloaded = load_config_from_path(&config_path).expect("Reload should succeed");
    assert_eq!(reloaded.language, Language::English);
    assert_eq!(reloaded.sources[0].name, "Meine benutzerdefinierte Quelle");
    assert_eq!(reloaded.targets[1].name, "Kundenprojekt Hauptseite");
    assert_eq!(reloaded.harnesses[custom_harness_idx].name, "Custom Harness Environment");
}

#[test]
fn test_left_pane_width_constraints_and_auto_adjust() {
    use skill_manager::services::i18n::{default_left_pane_width, min_left_pane_width};

    let de_min = min_left_pane_width(Language::Deutsch);
    let en_min = min_left_pane_width(Language::English);

    // German requires wider constraint (at least 450px) to prevent button truncation
    assert!(de_min >= 450.0);
    assert!(en_min >= 340.0);
    assert!(de_min > en_min);

    let de_default = default_left_pane_width(Language::Deutsch);
    assert!(de_default >= de_min);

    // Auto-adjust simulation: switching from English width (340px) to German must expand to at least de_min
    let mut current_width = 340.0;
    if current_width < de_min {
        current_width = de_min;
    }
    assert_eq!(current_width, de_min);
    assert!(current_width >= 450.0);
}

#[test]
fn test_left_pane_width_persistence() {
    let mock_dir = common::create_mock_config_dir();
    let config_path = mock_dir.path().join("config.json");

    let mut config = AppConfig::default_seeded();
    assert_eq!(config.left_pane_width, None);

    config.left_pane_width = Some(480.0);
    save_config_to_path(&config, &config_path).expect("Saving config should succeed");

    let reloaded = load_config_from_path(&config_path).expect("Loading config should succeed");
    assert_eq!(reloaded.left_pane_width, Some(480.0));
}


