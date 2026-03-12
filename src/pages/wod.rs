use crate::auth::{AuthUser, UserRole};
use crate::db::{Wod, WodMovement};
use leptos::prelude::*;

#[server]
async fn list_wods() -> Result<Vec<Wod>, ServerFnError> {
    let _user = crate::auth::session::require_auth().await?;
    let pool = crate::db::db().await?;
    crate::db::list_wods_db(&pool)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
async fn create_wod(
    title: String,
    description: String,
    workout_type: String,
    time_cap_minutes: String,
    programmed_date: String,
) -> Result<String, ServerFnError> {
    let user = crate::auth::session::require_role(UserRole::Coach).await?;
    let pool = crate::db::db().await?;
    let user_uuid: uuid::Uuid = user
        .id
        .parse()
        .map_err(|e: uuid::Error| ServerFnError::new(e.to_string()))?;
    let time_cap = if time_cap_minutes.is_empty() {
        None
    } else {
        time_cap_minutes.parse::<i32>().ok()
    };
    let desc = if description.is_empty() { None } else { Some(description.as_str()) };
    crate::db::create_wod_db(
        &pool,
        &title,
        desc,
        &workout_type,
        time_cap,
        &programmed_date,
        Some(user_uuid),
    )
    .await
    .map(|id| id.to_string())
    .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
async fn delete_wod(id: String) -> Result<(), ServerFnError> {
    crate::auth::session::require_role(UserRole::Coach).await?;
    let pool = crate::db::db().await?;
    let uuid: uuid::Uuid = id
        .parse()
        .map_err(|e: uuid::Error| ServerFnError::new(e.to_string()))?;
    crate::db::delete_wod_db(&pool, uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
async fn get_wod_movements(wod_id: String) -> Result<Vec<WodMovement>, ServerFnError> {
    let _user = crate::auth::session::require_auth().await?;
    let pool = crate::db::db().await?;
    let uuid: uuid::Uuid = wod_id
        .parse()
        .map_err(|e: uuid::Error| ServerFnError::new(e.to_string()))?;
    crate::db::get_wod_movements_db(&pool, uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
async fn add_wod_movement(
    wod_id: String,
    exercise_id: String,
    reps: String,
    sets: String,
    weight_kg: String,
    notes: String,
) -> Result<(), ServerFnError> {
    crate::auth::session::require_role(UserRole::Coach).await?;
    let pool = crate::db::db().await?;
    let wod_uuid: uuid::Uuid = wod_id
        .parse()
        .map_err(|e: uuid::Error| ServerFnError::new(e.to_string()))?;
    let ex_uuid: uuid::Uuid = exercise_id
        .parse()
        .map_err(|e: uuid::Error| ServerFnError::new(e.to_string()))?;
    let reps_opt = if reps.is_empty() { None } else { reps.parse::<i32>().ok() };
    let sets_opt = if sets.is_empty() { None } else { sets.parse::<i32>().ok() };
    let weight_opt = if weight_kg.is_empty() { None } else { weight_kg.parse::<f32>().ok() };
    let notes_opt = if notes.is_empty() { None } else { Some(notes.as_str()) };
    crate::db::add_wod_movement_db(&pool, wod_uuid, ex_uuid, reps_opt, sets_opt, weight_opt, notes_opt, 0)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
async fn delete_wod_movement(id: String) -> Result<(), ServerFnError> {
    crate::auth::session::require_role(UserRole::Coach).await?;
    let pool = crate::db::db().await?;
    let uuid: uuid::Uuid = id
        .parse()
        .map_err(|e: uuid::Error| ServerFnError::new(e.to_string()))?;
    crate::db::delete_wod_movement_db(&pool, uuid)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
async fn list_exercises_for_wod() -> Result<Vec<(String, String)>, ServerFnError> {
    let _user = crate::auth::session::require_auth().await?;
    let pool = crate::db::db().await?;
    crate::db::list_exercises_db(&pool)
        .await
        .map(|exs| exs.into_iter().map(|e| (e.id, e.name)).collect())
        .map_err(|e| ServerFnError::new(e.to_string()))
}

fn wod_type_label(t: &str) -> &'static str {
    match t {
        "amrap"    => "AMRAP",
        "fortime"  => "FOR TIME",
        "emom"     => "EMOM",
        "tabata"   => "TABATA",
        "strength" => "STRENGTH",
        _          => "CUSTOM",
    }
}

fn wod_type_class(t: &str) -> &'static str {
    match t {
        "amrap"    => "wod-badge--amrap",
        "fortime"  => "wod-badge--fortime",
        "emom"     => "wod-badge--emom",
        "tabata"   => "wod-badge--tabata",
        "strength" => "wod-badge--strength",
        _          => "wod-badge--custom",
    }
}

#[component]
pub fn WodPage() -> impl IntoView {
    let user = use_context::<AuthUser>().unwrap();
    let is_coach = matches!(user.role, UserRole::Coach | UserRole::Admin);

    let create_action = ServerAction::<CreateWod>::new();
    let delete_action = ServerAction::<DeleteWod>::new();
    // Movement deletion lives here so the modal renders at page level
    let del_mov_action = ServerAction::<DeleteWodMovement>::new();

    let wods = Resource::new(
        move || (create_action.version().get(), delete_action.version().get()),
        |_| list_wods(),
    );

    let show_form = RwSignal::new(false);
    let title_input = RwSignal::new(String::new());
    let desc_input = RwSignal::new(String::new());
    let type_input = RwSignal::new("fortime".to_string());
    let cap_input = RwSignal::new(String::new());
    let date_input = RwSignal::new(String::new());

    let expanded_wod: RwSignal<Option<String>> = RwSignal::new(None);

    let show_delete_wod = RwSignal::new(false);
    let pending_delete_wod_id = RwSignal::new(String::new());

    // Signals for movement deletion — passed into WodMovementsPanel as props
    let show_delete_mov = RwSignal::new(false);
    let pending_delete_mov_id = RwSignal::new(String::new());
    let del_mov_version = del_mov_action.version();

    let fab_view = if is_coach {
        view! {
            <button
                class={move || if show_form.get() { "fab fab--active" } else { "fab" }}
                on:click=move |_| show_form.update(|v| *v = !*v)
            >
                <span class="fab-icon"></span>
            </button>
        }.into_any()
    } else {
        view! { }.into_any()
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
            }.into_any()
        } else {
            view! { }.into_any()
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
                        <p class="empty-title">"No WODs programmed yet"</p>
                        {is_coach.then(|| view! {
                            <p class="empty-sub">"Use + to program today's WOD"</p>
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
                                    expanded_wod=expanded_wod
                                    pending_delete_wod_id=pending_delete_wod_id
                                    show_delete_wod=show_delete_wod
                                    del_mov_version=del_mov_version
                                    show_delete_mov=show_delete_mov
                                    pending_delete_mov_id=pending_delete_mov_id
                                />
                            }
                        }).collect_view()}
                    </div>
                }.into_any(),
            })}
        </Suspense>
    }.into_any();

    let wod_modal = view! {
        <WodDeleteModal
            show_delete=show_delete_wod
            pending_delete_id=pending_delete_wod_id
            delete_action=delete_action
            msg="Delete this WOD?"
            sub="All movements will also be removed. This cannot be undone."
            btn_label="Delete"
        />
    }.into_any();

    let mov_modal = view! {
        <MovDeleteModal
            show_delete=show_delete_mov
            pending_delete_id=pending_delete_mov_id
            delete_action=del_mov_action
        />
    }.into_any();

    view! {
        <div class="wod-page">
            {fab_view}
            {form_view}
            {list_view}
            {wod_modal}
            {mov_modal}
        </div>
    }
}

