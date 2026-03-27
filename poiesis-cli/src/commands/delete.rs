use poiesis_core::{parse_content, sections, to_raw, Config, UpdateParams, WpClient};

use crate::util::{fatal, fatal_err};

pub async fn run(id: &str, section: Option<String>, force: bool) {
    let config = Config::load().unwrap_or_else(|e| fatal_err(&e));
    let client = WpClient::new(&config).unwrap_or_else(|e| fatal_err(&e));

    let post_id = WpClient::validate_post_id(id).unwrap_or_else(|e| fatal_err(&e));

    if let Some(sid) = section {
        // Delete a section — update the post content
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
        sections::delete_section(&mut doc, &sid).unwrap_or_else(|e| fatal_err(&e));
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
            "Deleted section [{}] from {} {}",
            sid, updated.post_type, updated.id
        );
    } else {
        // Trash/delete the entire post
        if !force {
            fatal("This will trash the entire post. Use --force to confirm, or --section <sid> to delete a section.");
        }

        // Determine if post or page
        let is_page = match client.get_post(post_id).await {
            Ok(p) => p.post_type == "page",
            Err(_) => true,
        };

        if is_page {
            client
                .delete_page(post_id)
                .await
                .unwrap_or_else(|e| fatal_err(&e));
        } else {
            client
                .delete_post(post_id)
                .await
                .unwrap_or_else(|e| fatal_err(&e));
        }

        println!(
            "Trashed {} {}",
            if is_page { "page" } else { "post" },
            post_id
        );
    }
}
