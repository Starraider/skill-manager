use dioxus::prelude::*;
use std::collections::HashSet;
use std::path::PathBuf;

use crate::components::common::{pick_folder, StatusPill};
use crate::components::resize_handle::ResizeHandle;
use crate::models::config::SourceConfig;
use crate::models::skill::Skill;
use crate::models::state::AppState;
use crate::services::config_store::{expand_tilde, save_config};
use crate::services::fs_scanner::scan_skills;
use crate::services::i18n::{min_left_pane_width, t, tr};

fn open_source_add_dialog(
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

fn open_source_edit_dialog(
    source: &SourceConfig,
    mut editing_id: Signal<Option<String>>,
    mut name: Signal<String>,
    mut path: Signal<String>,
    mut error: Signal<Option<String>>,
    mut is_edit: Signal<bool>,
    mut is_open: Signal<bool>,
) {
    editing_id.set(Some(source.id.clone()));
    name.set(source.name.clone());
    path.set(source.path.to_string_lossy().to_string());
    error.set(None);
    is_edit.set(true);
    is_open.set(true);
}

fn save_source_modal(
    mut state: AppState,
    mut is_modal_open: Signal<bool>,
    modal_is_edit: Signal<bool>,
    editing_source_id: Signal<Option<String>>,
    form_name: Signal<String>,
    form_path: Signal<String>,
    mut form_error: Signal<Option<String>>,
) {
    let lang = (state.current_language)();
    let trimmed_name = form_name().trim().to_string();
    let trimmed_path = form_path().trim().to_string();

    if trimmed_name.is_empty() {
        form_error.set(Some(t(lang, "sources.err_name_required")));
        return;
    }
    if trimmed_path.is_empty() {
        form_error.set(Some(t(lang, "sources.err_path_required")));
        return;
    }

    let path_buf = PathBuf::from(trimmed_path);
    let mut current_config = (state.config)();

    if modal_is_edit() {
        if let Some(ref id) = editing_source_id() {
            if let Some(s) = current_config.sources.iter_mut().find(|s| s.id == *id) {
                s.name = trimmed_name.clone();
                s.path = path_buf;
            }
        }
        let _ = save_config(&current_config);
        state.config.set(current_config);
        state
            .status_message
            .set(Some(tr(lang, "sources.msg_updated", &[("name", &trimmed_name)])));
    } else {
        let id = format!(
            "source-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis()
        );
        let new_source = SourceConfig {
            id: id.clone(),
            name: trimmed_name.clone(),
            path: path_buf,
        };
        current_config.sources.push(new_source);
        let _ = save_config(&current_config);
        state.config.set(current_config);
        state.selected_source_id.set(Some(id));
        state
            .status_message
            .set(Some(tr(lang, "sources.msg_added", &[("name", &trimmed_name)])));
    }

    is_modal_open.set(false);
}

#[component]
pub fn TabSources() -> Element {
    let mut state: AppState = use_context();
    let lang = (state.current_language)();
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

    // Modal state for Add and Edit Source
    let mut is_modal_open = use_signal(|| false);
    let modal_is_edit = use_signal(|| false);
    let editing_source_id = use_signal(|| None::<String>);
    let mut form_name = use_signal(String::new);
    let mut form_path = use_signal(String::new);
    let mut form_error = use_signal(|| None::<String>);

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

    let selected_source_for_header = selected_source.clone();
    let pane_width = (state.left_pane_width)();
    let min_width = min_left_pane_width(lang);
    let discovered_title = if let Some(ref src) = selected_source {
        tr(lang, "sources.skills_heading", &[("name", &src.name), ("count", &discovered_skills.len().to_string())])
    } else {
        t(lang, "sources.discovered_skills")
    };

    rsx! {
        div { class: "split-layout",
            // Left Pane: Source Folders
            div {
                class: "pane-left",
                style: "width: {pane_width}px; min-width: {min_width}px;",
                div { class: "pane-header",
                    span { class: "pane-title", "{t(lang, \"sources.title\")}" }
                    div { class: "pane-header-actions",
                        button {
                            class: "btn btn-primary btn-sm",
                            onclick: move |_| {
                                open_source_add_dialog(
                                    editing_source_id,
                                    form_name,
                                    form_path,
                                    form_error,
                                    modal_is_edit,
                                    is_modal_open,
                                );
                            },
                            "{t(lang, \"sources.btn_add\")}"
                        }
                        button {
                            class: "btn btn-secondary btn-sm",
                            onclick: {
                                let src_opt = selected_source.clone();
                                move |_| {
                                    if let Some(ref s) = src_opt {
                                        open_source_edit_dialog(
                                            s,
                                            editing_source_id,
                                            form_name,
                                            form_path,
                                            form_error,
                                            modal_is_edit,
                                            is_modal_open,
                                        );
                                    }
                                }
                            },
                            disabled: selected_source.is_none(),
                            "{t(lang, \"sources.btn_edit\")}"
                        }
                        button {
                            class: "btn btn-danger btn-sm",
                            onclick: on_remove_source,
                            disabled: selected_source_id.is_none(),
                            "{t(lang, \"sources.btn_remove\")}"
                        }
                    }
                }
                div { class: "pane-body",
                    if config.sources.is_empty() {
                        div { class: "empty-state",
                            p { "{t(lang, \"sources.empty\")}" }
                            button {
                                class: "btn btn-secondary btn-sm",
                                onclick: move |_| {
                                    open_source_add_dialog(
                                        editing_source_id,
                                        form_name,
                                        form_path,
                                        form_error,
                                        modal_is_edit,
                                        is_modal_open,
                                    );
                                },
                                "{t(lang, \"sources.btn_add_folder\")}"
                            }
                        }
                    } else {
                        for source in &config.sources {
                            {
                                let is_selected = Some(&source.id) == selected_source_id.as_ref();
                                let is_active = expand_tilde(&source.path).exists();
                                let sid = source.id.clone();
                                let s_clone = source.clone();
                                rsx! {
                                    div {
                                        key: "{source.id}",
                                        class: if is_selected { "item-card selected" } else { "item-card" },
                                        onclick: move |_| {
                                            state.selected_source_id.set(Some(sid.clone()));
                                            selected_skill_names.set(HashSet::new());
                                        },
                                        ondoubleclick: {
                                            let s = s_clone.clone();
                                            move |_| {
                                                open_source_edit_dialog(
                                                    &s,
                                                    editing_source_id,
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
                                                span { class: "item-name", "{source.name}" }
                                                button {
                                                    class: "btn-icon",
                                                    title: "{t(lang, \"sources.tooltip_edit\")}",
                                                    onclick: {
                                                        let s = s_clone.clone();
                                                        move |e| {
                                                            e.stop_propagation();
                                                            state.selected_source_id.set(Some(s.id.clone()));
                                                            selected_skill_names.set(HashSet::new());
                                                            open_source_edit_dialog(
                                                                &s,
                                                                editing_source_id,
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
                                            }
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

            // Draggable Resize Handle
            ResizeHandle {}

            // Right Pane: Discovered Skills
            div { class: "pane-right",
                div { class: "pane-header",
                    div { style: "display: flex; align-items: center; gap: 10px;",
                        span { class: "pane-title", "{discovered_title}" }
                        if selected_source_for_header.is_some() {
                            button {
                                class: "btn btn-secondary btn-sm",
                                onclick: {
                                    let src_opt = selected_source_for_header.clone();
                                    move |_| {
                                        if let Some(ref s) = src_opt {
                                            open_source_edit_dialog(
                                                s,
                                                editing_source_id,
                                                form_name,
                                                form_path,
                                                form_error,
                                                modal_is_edit,
                                                is_modal_open,
                                            );
                                        }
                                    }
                                },
                                title: "{t(lang, \"sources.btn_edit_source_title\")}",
                                "{t(lang, \"sources.btn_edit_source\")}"
                            }
                        }
                    }
                    button {
                        class: "btn btn-primary btn-sm",
                        onclick: move |_| state.is_copy_modal_open.set(true),
                        disabled: selected_skill_names().is_empty(),
                        "{tr(lang, \"sources.btn_copy_skills\", &[(\"count\", &selected_skill_names().len().to_string())])}"
                    }
                }

                div { class: "pane-body", style: "padding: 0;",
                    if selected_source.is_none() {
                        div { class: "empty-state",
                            p { "{t(lang, \"sources.empty_select_prompt\")}" }
                        }
                    } else if discovered_skills.is_empty() {
                        div { class: "empty-state",
                            p { "{t(lang, \"sources.empty_skills\")}" }
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
                                        th { "{t(lang, \"sources.th_skill_name\")}" }
                                        th { "{t(lang, \"sources.th_folder\")}" }
                                    }
                                }
                                tbody {
                                    for skill in &discovered_skills {
                                        {
                                            let s_name = skill.name.clone();
                                            let is_checked = selected_skill_names().contains(&skill.name);
                                            let fallback_desc = t(lang, "sources.no_description");
                                            let tooltip_text = skill
                                                .description
                                                .as_deref()
                                                .filter(|d| !d.trim().is_empty())
                                                .or_else(|| skill.title.as_deref().filter(|t| !t.trim().is_empty()))
                                                .unwrap_or(&fallback_desc);
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
                                                    td {
                                                        span {
                                                            class: "skill-name-interactive",
                                                            title: "{tooltip_text}",
                                                            "{skill.name}"
                                                        }
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

        // Add / Edit Source Modal
        if is_modal_open() {
            div {
                class: "modal-backdrop",
                onclick: move |_| is_modal_open.set(false),
                div {
                    class: "modal-content",
                    onclick: move |e| e.stop_propagation(),
                    div { class: "modal-header",
                        h3 { class: "modal-title",
                            if modal_is_edit() { "{t(lang, \"sources.modal_edit_title\")}" } else { "{t(lang, \"sources.modal_add_title\")}" }
                        }
                        button {
                            class: "btn btn-secondary btn-sm",
                            onclick: move |_| is_modal_open.set(false),
                            "{t(lang, \"common.close\")}"
                        }
                    }
                    div { class: "modal-body",
                        div { class: "form-group",
                            label { class: "form-label", "{t(lang, \"sources.lbl_folder_name\")}" }
                            input {
                                class: "form-control",
                                r#type: "text",
                                placeholder: "{t(lang, \"sources.ph_folder_name\")}",
                                value: "{form_name}",
                                oninput: move |e| {
                                    form_name.set(e.value());
                                    form_error.set(None);
                                },
                                onkeydown: move |e| {
                                    if e.key() == Key::Enter {
                                        save_source_modal(
                                            state,
                                            is_modal_open,
                                            modal_is_edit,
                                            editing_source_id,
                                            form_name,
                                            form_path,
                                            form_error,
                                        );
                                    }
                                },
                            }
                            p { style: "font-size: 11px; color: var(--text-muted); margin-top: 4px;",
                                "{t(lang, \"sources.hint_folder_name\")}"
                            }
                        }
                        div { class: "form-group",
                            label { class: "form-label", "{t(lang, \"sources.lbl_folder_path\")}" }
                            div { class: "input-with-button",
                                input {
                                    class: "form-control",
                                    r#type: "text",
                                    placeholder: "{t(lang, \"sources.ph_folder_path\")}",
                                    value: "{form_path}",
                                    oninput: move |e| {
                                        form_path.set(e.value());
                                        form_error.set(None);
                                    },
                                    onkeydown: move |e| {
                                        if e.key() == Key::Enter {
                                            save_source_modal(
                                                state,
                                                is_modal_open,
                                                modal_is_edit,
                                                editing_source_id,
                                                form_name,
                                                form_path,
                                                form_error,
                                            );
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
                                            let dlg_title = t(lang, "sources.dlg_select_dir");
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
                                    "{t(lang, \"sources.btn_browse\")}"
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
                            onclick: move |_| {
                                save_source_modal(
                                    state,
                                    is_modal_open,
                                    modal_is_edit,
                                    editing_source_id,
                                    form_name,
                                    form_path,
                                    form_error,
                                );
                            },
                            if modal_is_edit() { "{t(lang, \"common.save_changes\")}" } else { "{t(lang, \"sources.btn_add_source\")}" }
                        }
                    }
                }
            }
        }
    }
}

