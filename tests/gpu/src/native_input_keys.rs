// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Physical-key selection shared by the public input injector and headless regressions.

pub(crate) fn physical_a_keycode(first_key: u8, names: &[[u8; 4]]) -> Result<u8, String> {
    // GLFW names physical keys by the US positions, not by the layout's A character.
    let offset = names
        .iter()
        .position(|name| name == b"AC01")
        .ok_or("XKB keyboard map has no physical AC01 key")?;
    let offset = u8::try_from(offset).map_err(|_| "XKB A keycode offset is too large")?;
    first_key
        .checked_add(offset)
        .ok_or_else(|| "XKB A keycode is out of range".to_owned())
}

#[test]
fn physical_a_uses_ac01_instead_of_the_character_position() {
    let mut names = [[0; 4]; 15];
    names[0] = *b"AD01";
    names[14] = *b"AC01";
    assert_eq!(physical_a_keycode(24, &names), Ok(38));
}

#[test]
fn physical_a_rejects_missing_names_and_overflow() {
    assert!(physical_a_keycode(24, &[*b"AD01"]).is_err());
    assert!(physical_a_keycode(255, &[*b"AD01", *b"AC01"]).is_err());
}
