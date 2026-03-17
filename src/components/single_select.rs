use super::SelectOption;
use leptos::portal::Portal;
use leptos::prelude::*;

/// Reusable searchable single-select dropdown component.
///
/// - `options`: all available choices
/// - `selected`: signal holding the currently selected value
/// - `placeholder`: text shown when nothing is selected
#[component]
pub fn SingleSelect(
    options: Vec<SelectOption>,
    selected: RwSignal<String>,
    #[prop(default = "Select...")] placeholder: &'static str,
) -> impl IntoView {
    let open = RwSignal::new(false);
    let search = RwSignal::new(String::new());
    let all_options = RwSignal::new(options);
    let trigger_ref = NodeRef::<leptos::html::Button>::new();

    // Position signals
    let dropdown_top = RwSignal::new(0.0_f64);
    let dropdown_bottom = RwSignal::new(0.0_f64);
    let dropdown_left = RwSignal::new(0.0_f64);
    let dropdown_width = RwSignal::new(0.0_f64);
    let dropdown_max_height = RwSignal::new(300.0_f64);
    let opens_up = RwSignal::new(false);

    let toggle = move |_: leptos::ev::MouseEvent| {
        if !open.get_untracked() {
            #[cfg(target_arch = "wasm32")]
            if let Some(el) = trigger_ref.get_untracked() {
                let el_ref: &web_sys::Element = el.as_ref();
                let rect = el_ref.get_bounding_client_rect();

                let vp_height = web_sys::window()
                    .and_then(|w| w.inner_height().ok())
                    .and_then(|v| v.as_f64())
                    .unwrap_or(667.0);

                // ~70px for bottom nav + safe area
                let nav_h = 70.0_f64;
                let space_below = (vp_height - rect.bottom() - nav_h).max(0.0);
                let space_above = rect.top();

                let flip = space_below < 200.0 && space_above > space_below;
                opens_up.set(flip);
                dropdown_left.set(rect.left());
                dropdown_width.set(rect.width());

                if flip {
                    // Anchor bottom edge of dropdown to top of trigger
                    dropdown_bottom.set(vp_height - rect.top() + 4.0);
                    dropdown_max_height.set((space_above - 12.0).clamp(120.0, 360.0));
                } else {
                    dropdown_top.set(rect.bottom() + 4.0);
                    dropdown_max_height.set((space_below - 12.0).clamp(120.0, 360.0));
                }
            }
        } else {
            search.set(String::new());
        }
        open.update(|v| *v = !*v);
    };

    let close = move || {
        open.set(false);
        search.set(String::new());
    };

    view! {
        <div class="single-select">
            <button node_ref=trigger_ref type="button" class="single-select__trigger" on:click=toggle>
                <span class="single-select__label">
                    {move || {
                        let val = selected.get();
                        if val.is_empty() {
                            placeholder.to_string()
                        } else {
                            let opts = all_options.get();
                            opts.iter()
                                .find(|o| o.value == val)
                                .map(|o| o.label.clone())
                                .unwrap_or(val)
                        }
                    }}
                </span>
                <span class="single-select__arrow" class:open=move || open.get()></span>
            </button>

            // Dropdown rendered into document.body via Portal to escape any overflow clipping
            <Portal>
                {move || {
                    if !open.get() {
                        return ().into_any();
                    }

                    let query = search.get().to_lowercase();
                    let opts = all_options.get();
                    let filtered: Vec<_> = opts
                        .iter()
                        .filter(|o| query.is_empty() || o.label.to_lowercase().contains(&query))
                        .cloned()
                        .collect();

                    let max_h = dropdown_max_height.get_untracked();
                    let style = if opens_up.get_untracked() {
                        format!(
                            "bottom: {}px; left: {}px; min-width: {}px; max-height: {}px;",
                            dropdown_bottom.get_untracked(),
                            dropdown_left.get_untracked(),
                            dropdown_width.get_untracked(),
                            max_h,
                        )
                    } else {
                        format!(
                            "top: {}px; left: {}px; min-width: {}px; max-height: {}px;",
                            dropdown_top.get_untracked(),
                            dropdown_left.get_untracked(),
                            dropdown_width.get_untracked(),
                            max_h,
                        )
                    };

                    view! {
                        <div class="single-select__backdrop" on:click=move |_| close()></div>
                        <div class="single-select__dropdown" style=style>
                            <input
                                type="text"
                                class="single-select__search"
                                placeholder="Search..."
                                prop:value=move || search.get()
                                on:input=move |ev| search.set(event_target_value(&ev))
                                on:click=move |ev| ev.stop_propagation()
                            />
                            <div class="single-select__options">
                                {filtered.into_iter().map(|opt| {
                                    let val_check = opt.value.clone();
                                    let val_click = opt.value.clone();
                                    let label = opt.label.clone();
                                    view! {
                                        <div
                                            class="single-select__option"
                                            class:selected=move || selected.get() == val_check
                                            on:click=move |ev| {
                                                ev.stop_propagation();
                                                selected.set(val_click.clone());
                                                close();
                                            }
                                        >
                                            <span class="single-select__check"></span>
                                            {label}
                                        </div>
                                    }
                                }).collect_view()}
                                {move || {
                                    let q = search.get().to_lowercase();
                                    let opts = all_options.get();
                                    let count = opts
                                        .iter()
                                        .filter(|o| q.is_empty() || o.label.to_lowercase().contains(&q))
                                        .count();
                                    (count == 0).then(|| view! {
                                        <div class="single-select__empty">"No matches"</div>
                                    })
                                }}
                            </div>
                        </div>
                    }.into_any()
                }}
            </Portal>
        </div>
    }
}
