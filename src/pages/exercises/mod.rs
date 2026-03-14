mod exercise_card;
mod exercise_form;
pub(crate) mod helpers;
mod server_fns;

use crate::components::{DeleteModal, MultiSelect, SelectOption};
use crate::db::Exercise;
use exercise_card::ExerciseCard;
use exercise_form::ExerciseForm;
use leptos::prelude::*;

pub(crate) use helpers::*;
pub(super) use server_fns::*;

#[component]
pub fn ExercisesPage() -> impl IntoView {
    use crate::auth::{AuthUser, UserRole};
    let is_coach = use_context::<AuthUser>()
        .map(|u| matches!(u.role, UserRole::Coach | UserRole::Admin))
        .unwrap_or(false);

    let create_action = ServerAction::<CreateExercise>::new();
    let delete_action = ServerAction::<DeleteExercise>::new();
    let update_action = ServerAction::<UpdateExercise>::new();

    let exercises = Resource::new(
        move || {
            (
                create_action.version().get(),
                delete_action.version().get(),
                update_action.version().get(),
            )
        },
        |_| list_exercises(),
    );

    let active_filters: RwSignal<Vec<String>> = RwSignal::new(vec![]);
    let show_form = RwSignal::new(false);
    let expanded_video = RwSignal::new(Option::<String>::None);
    let editing_exercise: RwSignal<Option<String>> = RwSignal::new(None);
    let show_delete = RwSignal::new(false);
    let pending_delete_id = RwSignal::new(String::new());

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

    let filter_options: Vec<SelectOption> = CATEGORIES
        .iter()
        .map(|(v, l, _, _)| SelectOption {
            value: v.to_string(),
            label: l.to_string(),
        })
        .collect();

    view! {
        <div class="exercises-page">
            {fab_view}
            {move || {
                if show_form.get() {
                    view! { <ExerciseForm create_action=create_action show_form=show_form/> }.into_any()
                } else {
                    ().into_view().into_any()
                }
            }}
            <div style=move || if show_form.get() { "display:none" } else { "" }>
                <MultiSelect options=filter_options selected=active_filters placeholder="All Categories"/>
                <Suspense fallback=|| view! { <p class="loading">"Loading movements..."</p> }>
                    {move || {
                        let filters = active_filters.get();
                        exercises.get().map(|result| {
                            match result {
                                Ok(list) if list.is_empty() => {
                                    view! {
                                        <div class="empty-state">
                                            <p class="empty-title">"No movements yet"</p>
                                            <p class="empty-sub">"Tap + to build your library"</p>
                                        </div>
                                    }.into_any()
                                }
                                Ok(list) => {
                                    let filtered: Vec<Exercise> = if filters.is_empty() {
                                        list
                                    } else {
                                        list.into_iter().filter(|e| filters.contains(&e.category)).collect()
                                    };
                                    view! {
                                        <div class="exercise-grid">
                                            {filtered.into_iter().map(|ex| {
                                                view! {
                                                    <ExerciseCard
                                                        exercise=ex
                                                        expanded_video=expanded_video
                                                        editing_exercise=editing_exercise
                                                        update_action=update_action
                                                        pending_delete_id=pending_delete_id
                                                        show_delete=show_delete
                                                        is_coach=is_coach
                                                    />
                                                }
                                            }).collect_view()}
                                        </div>
                                    }.into_any()
                                }
                                Err(e) => view! { <p class="error">{format!("Error: {}", e)}</p> }.into_any(),
                            }
                        })
                    }}
                </Suspense>
            </div>
            <DeleteModal
                show=show_delete
                title="Delete this movement?"
                subtitle="This cannot be undone."
                confirm_label="Delete"
                on_confirm=move || {
                    delete_action.dispatch(DeleteExercise {
                        id: pending_delete_id.get_untracked(),
                    });
                }
            />
        </div>
    }
}
