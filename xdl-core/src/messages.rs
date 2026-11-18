//! Message templates loaded from `resources/messages.txt`.
//!
//! The templates are compiled into the binary (`include_str!`), so there is no
//! runtime file to lose. They are parsed once, on first use.
//!
//! An unknown key does not render as an empty string: it renders as
//! `[missing message: key]`, so a typo shows up in the output.

use std::collections::HashMap;
use std::fmt::Display;
use std::sync::OnceLock;

const RESOURCE: &str = include_str!("../resources/messages.txt");

fn table() -> &'static HashMap<&'static str, &'static str> {
    static TABLE: OnceLock<HashMap<&'static str, &'static str>> = OnceLock::new();
    TABLE.get_or_init(|| parse(RESOURCE))
}

/// Parse `key = template` lines. Blank lines and `#` comments are skipped.
fn parse(source: &'static str) -> HashMap<&'static str, &'static str> {
    source
        .lines()
        .filter(|line| !line.trim().is_empty() && !line.trim_start().starts_with('#'))
        .filter_map(|line| line.split_once(" = "))
        .map(|(key, template)| (key.trim(), template))
        .collect()
}

/// Render the template for `key`, replacing each `{name}` with its value.
/// Placeholders with no matching argument are left as written.
pub fn render(key: &str, args: &[(&str, &dyn Display)]) -> String {
    let Some(template) = table().get(key) else {
        return format!("[missing message: {key}]");
    };
    let mut out = (*template).to_string();
    for (name, value) in args {
        out = out.replace(&format!("{{{name}}}"), &value.to_string());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn substitutes_named_placeholders() {
        let text = render(
            "type_mismatch",
            &[("expected", &"FLOAT"), ("actual", &"STRING")],
        );
        assert_eq!(text, "Type mismatch: expected FLOAT, got STRING");
    }

    #[test]
    fn unknown_key_is_visible_not_empty() {
        assert_eq!(render("no_such_key", &[]), "[missing message: no_such_key]");
    }

    #[test]
    fn every_error_variant_has_a_template() {
        // One template per XdlError variant (27). A missing or duplicated key
        // would otherwise only show up as a "[missing message]" at runtime.
        assert_eq!(table().len(), 27, "parsed {} templates", table().len());
    }
}
