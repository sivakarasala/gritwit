mod helpers;
mod section_movements_panel;
mod server_fns;
pub mod week_calendar;
mod wod_card;
mod wod_form;
mod wod_section_card;
mod wod_sections_panel;

use crate::auth::{AuthUser, UserRole};
use crate::components::DeleteModal;
use leptos::prelude::*;

pub(super) use helpers::*;
pub(super) use server_fns::*;

use week_calendar::WeeklyCalendar;
use wod_card::WodCard;
use wod_form::WodForm;

// ---- WodPage ----

#[component]
pub fn WodPage() -> impl IntoView {
    let is_coach = use_context::<AuthUser>()
        .map(|u| matches!(u.role, UserRole::Coach | UserRole::Admin))
        .unwrap_or(false);

    let create_action = ServerAction::<CreateWod>::new();
    let delete_action = ServerAction::<DeleteWod>::new();
    let update_action = ServerAction::<UpdateWod>::new();

    let selected_date: RwSignal<String> = RwSignal::new(week_calendar::today_iso());

    let wods = Resource::new(
        move || {
            (
                selected_date.get(),
                create_action.version().get(),
                delete_action.version().get(),
                update_action.version().get(),
            )
        },
        |(date, _, _, _)| list_wods_for_date(date),
    );

    let show_form = RwSignal::new(false);
    let title_input = RwSignal::new(String::new());
    let desc_input = RwSignal::new(String::new());
    let type_input = RwSignal::new("fortime".to_string());
    let cap_input = RwSignal::new(String::new());
    let date_input = RwSignal::new(String::new());

    let editing_wod: RwSignal<Option<String>> = RwSignal::new(None);

    let show_delete_wod = RwSignal::new(false);
    let pending_delete_wod_id = RwSignal::new(String::new());

    let fab_view = if is_coach {
        view! {
            <button
                class={move || if show_form.get() { "fab fab--active" } else { "fab" }}
                on:click=move |_| show_form.update(|v| *v = !*v)
            >
                <span class="fab-icon"></span>
            </button>
        }
        .into_any()
    } else {
        ().into_view().into_any()
    };

    let form_view = move || {
        if is_coach && show_form.get() {
            view! {
                <WodForm
                    create_action=create_action
                    show_form=show_form
                    title_input=title_input
                    desc_input=desc_input
                    type_input=type_input
                    cap_input=cap_input
                    date_input=date_input
                />
            }
            .into_any()
        } else {
            ().into_view().into_any()
        }
    };

    let list_view = view! {
        <Suspense fallback=|| view! { <p class="loading">"Loading WODs..."</p> }>
            {move || wods.get().map(|result| match result {
                Err(e) => view! {
                    <p class="wod-error">{format!("Error: {}", e)}</p>
                }.into_any(),
                Ok(list) if list.is_empty() => view! {
                    <div class="empty-state">
                        <p class="empty-title">"No WOD for this day"</p>
                        {is_coach.then(|| view! {
                            <p class="empty-sub">"Use + to program a WOD"</p>
                        })}
                    </div>
                }.into_any(),
                Ok(list) => view! {
                    <div class="wod-list">
                        {list.into_iter().map(|wod| {
                            view! {
                                <WodCard
                                    wod=wod
                                    is_coach=is_coach
                                    editing_wod=editing_wod
                                    update_action=update_action
                                    pending_delete_wod_id=pending_delete_wod_id
                                    show_delete_wod=show_delete_wod
                                />
                            }
                        }).collect_view()}
                    </div>
                }.into_any(),
            })}
        </Suspense>
    }
    .into_any();

    view! {
        <div class="wod-page">
            {fab_view}
            {form_view}
            <div style=move || if show_form.get() { "display:none" } else { "" }>
                <WeeklyCalendar selected_date=selected_date />
                {list_view}
                <DeleteModal
                    show=show_delete_wod
                    title="Delete this WOD?"
                    subtitle="All sections and movements will also be removed. This cannot be undone."
                    confirm_label="Delete"
                    on_confirm=move || {
                        delete_action.dispatch(DeleteWod {
                            id: pending_delete_wod_id.get_untracked(),
                        });
                    }
                />
            </div>
        </div>
    }
}
