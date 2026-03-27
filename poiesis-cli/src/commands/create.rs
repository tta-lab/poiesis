use poiesis_core::markdown::markdown_to_html;
use poiesis_core::{Config, CreateParams, PostStatus, WpClient};

use crate::util::{fatal, fatal_err, try_read_stdin};

pub async fn run(title: Option<String>, post_type: Option<String>, status: Option<String>) {
    let config = Config::load().unwrap_or_else(|e| fatal_err(&e));
    let client = WpClient::new(&config).unwrap_or_else(|e| fatal_err(&e));

    let stdin = try_read_stdin().unwrap_or_else(|e| fatal(&format!("failed to read stdin: {}", e)));
    let markdown = match stdin {
        Some(m) => m,
        None => fatal("expected markdown content on stdin"),
    };

    let markdown = markdown.trim().to_string();
    if markdown.is_empty() {
        fatal("stdin content is empty");
    }

    // Extract title from first heading if not provided
    let (title, content_md) = if let Some(t) = title {
        (t, markdown.clone())
    } else {
        extract_title_and_content(&markdown)
    };

    // Convert markdown to HTML
    let html = markdown_to_html(&content_md);
    // Wrap in a paragraph block
    let raw_content = format!("<!-- wp:paragraph -->{html}<!-- /wp:paragraph -->");

    let params = CreateParams {
        title,
        content: raw_content,
        status: status.as_deref().map(PostStatus::parse),
        ..Default::default()
    };

    let post = match post_type.as_deref() {
        Some("page") => client
            .create_page(&params)
            .await
            .unwrap_or_else(|e| fatal_err(&e)),
        _ => client
            .create_post(&params)
            .await
            .unwrap_or_else(|e| fatal_err(&e)),
    };
    println!("Created {} {} ({})", post.post_type, post.id, post.status);
    println!("Link: {}", post.link);
}

/// Extract title from first `# Heading` and return (title, remaining_content)
fn extract_title_and_content(markdown: &str) -> (String, String) {
    let first_line = markdown.lines().next().unwrap_or("");
    if first_line.starts_with("# ") {
        let title = first_line
            .strip_prefix("# ")
            .unwrap_or("")
            .trim()
            .to_string();
        let rest = markdown
            .lines()
            .skip(1)
            .collect::<Vec<_>>()
            .join("\n")
            .trim_start()
            .to_string();
        (title, rest)
    } else {
        fatal("no title found in stdin — use `--title` or start content with `# Title`");
    }
}
