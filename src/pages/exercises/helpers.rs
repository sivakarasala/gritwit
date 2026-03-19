use crate::components::SelectOption;

/// Convert a YouTube or Vimeo URL to its embed URL. Returns None for local/other URLs.
pub fn to_embed_url(url: &str) -> Option<String> {
    if url.contains("youtube.com/watch") {
        if let Some(pos) = url.find("v=") {
            let id = &url[pos + 2..];
            let id = id.split('&').next().unwrap_or(id);
            return Some(format!(
                "https://www.youtube.com/embed/{}?playsinline=1&enablejsapi=1",
                id
            ));
        }
    }
    if url.contains("youtu.be/") {
        if let Some(pos) = url.find("youtu.be/") {
            let id = &url[pos + 9..];
            let id = id.split('?').next().unwrap_or(id);
            return Some(format!(
                "https://www.youtube.com/embed/{}?playsinline=1&enablejsapi=1",
                id
            ));
        }
    }
    if url.contains("vimeo.com/") {
        if let Some(pos) = url.rfind('/') {
            let id = &url[pos + 1..];
            let id = id.split('?').next().unwrap_or(id);
            if id.chars().all(|c| c.is_ascii_digit()) {
                return Some(format!("https://player.vimeo.com/video/{}", id));
            }
        }
    }
    None
}

/// Single source of truth for exercise categories.
/// Each entry: (value, label, badge, css_class)
pub const CATEGORIES: &[(&str, &str, &str, &str)] = &[
    ("conditioning", "Conditioning", "CON", "badge--conditioning"),
    ("gymnastics", "Gymnastics", "GYM", "badge--gymnastics"),
    (
        "weightlifting",
        "Weightlifting",
        "WL",
        "badge--weightlifting",
    ),
    ("powerlifting", "Powerlifting", "PWR", "badge--powerlifting"),
    ("cardio", "Cardio", "CRD", "badge--cardio"),
    ("bodybuilding", "Bodybuilding", "BB", "badge--bodybuilding"),
    ("strongman", "Strongman", "STR", "badge--strongman"),
    ("plyometrics", "Plyometrics", "PLY", "badge--plyometrics"),
    ("calisthenics", "Calisthenics", "CAL", "badge--calisthenics"),
    ("mobility", "Mobility", "MOB", "badge--mobility"),
    ("yoga", "Yoga", "YGA", "badge--yoga"),
    ("meditation", "Meditation", "MED", "badge--meditation"),
    ("breathing", "Breathing", "BRE", "badge--breathing"),
    ("chanting", "Chanting", "CHN", "badge--chanting"),
    ("sports", "Sports", "SPT", "badge--sports"),
    ("warmup", "Warm Up", "WRM", "badge--warmup"),
    ("cooldown", "Cool Down", "CLD", "badge--cooldown"),
];

pub fn category_select_options() -> Vec<SelectOption> {
    CATEGORIES
        .iter()
        .map(|(val, label, _, _)| SelectOption {
            value: val.to_string(),
            label: label.to_string(),
        })
        .collect()
}

/// Returns the default scoring type for a given category.
pub fn default_scoring_type(category: &str) -> &'static str {
    match category {
        "gymnastics" | "calisthenics" | "plyometrics" | "warmup" => "reps_only",
        "cardio" | "conditioning" => "distance",
        "yoga" | "mobility" | "meditation" | "breathing" | "chanting" | "cooldown" | "sports" => {
            "time"
        }
        _ => "weight_and_reps",
    }
}

pub fn category_color(cat: &str) -> &'static str {
    match cat {
        "conditioning" => "#e74c3c",
        "gymnastics" => "#9b59b6",
        "weightlifting" => "#3498db",
        "powerlifting" => "#2980b9",
        "cardio" => "#e67e22",
        "bodybuilding" => "#c0392b",
        "strongman" => "#7f8c8d",
        "plyometrics" => "#d35400",
        "calisthenics" => "#16a085",
        "mobility" => "#1abc9c",
        "yoga" => "#8e44ad",
        "meditation" => "#2ecc71",
        "breathing" => "#7fb3d3",
        "chanting" => "#f39c12",
        "sports" => "#f1c40f",
        "warmup" => "#ffa500",
        "cooldown" => "#6495ed",
        _ => "#888",
    }
}

pub fn scoring_type_options() -> Vec<SelectOption> {
    vec![
        SelectOption {
            value: "weight_and_reps".into(),
            label: "Weight & Reps".into(),
        },
        SelectOption {
            value: "reps_only".into(),
            label: "Reps Only".into(),
        },
        SelectOption {
            value: "distance".into(),
            label: "Distance".into(),
        },
        SelectOption {
            value: "calories".into(),
            label: "Calories".into(),
        },
        SelectOption {
            value: "time".into(),
            label: "Time".into(),
        },
    ]
}
