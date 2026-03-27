use poiesis_core::{parse_content, sections, to_raw, Config, UpdateParams, WpClient};

use crate::util::fatal_err;

pub async fn run(id: &str, section: &str, new_name: &str) {
    let config = Config::load().unwrap_or_else(|e| fatal_err(&e));
    let client = WpClient::new(&config).unwrap_or_else(|e| fatal_err(&e));

    let post_id = WpClient::validate_post_id(id).unwrap_or_else(|e| fatal_err(&e));

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
    sections::rename_section(&mut doc, section, new_name).unwrap_or_else(|e| fatal_err(&e));
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

    println!(
        "Renamed section in {} {} to '{}'",
        updated.post_type, updated.id, new_name
    );
}
