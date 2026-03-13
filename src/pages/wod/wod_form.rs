use leptos::prelude::*;

use super::CreateWod;

#[component]
pub fn WodForm(
    create_action: ServerAction<CreateWod>,
    show_form: RwSignal<bool>,
    title_input: RwSignal<String>,
    desc_input: RwSignal<String>,
    type_input: RwSignal<String>,
    cap_input: RwSignal<String>,
    date_input: RwSignal<String>,
) -> impl IntoView {
    view! {
        <form
            class="wod-form"
            on:submit=move |ev| {
                ev.prevent_default();
                let t = title_input.get_untracked();
                if t.is_empty() { return; }
                create_action.dispatch(CreateWod {
                    title: t,
                    description: desc_input.get_untracked(),
                    workout_type: type_input.get_untracked(),
                    time_cap_minutes: cap_input.get_untracked(),
                    programmed_date: date_input.get_untracked(),
                });
                title_input.set(String::new());
                desc_input.set(String::new());
                cap_input.set(String::new());
                show_form.set(false);
            }
        >
            <div class="form-row">
                <input
                    type="date"
                    prop:value=move || date_input.get()
                    on:input=move |ev| date_input.set(event_target_value(&ev))
                />
                <select
                    prop:value=move || type_input.get()
                    on:change=move |ev| type_input.set(event_target_value(&ev))
                >
                    <option value="fortime">"For Time"</option>
                    <option value="amrap">"AMRAP"</option>
                    <option value="emom">"EMOM"</option>
                    <option value="tabata">"Tabata"</option>
                    <option value="strength">"Strength"</option>
                    <option value="custom">"Custom"</option>
                </select>
            </div>
            <input
                type="text"
                placeholder="WOD title (e.g. Fran)"
                prop:value=move || title_input.get()
                on:input=move |ev| title_input.set(event_target_value(&ev))
            />
            <input
                type="text"
                placeholder="Description (optional)"
                prop:value=move || desc_input.get()
                on:input=move |ev| desc_input.set(event_target_value(&ev))
            />
            <input
                type="number"
                placeholder="Time cap (minutes)"
                prop:value=move || cap_input.get()
                on:input=move |ev| cap_input.set(event_target_value(&ev))
            />
            <button
                type="submit"
                class="form-submit"
                disabled=move || create_action.pending().get()
            >
                {move || if create_action.pending().get() {
                    view! { <span class="spinner"></span>" Creating..." }.into_any()
                } else {
                    view! { "Create WOD" }.into_any()
                }}
            </button>
        </form>
    }
}
