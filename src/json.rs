//! Minimal JSON output helpers.

pub fn escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\x08' => out.push_str("\\b"),
            '\x0c' => out.push_str("\\f"),
            c if (c as u32) < 0x20 => {
                out.push_str(&format!("\\u{:04x}", c as u32));
            }
            c => out.push(c),
        }
    }
    out
}

pub fn quoted(s: &str) -> String {
    format!("\"{}\"", escape(s))
}

pub fn opt_num<T: std::fmt::Display>(v: Option<T>) -> String {
    match v {
        Some(n) => n.to_string(),
        None => "null".into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escape_passes_through_plain_ascii() {
        assert_eq!(escape("hello world"), "hello world");
        assert_eq!(escape("NVIDIA GeForce RTX 5090"), "NVIDIA GeForce RTX 5090");
    }

    #[test]
    fn escape_quotes_and_backslashes() {
        assert_eq!(escape("a\"b"), "a\\\"b");
        assert_eq!(escape("a\\b"), "a\\\\b");
    }

    #[test]
    fn escape_whitespace_control_chars() {
        assert_eq!(escape("\n"), "\\n");
        assert_eq!(escape("\r"), "\\r");
        assert_eq!(escape("\t"), "\\t");
        assert_eq!(escape("\x08"), "\\b");
        assert_eq!(escape("\x0c"), "\\f");
    }

    #[test]
    fn escape_other_control_chars_use_unicode_form() {
        assert_eq!(escape("\x01"), "\\u0001");
        assert_eq!(escape("\x1f"), "\\u001f");
    }

    #[test]
    fn quoted_wraps_and_escapes() {
        assert_eq!(quoted("hi"), "\"hi\"");
        assert_eq!(quoted("a\"b"), "\"a\\\"b\"");
    }

    #[test]
    fn opt_num_emits_value_or_null() {
        assert_eq!(opt_num::<u32>(Some(42)), "42");
        assert_eq!(opt_num::<i32>(Some(-100)), "-100");
        assert_eq!(opt_num::<u32>(None), "null");
    }
}
