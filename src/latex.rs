pub fn latex_escape(s: &str) -> String {
    let mut escaped = String::new();
    for c in s.chars() {
        match c {
            '&' => escaped.push_str("\\&"),
            '%' => escaped.push_str("\\%"),
            '$' => escaped.push_str("\\$"),
            '#' => escaped.push_str("\\#"),
            '_' => escaped.push_str("\\_"),
            '{' => escaped.push_str("\\{"),
            '}' => escaped.push_str("\\}"),
            '~' => escaped.push_str("\\textasciitilde{}"),
            '^' => escaped.push_str("\\textasciicircum{}"),
            '\\' => escaped.push_str("\\textbackslash{}"),
            '<' => escaped.push_str("\\textless{}"),
            '>' => escaped.push_str("\\textgreater{}"),
            '|' => escaped.push_str("\\textbar{}"),
            _ => escaped.push(c),
        }
    }
    escaped
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_latex_escape_empty_string() {
        assert_eq!(latex_escape(""), "");
    }

    #[test]
    fn test_latex_escape_no_special_chars() {
        assert_eq!(latex_escape("Hello World"), "Hello World");
    }

    #[test]
    fn test_latex_escape_all_special_chars() {
        let input = "&%$#_{}~^\\<>|";
        let expected = r"\&\%\$\#\_\{\}\textasciitilde{}\textasciicircum{}\textbackslash{}\textless{}\textgreater{}\textbar{}";
        assert_eq!(latex_escape(input), expected);
    }

    #[test]
    fn test_latex_escape_mixed_chars() {
        let input = "Invoice #123 for $100 & more.";
        let expected = "Invoice \\#123 for \\$100 \\& more.";
        assert_eq!(latex_escape(input), expected);
    }

    #[test]
    fn test_latex_escape_with_spaces_and_newlines() {
        let input = "Line 1\nLine 2 & Line 3";
        let expected = "Line 1\nLine 2 \\& Line 3";
        assert_eq!(latex_escape(input), expected);
    }

    #[test]
    fn test_latex_escape_only_one_special_char() {
        assert_eq!(latex_escape("$"), "\\$");
        assert_eq!(latex_escape("&"), "\\&");
        assert_eq!(latex_escape("#"), "\\#");
    }
}

