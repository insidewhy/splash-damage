use evdev::Key;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(rename = "remap")]
    pub remaps: Vec<RemapEntry>,
}

#[derive(Debug, Deserialize)]
pub struct RemapEntry {
    pub from: String,
    pub to: String,
    #[serde(default)]
    pub exclude: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct KeyCombo {
    pub modifiers: Vec<Key>,
    pub key: Key,
}

#[derive(Debug, Clone)]
pub struct RemapRule {
    pub from: KeyCombo,
    pub to: KeyCombo,
    pub exclude: Vec<String>,
}

pub fn load_config(path: &Path) -> anyhow::Result<Vec<RemapRule>> {
    let content = std::fs::read_to_string(path)?;
    let config: Config = toml::from_str(&content)?;

    config
        .remaps
        .into_iter()
        .map(|entry| {
            let from = parse_key_combo(&entry.from)?;
            let to = parse_key_combo(&entry.to)?;
            Ok(RemapRule {
                from,
                to,
                exclude: entry.exclude,
            })
        })
        .collect()
}

fn parse_key_combo(s: &str) -> anyhow::Result<KeyCombo> {
    let parts: Vec<&str> = s.split('+').map(str::trim).collect();
    if parts.is_empty() {
        anyhow::bail!("empty key combo");
    }

    let mut modifiers = Vec::new();
    for part in &parts[..parts.len() - 1] {
        modifiers.push(parse_modifier(part)?);
    }

    let key = parse_key(parts.last().unwrap())?;
    Ok(KeyCombo { modifiers, key })
}

fn parse_modifier(s: &str) -> anyhow::Result<Key> {
    match s.to_lowercase().as_str() {
        "ctrl" | "control" => Ok(Key::KEY_LEFTCTRL),
        "shift" => Ok(Key::KEY_LEFTSHIFT),
        "alt" => Ok(Key::KEY_LEFTALT),
        "super" | "meta" | "cmd" => Ok(Key::KEY_LEFTMETA),
        other => anyhow::bail!("unknown modifier: {other}"),
    }
}

fn parse_key(s: &str) -> anyhow::Result<Key> {
    KEYNAME_MAP
        .get(s.to_lowercase().as_str())
        .copied()
        .ok_or_else(|| anyhow::anyhow!("unknown key: {s}"))
}

static KEYNAME_MAP: std::sync::LazyLock<HashMap<&'static str, Key>> =
    std::sync::LazyLock::new(|| {
        let mut m = HashMap::new();
        m.insert("a", Key::KEY_A);
        m.insert("b", Key::KEY_B);
        m.insert("c", Key::KEY_C);
        m.insert("d", Key::KEY_D);
        m.insert("e", Key::KEY_E);
        m.insert("f", Key::KEY_F);
        m.insert("g", Key::KEY_G);
        m.insert("h", Key::KEY_H);
        m.insert("i", Key::KEY_I);
        m.insert("j", Key::KEY_J);
        m.insert("k", Key::KEY_K);
        m.insert("l", Key::KEY_L);
        m.insert("m", Key::KEY_M);
        m.insert("n", Key::KEY_N);
        m.insert("o", Key::KEY_O);
        m.insert("p", Key::KEY_P);
        m.insert("q", Key::KEY_Q);
        m.insert("r", Key::KEY_R);
        m.insert("s", Key::KEY_S);
        m.insert("t", Key::KEY_T);
        m.insert("u", Key::KEY_U);
        m.insert("v", Key::KEY_V);
        m.insert("w", Key::KEY_W);
        m.insert("x", Key::KEY_X);
        m.insert("y", Key::KEY_Y);
        m.insert("z", Key::KEY_Z);
        m.insert("0", Key::KEY_0);
        m.insert("1", Key::KEY_1);
        m.insert("2", Key::KEY_2);
        m.insert("3", Key::KEY_3);
        m.insert("4", Key::KEY_4);
        m.insert("5", Key::KEY_5);
        m.insert("6", Key::KEY_6);
        m.insert("7", Key::KEY_7);
        m.insert("8", Key::KEY_8);
        m.insert("9", Key::KEY_9);
        m.insert("space", Key::KEY_SPACE);
        m.insert("enter", Key::KEY_ENTER);
        m.insert("tab", Key::KEY_TAB);
        m.insert("escape", Key::KEY_ESC);
        m.insert("esc", Key::KEY_ESC);
        m.insert("backspace", Key::KEY_BACKSPACE);
        m.insert("delete", Key::KEY_DELETE);
        m.insert("up", Key::KEY_UP);
        m.insert("down", Key::KEY_DOWN);
        m.insert("left", Key::KEY_LEFT);
        m.insert("right", Key::KEY_RIGHT);
        m.insert("home", Key::KEY_HOME);
        m.insert("end", Key::KEY_END);
        m.insert("pageup", Key::KEY_PAGEUP);
        m.insert("pagedown", Key::KEY_PAGEDOWN);
        m.insert("f1", Key::KEY_F1);
        m.insert("f2", Key::KEY_F2);
        m.insert("f3", Key::KEY_F3);
        m.insert("f4", Key::KEY_F4);
        m.insert("f5", Key::KEY_F5);
        m.insert("f6", Key::KEY_F6);
        m.insert("f7", Key::KEY_F7);
        m.insert("f8", Key::KEY_F8);
        m.insert("f9", Key::KEY_F9);
        m.insert("f10", Key::KEY_F10);
        m.insert("f11", Key::KEY_F11);
        m.insert("f12", Key::KEY_F12);
        m
    });
