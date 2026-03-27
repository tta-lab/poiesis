use pulldown_cmark::{html, Event, HeadingLevel, Options, Parser, Tag};

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

/// Convert markdown to Gutenberg block markup.
/// Each top-level element becomes its own block:
/// - Headings → wp:heading {"level":N}
/// - Paragraphs → wp:paragraph
/// - Horizontal rules → wp:separator (self-closing)
/// - Lists → wp:list
/// - Block quotes → wp:quote
/// - Code blocks → wp:code
pub fn markdown_to_raw_gutenberg(markdown: &str) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);

    let parser = Parser::new_ext(markdown, options);
    let events: Vec<Event<'_>> = parser.collect();

    let mut out = String::new();
    let mut i = 0;

    while i < events.len() {
        match &events[i] {
            Event::Start(Tag::Heading { level, .. }) => {
                let n = heading_level_num(*level);
                let j = find_end_idx(&events, i);
                let mut html_buf = String::new();
                html::push_html(&mut html_buf, events[i..=j].iter().cloned());
                let html_buf = html_buf.trim().to_string();
                let attrs = format!(r#"{{"level":{n}}}"#);
                out.push_str(&format!(
                    "<!-- wp:heading {attrs} -->{html_buf}<!-- /wp:heading -->\n"
                ));
                i = j + 1;
            }
            Event::Start(Tag::Paragraph) => {
                let j = find_end_idx(&events, i);
                let mut html_buf = String::new();
                html::push_html(&mut html_buf, events[i..=j].iter().cloned());
                let html_buf = html_buf.trim().to_string();
                out.push_str(&format!(
                    "<!-- wp:paragraph -->{html_buf}<!-- /wp:paragraph -->\n"
                ));
                i = j + 1;
            }
            Event::Rule => {
                out.push_str("<!-- wp:separator /-->\n");
                i += 1;
            }
            Event::Start(Tag::List(_)) => {
                let j = find_end_idx(&events, i);
                let mut html_buf = String::new();
                html::push_html(&mut html_buf, events[i..=j].iter().cloned());
                let html_buf = html_buf.trim().to_string();
                out.push_str(&format!("<!-- wp:list -->{html_buf}<!-- /wp:list -->\n"));
                i = j + 1;
            }
            Event::Start(Tag::BlockQuote(_)) => {
                let j = find_end_idx(&events, i);
                let mut html_buf = String::new();
                html::push_html(&mut html_buf, events[i..=j].iter().cloned());
                let html_buf = html_buf.trim().to_string();
                out.push_str(&format!("<!-- wp:quote -->{html_buf}<!-- /wp:quote -->\n"));
                i = j + 1;
            }
            Event::Start(Tag::CodeBlock(_)) => {
                let j = find_end_idx(&events, i);
                let mut html_buf = String::new();
                html::push_html(&mut html_buf, events[i..=j].iter().cloned());
                let html_buf = html_buf.trim().to_string();
                out.push_str(&format!("<!-- wp:code -->{html_buf}<!-- /wp:code -->\n"));
                i = j + 1;
            }
            _ => {
                i += 1;
            }
        }
    }

    out.trim_end().to_string()
}

fn heading_level_num(level: HeadingLevel) -> usize {
    match level {
        HeadingLevel::H1 => 1,
        HeadingLevel::H2 => 2,
        HeadingLevel::H3 => 3,
        HeadingLevel::H4 => 4,
        HeadingLevel::H5 => 5,
        HeadingLevel::H6 => 6,
    }
}

/// Find the index of the End event matching the Start at `start_idx`
fn find_end_idx(events: &[Event<'_>], start_idx: usize) -> usize {
    let mut depth = 0i32;
    for (j, event) in events[start_idx..].iter().enumerate() {
        match event {
            Event::Start(_) => depth += 1,
            Event::End(_) => {
                depth -= 1;
                if depth == 0 {
                    return start_idx + j;
                }
            }
            _ => {}
        }
    }
    events.len().saturating_sub(1)
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

    #[test]
    fn test_gutenberg_heading_block() {
        let raw = markdown_to_raw_gutenberg("## My Section");
        assert!(raw.contains(r#"<!-- wp:heading {"level":2} -->"#));
        assert!(raw.contains("<h2>My Section</h2>"));
        assert!(raw.contains("<!-- /wp:heading -->"));
    }

    #[test]
    fn test_gutenberg_paragraph_block() {
        let raw = markdown_to_raw_gutenberg("Hello world.");
        assert!(raw.contains("<!-- wp:paragraph -->"));
        assert!(raw.contains("<p>Hello world.</p>"));
        assert!(raw.contains("<!-- /wp:paragraph -->"));
    }

    #[test]
    fn test_gutenberg_separator_block() {
        let raw = markdown_to_raw_gutenberg("---");
        assert!(raw.contains("<!-- wp:separator /-->"));
    }

    #[test]
    fn test_gutenberg_list_block() {
        let raw = markdown_to_raw_gutenberg("- Item A\n- Item B");
        assert!(raw.contains("<!-- wp:list -->"));
        assert!(raw.contains("Item A"));
        assert!(raw.contains("<!-- /wp:list -->"));
    }

    #[test]
    fn test_gutenberg_multiple_blocks() {
        let md = "## Title\n\nParagraph text.\n\n- Item 1\n- Item 2";
        let raw = markdown_to_raw_gutenberg(md);
        assert!(raw.contains("wp:heading"));
        assert!(raw.contains("wp:paragraph"));
        assert!(raw.contains("wp:list"));
        // Each block should be separate
        let heading_pos = raw.find("wp:heading").unwrap();
        let para_pos = raw.find("wp:paragraph").unwrap();
        let list_pos = raw.find("wp:list").unwrap();
        assert!(heading_pos < para_pos);
        assert!(para_pos < list_pos);
    }
}
