use tracing::Level;

/// Escapes a given string for safe inclusion in Markdown documents.
///
/// This function replaces special Markdown characters with their corresponding escape sequences.
pub fn markdown_escape(initial: &str) -> String {
    let mut escaped = String::with_capacity(initial.len() * 2);
    let mut previous_digit = false;
    let mut previous_space = false;

    for c in initial.chars() {
        let escape_this_char = match c {
            '\\' | '`' | '*' | '_' | '{' | '}' | '[' | ']' | '(' | ')' | '#' | '+' | '-' | '!' => true,
            '.' => previous_digit, // escape a '.' to avoid number list from "1."
            '>' => previous_space, // escape a '>' when it follows white space
            _ => false,
        };

        previous_digit = c.is_digit(10);
        previous_space = c.is_whitespace();

        if escape_this_char {
            escaped.push('\\');
        }
        escaped.push(c);

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