#[component]
fn WodMovementsPanel(
    wod_id: String,
    is_coach: bool,
    del_mov_version: RwSignal<usize>,
    show_delete_mov: RwSignal<bool>,
    pending_delete_mov_id: RwSignal<String>,
) -> impl IntoView {
    let add_action = ServerAction::<AddWodMovement>::new();

    let wid_resource = wod_id.clone();
    let movements = Resource::new(
        move || (wid_resource.clone(), add_action.version().get(), del_mov_version.get()),
        |(id, _, _)| get_wod_movements(id),
    );

    let exercises = Resource::new(|| (), |_| list_exercises_for_wod());

    let show_add = RwSignal::new(false);
    let ex_id = RwSignal::new(String::new());
    let reps_input = RwSignal::new(String::new());
    let sets_input = RwSignal::new(String::new());
    let weight_input = RwSignal::new(String::new());
    let notes_input = RwSignal::new(String::new());

    let wid_submit = wod_id.clone();

    view! {
        <div class="wod-movements">
            <Suspense fallback=|| view! { <span class="wod-mov-loading">"Loading..."</span> }>
                {move || movements.get().map(|res| match res {
                    Err(_) => view! {
                        <p class="wod-mov-error">"Could not load movements"</p>
                    }.into_any(),
                    Ok(movs) if movs.is_empty() => view! {
                        <p class="wod-mov-empty">"No movements yet"</p>
                    }.into_any(),
                    Ok(movs) => view! {
                        <ol class="wod-mov-list">
                            {movs.into_iter().map(|m| {
                                let parts: Vec<String> = [
                                    m.sets.map(|s| format!("{}×", s)),
                                    m.reps.map(|r| format!("{} reps", r)),
                                    m.weight_kg.map(|w| format!("@ {}kg", w)),
                                ]
                                .into_iter()
                                .flatten()
                                .collect();
                                let detail = parts.join(" ");
                                let mid = m.id.clone();
                                view! {
                                    <li class="wod-mov-item">
                                        <span class="wod-mov-name">{m.exercise_name}</span>
                                        {(!detail.is_empty()).then(|| view! {
                                            <span class="wod-mov-detail">{detail}</span>
                                        })}
                                        {is_coach.then(|| view! {
                                            <button
                                                class="wod-mov-delete"
                                                on:click=move |_| {
                                                    pending_delete_mov_id.set(mid.clone());
                                                    show_delete_mov.set(true);
                                                }
                                            >"×"</button>
                                        })}
                                    </li>
                                }
                            }).collect_view()}
                        </ol>
                    }.into_any(),
                })}
            </Suspense>

            {is_coach.then(move || view! {
                <div class="wod-add-movement">
                    {move || (!show_add.get()).then(|| view! {
                        <button
                            class="wod-add-mov-btn"
                            on:click=move |_| show_add.set(true)
                        >"+ Add Movement"</button>
                    })}
                    {move || show_add.get().then(|| {
                        let wid = wid_submit.clone();
                        view! {
                            <form
                                class="wod-mov-form"
                                on:submit=move |ev| {
                                    ev.prevent_default();
                                    if ex_id.get_untracked().is_empty() { return; }
                                    add_action.dispatch(AddWodMovement {
                                        wod_id: wid.clone(),
                                        exercise_id: ex_id.get_untracked(),
                                        reps: reps_input.get_untracked(),
                                        sets: sets_input.get_untracked(),
                                        weight_kg: weight_input.get_untracked(),
                                        notes: notes_input.get_untracked(),
                                    });
                                    reps_input.set(String::new());
                                    sets_input.set(String::new());
                                    weight_input.set(String::new());
                                    notes_input.set(String::new());
                                    show_add.set(false);
                                }
                            >
                                <Suspense fallback=|| view! { <span>"Loading..."</span> }>
                                    {move || exercises.get().map(|res| match res {
                                        Ok(exs) => view! {
                                            <select
                                                prop:value=move || ex_id.get()
                                                on:change=move |ev| ex_id.set(event_target_value(&ev))
                                            >
                                                <option value="">"Select exercise"</option>
                                                {exs.into_iter().map(|(id, name)| view! {
                                                    <option value={id}>{name}</option>
                                                }).collect_view()}
                                            </select>
                                        }.into_any(),
                                        Err(_) => view! {
                                            <span>"Failed to load exercises"</span>
                                        }.into_any(),
                                    })}
                                </Suspense>
                                <div class="form-row">
                                    <input
                                        type="number"
                                        placeholder="Sets"
                                        prop:value=move || sets_input.get()
                                        on:input=move |ev| sets_input.set(event_target_value(&ev))
                                    />
                                    <input
                                        type="number"
                                        placeholder="Reps"
                                        prop:value=move || reps_input.get()
                                        on:input=move |ev| reps_input.set(event_target_value(&ev))
                                    />
                                    <input
                                        type="number"
                                        placeholder="kg"
                                        prop:value=move || weight_input.get()
                                        on:input=move |ev| weight_input.set(event_target_value(&ev))
                                    />
                                </div>
                                <div class="wod-mov-form-btns">
                                    <button type="submit" class="form-submit">"Add"</button>
                                    <button
                                        type="button"
                                        class="wod-cancel-btn"
                                        on:click=move |_| show_add.set(false)
                                    >"Cancel"</button>
                                </div>
                            </form>
                        }
                    })}
                </div>
            })}
        </div>
    }
}

