mod common;

use skill_manager::models::config::AppConfig;
use skill_manager::services::config_store::{
    expand_tilde, load_config_from_path, save_config_to_path,
};
use std::fs;
use std::path::{Path, PathBuf};

#[test]
fn test_tilde_expansion_resolves_home_dir() {
    let home = dirs::home_dir().expect("Home directory should be resolvable");

    let path_with_tilde_slash = Path::new("~/projects/skill-manager");
    let expanded1 = expand_tilde(path_with_tilde_slash);
    assert_eq!(expanded1, home.join("projects/skill-manager"));

    let path_tilde_only = Path::new("~");
    let expanded2 = expand_tilde(path_tilde_only);
    assert_eq!(expanded2, home);

    let normal_path = Path::new("/var/log/app.log");
    let expanded3 = expand_tilde(normal_path);
    assert_eq!(expanded3, PathBuf::from("/var/log/app.log"));
}

#[test]
fn test_default_config_seeding_when_file_missing() {
    let mock_dir = common::create_mock_config_dir();
    let config_path = mock_dir.path().join("non_existent_config.json");

    assert!(!config_path.exists());
    let loaded = load_config_from_path(&config_path).expect("Should succeed with default config");

    assert_eq!(loaded, AppConfig::default_seeded());
    assert_eq!(loaded.harnesses.len(), 4);
    assert!(loaded.targets.iter().any(|t| t.is_global && t.is_pinned));
}

#[test]
fn test_atomic_config_save_replaces_cleanly() {
    let mock_dir = common::create_mock_config_dir();
    let config_path = mock_dir.path().join("config.json");

    let mut config = AppConfig::default_seeded();
    config
        .sources
        .push(skill_manager::models::config::SourceConfig {
            id: "source-1".to_string(),
            name: "Test Source".to_string(),
            path: PathBuf::from("/mock/source"),
        });

    save_config_to_path(&config, &config_path).expect("Saving config should succeed");
    assert!(config_path.exists());

    let loaded = load_config_from_path(&config_path).expect("Loading config should succeed");
    assert_eq!(loaded.sources.len(), 1);
    assert_eq!(loaded.sources[0].name, "Test Source");

    // Overwrite with modified config
    config.sources[0].name = "Updated Source Name".to_string();
    save_config_to_path(&config, &config_path).expect("Re-saving config should succeed atomically");

    let reloaded = load_config_from_path(&config_path).expect("Reloading should succeed");
    assert_eq!(reloaded.sources[0].name, "Updated Source Name");
}

#[test]
fn test_corrupt_config_recovers_to_default() {
    let mock_dir = common::create_mock_config_dir();
    let config_path = mock_dir.path().join("corrupt_config.json");

    fs::write(&config_path, "{ broken json content: true,,, }").expect("Write corrupt file");
    let loaded = load_config_from_path(&config_path)
        .expect("Loading corrupt config should recover to default");

    assert_eq!(loaded, AppConfig::default_seeded());
}
