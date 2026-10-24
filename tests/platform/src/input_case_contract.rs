// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Pure event-oracle checks shared by native observers and headless regression tests.

use serde_json::Value;

pub(crate) fn matches_events(spec: &Value, observed: &[Value]) -> bool {
    let Some(expected) = spec["expected"].as_array() else {
        return false;
    };
    if expected.is_empty() {
        return false;
    }
    let normalized: Vec<_> = observed
        .iter()
        .filter(|e| spec["family"] != "repeat" || e["action"] != "Repeat")
        .collect();
    if normalized.len() != expected.len() {
        return false;
    }
    if spec["family"] == "repeat"
        && (observed.len() < 3
            || observed.first().is_none_or(|e| e["action"] != "Press")
            || observed.last().is_none_or(|e| e["action"] != "Release")
            || !observed[1..observed.len() - 1].iter().all(|e| {
                e["kind"] == "key"
                    && e["key"] == 65
                    && e["action"] == "Repeat"
                    && e["scancode"] == expected[0]["scancode"]
                    && e["modifiers"]
                        .as_i64()
                        .is_some_and(|m| m.trailing_zeros() >= 4)
            }))
    {
        return false;
    }
    normalized
        .into_iter()
        .zip(expected)
        .all(|(actual, expected)| {
            expected.as_object().is_some_and(|fields| {
                fields.iter().all(|(name, value)| {
                    if name == "modifiers" {
                        actual[name].as_i64().is_some_and(|bits| {
                            bits & spec["modifier_mask"].as_i64().unwrap_or(15)
                                == value.as_i64().unwrap_or(-1)
                        })
                    } else {
                        actual[name] == *value
                    }
                })
            })
        })
}

pub(crate) fn lock_events_match(spec: &Value, observed: &[Value]) -> bool {
    if spec["family"] != "lock-modifiers" {
        return true;
    }
    if observed.len() != 12 {
        return false;
    }
    let Some(bit) = spec["lock_bit"].as_i64() else {
        return false;
    };
    let masks: Option<Vec<_>> = [2, 3, 4, 5, 8, 9, 10, 11]
        .into_iter()
        .map(|i| observed[i]["modifiers"].as_i64())
        .collect();
    masks.is_some_and(|m| {
        m[..4].iter().all(|v| v & bit == m[0] & bit)
            && m[4..].iter().all(|v| v & bit == m[4] & bit)
            && m[0] & bit != m[4] & bit
    })
}
