use crate::db::Exercise;
use leptos::prelude::*;

#[server]
async fn list_exercises() -> Result<Vec<Exercise>, ServerFnError> {
    let _user = crate::auth::session::require_auth().await?;
    let pool = crate::db::db().await?;
    crate::db::list_exercises_db(&pool)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
async fn create_exercise(
    name: String,
    category: String,
    movement_type: String,
    description: String,
) -> Result<(), ServerFnError> {
    let user = crate::auth::session::require_auth().await?;
    let pool = crate::db::db().await?;
    let user_uuid: uuid::Uuid = user.id.parse().map_err(|e: uuid::Error| ServerFnError::new(e.to_string()))?;
    let mt = if movement_type.is_empty() {
        None
    } else {
        Some(movement_type.as_str())
    };
    let desc = if description.is_empty() {
        None
    } else {
        Some(description.as_str())
    };
    crate::db::create_exercise_db(&pool, &name, &category, mt, &[], desc, Some(user_uuid))
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
async fn delete_exercise(id: String) -> Result<(), ServerFnError> {
    let _user = crate::auth::session::require_auth().await?;
    let pool = crate::db::db().await?;
    let uuid: uuid::Uuid = id.parse().map_err(|e: uuid::Error| ServerFnError::new(e.to_string()))?;
    crate::db::delete_exercise_db(&pool, uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

fn category_badge(cat: &str) -> &'static str {
    match cat {
        "crossfit" => "WOD",
        "strength" => "STR",
        "meditation" => "MED",
        "breathing" => "BRE",
        "chanting" => "CHT",
        _ => "GEN",
    }
}

fn category_class(cat: &str) -> &'static str {
    match cat {
        "crossfit" => "badge--crossfit",
        "strength" => "badge--strength",
        "meditation" => "badge--meditation",
        "breathing" => "badge--breathing",
        "chanting" => "badge--chanting",
        _ => "",
    }
}

#[component]
pub fn ExercisesPage() -> impl IntoView {
    let create_action = ServerAction::<CreateExercise>::new();
    let delete_action = ServerAction::<DeleteExercise>::new();

    let exercises = Resource::new(
        move || (create_action.version().get(), delete_action.version().get()),
        |_| list_exercises(),
    );

    let name_input = RwSignal::new(String::new());
    let category_input = RwSignal::new("crossfit".to_string());
    let movement_type_input = RwSignal::new(String::new());
    let description_input = RwSignal::new(String::new());
    let active_filter = RwSignal::new("all".to_string());
    let show_form = RwSignal::new(false);

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let name = name_input.get_untracked();
        if !name.is_empty() {
            create_action.dispatch(CreateExercise {
                name,
                category: category_input.get_untracked(),
                movement_type: movement_type_input.get_untracked(),
                description: description_input.get_untracked(),
            });
            name_input.set(String::new());
            description_input.set(String::new());
            show_form.set(false);
        }
    };

    view! {
        <div class="exercises-page">
            <div class="exercises-header">
                <h1>"Movements"</h1>
                <button
                    class="add-btn"
                    on:click=move |_| show_form.update(|v| *v = !*v)
                >
                    {move || if show_form.get() { "Cancel" } else { "+ Add" }}
                </button>
            </div>

            // Filter pills
            <div class="filter-pills">
                {["all", "crossfit", "strength", "meditation", "breathing", "chanting"].into_iter().map(|cat| {
                    let cat_str = cat.to_string();
                    let label = match cat {
                        "all" => "All",
                        "crossfit" => "CrossFit",
                        "strength" => "Strength",
                        "meditation" => "Meditation",
                        "breathing" => "Breathing",
                        "chanting" => "Chanting",
                        _ => cat,
                    };
                    let cat_active = cat_str.clone();
                    let cat_click = cat_str.clone();
                    view! {
                        <button
                            class="filter-pill"
                            class:active=move || active_filter.get() == cat_active
                            on:click=move |_| active_filter.set(cat_click.clone())
                        >
                            {label}
                        </button>
                    }
                }).collect_view()}
            </div>

            // Collapsible add form
            {move || show_form.get().then(|| view! {
                <form class="exercise-form" on:submit=on_submit>
                    <input
                        type="text"
                        placeholder="Exercise name"
                        prop:value=move || name_input.get()
                        on:input=move |ev| name_input.set(event_target_value(&ev))
                    />
                    <div class="form-row">
                        <select
                            prop:value=move || category_input.get()
                            on:change=move |ev| category_input.set(event_target_value(&ev))
                        >
                            <option value="crossfit">"CrossFit"</option>
                            <option value="strength">"Strength"</option>
                            <option value="meditation">"Meditation"</option>
                            <option value="breathing">"Breathing"</option>
                            <option value="chanting">"Chanting"</option>
                        </select>
                        <input
                            type="text"
                            placeholder="Type (e.g. Olympic)"
                            prop:value=move || movement_type_input.get()
                            on:input=move |ev| movement_type_input.set(event_target_value(&ev))
                        />
                    </div>
                    <input
                        type="text"
                        placeholder="Description (optional)"
                        prop:value=move || description_input.get()
                        on:input=move |ev| description_input.set(event_target_value(&ev))
                    />
                    <button type="submit" class="form-submit">"Add Movement"</button>
                </form>
            })}

            <Suspense fallback=|| view! { <p class="loading">"Loading movements..."</p> }>
                {move || {
                    let filter = active_filter.get();
                    exercises.get().map(|result| {
                        match result {
                            Ok(list) if list.is_empty() => {
                                view! {
                                    <div class="empty-state">
                                        <p class="empty-title">"No movements yet"</p>
                                        <p class="empty-sub">"Tap + Add to build your library"</p>
                                    </div>
                                }.into_any()
                            }
                            Ok(list) => {
                                let filtered: Vec<Exercise> = if filter == "all" {
                                    list
                                } else {
                                    list.into_iter().filter(|e| e.category == filter).collect()
                                };
                                view! {
                                    <div class="exercise-grid">
                                        {filtered.into_iter().map(|ex| {
                                            let id = ex.id.clone();
                                            let cat = ex.category.clone();
                                            let badge_text = category_badge(&cat);
                                            let badge_cls = category_class(&cat);
                                            view! {
                                                <div class="exercise-card">
                                                    <div class="exercise-card-top">
                                                        <span class={format!("exercise-badge {}", badge_cls)}>{badge_text}</span>
                                                        <button
                                                            class="exercise-delete"
                                                            on:click={
                                                                let id = id.clone();
                                                                move |_| {
                                                                    delete_action.dispatch(DeleteExercise { id: id.clone() });
                                                                }
                                                            }
                                                        >"x"</button>
                                                    </div>
                                                    <h3 class="exercise-name">{ex.name}</h3>
                                                    {ex.movement_type.map(|mt| view! {
                                                        <span class="exercise-type">{mt}</span>
                                                    })}
                                                </div>
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
    }
}
