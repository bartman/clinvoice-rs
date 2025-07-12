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
