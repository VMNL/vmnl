// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Explicit operator expectations, coverage accounting and physical action plan.

use serde_json::Value;
use std::collections::BTreeSet;

pub(crate) struct Step {
    pub name: String,
    pub command: String,
    pub prompt: String,
}

pub(crate) struct Plan {
    pub steps: Vec<Step>,
    pub exclusions: Vec<(String, String)>,
}

fn text<'a>(value: &'a Value, key: &str) -> Result<&'a str, String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .filter(|s| !s.trim().is_empty() && s.len() <= 160 && !s.chars().any(char::is_control))
        .ok_or_else(|| format!("missing/invalid profile text: {key}"))
}

fn index(value: &Value) -> Result<usize, String> {
    value
        .as_u64()
        .and_then(|v| usize::try_from(v).ok())
        .ok_or("invalid profile index".into())
}

fn array<'a>(profile: &'a Value, key: &str) -> Result<&'a [Value], String> {
    profile
        .get(key)
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .ok_or_else(|| format!("profile requires {key} array"))
}

pub(crate) fn validate_identity(profile: &Value) -> Result<(), String> {
    if profile["schema_version"] != 1 {
        return Err("physical profile schema_version must be 1".into());
    }
    text(profile, "model")?;
    if !["usb", "bluetooth"].contains(&text(profile, "connection")?) {
        return Err("connection must be usb or bluetooth".into());
    }
    Ok(())
}

fn cover(
    coverage: &mut BTreeSet<(String, usize)>,
    kind: &str,
    i: usize,
    counts: [usize; 3],
) -> Result<(), String> {
    let count = match kind {
        "button" => counts[0],
        "axis" => counts[1],
        "hat" => counts[2],
        _ => return Err("unknown physical control kind".into()),
    };
    if i >= count || !coverage.insert((kind.into(), i)) {
        return Err(format!("duplicate/out-of-range {kind} {i}"));
    }
    Ok(())
}

fn add(steps: &mut Vec<Step>, name: String, command: String, prompt: String) {
    steps.push(Step {
        name,
        command,
        prompt,
    });
}

fn button_steps(
    profile: &Value,
    mapped: bool,
    counts: [usize; 3],
    coverage: &mut BTreeSet<(String, usize)>,
    steps: &mut Vec<Step>,
) -> Result<(usize, String), String> {
    let buttons = array(profile, "buttons")?;
    if buttons.is_empty() {
        return Err("at least one tested physical button is required for held disconnect".into());
    }
    let mut held = None;
    for button in buttons {
        let i = index(&button["index"])?;
        cover(coverage, "button", i, counts)?;
        let label = text(button, "label")?;
        let mapping = if button["mapped"].is_null() {
            "-".into()
        } else {
            let m = index(&button["mapped"])?;
            if !mapped || m >= 15 {
                return Err("mapped button requires an available mapping and index 0..14".into());
            }
            m.to_string()
        };
        held.get_or_insert((i, mapping.clone()));
        for (phase, down) in [
            ("Initial release", false),
            ("Press", true),
            ("Release", false),
        ] {
            add(
                steps,
                format!("Button {i} {label}: {phase}"),
                format!("BUTTON\t{i}\t{}\t{mapping}", u8::from(down)),
                format!(
                    "{} {label} (bouton brut {i}).",
                    if down {
                        "Maintiens enfoncé"
                    } else {
                        "Relâche"
                    }
                ),
            );
        }
    }
    held.ok_or("no held-disconnect button".into())
}

fn validate_mapped_axis(axis: &Value, tolerance: f64) -> Result<(), String> {
    if axis["gamepad"].is_null() {
        return Ok(());
    }
    let gamepad = &axis["gamepad"];
    let values = ["minimum", "neutral", "maximum"].map(|key| gamepad[key].as_f64());
    let [Some(low), Some(center), Some(high)] = values else {
        return Err("mapped axis requires three independent expectations".into());
    };
    if !low.is_finite()
        || !center.is_finite()
        || !high.is_finite()
        || (high - low).abs() <= 2.0 * tolerance
        || !(low.min(high)..=low.max(high)).contains(&center)
        || ((center - low).abs() > 0.0 && (center - low).abs() <= 2.0 * tolerance)
        || ((center - high).abs() > 0.0 && (center - high).abs() <= 2.0 * tolerance)
    {
        return Err("mapped axis has overlapping/invalid expectation bands".into());
    }
    Ok(())
}

