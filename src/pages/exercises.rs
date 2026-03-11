use crate::db::Exercise;
use leptos::prelude::*;

#[server]
async fn list_exercises() -> Result<Vec<Exercise>, ServerFnError> {
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
    let pool = crate::db::db().await?;
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
    crate::db::create_exercise_db(&pool, &name, &category, mt, &[], desc)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
async fn delete_exercise(id: String) -> Result<(), ServerFnError> {
    let pool = crate::db::db().await?;
    let uuid: uuid::Uuid = id.parse().map_err(|e: uuid::Error| ServerFnError::new(e.to_string()))?;
    crate::db::delete_exercise_db(&pool, uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
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
        }
    };

    view! {
        <div class="exercises-page">
            <h1>"Exercise Library"</h1>

            <form class="exercise-form" on:submit=on_submit>
                <input
                    type="text"
                    placeholder="Exercise name"
                    prop:value=move || name_input.get()
                    on:input=move |ev| name_input.set(event_target_value(&ev))
                />
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
                    placeholder="Movement type (optional)"
                    prop:value=move || movement_type_input.get()
                    on:input=move |ev| movement_type_input.set(event_target_value(&ev))
                />
                <input
                    type="text"
                    placeholder="Description (optional)"
                    prop:value=move || description_input.get()
                    on:input=move |ev| description_input.set(event_target_value(&ev))
                />
                <button type="submit">"Add Exercise"</button>
            </form>

            <Suspense fallback=|| view! { <p>"Loading exercises..."</p> }>
                {move || {
                    exercises.get().map(|result| {
                        match result {
                            Ok(list) if list.is_empty() => {
                                view! { <p class="empty">"No exercises yet. Add one above!"</p> }.into_any()
                            }
                            Ok(list) => {
                                view! {
                                    <table class="exercises-table">
                                        <thead>
                                            <tr>
                                                <th>"Name"</th>
                                                <th>"Category"</th>
                                                <th>"Type"</th>
                                                <th></th>
                                            </tr>
                                        </thead>
                                        <tbody>
                                            {list.into_iter().map(|ex| {
                                                let id = ex.id.clone();
                                                view! {
                                                    <tr>
                                                        <td>{ex.name}</td>
                                                        <td>{ex.category}</td>
                                                        <td>{ex.movement_type.unwrap_or_default()}</td>
                                                        <td>
                                                            <button
                                                                class="btn-delete"
                                                                on:click={
                                                                    let id = id.clone();
                                                                    move |_| {
                                                                        delete_action.dispatch(DeleteExercise { id: id.clone() });
                                                                    }
                                                                }
                                                            >"Delete"</button>
                                                        </td>
                                                    </tr>
                                                }
                                            }).collect_view()}
                                        </tbody>
                                    </table>
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
