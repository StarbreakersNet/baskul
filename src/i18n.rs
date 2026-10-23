//! Traductions du daemon (menus, état de la connexion). Les textes sont dans
//! `locales/*.json`, section « daemon », à côté de ceux de l'app (section
//! « app », lue par vue-i18n). `{nom}` est remplacé par la valeur donnée.

use crate::config::Language;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::OnceLock;

const FR: &str = include_str!("../locales/fr.json");
const EN: &str = include_str!("../locales/en.json");

static TEXTS: OnceLock<HashMap<String, String>> = OnceLock::new();

/// Choisit la langue, une fois pour toutes, au démarrage.
pub fn init(pref: Language) {
    let lang = match pref {
        Language::Fr => "fr",
        Language::En => "en",
        Language::System => system_language(),
    };
    let _ = TEXTS.set(load(lang));
}

/// Langue du système : français ou, à défaut, anglais.
fn system_language() -> &'static str {
    let locale = sys_locale::get_locale().unwrap_or_default();
    if locale.to_ascii_lowercase().starts_with("fr") { "fr" } else { "en" }
}

/// Textes de la langue, complétés par l'anglais pour les clés absentes.
fn load(lang: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for text in [EN, if lang == "fr" { FR } else { EN }] {
        if let Ok(v) = serde_json::from_str::<Value>(text) {
            flatten(&v["daemon"], "", &mut map);
        }
    }
    map
}

fn flatten(v: &Value, prefix: &str, out: &mut HashMap<String, String>) {
    match v {
        Value::Object(o) => {
            for (k, v) in o {
                let key = if prefix.is_empty() { k.clone() } else { format!("{prefix}.{k}") };
                flatten(v, &key, out);
            }
        }
        Value::String(s) => {
            out.insert(prefix.to_string(), s.clone());
        }
        _ => {}
    }
}

/// Texte traduit (français si la langue n'a pas été choisie, dans les tests).
pub fn t(key: &str) -> String {
    tf(key, &[])
}

/// Texte traduit, avec ses valeurs : `tf("status.connected", &[("peer", "Mac")])`.
pub fn tf(key: &str, args: &[(&str, &str)]) -> String {
    let texts = TEXTS.get_or_init(|| load("fr"));
    let mut s = texts.get(key).cloned().unwrap_or_else(|| key.to_string());
    for (name, value) in args {
        s = s.replace(&format!("{{{name}}}"), value);
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_keys_in_every_language() {
        let (mut fr, mut en) = (HashMap::new(), HashMap::new());
        flatten(&serde_json::from_str::<Value>(FR).unwrap()["daemon"], "", &mut fr);
        flatten(&serde_json::from_str::<Value>(EN).unwrap()["daemon"], "", &mut en);
        let mut a: Vec<_> = fr.keys().collect();
        let mut b: Vec<_> = en.keys().collect();
        a.sort();
        b.sort();
        assert_eq!(a, b);
        assert!(!fr.is_empty());
    }

    #[test]
    fn english_and_placeholders() {
        let en = load("en");
        assert_eq!(en["menu.settings"], "Settings…");
        assert_eq!(tf("status.connected", &[("peer", "Mac")]), "Connecté à Mac");
        assert_eq!(t("cle.inconnue"), "cle.inconnue");
    }
}
