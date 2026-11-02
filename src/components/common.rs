use crate::models::state::AppState;
use crate::services::i18n::t;
use dioxus::prelude::*;
use std::path::{Path, PathBuf};

#[component]
pub fn StatusPill(
    active: bool,
    #[props(default)] active_text: Option<String>,
    #[props(default)] missing_text: Option<String>,
) -> Element {
    let state: AppState = use_context();
    let lang = (state.current_language)();

    let display_active = active_text.unwrap_or_else(|| t(lang, "common.active"));
    let display_missing = missing_text.unwrap_or_else(|| t(lang, "common.missing"));

    if active {
        rsx! {
            span { class: "badge badge-active", "{display_active}" }
        }
    } else {
        rsx! {
            span { class: "badge badge-missing", "{display_missing}" }
        }
    }
}

#[component]
pub fn PinnedBadge(#[props(default)] text: Option<String>) -> Element {
    let state: AppState = use_context();
    let lang = (state.current_language)();
    let display_text = text.unwrap_or_else(|| t(lang, "common.pinned"));

    rsx! {
        span { class: "badge badge-pinned", "{display_text}" }
    }
}

#[component]
pub fn HarnessBadge(name: String) -> Element {
    rsx! {
        span { class: "badge badge-harness", "{name}" }
    }
}

/// Helper to trigger a native folder picker dialog.
pub async fn pick_folder(title: &str, starting_directory: Option<&Path>) -> Option<PathBuf> {
    let mut dialog = rfd::AsyncFileDialog::new().set_title(title);
    if let Some(dir) = starting_directory {
        dialog = dialog.set_directory(dir);
    }
    let handle = dialog.pick_folder().await;
    handle.map(|h| h.path().to_path_buf())
}
