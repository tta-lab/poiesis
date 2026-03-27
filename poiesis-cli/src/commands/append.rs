use poiesis_core::{parse_content, rebuild_sections, to_raw, Config, UpdateParams, WpClient};

use crate::util::{fatal, fatal_err, try_read_stdin};

pub async fn run(id: &str) {
    let config = Config::load().unwrap_or_else(|e| fatal_err(&e));
    let client = WpClient::new(&config).unwrap_or_else(|e| fatal_err(&e));

    let post_id = WpClient::validate_post_id(id).unwrap_or_else(|e| fatal_err(&e));

    let stdin = try_read_stdin().unwrap_or_else(|e| fatal(&format!("failed to read stdin: {}", e)));
    let content = match stdin {
        Some(c) if !c.trim().is_empty() => c,
        _ => fatal("expected markdown content on stdin"),
    };

    let (post, is_page) = match client.get_post(post_id).await {
        Ok(p) => (p, false),
        Err(_) => (
            client
                .get_page(post_id)
                .await
                .unwrap_or_else(|e| fatal_err(&e)),
            true,
        ),
    };

    let mut doc = parse_content(&post.content.raw);

    // Append to end of document
    let appended = if doc.markdown.is_empty() {
        content.trim().to_string()
    } else {
        format!("{}\n\n{}", doc.markdown.trim_end(), content.trim())
    };
    doc.markdown = appended;
    rebuild_sections(&mut doc);

    let new_raw = to_raw(&doc);
    let params = UpdateParams {
        content: Some(new_raw),
        ..Default::default()
    };

    let updated = if is_page {
        client
            .update_page(post_id, &params)
            .await
            .unwrap_or_else(|e| fatal_err(&e))
    } else {
        client
            .update_post(post_id, &params)
            .await
            .unwrap_or_else(|e| fatal_err(&e))
    };

    println!("Appended to {} {}", updated.post_type, updated.id);
}
