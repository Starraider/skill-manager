use dioxus::prelude::*;
use std::collections::HashSet;

use crate::components::common::{pick_folder, StatusPill};
use crate::models::config::SourceConfig;
use crate::models::skill::Skill;
use crate::models::state::AppState;
use crate::services::config_store::{expand_tilde, save_config};
use crate::services::fs_scanner::scan_skills;

#[component]
pub fn TabSources() -> Element {
    let mut state: AppState = use_context();
    let config = (state.config)();
    let selected_source_id = (state.selected_source_id)();
    let mut selected_skill_names = state.selected_skill_names;

    let selected_source = config
        .sources
        .iter()
        .find(|s| Some(&s.id) == selected_source_id.as_ref())
        .cloned();

    // Scan skills for the active source
    let discovered_skills: Vec<Skill> = if let Some(ref source) = selected_source {
        scan_skills(&source.path).unwrap_or_default()
    } else {
        Vec::new()
    };

    let on_add_source = move |_| {
        spawn(async move {
            if let Some(folder) = pick_folder("Select Source Directory", None).await {
                let name = folder
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| "Source".to_string());
                let id = format!(
                    "source-{}",
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_millis()
                );

                let mut current_config = (state.config)();
                let new_source = SourceConfig {
                    id: id.clone(),
                    name,
                    path: folder,
                };
                current_config.sources.push(new_source);
                let _ = save_config(&current_config);
                state.config.set(current_config);
                state.selected_source_id.set(Some(id));
            }
        });
    };

    let on_remove_source = {
        let current_id = selected_source_id.clone();
        move |_| {
            if let Some(ref id_to_remove) = current_id {
                let mut current_config = (state.config)();
                current_config.sources.retain(|s| s.id != *id_to_remove);
                let _ = save_config(&current_config);
                state.config.set(current_config.clone());
                state
                    .selected_source_id
                    .set(current_config.sources.first().map(|s| s.id.clone()));
                selected_skill_names.set(HashSet::new());
            }
        }
    };

    let all_selected = !discovered_skills.is_empty()
        && discovered_skills
            .iter()
            .all(|s| selected_skill_names().contains(&s.name));

    let on_toggle_select_all = {
        let skills = discovered_skills.clone();
        move |_| {
            if all_selected {
                selected_skill_names.set(HashSet::new());
            } else {
                let all: HashSet<String> = skills.iter().map(|s| s.name.clone()).collect();
                selected_skill_names.set(all);
            }
        }
    };

    rsx! {
        div { class: "split-layout",
            // Left Pane: Source Folders
            div { class: "pane-left",
                div { class: "pane-header",
                    span { class: "pane-title", "Source Repositories" }
                    div { style: "display: flex; gap: 6px;",
                        button {
                            class: "btn btn-primary btn-sm",
                            onclick: on_add_source,
                            "+ Add"
                        }
                        button {
                            class: "btn btn-danger btn-sm",
                            onclick: on_remove_source,
                            disabled: selected_source_id.is_none(),
                            "Remove"
                        }
                    }
                }
                div { class: "pane-body",
                    if config.sources.is_empty() {
                        div { class: "empty-state",
                            p { "No sources added yet." }
                            button {
                                class: "btn btn-secondary btn-sm",
                                onclick: on_add_source,
                                "Add Source Folder"
                            }
                        }
                    } else {
                        for source in &config.sources {
                            {
                                let is_selected = Some(&source.id) == selected_source_id.as_ref();
                                let is_active = expand_tilde(&source.path).exists();
                                let sid = source.id.clone();
                                rsx! {
                                    div {
                                        key: "{source.id}",
                                        class: if is_selected { "item-card selected" } else { "item-card" },
                                        onclick: move |_| {
                                            state.selected_source_id.set(Some(sid.clone()));
                                            selected_skill_names.set(HashSet::new());
                                        },
                                        div { class: "item-header",
                                            span { class: "item-name", "{source.name}" }
                                            StatusPill { active: is_active }
                                        }
                                        div { class: "item-path", "{source.path.display()}" }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Right Pane: Discovered Skills
            div { class: "pane-right",
                div { class: "pane-header",
                    div { style: "display: flex; align-items: center; gap: 12px;",
                        span { class: "pane-title",
                            if let Some(ref src) = selected_source {
                                "Skills in {src.name} ({discovered_skills.len()})"
                            } else {
                                "Discovered Skills"
                            }
                        }
                    }
                    button {
                        class: "btn btn-primary btn-sm",
                        onclick: move |_| state.is_copy_modal_open.set(true),
                        disabled: selected_skill_names().is_empty(),
                        "Copy Skills ({selected_skill_names().len()})"
                    }
                }

                div { class: "pane-body", style: "padding: 0;",
                    if selected_source.is_none() {
                        div { class: "empty-state",
                            p { "Select or add a source directory on the left to inspect skills." }
                        }
                    } else if discovered_skills.is_empty() {
                        div { class: "empty-state",
                            p { "No valid skills with a SKILL.md marker found in this source." }
                        }
                    } else {
                        div { class: "table-container",
                            table {
                                thead {
                                    tr {
                                        th { style: "width: 40px;",
                                            input {
                                                r#type: "checkbox",
                                                checked: all_selected,
                                                onchange: on_toggle_select_all
                                            }
                                        }
                                        th { "Skill Name" }
                                        th { "Title" }
                                        th { "Description" }
                                        th { "Folder" }
                                    }
                                }
                                tbody {
                                    for skill in &discovered_skills {
                                        {
                                            let s_name = skill.name.clone();
                                            let is_checked = selected_skill_names().contains(&skill.name);
                                            rsx! {
                                                tr { key: "{skill.name}",
                                                    td {
                                                        input {
                                                            r#type: "checkbox",
                                                            checked: is_checked,
                                                            onchange: {
                                                                let name = s_name.clone();
                                                                move |_| {
                                                                    let mut set = selected_skill_names();
                                                                    if set.contains(&name) {
                                                                        set.remove(&name);
                                                                    } else {
                                                                        set.insert(name.clone());
                                                                    }
                                                                    selected_skill_names.set(set);
                                                                }
                                                            }
                                                        }
                                                    }
                                                    td { style: "font-weight: 600;", "{skill.name}" }
                                                    td { "{skill.title.as_deref().unwrap_or(\"-\")}" }
                                                    td { style: "max-width: 320px; color: var(--text-muted);",
                                                        "{skill.description.as_deref().unwrap_or(\"-\")}"
                                                    }
                                                    td { style: "font-size: 11px; color: var(--text-muted);",
                                                        "{skill.folder_path.file_name().unwrap_or_default().to_string_lossy()}"
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
