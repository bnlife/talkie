//! Integration tests for settings persistence, especially the
//! `last_active_conversation_id` field.
//!
//! These test the `config` module directly (serde serialization / deserialization)
//! without a Tauri runtime.

use std::path::PathBuf;

use talkie::config;
use talkie::models::Settings;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

use std::sync::atomic::{AtomicU32, Ordering};

static TEST_COUNTER: AtomicU32 = AtomicU32::new(0);

/// Create a unique temporary config file path (one per test invocation).
/// Each call gets a fresh filename so parallel test execution doesn't race.
fn temp_config_path() -> PathBuf {
    let mut p = std::env::temp_dir();
    let n = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
    p.push(format!("talkie_test_config_{}_{}.json", std::process::id(), n));
    // Clean up any leftover from a previous crash.
    let _ = std::fs::remove_file(&p);
    p
}

// ===========================================================================
// Default / new-field compatibility
// ===========================================================================

/// A fresh config (no file on disk) should return default Settings with
/// `last_active_conversation_id = None`.
#[test]
fn test_config_default_none_when_file_missing() {
    let path = temp_config_path();
    // Ensure the file does NOT exist.
    let _ = std::fs::remove_file(&path);

    let settings = config::load(path.clone()).expect("load should succeed for missing file");
    assert_eq!(settings.last_active_conversation_id, None);

    // Clean up.
    let _ = std::fs::remove_file(&path);
}

/// An old-style config JSON *without* `last_active_conversation_id` must
/// still deserialize correctly (serde(default) on the field).
#[test]
fn test_config_backwards_compat_old_format() {
    let path = temp_config_path();
    let old_json = r#"{
        "base_url": "https://api.example.com",
        "api_key": "sk-old",
        "model": "gpt-3.5",
        "temperature": 0.5
    }"#;
    std::fs::write(&path, old_json).expect("write old-format config");

    let settings = config::load(path.clone()).expect("old format should load");
    assert_eq!(settings.base_url, "https://api.example.com");
    assert_eq!(settings.api_key, "sk-old");
    assert_eq!(settings.model, "gpt-3.5");
    assert_eq!(settings.temperature, 0.5);
    // The new field should gracefully default to None.
    assert_eq!(settings.last_active_conversation_id, None);

    let _ = std::fs::remove_file(&path);
}

// ===========================================================================
// Save / Load round-trips
// ===========================================================================

/// Save a config with `last_active_conversation_id = Some(...)`, load it back,
/// and verify the field is preserved.
#[test]
fn test_config_save_and_load_last_active() {
    let path = temp_config_path();

    let mut settings = Settings::default();
    settings.last_active_conversation_id = Some("conv-abc".into());

    config::save(path.clone(), &settings).expect("save should succeed");

    let loaded = config::load(path.clone()).expect("load should succeed");
    assert_eq!(
        loaded.last_active_conversation_id,
        Some("conv-abc".into()),
        "last_active_conversation_id should survive a save/load round-trip"
    );

    let _ = std::fs::remove_file(&path);
}

/// Saving with `last_active_conversation_id = None` should persist as null
/// and reload correctly.
#[test]
fn test_config_save_with_none_last_active() {
    let path = temp_config_path();

    let settings = Settings::default(); // last_active = None

    config::save(path.clone(), &settings).expect("save should succeed");

    let loaded = config::load(path.clone()).expect("load should succeed");
    assert_eq!(loaded.last_active_conversation_id, None);

    let _ = std::fs::remove_file(&path);
}

/// Overwrite an existing `last_active_conversation_id` with a new value.
#[test]
fn test_config_update_last_active() {
    let path = temp_config_path();

    // Save with one value.
    let mut settings = Settings::default();
    settings.last_active_conversation_id = Some("conv-old".into());
    config::save(path.clone(), &settings).expect("first save");

    // Overwrite with a new value.
    settings.last_active_conversation_id = Some("conv-new".into());
    config::save(path.clone(), &settings).expect("second save");

    let loaded = config::load(path.clone()).expect("load after update");
    assert_eq!(
        loaded.last_active_conversation_id,
        Some("conv-new".into()),
        "last_active should reflect the most recent write"
    );

    let _ = std::fs::remove_file(&path);
}

/// Save a full Settings, clear it back to None, and verify.
#[test]
fn test_config_clear_last_active() {
    let path = temp_config_path();

    let mut settings = Settings::default();
    settings.last_active_conversation_id = Some("conv-xyz".into());
    config::save(path.clone(), &settings).expect("save with id");

    settings.last_active_conversation_id = None;
    config::save(path.clone(), &settings).expect("save clearing id");

    let loaded = config::load(path.clone()).expect("load after clear");
    assert_eq!(loaded.last_active_conversation_id, None);

    let _ = std::fs::remove_file(&path);
}

/// Verify the on-disk JSON actually contains the field when it's set.
#[test]
fn test_config_json_contains_last_active() {
    let path = temp_config_path();

    let mut settings = Settings::default();
    settings.last_active_conversation_id = Some("conv-json-check".into());
    config::save(path.clone(), &settings).expect("save");

    let content = std::fs::read_to_string(&path).expect("read config file");
    assert!(
        content.contains("last_active_conversation_id"),
        "JSON should contain the last_active_conversation_id key"
    );
    assert!(
        content.contains("conv-json-check"),
        "JSON should contain the saved conversation id value"
    );

    let _ = std::fs::remove_file(&path);
}

/// Verify the on-disk JSON does NOT contain the field when it's None
/// (serde skips `Option::None` by default, keeping the file clean).
#[test]
fn test_config_json_omits_last_active_when_none() {
    let path = temp_config_path();

    let settings = Settings::default(); // last_active = None
    config::save(path.clone(), &settings).expect("save");

    let content = std::fs::read_to_string(&path).expect("read config file");
    assert!(
        !content.contains("last_active_conversation_id"),
        "JSON should omit the field when last_active is None"
    );

    let _ = std::fs::remove_file(&path);
}
