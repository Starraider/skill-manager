use dioxus::prelude::*;
use std::path::PathBuf;

use crate::components::common::pick_folder;
use crate::models::config::HarnessConfig;
use crate::models::state::AppState;
use crate::services::config_store::save_config;

#[component]
pub fn TabHarnesses() -> Element {
    let mut state: AppState = use_context();
    let config = (state.config)();

    let mut selected_harness_id = use_signal(|| config.harnesses.first().map(|h| h.id.clone()));

    let selected_harness = config
        .harnesses
        .iter()
        .find(|h| Some(&h.id) == selected_harness_id().as_ref())
        .cloned();

    // Local form state for the selected harness
    let mut form_name = use_signal(String::new);
    let mut form_rel_path = use_signal(String::new);
    let mut form_global_path = use_signal(String::new);

    // Sync form state when selection changes
    use_effect(move || {
        let current_id = selected_harness_id();
        let cfg = (state.config)();
        if let Some(h) = cfg
            .harnesses
            .iter()
            .find(|h| Some(&h.id) == current_id.as_ref())
        {
            form_name.set(h.name.clone());
            form_rel_path.set(h.relative_path.to_string_lossy().to_string());
            form_global_path.set(
                h.global_path
                    .as_ref()
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_default(),
            );
        }
    });

    let on_add_harness = move |_| {
        let mut current_config = (state.config)();
        let id = format!(
            "harness-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis()
        );
        let new_harness = HarnessConfig {
            id: id.clone(),
            name: "New Harness".to_string(),
            relative_path: PathBuf::from(".new-harness/skills"),
            global_path: None,
            is_pinned: false,
        };
        current_config.harnesses.push(new_harness);
        let _ = save_config(&current_config);
        state.config.set(current_config);
        selected_harness_id.set(Some(id));
    };

    let on_remove_harness = {
        let current_id = selected_harness_id();
        move |_| {
            if let Some(ref id) = current_id {
                let mut current_config = (state.config)();
                current_config.harnesses.retain(|h| h.id != *id);
                let _ = save_config(&current_config);
                state.config.set(current_config.clone());
                selected_harness_id.set(current_config.harnesses.first().map(|h| h.id.clone()));
            }
        }
    };

    let on_save_changes = move |_| {
        if let Some(ref id) = selected_harness_id() {
            let mut current_config = (state.config)();
            if let Some(h) = current_config.harnesses.iter_mut().find(|h| h.id == *id) {
                h.name = form_name();
                h.relative_path = PathBuf::from(form_rel_path());
                let gp_str = form_global_path().trim().to_string();
                h.global_path = if gp_str.is_empty() {
                    None
                } else {
                    Some(PathBuf::from(gp_str))
                };
            }
            let _ = save_config(&current_config);
            state.config.set(current_config);
            state
                .status_message
                .set(Some("Harness configuration saved.".to_string()));
        }
    };

    let on_pick_global_path = move |_| {
        spawn(async move {
            if let Some(folder) = pick_folder("Select Global Harness Directory", None).await {
                form_global_path.set(folder.to_string_lossy().to_string());
            }
        });
    };

    rsx! {
        div { class: "split-layout",
            // Left Pane: Harness List
            div { class: "pane-left",
                div { class: "pane-header",
                    span { class: "pane-title", "Configured Harnesses" }
                    div { style: "display: flex; gap: 6px;",
                        button {
                            class: "btn btn-primary btn-sm",
                            onclick: on_add_harness,
                            "+ Add"
                        }
                        button {
                            class: "btn btn-danger btn-sm",
                            onclick: on_remove_harness,
                            disabled: selected_harness.is_none(),
                            "Remove"
                        }
                    }
                }
                div { class: "pane-body",
                    for harness in &config.harnesses {
                        {
                            let is_selected = Some(&harness.id) == selected_harness_id().as_ref();
                            let hid = harness.id.clone();
                            rsx! {
                                div {
                                    key: "{harness.id}",
                                    class: if is_selected { "item-card selected" } else { "item-card" },
                                    onclick: move |_| {
                                        selected_harness_id.set(Some(hid.clone()));
                                    },
                                    div { class: "item-header",
                                        span { class: "item-name", "{harness.name}" }
                                    }
                                    div { class: "item-path", "{harness.relative_path.display()}" }
                                }
                            }
                        }
                    }
                }
            }

            // Right Pane: Edit Harness Form
            div { class: "pane-right",
                div { class: "pane-header",
                    span { class: "pane-title",
                        if let Some(ref h) = selected_harness {
                            "Configure Harness: {h.name}"
                        } else {
                            "Harness Details"
                        }
                    }
                    button {
                        class: "btn btn-primary btn-sm",
                        onclick: on_save_changes,
                        disabled: selected_harness.is_none(),
                        "Save Changes"
                    }
                }

                div { class: "pane-body", style: "padding: 24px; max-width: 640px;",
                    if selected_harness.is_none() {
                        div { class: "empty-state",
                            p { "Select a harness on the left to configure its directory paths." }
                        }
                    } else {
                        div {
                            div { class: "form-group",
                                label { class: "form-label", "Harness Display Name" }
                                input {
                                    class: "form-control",
                                    r#type: "text",
                                    value: "{form_name}",
                                    oninput: move |e| form_name.set(e.value())
                                }
                            }

                            div { class: "form-group",
                                label { class: "form-label", "Relative Project Subfolder Path" }
                                input {
                                    class: "form-control",
                                    r#type: "text",
                                    placeholder: "e.g. .claude/skills",
                                    value: "{form_rel_path}",
                                    oninput: move |e| form_rel_path.set(e.value())
                                }
                                p { style: "font-size: 11px; color: var(--text-muted); margin-top: 4px;",
                                    "Relative subpath inside project repositories where skills should be deployed."
                                }
                            }

                            div { class: "form-group",
                                label { class: "form-label", "Global Harness Directory Path (Optional)" }
                                div { class: "input-with-button",
                                    input {
                                        class: "form-control",
                                        r#type: "text",
                                        placeholder: "e.g. ~/.claude/skills",
                                        value: "{form_global_path}",
                                        oninput: move |e| form_global_path.set(e.value())
                                    }
                                    button {
                                        class: "btn btn-secondary",
                                        onclick: on_pick_global_path,
                                        "Browse..."
                                    }
                                }
                                p { style: "font-size: 11px; color: var(--text-muted); margin-top: 4px;",
                                    "System-wide folder where skills are installed when deploying to the Global target."
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
