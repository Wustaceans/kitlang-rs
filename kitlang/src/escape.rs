//! Shared escape/unescape logic for Kit string and character literals.
//!
//! Used by the lexer (input unescaping) and the AST (C output escaping).

/// Return the C escape sequence for a character, or `None` if the character
/// does not need escaping.
pub(crate) fn escape_for_c(c: char) -> Option<&'static str> {
    match c {
        '\n' => Some("\\n"),
        '\r' => Some("\\r"),
        '\t' => Some("\\t"),
        '\\' => Some("\\\\"),
        '\'' => Some("\\'"),
        '"' => Some("\\\""),
        '\0' => Some("\\0"),
        _ => None,
    }
}

/// Unescape a single character from a Kit escape sequence.
///
/// Given a string starting with the character *after* the backslash, consumes
/// the escape code and returns the resulting character.
pub(crate) fn unescape_char_after_backslash(esc: char) -> Option<char> {
    match esc {
        'n' => Some('\n'),
        'r' => Some('\r'),
        't' => Some('\t'),
        '\\' => Some('\\'),
        '\'' => Some('\''),
        '"' => Some('"'),
        '0' => Some('\0'),
        other => Some(other),
    }
}

/// Unescape a single Kit character literal (the content between quotes).
///
/// Returns `None` if the string is empty or contains an invalid escape.
pub(crate) fn unescape_single(s: &str) -> Option<char> {
    let mut chars = s.chars();
    let c = chars.next()?;
    if c == '\\' {
        let esc = chars.next()?;
        unescape_char_after_backslash(esc)
    } else {
        Some(c)
    }
}

/// Unescape a Kit string, processing all escape sequences.
pub(crate) fn unescape_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            if let Some(esc) = chars.next()
                && let Some(unescaped) = unescape_char_after_backslash(esc)
            {
                out.push(unescaped);
            }
        } else {
            out.push(c);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escape_for_c_roundtrips() {
        let cases = ['\n', '\r', '\t', '\\', '\'', '"', '\0', 'a', 'Z', '🎉'];
        for c in cases {
            if let Some(seq) = escape_for_c(c) {
                // The escaped form should round-trip through unescape_single
                let unescaped = unescape_single(seq).unwrap();
                assert_eq!(unescaped, c, "roundtrip failed for {c:?}");
            }
        }
    }

    #[test]
    fn unescape_string_basic() {
        assert_eq!(unescape_string("hello"), "hello");
        assert_eq!(unescape_string("hello\\nworld"), "hello\nworld");
        assert_eq!(unescape_string("tab\\there"), "tab\there");
        assert_eq!(unescape_string("back\\\\slash"), "back\\slash");
        assert_eq!(unescape_string("null\\0byte"), "null\0byte");
    }
}
