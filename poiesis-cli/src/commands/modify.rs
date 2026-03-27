use poiesis_core::{parse_content, sections, to_raw, Config, PostStatus, UpdateParams, WpClient};

use crate::util::{fatal, fatal_err, try_read_stdin};

pub async fn run(
    id: &str,
    section: Option<String>,
    title: Option<String>,
    status: Option<String>,
    slug: Option<String>,
) {
    let config = Config::load().unwrap_or_else(|e| fatal_err(&e));
    let client = WpClient::new(&config).unwrap_or_else(|e| fatal_err(&e));

    let post_id = WpClient::validate_post_id(id).unwrap_or_else(|e| fatal_err(&e));

    let stdin = try_read_stdin().unwrap_or_else(|e| fatal(&format!("failed to read stdin: {}", e)));

    // Determine if this is a post or page
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

    let new_content = if let Some(md_input) = stdin {
        if md_input.trim().is_empty() {
            if section.is_some() {
                fatal("stdin content is empty — cannot modify section with empty content");
            }
            None
        } else {
            // Modify content
            let mut doc = parse_content(&post.content.raw);

            if let Some(sid) = &section {
                // Section-level replace
                sections::replace_section(&mut doc, sid, md_input.trim())
                    .unwrap_or_else(|e| fatal_err(&e));
            } else {
                // Replace full content
                doc.markdown = md_input.trim().to_string();
            }

            Some(to_raw(&doc))
        }
    } else {
        if section.is_some() {
            fatal("section modify requires content on stdin");
        }
        None
    };

    // Build update params
    let params = UpdateParams {
        title,
        content: new_content,
        status: status.as_deref().map(PostStatus::parse),
        slug,
        ..Default::default()
    };

    // Check if we have anything to update
    if params.title.is_none()
        && params.content.is_none()
        && params.status.is_none()
        && params.slug.is_none()
    {
        fatal("nothing to modify — provide stdin content or --title/--status/--slug flags");
    }

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
        "Updated {} {} ({})",
        updated.post_type, updated.id, updated.status
    );
}
