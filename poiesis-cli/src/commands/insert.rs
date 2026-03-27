use poiesis_core::{parse_content, sections, to_raw, Config, UpdateParams, WpClient};

use crate::util::{fatal, fatal_err, try_read_stdin};

pub async fn run(id: &str, section: &str, before: bool, after: bool) {
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

    if before {
        sections::insert_before(&mut doc, section, content.trim())
            .unwrap_or_else(|e| fatal_err(&e));
    } else if after {
        sections::insert_after(&mut doc, section, content.trim()).unwrap_or_else(|e| fatal_err(&e));
    } else {
        fatal("must specify either --before or --after");
    }

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

    println!("Updated {} {}", updated.post_type, updated.id);
}
