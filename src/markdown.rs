use tracing::Level;

/// Escapes a given string for safe inclusion in Markdown documents.
///
/// This function replaces special Markdown characters with their corresponding escape sequences.
pub fn markdown_escape(initial: &str) -> String {
    let mut escaped = String::new();
    for c in initial.chars() {
        match c {
            '\\' | '`' | '*' | '_' | '{' | '}' | '[' | ']' | '(' | ')' | '#' | '+' | '-' | '!' => {
                escaped.push('\\');
                escaped.push(c);
            }
            _ => escaped.push(c),
        }
    }
    if tracing::enabled!(Level::TRACE) && escaped != initial {
        tracing::trace!("MARKDOWN  {}  =>  {}", initial, escaped);
    }
    escaped
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_markdown_escape_empty_string() {
        assert_eq!(markdown_escape(""), "");
    }

    #[test]
    fn test_markdown_escape_no_special_chars() {
        assert_eq!(markdown_escape("Hello World"), "Hello World");
    }

    #[test]
    fn test_markdown_escape_all_special_chars() {
        let input = r"`*_{}[]()#+-.!";
        let expected = r"\`\*\_\{\}\[\]\(\)\#\+\-.\!";
        assert_eq!(markdown_escape(input), expected);
    }

    #[test]
    fn test_markdown_escape_mixed_chars() {
        let input = "Invoice #123 for *important* stuff.";
        let expected = r#"Invoice \#123 for \*important\* stuff."#;
        assert_eq!(markdown_escape(input), expected);
    }
}
