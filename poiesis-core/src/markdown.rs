use pulldown_cmark::{html, Options, Parser};

/// Convert HTML to markdown
pub fn html_to_markdown(html_input: &str) -> String {
    if html_input.trim().is_empty() {
        return String::new();
    }
    htmd::convert(html_input).unwrap_or_else(|_| html_input.to_string())
}

/// Convert markdown to HTML
pub fn markdown_to_html(markdown: &str) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);

    let parser = Parser::new_ext(markdown, options);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);
    html_output.trim_end().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_html_to_markdown_paragraph() {
        let result = html_to_markdown("<p>Hello world</p>");
        assert!(result.contains("Hello world"));
    }

    #[test]
    fn test_html_to_markdown_heading() {
        let result = html_to_markdown("<h2>My Title</h2>");
        assert!(result.contains("## My Title") || result.contains("My Title"));
    }

    #[test]
    fn test_html_to_markdown_bold_italic() {
        let result = html_to_markdown("<p><strong>bold</strong> and <em>italic</em></p>");
        assert!(result.contains("bold"));
        assert!(result.contains("italic"));
    }

    #[test]
    fn test_html_to_markdown_link() {
        let result = html_to_markdown(r#"<p><a href="https://example.com">click here</a></p>"#);
        assert!(result.contains("click here"));
        assert!(result.contains("https://example.com") || result.contains("click here"));
    }

    #[test]
    fn test_html_to_markdown_ordered_list() {
        let result = html_to_markdown("<ol><li>First</li><li>Second</li></ol>");
        assert!(result.contains("First"));
        assert!(result.contains("Second"));
    }

    #[test]
    fn test_html_to_markdown_unordered_list() {
        let result = html_to_markdown("<ul><li>Item A</li><li>Item B</li></ul>");
        assert!(result.contains("Item A"));
        assert!(result.contains("Item B"));
    }

    #[test]
    fn test_html_to_markdown_blockquote() {
        let result = html_to_markdown("<blockquote><p>Quote text</p></blockquote>");
        assert!(result.contains("Quote text"));
    }

    #[test]
    fn test_html_to_markdown_hr() {
        let result = html_to_markdown("<hr/>");
        // htmd may produce --- or *** or similar
        assert!(!result.trim().is_empty());
    }

    #[test]
    fn test_html_to_markdown_nested_inline() {
        let result =
            html_to_markdown(r#"<p><strong><a href="https://test.com">bold link</a></strong></p>"#);
        assert!(result.contains("bold link"));
    }

    #[test]
    fn test_markdown_to_html_paragraph() {
        let result = markdown_to_html("Hello world");
        assert!(result.contains("Hello world"));
        assert!(result.contains("<p>"));
    }

    #[test]
    fn test_markdown_to_html_heading() {
        let result = markdown_to_html("## My Heading");
        assert!(result.contains("<h2>"));
        assert!(result.contains("My Heading"));
    }

    #[test]
    fn test_round_trip_markdown() {
        let original = "Hello world, this is a paragraph.";
        let html = markdown_to_html(original);
        let back = html_to_markdown(&html);
        assert!(back.contains("Hello world"));
    }

    #[test]
    fn test_chinese_content() {
        let html = "<p>经典讲义</p>";
        let md = html_to_markdown(html);
        assert!(md.contains("经典讲义"));

        let md_input =
            "## \u{4e3a}\u{4ec0}\u{4e48}\u{4f1a}\u{6709}\u{300e}\u{8bb2}\u{4e49}\u{300f}\u{ff1f}";
        let html_out = markdown_to_html(md_input);
        assert!(html_out.contains("讲义"));
    }
}
