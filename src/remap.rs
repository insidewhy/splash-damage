use evdev::{EventType, InputEvent, Key};
use std::collections::HashSet;

use crate::config::RemapRule;
use crate::window::SharedActiveWindow;

const KEY_PRESS: i32 = 1;
const KEY_RELEASE: i32 = 0;

const LEFT_RIGHT_MODIFIER_PAIRS: &[(Key, Key)] = &[
    (Key::KEY_LEFTCTRL, Key::KEY_RIGHTCTRL),
    (Key::KEY_LEFTSHIFT, Key::KEY_RIGHTSHIFT),
    (Key::KEY_LEFTALT, Key::KEY_RIGHTALT),
    (Key::KEY_LEFTMETA, Key::KEY_RIGHTMETA),
];

pub struct Remapper {
    rules: Vec<RemapRule>,
    active_window: SharedActiveWindow,
    pressed_keys: HashSet<Key>,
}

impl Remapper {
    pub fn new(rules: Vec<RemapRule>, active_window: SharedActiveWindow) -> Self {
        Self {
            rules,
            active_window,
            pressed_keys: HashSet::new(),
        }
    }

    pub fn process_event(&mut self, event: InputEvent) -> Vec<InputEvent> {
        if event.event_type() != EventType::KEY {
            return vec![event];
        }

        let key = Key::new(event.code());
        let value = event.value();

        match value {
            KEY_PRESS => {
                self.pressed_keys.insert(key);
            }
            KEY_RELEASE => {
                self.pressed_keys.remove(&key);
            }
            _ => return vec![event], // repeat events pass through
        }

        if let Some(rule) = self.find_matching_rule(key) {
            self.apply_remap(&rule, key, value)
        } else {
            vec![event]
        }
    }

    fn find_matching_rule(&self, trigger_key: Key) -> Option<RemapRule> {
        let window_class = self
            .active_window
            .try_read()
            .ok()
            .and_then(|w| w.as_ref().map(|w| w.resource_class.clone()))
            .unwrap_or_default();

        for rule in &self.rules {
            if rule.from.key != trigger_key {
                continue;
            }

            let all_modifiers_held = rule.from.modifiers.iter().all(|m| self.is_modifier_held(*m));
            if !all_modifiers_held {
                continue;
            }

            let excluded = rule
                .exclude
                .iter()
                .any(|exc| window_class.eq_ignore_ascii_case(exc));
            if excluded {
                continue;
            }

            return Some(rule.clone());
        }

        None
    }

    fn is_modifier_held(&self, modifier: Key) -> bool {
        if self.pressed_keys.contains(&modifier) {
            return true;
        }
        for (left, right) in LEFT_RIGHT_MODIFIER_PAIRS {
            if modifier == *left && self.pressed_keys.contains(right) {
                return true;
            }
            if modifier == *right && self.pressed_keys.contains(left) {
                return true;
            }
        }
        false
    }

    fn apply_remap(&self, rule: &RemapRule, _trigger_key: Key, value: i32) -> Vec<InputEvent> {
        let mut events = Vec::new();

        if value == KEY_PRESS {
            // Release the "from" modifiers that aren't in "to"
            for from_mod in &rule.from.modifiers {
                let needed_in_to = rule.to.modifiers.iter().any(|to_mod| {
                    self.same_modifier_group(*from_mod, *to_mod)
                });
                if !needed_in_to {
                    events.push(key_event(*from_mod, KEY_RELEASE));
                }
            }

            // Press the "to" modifiers that aren't already held from "from"
            for to_mod in &rule.to.modifiers {
                let already_from = rule.from.modifiers.iter().any(|from_mod| {
                    self.same_modifier_group(*from_mod, *to_mod)
                });
                if !already_from {
                    events.push(key_event(*to_mod, KEY_PRESS));
                }
            }

            events.push(key_event(rule.to.key, KEY_PRESS));
        } else {
            events.push(key_event(rule.to.key, KEY_RELEASE));

            // Release "to" modifiers we injected, re-press "from" modifiers
            for to_mod in &rule.to.modifiers {
                let was_from = rule.from.modifiers.iter().any(|from_mod| {
                    self.same_modifier_group(*from_mod, *to_mod)
                });
                if !was_from {
                    events.push(key_event(*to_mod, KEY_RELEASE));
                }
            }

            for from_mod in &rule.from.modifiers {
                let is_to = rule.to.modifiers.iter().any(|to_mod| {
                    self.same_modifier_group(*from_mod, *to_mod)
                });
                if !is_to {
                    events.push(key_event(*from_mod, KEY_PRESS));
                }
            }
        }

        events.push(syn_event());
        events
    }

    fn same_modifier_group(&self, a: Key, b: Key) -> bool {
        if a == b {
            return true;
        }
        for (left, right) in LEFT_RIGHT_MODIFIER_PAIRS {
            if (a == *left || a == *right) && (b == *left || b == *right) {
                return true;
            }
        }
        false
    }
}

fn key_event(key: Key, value: i32) -> InputEvent {
    InputEvent::new(EventType::KEY, key.code(), value)
}

fn syn_event() -> InputEvent {
    InputEvent::new(EventType::SYNCHRONIZATION, 0, 0)
}
