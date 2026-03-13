use leptos::prelude::*;

/// A single option for the multi-select dropdown.
#[derive(Clone, Debug, PartialEq)]
pub struct SelectOption {
    pub value: String,
    pub label: String,
}

/// Reusable searchable multi-select dropdown component.
///
/// - `options`: all available choices
/// - `selected`: signal holding the currently selected values
/// - `placeholder`: text shown when nothing is selected (e.g. "All Categories")
#[component]
pub fn MultiSelect(
    options: Vec<SelectOption>,
    selected: RwSignal<Vec<String>>,
    #[prop(default = "Select...")] placeholder: &'static str,
) -> impl IntoView {
    let open = RwSignal::new(false);
    let search = RwSignal::new(String::new());

    let toggle = move |_: leptos::ev::MouseEvent| {
        open.update(|v| *v = !*v);
        if !open.get_untracked() {
            search.set(String::new());
        }
    };

    let close = move || {
        open.set(false);
        search.set(String::new());
    };

    let all_options = RwSignal::new(options);

    view! {
        <div class="multi-select">
            // Trigger button
            <button type="button" class="multi-select__trigger" on:click=toggle>
                <span class="multi-select__label">
                    {move || {
                        let sel = selected.get();
                        if sel.is_empty() {
                            placeholder.to_string()
                        } else {
                            format!("{} selected", sel.len())
                        }
                    }}
                </span>
                <span class="multi-select__arrow" class:open=move || open.get()></span>
            </button>

            // Selected chips
            {move || {
                let sel = selected.get();
                if sel.is_empty() {
                    ().into_any()
                } else {
                    let opts = all_options.get();
                    let chips: Vec<_> = opts
                        .iter()
                        .filter(|o| sel.contains(&o.value))
                        .cloned()
                        .collect();
                    view! {
                        <div class="multi-select__chips">
                            {chips.into_iter().map(|opt| {
                                let val = opt.value.clone();
                                view! {
                                    <span class="multi-select__chip">
                                        {opt.label}
                                        <button
                                            type="button"
                                            class="multi-select__chip-remove"
                                            on:click=move |ev| {
                                                ev.stop_propagation();
                                                let v = val.clone();
                                                selected.update(|s| s.retain(|x| x != &v));
                                            }
                                        >"×"</button>
                                    </span>
                                }
                            }).collect_view()}
                            <button
                                type="button"
                                class="multi-select__clear"
                                on:click=move |ev| {
                                    ev.stop_propagation();
                                    selected.set(vec![]);
                                }
                            >"Clear all"</button>
                        </div>
                    }.into_any()
                }
            }}

            // Dropdown overlay
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

                view! {
                    <div class="multi-select__backdrop" on:click=move |_| close()></div>
                    <div class="multi-select__dropdown">
                        <input
                            type="text"
                            class="multi-select__search"
                            placeholder="Search..."
                            prop:value=move || search.get()
                            on:input=move |ev| search.set(event_target_value(&ev))
                            on:click=move |ev| ev.stop_propagation()
                        />
                        <div class="multi-select__options">
                            {filtered.into_iter().map(|opt| {
                                let val_check = opt.value.clone();
                                let val_toggle = opt.value.clone();
                                view! {
                                    <label
                                        class="multi-select__option"
                                        on:click=move |ev| ev.stop_propagation()
                                    >
                                        <input
                                            type="checkbox"
                                            prop:checked=move || selected.get().contains(&val_check)
                                            on:change=move |_| {
                                                let v = val_toggle.clone();
                                                selected.update(|s| {
                                                    if s.contains(&v) {
                                                        s.retain(|x| x != &v);
                                                    } else {
                                                        s.push(v);
                                                    }
                                                });
                                            }
                                        />
                                        <span class="multi-select__check"></span>
                                        {opt.label}
                                    </label>
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
                                    <div class="multi-select__empty">"No matches"</div>
                                })
                            }}
                        </div>
                    </div>
                }.into_any()
            }}
        </div>
    }
}
