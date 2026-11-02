use dioxus::prelude::*;

use crate::components::common::{pick_folder, HarnessBadge, PinnedBadge, StatusPill};
use crate::models::config::TargetConfig;
use crate::models::skill::InstalledSkill;
use crate::models::state::AppState;
use crate::services::config_store::{expand_tilde, save_config};
use crate::services::deployer::remove_skill_from_target;
use crate::services::fs_scanner::scan_installed_skills;

#[component]
pub fn TabTargets() -> Element {
    let mut state: AppState = use_context();
    let config = (state.config)();
    let selected_target_id = (state.selected_target_id)();

    // Ensure a default target is selected if none currently selected
    let active_target_id =
        selected_target_id.or_else(|| config.targets.first().map(|t| t.id.clone()));

    let selected_target = config
        .targets
        .iter()
        .find(|t| Some(&t.id) == active_target_id.as_ref())
        .cloned();

    // Revision counter to trigger re-scanning after skill removal
    let mut scan_rev = use_signal(|| 0);

    let installed_skills: Vec<InstalledSkill> = if let Some(ref target) = selected_target {
        let _ = scan_rev(); // track revision
        scan_installed_skills(target, &config.harnesses)
    } else {
        Vec::new()
    };

    let on_add_target = move |_| {
        spawn(async move {
            if let Some(folder) = pick_folder("Select Project Target Directory", None).await {
                let name = folder
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| "Target Project".to_string());
                let id = format!(
                    "target-{}",
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_millis()
                );

                let mut current_config = (state.config)();
                let new_target = TargetConfig {
                    id: id.clone(),
                    name,
                    path: folder,
                    is_global: false,
                    is_pinned: false,
                };
                current_config.targets.push(new_target);
                let _ = save_config(&current_config);
                state.config.set(current_config);
                state.selected_target_id.set(Some(id));
            }
        });
    };

    let on_remove_target = {
        let current_target = selected_target.clone();
        move |_| {
            if let Some(ref target) = current_target {
                if target.is_pinned {
                    return;
                }
                let mut current_config = (state.config)();
                current_config.targets.retain(|t| t.id != target.id);
                let _ = save_config(&current_config);
                state.config.set(current_config.clone());
                state
                    .selected_target_id
                    .set(current_config.targets.first().map(|t| t.id.clone()));
            }
        }
    };

    rsx! {
        div { class: "split-layout",
            // Left Pane: Targets List
            div { class: "pane-left",
                div { class: "pane-header",
                    span { class: "pane-title", "Target Environments" }
                    div { style: "display: flex; gap: 6px;",
                        button {
                            class: "btn btn-primary btn-sm",
                            onclick: on_add_target,
                            "+ Add"
                        }
                        button {
                            class: "btn btn-danger btn-sm",
                            onclick: on_remove_target,
                            disabled: selected_target.as_ref().map(|t| t.is_pinned).unwrap_or(true),
                            "Remove"
                        }
                    }
                }
                div { class: "pane-body",
                    for target in &config.targets {
                        {
                            let is_selected = Some(&target.id) == active_target_id.as_ref();
                            let is_active = target.is_global || expand_tilde(&target.path).exists();
                            let tid = target.id.clone();
                            rsx! {
                                div {
                                    key: "{target.id}",
                                    class: if is_selected { "item-card selected" } else { "item-card" },
                                    onclick: move |_| {
                                        state.selected_target_id.set(Some(tid.clone()));
                                    },
                                    div { class: "item-header",
                                        div { style: "display: flex; align-items: center; gap: 6px;",
                                            span { class: "item-name", "{target.name}" }
                                            if target.is_pinned {
                                                PinnedBadge {}
                                            }
                                        }
                                        StatusPill { active: is_active }
                                    }
                                    if target.is_global {
                                        div { class: "item-path", "Global user harness directories" }
                                    } else {
                                        div { class: "item-path", "{target.path.display()}" }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Right Pane: Installed Skills
            div { class: "pane-right",
                div { class: "pane-header",
                    span { class: "pane-title",
                        if let Some(ref target) = selected_target {
                            "Installed Skills in {target.name} ({installed_skills.len()})"
                        } else {
                            "Installed Skills"
                        }
                    }
                }

                div { class: "pane-body", style: "padding: 0;",
                    if selected_target.is_none() {
                        div { class: "empty-state",
                            p { "Select a target on the left to inspect installed skills." }
                        }
                    } else if installed_skills.is_empty() {
                        div { class: "empty-state",
                            p { "No skills currently installed in this target." }
                        }
                    } else {
                        div { class: "table-container",
                            table {
                                thead {
                                    tr {
                                        th { "Skill Name" }
                                        th { "Configured Harnesses" }
                                        th { "Installations" }
                                        th { style: "text-align: right;", "Actions" }
                                    }
                                }
                                tbody {
                                    for skill in &installed_skills {
                                        {
                                            let s_name = skill.name.clone();
                                            let target_opt = selected_target.clone();
                                            let harnesses_clone = config.harnesses.clone();
                                            rsx! {
                                                tr { key: "{skill.name}",
                                                    td { style: "font-weight: 600;", "{skill.name}" }
                                                    td {
                                                        for h in &skill.harness_names {
                                                            HarnessBadge { key: "{h}", name: h.clone() }
                                                        }
                                                    }
                                                    td { style: "color: var(--text-muted); font-size: 12px;",
                                                        "{skill.installed_paths.len()} path(s)"
                                                    }
                                                    td { style: "text-align: right;",
                                                        button {
                                                            class: "btn btn-danger btn-sm",
                                                            onclick: {
                                                                let name = s_name.clone();
                                                                let target_clone = target_opt.clone();
                                                                let harnesses = harnesses_clone.clone();
                                                                move |_| {
                                                                    if let Some(ref t) = target_clone {
                                                                        let target = t.clone();
                                                                        let h_list = harnesses.clone();
                                                                        let skill_to_delete = name.clone();
                                                                        spawn(async move {
                                                                            let s_del = skill_to_delete.clone();
                                                                            let res = tokio::task::spawn_blocking(move || {
                                                                                remove_skill_from_target(&s_del, &target, &h_list)
                                                                            })
                                                                            .await;

                                                                            match res {
                                                                                Ok(Ok(())) => {
                                                                                    state.status_message.set(Some(format!(
                                                                                        "Successfully removed skill '{}' from all harnesses.",
                                                                                        skill_to_delete
                                                                                    )));
                                                                                    scan_rev.set(scan_rev() + 1);
                                                                                }
                                                                                Ok(Err(e)) => {
                                                                                    state.status_message.set(Some(format!("Failed to remove skill: {}", e)));
                                                                                }
                                                                                Err(e) => {
                                                                                    state.status_message.set(Some(format!("Internal task error: {}", e)));
                                                                                }
                                                                            }
                                                                        });
                                                                    }
                                                                }
                                                            },
                                                            "Remove Skill"
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
}