#[component]
fn WodForm(
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
            <button type="submit" class="form-submit">"Create WOD"</button>
        </form>
    }
}

#[component]
fn WodCard(
    wod: Wod,
    is_coach: bool,
    expanded_wod: RwSignal<Option<String>>,
    pending_delete_wod_id: RwSignal<String>,
    show_delete_wod: RwSignal<bool>,
    del_mov_version: RwSignal<usize>,
    show_delete_mov: RwSignal<bool>,
    pending_delete_mov_id: RwSignal<String>,
) -> impl IntoView {
    let wid = wod.id.clone();
    let wid_del = wod.id.clone();
    let wid_exp = wod.id.clone();
    let wid_panel = wod.id.clone();
    let type_label = wod_type_label(&wod.workout_type);
    let type_cls = format!("wod-badge {}", wod_type_class(&wod.workout_type));
    let cap = wod.time_cap_minutes;
    let title = wod.title.clone();
    let desc = wod.description.clone();
    let date = wod.programmed_date.clone();

    view! {
        <div class="wod-card">
            <div class="wod-card-top">
                <div class="wod-card-meta">
                    <span class={type_cls}>{type_label}</span>
                    <span class="wod-date">{date}</span>
                </div>
                <div class="wod-card-actions">
                    <button
                        class="wod-expand-btn"
                        on:click=move |_| {
                            expanded_wod.update(|v| {
                                if v.as_ref() == Some(&wid) {
                                    *v = None;
                                } else {
                                    *v = Some(wid.clone());
                                }
                            });
                        }
                    >
                        {move || if expanded_wod.get().as_ref() == Some(&wid_exp) {
                            "▲"
                        } else {
                            "▼"
                        }}
                    </button>
                    {is_coach.then(|| view! {
                        <button
                            class="wod-delete"
                            on:click=move |_| {
                                pending_delete_wod_id.set(wid_del.clone());
                                show_delete_wod.set(true);
                            }
                        >"×"</button>
                    })}
                </div>
            </div>
            <h2 class="wod-title">{title}</h2>
            {desc.map(|d| view! {
                <p class="wod-desc">{d}</p>
            })}
            {cap.map(|c| view! {
                <span class="wod-timecap">"⏱ "{c}" min"</span>
            })}
            {move || {
                let is_exp = expanded_wod.get().as_ref() == Some(&wid_panel);
                is_exp.then(|| view! {
                    <WodMovementsPanel
                        wod_id=wid_panel.clone()
                        is_coach=is_coach
                        del_mov_version=del_mov_version
                        show_delete_mov=show_delete_mov
                        pending_delete_mov_id=pending_delete_mov_id
                    />
                })
            }}
        </div>
    }
}

