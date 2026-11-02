use crate::models::state::AppState;
use crate::services::config_store::save_config;
use crate::services::i18n::{default_left_pane_width, min_left_pane_width, t};
use dioxus::prelude::*;

#[component]
pub fn ResizeHandle() -> Element {
    let mut state: AppState = use_context();
    let lang = (state.current_language)();
    let mut is_resizing = use_signal(|| false);

    let on_mousedown = move |e: MouseEvent| {
        e.stop_propagation();
        is_resizing.set(true);
    };

    let on_mousemove = move |e: MouseEvent| {
        if is_resizing() {
            let current_l = (state.current_language)();
            let min_w = min_left_pane_width(current_l);
            let max_w = 750.0;
            let client_x = e.client_coordinates().x;
            let new_width = client_x.clamp(min_w, max_w);
            state.left_pane_width.set(new_width);

            let mut cfg = (state.config)();
            cfg.left_pane_width = Some(new_width);
            let _ = save_config(&cfg);
            state.config.set(cfg);
        }
    };

    let on_mouseup = move |_| {
        if is_resizing() {
            is_resizing.set(false);
        }
    };

    let on_dblclick = move |e: MouseEvent| {
        e.stop_propagation();
        let current_l = (state.current_language)();
        let def_w = default_left_pane_width(current_l);
        state.left_pane_width.set(def_w);

        let mut cfg = (state.config)();
        cfg.left_pane_width = Some(def_w);
        let _ = save_config(&cfg);
        state.config.set(cfg);
    };

    let tooltip = t(lang, "common.resize_tooltip");

    rsx! {
        div {
            class: if is_resizing() { "resize-handle active" } else { "resize-handle" },
            onmousedown: on_mousedown,
            ondoubleclick: on_dblclick,
            title: "{tooltip}",
            div { class: "resize-handle-line" }
        }
        if is_resizing() {
            div {
                class: "resize-overlay",
                onmousemove: on_mousemove,
                onmouseup: on_mouseup,
                onmouseleave: on_mouseup,
            }
        }
    }
}
