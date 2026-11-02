use dioxus::prelude::*;
use std::path::PathBuf;

use crate::components::common::{pick_folder, HarnessBadge, PinnedBadge, StatusPill};
use crate::components::resize_handle::ResizeHandle;
use crate::models::config::TargetConfig;
use crate::models::skill::InstalledSkill;
use crate::models::state::AppState;
use crate::services::config_store::{expand_tilde, save_config};
use crate::services::deployer::remove_skill_from_target;
use crate::services::fs_scanner::scan_installed_skills;
use crate::services::i18n::{min_left_pane_width, t, tr};

fn open_target_add_dialog(
    mut editing_id: Signal<Option<String>>,
    mut name: Signal<String>,
    mut path: Signal<String>,
    mut error: Signal<Option<String>>,
    mut is_edit: Signal<bool>,
    mut is_open: Signal<bool>,
) {
    editing_id.set(None);
    name.set(String::new());
    path.set(String::new());
    error.set(None);
    is_edit.set(false);
    is_open.set(true);
}

fn open_target_edit_dialog(
    target: &TargetConfig,
    mut editing_id: Signal<Option<String>>,
    mut name: Signal<String>,
    mut path: Signal<String>,
    mut error: Signal<Option<String>>,
    mut is_edit: Signal<bool>,
    mut is_open: Signal<bool>,
) {
    editing_id.set(Some(target.id.clone()));
    name.set(target.name.clone());
    path.set(if target.is_global {
        "Global user harness directories".to_string()
    } else {
        target.path.to_string_lossy().to_string()
    });
    error.set(None);
    is_edit.set(true);
    is_open.set(true);
}

fn save_target_modal(
    mut state: AppState,
    mut is_modal_open: Signal<bool>,
    modal_is_edit: Signal<bool>,
    editing_target_id: Signal<Option<String>>,
    active_target_id: Option<String>,
    form_name: Signal<String>,
    form_path: Signal<String>,
    mut form_error: Signal<Option<String>>,
) {
    let lang = (state.current_language)();
    let trimmed_name = form_name().trim().to_string();
    let trimmed_path = form_path().trim().to_string();

    if trimmed_name.is_empty() {
        form_error.set(Some(t(lang, "targets.err_name_required")));
        return;
    }

    let is_editing_global = modal_is_edit()
        && editing_target_id().as_deref() == Some("global");

    if !is_editing_global && trimmed_path.is_empty() {
        form_error.set(Some(t(lang, "targets.err_path_required")));
        return;
    }

    let mut current_config = (state.config)();

    if modal_is_edit() {
        let target_id_to_update = editing_target_id().or(active_target_id);
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
            .set(Some(tr(lang, "targets.msg_updated", &[("name", &trimmed_name)])));
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
            .set(Some(tr(lang, "targets.msg_added", &[("name", &trimmed_name)])));
    }

    is_modal_open.set(false);
}