#[component]
fn WodDeleteModal(
    show_delete: RwSignal<bool>,
    pending_delete_id: RwSignal<String>,
    delete_action: ServerAction<DeleteWod>,
    #[prop(into)] msg: String,
    #[prop(into)] sub: String,
    #[prop(into)] btn_label: String,
) -> impl IntoView {
    view! {
        <div
            class="confirm-overlay"
            style=move || if show_delete.get() { "display:flex" } else { "display:none" }
            on:click=move |_| show_delete.set(false)
        >
            <div class="confirm-dialog" on:click=move |ev| { ev.stop_propagation(); }>
                <p class="confirm-msg">{msg}</p>
                <p class="confirm-sub">{sub}</p>
                <div class="confirm-actions">
                    <button
                        class="confirm-cancel-btn"
                        on:click=move |_| show_delete.set(false)
                    >"Cancel"</button>
                    <button
                        class="confirm-delete-btn"
                        on:click=move |_| {
                            delete_action.dispatch(DeleteWod { id: pending_delete_id.get_untracked() });
                            show_delete.set(false);
                        }
                    >{btn_label.clone()}</button>
                </div>
            </div>
        </div>
    }
}

#[component]
fn MovDeleteModal(
    show_delete: RwSignal<bool>,
    pending_delete_id: RwSignal<String>,
    delete_action: ServerAction<DeleteWodMovement>,
) -> impl IntoView {
    view! {
        <div
            class="confirm-overlay"
            style=move || if show_delete.get() { "display:flex" } else { "display:none" }
            on:click=move |_| show_delete.set(false)
        >
            <div class="confirm-dialog" on:click=move |ev| { ev.stop_propagation(); }>
                <p class="confirm-msg">"Remove this movement?"</p>
                <p class="confirm-sub">"This cannot be undone."</p>
                <div class="confirm-actions">
                    <button
                        class="confirm-cancel-btn"
                        on:click=move |_| show_delete.set(false)
                    >"Cancel"</button>
                    <button
                        class="confirm-delete-btn"
                        on:click=move |_| {
                            delete_action.dispatch(DeleteWodMovement {
                                id: pending_delete_id.get_untracked(),
                            });
                            show_delete.set(false);
                        }
                    >"Remove"</button>
                </div>
            </div>
        </div>
    }
}