fn axis_steps(
    profile: &Value,
    counts: [usize; 3],
    coverage: &mut BTreeSet<(String, usize)>,
    steps: &mut Vec<Step>,
    mapped: bool,
) -> Result<(), String> {
    for axis in array(profile, "axes")? {
        let i = index(&axis["index"])?;
        cover(coverage, "axis", i, counts)?;
        let label = text(axis, "label")?;
        let values: Vec<f64> = ["minimum", "neutral", "maximum", "tolerance"]
            .into_iter()
            .map(|key| {
                axis[key]
                    .as_f64()
                    .filter(|n| n.is_finite())
                    .ok_or_else(|| format!("invalid axis {key}"))
            })
            .collect::<Result<_, _>>()?;
        let (min, rest, max, tolerance) = (values[0], values[1], values[2], values[3]);
        if !(-1.0..=1.0).contains(&min)
            || !(-1.0..=1.0).contains(&max)
            || min > rest
            || rest > max
            || !(0.001..=0.25).contains(&tolerance)
            || max - min <= 2.0 * tolerance
            || (rest > min && rest - min <= 2.0 * tolerance)
            || (rest < max && max - rest <= 2.0 * tolerance)
        {
            return Err(format!(
                "axis {i} has overlapping/invalid expectation bands"
            ));
        }
        validate_mapped_axis(axis, tolerance)?;
        for (phase, expected) in [
            ("Neutral", rest),
            ("Minimum", min),
            ("Maximum", max),
            ("Restored neutral", rest),
        ] {
            let mapped_fields = if axis["gamepad"].is_null() {
                String::new()
            } else {
                let gamepad = &axis["gamepad"];
                let index = index(&gamepad["index"])?;
                let key = match phase {
                    "Minimum" => "minimum",
                    "Maximum" => "maximum",
                    _ => "neutral",
                };
                let value = gamepad[key]
                    .as_f64()
                    .filter(|v| v.is_finite() && (-1.0..=1.0).contains(v))
                    .ok_or("invalid independent mapped-axis expectation")?;
                if !mapped || index >= 6 {
                    return Err("mapped axis requires an available gamepad and index 0..5".into());
                }
                format!("\t{index}\t{value}")
            };
            add(
                steps,
                format!("Axis {i} {label}: {phase}"),
                format!("AXIS\t{i}\t{expected}\t{tolerance}{mapped_fields}"),
                format!("Maintiens {label} à {phase}: attendu {expected} ± {tolerance}."),
            );
        }
    }
    Ok(())
}

fn mapped_exclusions(
    profile: &Value,
    mapped: bool,
    exclusions: &mut Vec<(String, String)>,
) -> Result<(), String> {
    for i in 0..15 {
        if !array(profile, "buttons")?
            .iter()
            .any(|b| b["mapped"].as_u64().and_then(|n| usize::try_from(n).ok()) == Some(i))
        {
            exclusions.push((
                format!("Mapped button {i}"),
                if mapped {
                    "not assigned to a tested raw button in this physical profile"
                } else {
                    "selected device has no gamepad mapping"
                }
                .into(),
            ));
        }
    }
    for i in 0..6 {
        if !array(profile, "axes")?.iter().any(|a| {
            a["gamepad"]["index"]
                .as_u64()
                .and_then(|n| usize::try_from(n).ok())
                == Some(i)
        }) {
            exclusions.push((
                format!("Mapped axis {i}"),
                if mapped {
                    "no independent mapped-axis expectations in this physical profile"
                } else {
                    "selected device has no gamepad mapping"
                }
                .into(),
            ));
        }
    }
    Ok(())
}

