mod exercise_card;
mod exercise_form;
pub(crate) mod helpers;
mod server_fns;

use std::collections::HashSet;

use crate::components::DeleteModal;
use crate::db::Exercise;
use exercise_card::ExerciseCard;
use exercise_form::ExerciseForm;
use leptos::prelude::*;

pub(crate) use helpers::*;
pub(super) use server_fns::*;

#[component]
pub fn ExercisesPage() -> impl IntoView {
    use crate::auth::{AuthUser, UserRole};
    let auth_user = use_context::<AuthUser>();
    let is_coach = auth_user
        .as_ref()
        .map(|u| matches!(u.role, UserRole::Coach | UserRole::Admin))
        .unwrap_or(false);
    let is_admin = auth_user
        .as_ref()
        .map(|u| matches!(u.role, UserRole::Admin))
        .unwrap_or(false);
    let current_user_id = auth_user.map(|u| u.id);

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

    let search: RwSignal<String> = RwSignal::new(String::new());
    let collapsed: RwSignal<HashSet<String>> = RwSignal::new(HashSet::new());
    let show_form = RwSignal::new(false);
    let expanded_id = RwSignal::new(Option::<String>::None);
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

    view! {
        <div class="exercises-page">
            {fab_view}
            {move || {
                show_form.get().then(|| view! {
                    <ExerciseForm create_action=create_action show_form=show_form/>
                })
            }}
            <div style=move || if show_form.get() { "display:none" } else { "" }>
                <div class="exercises-search">
                    <input
                        type="text"
                        class="exercises-search-input"
                        placeholder="Search movements…"
                        prop:value=move || search.get()
                        on:input=move |ev| search.set(event_target_value(&ev))
                    />
                </div>
                <Suspense fallback=|| view! { <p class="loading">"Loading movements..."</p> }>
                    {move || {
                        let current_user_id = current_user_id.clone();
                        let q = search.get().to_lowercase();
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
                                    let groups: Vec<(&'static str, &'static str, Vec<Exercise>)> =
                                        CATEGORIES.iter()
                                            .filter_map(|(cat_val, cat_label, _, _)| {
                                                let filtered: Vec<Exercise> = list.iter()
                                                    .filter(|e| {
                                                        e.category == *cat_val &&
                                                        (q.is_empty() || e.name.to_lowercase().contains(&q))
                                                    })
                                                    .cloned()
                                                    .collect();
                                                if filtered.is_empty() { None }
                                                else { Some((*cat_val, *cat_label, filtered)) }
                                            })
                                            .collect();

                                    if groups.is_empty() {
                                        view! {
                                            <div class="empty-state">
                                                <p class="empty-title">"No matches"</p>
                                                <p class="empty-sub">"Try a different search"</p>
                                            </div>
                                        }.into_any()
                                    } else {
                                        let total: usize = groups.iter().map(|(_, _, v)| v.len()).sum();
                                        let cat_count = groups.len();
                                        view! {
                                            <div>
                                                <div class="exercises-stats">
                                                    <span class="exercises-stat">
                                                        <span class="exercises-stat-num">{total}</span>
                                                        " movements"
                                                    </span>
                                                    <span class="exercises-stat-sep">"·"</span>
                                                    <span class="exercises-stat">
                                                        <span class="exercises-stat-num">{cat_count}</span>
                                                        " categories"
                                                    </span>
                                                </div>
                                                <div class="exercises-sections">
                                                    {groups.into_iter().map(|(cat_key, cat_label, cat_exercises)| {
                                                        let color = category_color(cat_key);
                                                        let count = cat_exercises.len();
                                                        view! {
                                                            <div class="exercises-section">
                                                                <div
                                                                    class="exercises-section-header"
                                                                    on:click=move |_| {
                                                                        collapsed.update(|s| {
                                                                            if s.contains(cat_key) {
                                                                                s.remove(cat_key);
                                                                            } else {
                                                                                s.insert(cat_key.to_string());
                                                                            }
                                                                        });
                                                                    }
                                                                >
                                                                    <div class="exercises-section-left">
                                                                        <div
                                                                            class="exercises-section-dot"
                                                                            style=format!("background:{}", color)
                                                                        ></div>
                                                                        <span class="exercises-section-name">{cat_label}</span>
                                                                        <span class="exercises-section-count">{count}</span>
                                                                    </div>
                                                                    <div class=move || {
                                                                        if collapsed.get().contains(cat_key) {
                                                                            "exercises-section-chevron"
                                                                        } else {
                                                                            "exercises-section-chevron open"
                                                                        }
                                                                    }></div>
                                                                </div>
                                                                {{
                                                                let uid_for_section = current_user_id.clone();
                                                                move || {
                                                                    if !collapsed.get().contains(cat_key) {
                                                                        cat_exercises.iter().map(|ex| {
                                                                            view! {
                                                                                <ExerciseCard
                                                                                    exercise=ex.clone()
                                                                                    expanded_id=expanded_id
                                                                                    editing_exercise=editing_exercise
                                                                                    update_action=update_action
                                                                                    pending_delete_id=pending_delete_id
                                                                                    show_delete=show_delete
                                                                                    is_coach=is_coach
                                                                                    is_admin=is_admin
                                                                                    current_user_id=uid_for_section.clone()
                                                                                />
                                                                            }
                                                                        }).collect_view()
                                                                        .into_any()
                                                                    } else {
                                                                        ().into_view().into_any()
                                                                    }
                                                                }}}
                                                            </div>
                                                        }
                                                    }).collect_view()}
                                                </div>
                                            </div>
                                        }.into_any()
                                    }
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
