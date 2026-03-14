use leptos::prelude::*;

use super::custom_log::{ExerciseEntry, SetData};
use super::set_row::SetRow;

/// Card for a single exercise with its sets.
#[component]
pub(super) fn ExerciseEntryCard(
    entry_key: usize,
    exercise_name: String,
    sets: Vec<SetData>,
    exercises: RwSignal<Vec<ExerciseEntry>>,
) -> impl IntoView {
    let remove = move |_| {
        exercises.update(|list| {
            list.retain(|e| e.key != entry_key);
        });
    };

    let add_set = move |_| {
        exercises.update(|list| {
            if let Some(e) = list.iter_mut().find(|e| e.key == entry_key) {
                let next = e.sets.len() as i32 + 1;
                e.sets.push(SetData::new(next));
            }
        });
    };

    view! {
        <div class="exercise-entry-card">
            <div class="exercise-entry-header">
                <span class="exercise-entry-name">{exercise_name}</span>
                <button class="exercise-entry-remove" on:click=remove>"×"</button>
            </div>

            <div class="exercise-sets">
                {sets
                    .into_iter()
                    .map(|set| {
                        view! {
                            <SetRow
                                entry_key=entry_key
                                set=set
                                exercises=exercises
                            />
                        }
                    })
                    .collect_view()}
            </div>

            <button class="add-set-btn" on:click=add_set>"+ Add Set"</button>
        </div>
    }
}