pub(crate) fn plan(profile: &Value, counts: [usize; 3], mapped: bool) -> Result<Plan, String> {
    validate_identity(profile)?;
    if counts[0] > 128 || counts[1] > 64 || counts[2] > 8 {
        return Err(
            "physical inventory exceeds protocol bounds (128 buttons/64 axes/8 hats)".into(),
        );
    }
    let mut coverage = BTreeSet::new();
    let mut steps = Vec::new();
    let (held, mapping) = button_steps(profile, mapped, counts, &mut coverage, &mut steps)?;
    axis_steps(profile, counts, &mut coverage, &mut steps, mapped)?;
    for hat in array(profile, "hats")? {
        let i = index(&hat["index"])?;
        cover(&mut coverage, "hat", i, counts)?;
        let label = text(hat, "label")?;
        for (name, mask) in [
            ("Centre", 0),
            ("Haut", 1),
            ("Droite", 2),
            ("Bas", 4),
            ("Gauche", 8),
            ("Haut-droite", 3),
            ("Bas-droite", 6),
            ("Bas-gauche", 12),
            ("Haut-gauche", 9),
            ("Centre final", 0),
        ] {
            add(
                &mut steps,
                format!("Hat {i} {label}: {name}"),
                format!("HAT\t{i}\t{mask}"),
                format!("Maintiens {label} vers {name}."),
            );
        }
    }
    let mut exclusions = Vec::new();
    for exclusion in array(profile, "exclusions")? {
        let kind = text(exclusion, "kind")?;
        let i = index(&exclusion["index"])?;
        cover(&mut coverage, kind, i, counts)?;
        exclusions.push((format!("{kind} {i}"), text(exclusion, "reason")?.into()));
    }
    for (kind, count) in ["button", "axis", "hat"].into_iter().zip(counts) {
        for i in 0..count {
            if !coverage.contains(&(kind.into(), i)) {
                return Err(format!("{kind} {i} must be tested or explicitly excluded"));
            }
        }
    }
    mapped_exclusions(profile, mapped, &mut exclusions)?;
    add(
        &mut steps,
        "Held disconnect preparation".into(),
        format!("BUTTON\t{held}\t1\t{mapping}"),
        format!("Maintiens le bouton brut {held} enfoncé jusqu'à la déconnexion demandée ensuite."),
    );
    add(
        &mut steps,
        "Disconnect while held".into(),
        "DISCONNECT".into(),
        "Sans relâcher ce bouton, débranche la manette ou coupe sa connexion Bluetooth.".into(),
    );
    add(
        &mut steps,
        "Next absent batch".into(),
        "EMPTY".into(),
        "Laisse la manette déconnectée.".into(),
    );
    Ok(Plan { steps, exclusions })
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;
    fn profile() -> Value {
        serde_json::json!({"schema_version":1,"model":"test device","connection":"usb","buttons":[{"index":0,"label":"A","mapped":0}],"axes":[{"index":0,"label":"X","minimum":-1.0,"neutral":0.0,"maximum":1.0,"tolerance":0.1}],"hats":[{"index":0,"label":"Dpad"}],"exclusions":[]})
    }
    #[test]
    fn plans_independent_values_and_preserves_held_disconnect_order() {
        let p = plan(&profile(), [1, 1, 1], true).expect("valid profile");
        assert_eq!(p.steps[0].command, "BUTTON\t0\t0\t0");
        assert_eq!(p.steps[1].command, "BUTTON\t0\t1\t0");
        assert_eq!(p.steps[2].command, "BUTTON\t0\t0\t0");
        assert_eq!(p.steps[p.steps.len() - 3].command, "BUTTON\t0\t1\t0");
        assert_eq!(p.steps[p.steps.len() - 2].command, "DISCONNECT");
        assert_eq!(p.steps.last().expect("empty batch").command, "EMPTY");
    }
    #[test]
    fn mapped_axes_use_independent_expectations_and_report_omissions() {
        let mut p = profile();
        p["axes"][0]["gamepad"] =
            serde_json::json!({"index":2,"minimum":1.0,"neutral":0.0,"maximum":-1.0});
        let result = plan(&p, [1, 1, 1], true).expect("independent inverted mapping");
        assert!(result
            .steps
            .iter()
            .any(|s| s.command == "AXIS\t0\t-1\t0.1\t2\t1"));
        assert!(!result
            .exclusions
            .iter()
            .any(|(name, _)| name == "Mapped axis 2"));
        assert!(result
            .exclusions
            .iter()
            .any(|(name, _)| name == "Mapped axis 3"));
        p["axes"][0]["gamepad"]["minimum"] = serde_json::json!(-1.0);
        assert!(plan(&p, [1, 1, 1], true).is_err());
    }
    #[test]
    fn rejects_implicit_missing_controls_duplicates_and_overlapping_axis_bands() {
        assert!(plan(&profile(), [2, 1, 1], true).is_err());
        let mut p = profile();
        p["axes"][0]["tolerance"] = serde_json::json!(0.6);
        assert!(plan(&p, [1, 1, 1], true).is_err());
        p = profile();
        p["exclusions"] = serde_json::json!([{"kind":"button","index":0,"reason":"duplicate"}]);
        assert!(plan(&p, [1, 1, 1], true).is_err());
        p = profile();
        p["exclusions"] =
            serde_json::json!([{"kind":"button","index":1,"reason":"not accessible"}]);
        assert!(plan(&p, [2, 1, 1], true)
            .expect("explicit exclusion")
            .exclusions
            .iter()
            .any(|(name, _)| name == "button 1"));
        assert!(plan(&profile(), [1, 1, 1], false).is_err());
    }
}
