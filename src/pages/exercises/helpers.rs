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

pub fn category_badge(cat: &str) -> &'static str {
    CATEGORIES
        .iter()
        .find(|(v, _, _, _)| *v == cat)
        .map(|(_, _, b, _)| *b)
        .unwrap_or("GEN")
}

pub fn category_class(cat: &str) -> &'static str {
    CATEGORIES
        .iter()
        .find(|(v, _, _, _)| *v == cat)
        .map(|(_, _, _, c)| *c)
        .unwrap_or("")
}

pub fn category_select_options() -> Vec<SelectOption> {
    CATEGORIES
        .iter()
        .map(|(val, label, _, _)| SelectOption {
            value: val.to_string(),
            label: label.to_string(),
        })
        .collect()
}