#[component]
pub fn TabTargets() -> Element {
    let mut state: AppState = use_context();
    let lang = (state.current_language)();
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
    let modal_is_edit = use_signal(|| false);
    let editing_target_id = use_signal(|| None::<String>);
    let mut form_name = use_signal(String::new);
    let mut form_path = use_signal(String::new);
    let mut form_error = use_signal(|| None::<String>);

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

    let selected_target_for_header = selected_target.clone();
    let is_editing_global = modal_is_edit() && editing_target_id().as_deref() == Some("global");
    let pane_width = (state.left_pane_width)();
    let min_width = min_left_pane_width(lang);
    let installed_title = if let Some(ref target) = selected_target {
        tr(lang, "targets.skills_heading", &[("name", &target.name), ("count", &installed_skills.len().to_string())])
    } else {
        t(lang, "targets.installed_skills")
    };

    rsx! {
        div { class: "split-layout",
            // Left Pane: Targets List
            div {
                class: "pane-left",
                style: "width: {pane_width}px; min-width: {min_width}px;",
                div { class: "pane-header",
                    span { class: "pane-title", "{t(lang, \"targets.title\")}" }
                    div { class: "pane-header-actions",
                        button {
                            class: "btn btn-primary btn-sm",
                            onclick: move |_| {
                                open_target_add_dialog(
                                    editing_target_id,
                                    form_name,
                                    form_path,
                                    form_error,
                                    modal_is_edit,
                                    is_modal_open,
                                );
                            },
                            "{t(lang, \"targets.btn_add\")}"
                        }
                        button {
                            class: "btn btn-secondary btn-sm",
                            onclick: {
                                let target_opt = selected_target.clone();
                                move |_| {
                                    if let Some(ref t) = target_opt {
                                        open_target_edit_dialog(
                                            t,
                                            editing_target_id,
                                            form_name,
                                            form_path,
                                            form_error,
                                            modal_is_edit,
                                            is_modal_open,
                                        );
                                    }
                                }
                            },
                            disabled: selected_target.is_none(),
                            "{t(lang, \"targets.btn_edit\")}"
                        }
                        button {
                            class: "btn btn-danger btn-sm",
                            onclick: on_remove_target,
                            disabled: selected_target.as_ref().map(|t| t.is_pinned).unwrap_or(true),
                            "{t(lang, \"targets.btn_remove\")}"
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
                                    ondoubleclick: {
                                        let t = t_clone.clone();
                                        move |_| {
                                            open_target_edit_dialog(
                                                &t,
                                                editing_target_id,
                                                form_name,
                                                form_path,
                                                form_error,
                                                modal_is_edit,
                                                is_modal_open,
                                            );
                                        }
                                    },
                                    div { class: "item-header",
                                        div { style: "display: flex; align-items: center; gap: 6px; overflow: hidden;",
                                            span { class: "item-name", "{target.name}" }
                                            button {
                                                class: "btn-icon",
                                                title: "{t(lang, \"targets.tooltip_edit\")}",
                                                onclick: {
                                                    let t = t_clone.clone();
                                                    move |e| {
                                                        e.stop_propagation();
                                                        state.selected_target_id.set(Some(t.id.clone()));
                                                        open_target_edit_dialog(
                                                            &t,
                                                            editing_target_id,
                                                            form_name,
                                                            form_path,
                                                            form_error,
                                                            modal_is_edit,
                                                            is_modal_open,
                                                        );
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
                                        div { class: "item-path", "{t(lang, \"targets.global_path\")}" }
                                    } else {
                                        div { class: "item-path", "{target.path.display()}" }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Draggable Resize Handle
            ResizeHandle {}

            // Right Pane: Installed Skills
            div { class: "pane-right",
                div { class: "pane-header",
                    div { style: "display: flex; align-items: center; gap: 10px;",
                        span { class: "pane-title", "{installed_title}" }
                        if selected_target_for_header.is_some() {
                            button {
                                class: "btn btn-secondary btn-sm",
                                onclick: {
                                    let target_opt = selected_target_for_header.clone();
                                    move |_| {
                                        if let Some(ref t) = target_opt {
                                            open_target_edit_dialog(
                                                t,
                                                editing_target_id,
                                                form_name,
                                                form_path,
                                                form_error,
                                                modal_is_edit,
                                                is_modal_open,
                                            );
                                        }
                                    }
                                },
                                title: "{t(lang, \"targets.btn_edit_target_title\")}",
                                "{t(lang, \"targets.btn_edit_target\")}"
                            }
                        }
                    }
                }

                div { class: "pane-body", style: "padding: 0;",
                    if selected_target.is_none() {
                        div { class: "empty-state",
                            p { "{t(lang, \"targets.empty_select_prompt\")}" }
                        }
                    } else if installed_skills.is_empty() {
                        div { class: "empty-state",
                            p { "{t(lang, \"targets.empty_skills\")}" }
                        }
                    } else {
                        div { class: "table-container",
                            table {
                                thead {
                                    tr {
                                        th { "{t(lang, \"targets.th_skill_name\")}" }
                                        th { "{t(lang, \"targets.th_configured_harnesses\")}" }
                                        th { "{t(lang, \"targets.th_installations\")}" }
                                        th { style: "text-align: right;", "{t(lang, \"targets.th_actions\")}" }
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
                                                        "{tr(lang, \"targets.paths_count\", &[(\"count\", &skill.installed_paths.len().to_string())])}"
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
                                                                                    state.status_message.set(Some(tr(
                                                                                        lang,
                                                                                        "targets.msg_remove_success",
                                                                                        &[("name", &skill_to_delete)],
                                                                                    )));
                                                                                    scan_rev.set(scan_rev() + 1);
                                                                                }
                                                                                Ok(Err(e)) => {
                                                                                    state.status_message.set(Some(tr(
                                                                                        lang,
                                                                                        "targets.msg_remove_failed",
                                                                                        &[("error", &e.to_string())],
                                                                                    )));
                                                                                }
                                                                                Err(e) => {
                                                                                    state.status_message.set(Some(tr(
                                                                                        lang,
                                                                                        "targets.msg_remove_internal_error",
                                                                                        &[("error", &e.to_string())],
                                                                                    )));
                                                                                }
                                                                            }
                                                                        });
                                                                    }
                                                                }
                                                            },
                                                            "{t(lang, \"targets.btn_remove_skill\")}"
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
            div {
                class: "modal-backdrop",
                onclick: move |_| is_modal_open.set(false),
                div {
                    class: "modal-content",
                    onclick: move |e| e.stop_propagation(),
                    div { class: "modal-header",
                        h3 { class: "modal-title",
                            if modal_is_edit() { "{t(lang, \"targets.modal_edit_title\")}" } else { "{t(lang, \"targets.modal_add_title\")}" }
                        }
                        button {
                            class: "btn btn-secondary btn-sm",
                            onclick: move |_| is_modal_open.set(false),
                            "{t(lang, \"common.close\")}"
                        }
                    }
                    div { class: "modal-body",
                        div { class: "form-group",
                            label { class: "form-label", "{t(lang, \"targets.lbl_destination_name\")}" }
                            input {
                                class: "form-control",
                                r#type: "text",
                                placeholder: "{t(lang, \"targets.ph_destination_name\")}",
                                value: "{form_name}",
                                oninput: move |e| {
                                    form_name.set(e.value());
                                    form_error.set(None);
                                },
                                onkeydown: {
                                    let active_id = active_target_id.clone();
                                    move |e| {
                                        if e.key() == Key::Enter {
                                            save_target_modal(
                                                state,
                                                is_modal_open,
                                                modal_is_edit,
                                                editing_target_id,
                                                active_id.clone(),
                                                form_name,
                                                form_path,
                                                form_error,
                                            );
                                        }
                                    }
                                },
                            }
                            p { style: "font-size: 11px; color: var(--text-muted); margin-top: 4px;",
                                "{t(lang, \"targets.hint_destination_name\")}"
                            }
                        }
                        div { class: "form-group",
                            label { class: "form-label", "{t(lang, \"targets.lbl_destination_path\")}" }
                            if !is_editing_global {
                                div { class: "input-with-button",
                                    input {
                                        class: "form-control",
                                        r#type: "text",
                                        placeholder: "{t(lang, \"targets.ph_destination_path\")}",
                                        value: "{form_path}",
                                        oninput: move |e| {
                                            form_path.set(e.value());
                                            form_error.set(None);
                                        },
                                        onkeydown: {
                                            let active_id = active_target_id.clone();
                                            move |e| {
                                                if e.key() == Key::Enter {
                                                    save_target_modal(
                                                        state,
                                                        is_modal_open,
                                                        modal_is_edit,
                                                        editing_target_id,
                                                        active_id.clone(),
                                                        form_name,
                                                        form_path,
                                                        form_error,
                                                    );
                                                }
                                            }
                                        },
                                    }
                                    button {
                                        class: "btn btn-secondary",
                                        r#type: "button",
                                        onclick: move |_| {
                                            let cur_path = form_path();
                                            spawn(async move {
                                                let start = if !cur_path.is_empty() {
                                                    Some(PathBuf::from(&cur_path))
                                                } else {
                                                    None
                                                };
                                                let dlg_title = t(lang, "targets.dlg_select_dir");
                                                if let Some(folder) = pick_folder(&dlg_title, start.as_deref()).await {
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
                                        },
                                        "{t(lang, \"targets.btn_browse\")}"
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
                                    "{t(lang, \"targets.hint_global_paths\")}"
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
                            "{t(lang, \"common.cancel\")}"
                        }
                        button {
                            class: "btn btn-primary",
                            r#type: "button",
                            onclick: {
                                let active_id = active_target_id.clone();
                                move |_| {
                                    save_target_modal(
                                        state,
                                        is_modal_open,
                                        modal_is_edit,
                                        editing_target_id,
                                        active_id.clone(),
                                        form_name,
                                        form_path,
                                        form_error,
                                    );
                                }
                            },
                            if modal_is_edit() { "{t(lang, \"common.save_changes\")}" } else { "{t(lang, \"targets.btn_add_destination\")}" }
                        }
                    }
                }
            }
        }
    }
}

