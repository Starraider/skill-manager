use dioxus::prelude::*;
use std::path::PathBuf;

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

    // Modal state for Add and Edit Target
    let mut is_modal_open = use_signal(|| false);
    let mut modal_is_edit = use_signal(|| false);
    let mut editing_target_id = use_signal(|| None::<String>);
    let mut form_name = use_signal(String::new);
    let mut form_path = use_signal(String::new);
    let mut form_error = use_signal(|| None::<String>);

    let on_open_add = move |_| {
        editing_target_id.set(None);
        form_name.set(String::new());
        form_path.set(String::new());
        form_error.set(None);
        modal_is_edit.set(false);
        is_modal_open.set(true);
    };

    let on_open_edit = {
        let target_opt = selected_target.clone();
        move |_| {
            if let Some(ref t) = target_opt {
                editing_target_id.set(Some(t.id.clone()));
                form_name.set(t.name.clone());
                form_path.set(if t.is_global {
                    "Global user harness directories".to_string()
                } else {
                    t.path.to_string_lossy().to_string()
                });
                form_error.set(None);
                modal_is_edit.set(true);
                is_modal_open.set(true);
            }
        }
    };

    let on_browse_folder = move |_| {
        let cur_path = form_path();
        spawn(async move {
            let start = if !cur_path.is_empty() {
                Some(PathBuf::from(&cur_path))
            } else {
                None
            };
            if let Some(folder) = pick_folder("Select Project Target Directory", start.as_deref()).await {
                let folder_str = folder.to_string_lossy().to_string();
                form_path.set(folder_str);
                if form_name().trim().is_empty() {
                    if let Some(basename) = folder.file_name() {
                        form_name.set(basename.to_string_lossy().to_string());
                    }
                }
                form_error.set(None);
            }
        });
    };

    let on_save_modal = {
        let cur_active_id = active_target_id.clone();
        move || {
            let trimmed_name = form_name().trim().to_string();
            let trimmed_path = form_path().trim().to_string();

            if trimmed_name.is_empty() {
                form_error.set(Some("Please enter a name for the destination folder.".to_string()));
                return;
            }

            let is_editing_global = modal_is_edit()
                && editing_target_id().as_deref() == Some("global");

            if !is_editing_global && trimmed_path.is_empty() {
                form_error.set(Some("Please specify a directory path.".to_string()));
                return;
            }

            let mut current_config = (state.config)();

            if modal_is_edit() {
                let target_id_to_update = editing_target_id().or_else(|| cur_active_id.clone());
                if let Some(ref id) = target_id_to_update {
                    if let Some(t) = current_config.targets.iter_mut().find(|t| t.id == *id) {
                        t.name = trimmed_name.clone();
                        if !t.is_global {
                            t.path = PathBuf::from(trimmed_path);
                        }
                    }
                }
                let _ = save_config(&current_config);
                state.config.set(current_config);
                state
                    .status_message
                    .set(Some(format!("Updated destination '{}'.", trimmed_name)));
            } else {
                let id = format!(
                    "target-{}",
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_millis()
                );
                let new_target = TargetConfig {
                    id: id.clone(),
                    name: trimmed_name.clone(),
                    path: PathBuf::from(trimmed_path),
                    is_global: false,
                    is_pinned: false,
                };
                current_config.targets.push(new_target);
                let _ = save_config(&current_config);
                state.config.set(current_config);
                state.selected_target_id.set(Some(id));
                state
                    .status_message
                    .set(Some(format!("Added destination '{}'.", trimmed_name)));
            }

            is_modal_open.set(false);
        }
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
                            onclick: on_open_add,
                            "+ Add"
                        }
                        button {
                            class: "btn btn-secondary btn-sm",
                            onclick: on_open_edit,
                            disabled: selected_target.is_none(),
                            "Edit"
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
                            let t_clone = target.clone();
                            rsx! {
                                div {
                                    key: "{target.id}",
                                    class: if is_selected { "item-card selected" } else { "item-card" },
                                    onclick: move |_| {
                                        state.selected_target_id.set(Some(tid.clone()));
                                    },
                                    ondblclick: {
                                        let t = t_clone.clone();
                                        move |_| {
                                            editing_target_id.set(Some(t.id.clone()));
                                            form_name.set(t.name.clone());
                                            form_path.set(if t.is_global {
                                                "Global user harness directories".to_string()
                                            } else {
                                                t.path.to_string_lossy().to_string()
                                            });
                                            form_error.set(None);
                                            modal_is_edit.set(true);
                                            is_modal_open.set(true);
                                        }
                                    },
                                    div { class: "item-header",
                                        div { style: "display: flex; align-items: center; gap: 6px; overflow: hidden;",
                                            span { class: "item-name", "{target.name}" }
                                            button {
                                                class: "btn-icon",
                                                title: "Edit Name and Path",
                                                onclick: {
                                                    let t = t_clone.clone();
                                                    move |e| {
                                                        e.stop_propagation();
                                                        state.selected_target_id.set(Some(t.id.clone()));
                                                        editing_target_id.set(Some(t.id.clone()));
                                                        form_name.set(t.name.clone());
                                                        form_path.set(if t.is_global {
                                                            "Global user harness directories".to_string()
                                                        } else {
                                                            t.path.to_string_lossy().to_string()
                                                        });
                                                        form_error.set(None);
                                                        modal_is_edit.set(true);
                                                        is_modal_open.set(true);
                                                    }
                                                },
                                                "✎"
                                            }
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
                    div { style: "display: flex; align-items: center; gap: 10px;",
                        span { class: "pane-title",
                            if let Some(ref target) = selected_target {
                                "Installed Skills in {target.name} ({installed_skills.len()})"
                            } else {
                                "Installed Skills"
                            }
                        }
                        if selected_target.is_some() {
                            button {
                                class: "btn btn-secondary btn-sm",
                                onclick: on_open_edit,
                                title: "Edit Target Name and Path",
                                "✎ Edit"
                            }
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

        // Add / Edit Target Modal
        if is_modal_open() {
            let is_editing_global = modal_is_edit()
                && editing_target_id().as_deref() == Some("global");
            rsx! {
                div {
                    class: "modal-backdrop",
                    onclick: move |_| is_modal_open.set(false),
                    div {
                        class: "modal-content",
                        onclick: move |e| e.stop_propagation(),
                        div { class: "modal-header",
                            h3 { class: "modal-title",
                                if modal_is_edit() { "Edit Destination Folder" } else { "Add Destination Folder" }
                            }
                            button {
                                class: "btn btn-secondary btn-sm",
                                onclick: move |_| is_modal_open.set(false),
                                "✕"
                            }
                        }
                        div { class: "modal-body",
                            div { class: "form-group",
                                label { class: "form-label", "Destination Name" }
                                input {
                                    class: "form-control",
                                    r#type: "text",
                                    placeholder: "e.g. My Project",
                                    value: "{form_name}",
                                    oninput: move |e| {
                                        form_name.set(e.value());
                                        form_error.set(None);
                                    },
                                    onkeydown: {
                                        let mut save = on_save_modal;
                                        move |e| {
                                            if e.key() == Key::Enter {
                                                save();
                                            }
                                        }
                                    },
                                }
                                p { style: "font-size: 11px; color: var(--text-muted); margin-top: 4px;",
                                    "A custom display name independent of the filesystem path."
                                }
                            }
                            div { class: "form-group",
                                label { class: "form-label", "Destination Path" }
                                if !is_editing_global {
                                    div { class: "input-with-button",
                                        input {
                                            class: "form-control",
                                            r#type: "text",
                                            placeholder: "e.g. /Users/svenkalbhenn/Projekte/kunden-projekt",
                                            value: "{form_path}",
                                            oninput: move |e| {
                                                form_path.set(e.value());
                                                form_error.set(None);
                                            },
                                            onkeydown: {
                                                let mut save = on_save_modal;
                                                move |e| {
                                                    if e.key() == Key::Enter {
                                                        save();
                                                    }
                                                }
                                            },
                                        }
                                        button {
                                            class: "btn btn-secondary",
                                            r#type: "button",
                                            onclick: on_browse_folder,
                                            "Browse..."
                                        }
                                    }
                                } else {
                                    input {
                                        class: "form-control",
                                        r#type: "text",
                                        value: "{form_path}",
                                        disabled: true,
                                    }
                                    p { style: "font-size: 11px; color: var(--text-muted); margin-top: 4px;",
                                        "Global target paths are managed per-harness in the Harnesses tab."
                                    }
                                }
                            }
                            if let Some(ref err) = form_error() {
                                div { style: "color: var(--danger); font-size: 12px; margin-top: 8px; font-weight: 500;",
                                    "{err}"
                                }
                            }
                        }
                        div { class: "modal-footer",
                            button {
                                class: "btn btn-secondary",
                                r#type: "button",
                                onclick: move |_| is_modal_open.set(false),
                                "Cancel"
                            }
                            button {
                                class: "btn btn-primary",
                                r#type: "button",
                                onclick: move |_| on_save_modal(),
                                if modal_is_edit() { "Save Changes" } else { "Add Destination" }
                            }
                        }
                    }
                }
            }
        }
    }
}
