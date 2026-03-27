use poiesis_core::{parse_content, Config, WpClient};

use crate::display::{print_content_with_ids, print_heading_tree, print_post_header};
use crate::util::fatal_err;

pub async fn run(id: &str, tree: bool, section: Option<String>, json: bool) {
    let config = Config::load().unwrap_or_else(|e| fatal_err(&e));
    let client = WpClient::new(&config).unwrap_or_else(|e| fatal_err(&e));

    let post_id = WpClient::validate_post_id(id).unwrap_or_else(|e| fatal_err(&e));

    // Try post first, then page
    let post = match client.get_post(post_id).await {
        Ok(p) => p,
        Err(_) => client
            .get_page(post_id)
            .await
            .unwrap_or_else(|e| fatal_err(&e)),
    };

    if json {
        let json_str = serde_json::to_string_pretty(&post).unwrap();
        println!("{}", json_str);
        return;
    }

    let doc = parse_content(&post.content.raw);

    if let Some(sid) = section {
        // Show just the section content
        let sec = poiesis_core::find_section(&doc, &sid).unwrap_or_else(|e| fatal_err(&e));
        let section_content = &doc.markdown[sec.start..sec.end];
        println!("{}", section_content.trim());
        return;
    }

    if tree {
        print_heading_tree(&post, &doc.sections);
        return;
    }

    // Full detail: header + content with IDs
    print_post_header(&post);
    print_content_with_ids(&doc.markdown, &doc.sections);
}
