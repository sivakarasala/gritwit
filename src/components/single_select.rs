use super::SelectOption;
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

    view! {
        <div class="single-select">
            <button type="button" class="single-select__trigger" on:click=toggle>
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
                    <div class="single-select__backdrop" on:click=move |_| close()></div>
                    <div class="single-select__dropdown">
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
        </div>
    }
}
