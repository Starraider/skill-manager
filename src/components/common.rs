use dioxus::prelude::*;
use std::path::{Path, PathBuf};

#[component]
pub fn StatusPill(
    active: bool,
    #[props(default = "Active".to_string())] active_text: String,
    #[props(default = "Missing".to_string())] missing_text: String,
) -> Element {
    if active {
        rsx! {
            span { class: "badge badge-active", "{active_text}" }
        }
    } else {
        rsx! {
            span { class: "badge badge-missing", "{missing_text}" }
        }
    }
}

#[component]
pub fn PinnedBadge() -> Element {
    rsx! {
        span { class: "badge badge-pinned", "Pinned" }
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
