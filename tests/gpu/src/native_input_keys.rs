// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Physical-key selection shared by the public input injector and headless regressions.

pub(crate) fn physical_keycode(
    physical_name: &str,
    names: &[(String, u16)],
) -> Result<Option<u8>, String> {
    if physical_name.is_empty() {
        return Ok(None);
    }
    // GLFW names physical US positions, independently of the active layout's characters.
    let found = names
        .iter()
        .find(|(name, _)| name == physical_name)
        .or_else(|| {
            (physical_name == "RALT")
                .then(|| {
                    names
                        .iter()
                        .find(|(name, _)| matches!(name.as_str(), "LVL3" | "MDSW"))
                })
                .flatten()
        });
    found
        .map(|(_, code)| {
            u8::try_from(*code)
                .map_err(|_| format!("XKB {physical_name} keycode {code} is out of XTEST range"))
        })
        .transpose()
}

#[test]
fn physical_a_uses_ac01_instead_of_the_character_position() {
    let names = [("AD01".into(), 24), ("AC01".into(), 38)];
    assert_eq!(physical_keycode("AC01", &names), Ok(Some(38)));
}

#[test]
fn missing_fixtures_and_names_are_excluded_but_overflow_fails() {
    assert_eq!(physical_keycode("", &[("LSGT".into(), 94)]), Ok(None));
    assert_eq!(physical_keycode("FK25", &[]), Ok(None));
    assert!(physical_keycode("AC01", &[("AC01".into(), 256)]).is_err());
}

#[test]
fn right_alt_prefers_its_canonical_position_over_level_three() {
    let names = [("LVL3".into(), 92), ("RALT".into(), 108)];
    assert_eq!(physical_keycode("RALT", &names), Ok(Some(108)));
}

#[test]
fn right_alt_accepts_the_backend_level_three_fallback() {
    for alias in ["LVL3", "MDSW"] {
        assert_eq!(
            physical_keycode("RALT", &[(alias.into(), 92)]),
            Ok(Some(92))
        );
        assert_eq!(physical_keycode("AC01", &[(alias.into(), 92)]), Ok(None));
    }
}
