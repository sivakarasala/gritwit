pub fn phase_label(p: &str) -> &'static str {
    match p {
        "warmup" => "Warm-Up",
        "strength" => "Strength",
        "conditioning" => "Conditioning",
        "cooldown" => "Cool Down",
        "optional" => "Optional",
        "personal" => "Personal",
        _ => "Section",
    }
}

pub fn section_type_label(t: &str) -> &'static str {
    match t {
        "fortime" => "For Time",
        "amrap" => "AMRAP",
        "emom" => "EMOM",
        "strength" => "Strength",
        _ => "",
    }
}

pub fn phase_class(p: &str) -> &'static str {
    match p {
        "warmup" => "phase-badge--warmup",
        "strength" => "phase-badge--strength",
        "conditioning" => "phase-badge--conditioning",
        "cooldown" => "phase-badge--cooldown",
        "optional" => "phase-badge--optional",
        "personal" => "phase-badge--personal",
        _ => "",
    }
}

/// Derive a human-readable section label from structured fields.
/// Falls back to free-text title if provided.
pub fn section_derived_label(
    section_type: &str,
    time_cap: Option<i32>,
    rounds: Option<i32>,
    title: Option<&str>,
) -> String {
    if let Some(t) = title.filter(|s| !s.is_empty()) {
        return t.to_string();
    }
    match section_type {
        "fortime" => {
            let mut s = String::new();
            if let Some(r) = rounds {
                s.push_str(&format!("{} Rounds ", r));
            }
            s.push_str("For Time");
            if let Some(cap) = time_cap {
                s.push_str(&format!(" · {} min cap", cap));
            }
            s
        }
        "amrap" => {
            if let Some(cap) = time_cap {
                format!("{} min AMRAP", cap)
            } else {
                "AMRAP".to_string()
            }
        }
        "emom" => {
            if let Some(cap) = time_cap {
                format!("{} min EMOM", cap)
            } else {
                "EMOM".to_string()
            }
        }
        "strength" => "Strength".to_string(),
        other => section_type_label(other).to_string(),
    }
}

pub fn wod_type_label(t: &str) -> &'static str {
    match t {
        "amrap" => "AMRAP",
        "fortime" => "FOR TIME",
        "emom" => "EMOM",
        "tabata" => "TABATA",
        "strength" => "STRENGTH",
        _ => "CUSTOM",
    }
}

pub fn wod_type_class(t: &str) -> &'static str {
    match t {
        "amrap" => "wod-badge--amrap",
        "fortime" => "wod-badge--fortime",
        "emom" => "wod-badge--emom",
        "tabata" => "wod-badge--tabata",
        "strength" => "wod-badge--strength",
        _ => "wod-badge--custom",
    }
}
