// SPDX-License-Identifier: GPL-3.0-or-later
//! Tests for environment variable secret masking (Feature 6 — Env vars section).
//!
//! The `is_secret_env_key` domain function classifies env var keys as secret when
//! they contain PASS, SECRET, KEY, or TOKEN (case-insensitive substring match).
//! Tests also verify that env vars are correctly populated from the mock driver.
use std::sync::Arc;

use gtk_cross_platform::core::domain::container::is_secret_env_key;
use gtk_cross_platform::core::use_cases::container_use_case::ContainerUseCase;
use gtk_cross_platform::infrastructure::containers::mock_driver::MockContainerDriver;
use gtk_cross_platform::ports::use_cases::i_container_use_case::IContainerUseCase;

// ── is_secret_env_key ──────────────────────────────────────────────────────────

#[test]
fn mask_password_suffix() {
    assert!(is_secret_env_key("POSTGRES_PASSWORD"));
    assert!(is_secret_env_key("DB_PASSWORD"));
}

#[test]
fn mask_password_lowercase() {
    assert!(is_secret_env_key("password"));
    assert!(is_secret_env_key("db_password"));
}

#[test]
fn mask_secret_substring() {
    assert!(is_secret_env_key("API_SECRET"));
    assert!(is_secret_env_key("app_secret_value"));
}

#[test]
fn mask_token_substring() {
    assert!(is_secret_env_key("GITHUB_TOKEN"));
    assert!(is_secret_env_key("access_token"));
    assert!(is_secret_env_key("OAUTH_REFRESH_TOKEN"));
}

#[test]
fn mask_key_substring() {
    assert!(is_secret_env_key("AWS_ACCESS_KEY_ID"));
    assert!(is_secret_env_key("PRIVATE_KEY_PATH"));
    assert!(is_secret_env_key("api_key"));
}

#[test]
fn safe_key_not_masked() {
    assert!(!is_secret_env_key("NGINX_HOST"));
    assert!(!is_secret_env_key("TZ"));
    assert!(!is_secret_env_key("PORT"));
    assert!(!is_secret_env_key("LOG_LEVEL"));
    assert!(!is_secret_env_key("WORKER_CONCURRENCY"));
}

#[test]
fn empty_key_not_masked() {
    assert!(!is_secret_env_key(""));
}

// ── env var population from mock driver ───────────────────────────────────────

#[test]
fn container_env_vars_are_populated() {
    let uc = ContainerUseCase::new(Arc::new(MockContainerDriver::new()));
    let containers = uc.list(true).expect("list");
    let web = containers
        .iter()
        .find(|c| c.name == "web-server")
        .expect("find web-server");
    assert!(!web.env.is_empty(), "web-server should have env vars");
    assert!(
        web.env.iter().any(|e| e.starts_with("NGINX_HOST=")),
        "expected NGINX_HOST env var"
    );
}

#[test]
fn db_container_has_secret_env_vars() {
    let uc = ContainerUseCase::new(Arc::new(MockContainerDriver::new()));
    let containers = uc.list(true).expect("list");
    let db = containers.iter().find(|c| c.name == "db").expect("find db");
    assert!(
        db.env.iter().any(|e| e.starts_with("POSTGRES_PASSWORD=")),
        "db should have POSTGRES_PASSWORD"
    );
    // Verify the key is classified as secret
    let key = "POSTGRES_PASSWORD";
    assert!(is_secret_env_key(key));
}

#[test]
fn env_var_key_extraction_works() {
    // Simulate what the UI does: split KEY=VALUE and check the key
    let env_line = "POSTGRES_PASSWORD=secret123";
    let (key, _value) = env_line.split_once('=').unwrap();
    assert!(is_secret_env_key(key));
}

#[test]
fn env_var_with_empty_value_key_check() {
    let env_line = "EMPTY_VAR=";
    let (key, _value) = env_line.split_once('=').unwrap();
    // "EMPTY_VAR" does not contain pass/secret/key/token → not masked
    assert!(!is_secret_env_key(key));
}
