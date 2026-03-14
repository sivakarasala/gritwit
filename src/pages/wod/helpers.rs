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
