use crate::models::skill::Skill;
use crate::models::state::AppState;
use crate::services::deployer::deploy_skills;
use dioxus::prelude::*;
use std::collections::HashSet;

#[component]
pub fn CopyModal(skills: Vec<Skill>) -> Element {
    let mut state: AppState = use_context();
    let config = (state.config)();
    let selected_skill_names = (state.selected_skill_names)();

    // Track which target IDs are selected for deployment
    let mut selected_target_ids = use_signal(|| {
        let mut set = HashSet::new();
        // Pre-select global target if available
        if let Some(global) = config.targets.iter().find(|t| t.is_global) {
            set.insert(global.id.clone());
        }
        set
    });

    let mut is_deploying = use_signal(|| false);

    let selected_skills: Vec<Skill> = skills
        .into_iter()
        .filter(|s| selected_skill_names.contains(&s.name))
        .collect();

    let close_modal = move |_| {
        state.is_copy_modal_open.set(false);
    };

    let on_confirm = {
        let selected_skills = selected_skills.clone();
        let targets = config.targets.clone();
        let harnesses = config.harnesses.clone();

        move |_| {
            if is_deploying() {
                return;
            }
            is_deploying.set(true);

            let chosen_targets: Vec<_> = targets
                .iter()
                .filter(|t| selected_target_ids().contains(&t.id))
                .cloned()
                .collect();

            let skills_to_deploy = selected_skills.clone();
            let harnesses_to_deploy = harnesses.clone();

            spawn(async move {
                let deploy_result = tokio::task::spawn_blocking(move || {
                    deploy_skills(&skills_to_deploy, &chosen_targets, &harnesses_to_deploy)
                })
                .await
                .unwrap_or_default();

                let mut msg = format!(
                    "Copied {} skill destinations.",
                    deploy_result.copied_paths.len()
                );
                if !deploy_result.warnings.is_empty() {
                    msg.push_str(&format!(" Warnings: {}.", deploy_result.warnings.len()));
                }
                if !deploy_result.errors.is_empty() {
                    msg.push_str(&format!(" Errors: {}.", deploy_result.errors.len()));
                }

                state.status_message.set(Some(msg));
                state.is_copy_modal_open.set(false);
                is_deploying.set(false);
            });
        }
    };

    rsx! {
        div { class: "modal-backdrop",
            div { class: "modal-content",
                div { class: "modal-header",
                    h3 { class: "modal-title", "Copy Skills to Targets" }
                    button {
                        class: "btn btn-secondary btn-sm",
                        onclick: close_modal,
                        "✕"
                    }
                }
                div { class: "modal-body",
                    div { style: "margin-bottom: 16px;",
                        p { style: "font-weight: 500; margin-bottom: 6px;",
                            "Skills to Copy ({selected_skills.len()}):"
                        }
                        div { style: "display: flex; flex-wrap: wrap; gap: 6px;",
                            for skill in &selected_skills {
                                span { class: "badge badge-harness", "{skill.name}" }
                            }
                        }
                    }

                    p { style: "font-weight: 500; margin-bottom: 8px;", "Select Destination Targets:" }
                    div { style: "display: flex; flex-direction: column; gap: 8px;",
                        for target in &config.targets {
                            {
                                let tid = target.id.clone();
                                let is_checked = selected_target_ids().contains(&tid);
                                rsx! {
                                    label {
                                        key: "{target.id}",
                                        style: "display: flex; align-items: center; gap: 8px; padding: 8px; background: var(--surface-hover); border-radius: 6px; cursor: pointer;",
                                        input {
                                            r#type: "checkbox",
                                            checked: is_checked,
                                            onchange: {
                                                let tid = target.id.clone();
                                                move |_| {
                                                    let mut current = selected_target_ids();
                                                    if current.contains(&tid) {
                                                        current.remove(&tid);
                                                    } else {
                                                        current.insert(tid.clone());
                                                    }
                                                    selected_target_ids.set(current);
                                                }
                                            }
                                        }
                                        div { style: "flex: 1;",
                                            div { style: "font-weight: 600;",
                                                "{target.name}"
                                                if target.is_global {
                                                    span { style: "margin-left: 6px;", class: "badge badge-pinned", "Global" }
                                                }
                                            }
                                            if !target.is_global {
                                                div { style: "font-size: 11px; color: var(--text-muted);",
                                                    "{target.path.display()}"
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                div { class: "modal-footer",
                    button {
                        class: "btn btn-secondary",
                        onclick: close_modal,
                        disabled: is_deploying(),
                        "Cancel"
                    }
                    button {
                        class: "btn btn-primary",
                        onclick: on_confirm,
                        disabled: is_deploying() || selected_target_ids().is_empty(),
                        if is_deploying() { "Deploying..." } else { "Deploy Skills" }
                    }
                }
            }
        }
    }
}
