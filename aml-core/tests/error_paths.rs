use aml_core::parser::Document;
#[cfg(feature = "styler")]
use aml_core::styler::Style;

// ── Parser error paths ────────────────────────────────────────────────────

#[test]
fn unclosed_fg_tag_returns_error() {
    assert!(Document::try_new("<fr>text").is_err());
}

#[test]
fn unclosed_bg_tag_returns_error() {
    assert!(Document::try_new("<bb>text").is_err());
}

#[test]
fn unclosed_modifier_tag_returns_error() {
    assert!(Document::try_new("<mb>text").is_err());
}

#[test]
fn unclosed_shorthand_tag_returns_error() {
    assert!(Document::try_new("<s fr mbi>text").is_err());
}

#[test]
fn unclosed_reset_tag_returns_error() {
    assert!(Document::try_new("<>text").is_err());
}

#[test]
fn unclosed_raw_tag_returns_error() {
    assert!(Document::try_new("<!53m>text").is_err());
}

#[test]
fn invalid_color_spec_returns_error() {
    assert!(Document::try_new("<fz>text</f>").is_err());
}

#[test]
fn missing_closing_angle_bracket_returns_error() {
    assert!(Document::try_new("<fr text</f>").is_err());
}

#[test]
fn mismatched_close_tag_returns_error() {
    assert!(Document::try_new("<fr>text</b>").is_err());
}

#[test]
fn empty_fg_tag_no_color_returns_error() {
    assert!(Document::try_new("<f>text</f>").is_err());
}

#[test]
fn invalid_hex_color_returns_error() {
    assert!(Document::try_new("<f#zzzzzz>text</f>").is_err());
}

#[test]
fn rgb_missing_component_returns_error() {
    assert!(Document::try_new("<f255,0>text</f>").is_err());
}

#[test]
fn shorthand_with_no_args_returns_error() {
    assert!(Document::try_new("<s>text</s>").is_err());
}

// ── Parser success edge cases ─────────────────────────────────────────────

#[test]
fn valid_deeply_nested_tags_parse_ok() {
    let input = "<fr><bg><mi><mb>text</m></m></b></f>";
    assert!(Document::try_new(input).is_ok());
}

#[test]
fn escaped_angle_bracket_in_text_parses_ok() {
    let doc = Document::try_new("\\<not a tag>").unwrap();
    let rendered = doc.render();
    assert_eq!(rendered, "<not a tag>");
}

#[test]
fn empty_input_parses_ok() {
    assert!(Document::try_new("").is_ok());
}

#[test]
fn plain_text_only_parses_ok() {
    let doc = Document::try_new("hello world").unwrap();
    assert_eq!(doc.render(), "hello world");
}

// ── Styler error paths ────────────────────────────────────────────────────

#[cfg(feature = "styler")]
#[test]
fn empty_style_spec_returns_error() {
    assert!(Style::new("").is_err());
}

#[cfg(feature = "styler")]
#[test]
fn invalid_style_spec_returns_error() {
    assert!(Style::new("xyz").is_err());
}

#[cfg(feature = "styler")]
#[test]
fn style_with_invalid_color_returns_error() {
    assert!(Style::new("fz").is_err());
}

#[cfg(feature = "styler")]
#[test]
fn paint_str_with_invalid_spec_returns_error() {
    assert!(Style::paint_str("invalid", "text").is_err());
}

// ── Styler success paths ──────────────────────────────────────────────────

#[cfg(feature = "styler")]
#[test]
fn valid_style_paint_contains_text() {
    let s = Style::new("fr mbi").unwrap();
    let result = s.paint("hello");
    assert!(result.contains("hello"));
    assert!(result.ends_with("\x1b[0m"));
}

#[cfg(feature = "styler")]
#[test]
fn paint_str_valid_spec_works() {
    let result = Style::paint_str("fr", "test").unwrap();
    assert!(result.contains("test"));
    assert!(result.contains("\x1b["));
}
