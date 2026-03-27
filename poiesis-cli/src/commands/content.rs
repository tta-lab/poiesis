use poiesis_core::{find_section, parse_content, Config, WpClient};

use crate::display::print_content_with_ids;
use crate::util::fatal_err;

pub async fn run(id: &str, section: Option<String>) {
    let config = Config::load().unwrap_or_else(|e| fatal_err(&e));
    let client = WpClient::new(&config).unwrap_or_else(|e| fatal_err(&e));

    let post_id = WpClient::validate_post_id(id).unwrap_or_else(|e| fatal_err(&e));

    let post = match client.get_post(post_id).await {
        Ok(p) => p,
        Err(_) => client
            .get_page(post_id)
            .await
            .unwrap_or_else(|e| fatal_err(&e)),
    };

    let doc = parse_content(&post.content.raw);

    if let Some(sid) = section {
        let sec = find_section(&doc, &sid).unwrap_or_else(|e| fatal_err(&e));
        let section_content = &doc.markdown[sec.start..sec.end];
        println!("{}", section_content.trim());
    } else {
        print_content_with_ids(&doc.markdown, &doc.sections);
    }
}
