use poiesis_core::{Config, ListParams, PostStatus, WpClient};

use crate::display::print_post_table;
use crate::util::fatal_err;

pub async fn run(
    post_type: Option<String>,
    status: Option<String>,
    search: Option<String>,
    per_page: Option<u32>,
    page: Option<u32>,
) {
    let config = Config::load().unwrap_or_else(|e| fatal_err(&e));
    let client = WpClient::new(&config).unwrap_or_else(|e| fatal_err(&e));

    let params = ListParams {
        page,
        per_page,
        search,
        status: status.as_deref().map(|s| vec![PostStatus::parse(s)]),
        ..Default::default()
    };

    let (posts, total) = match post_type.as_deref() {
        Some("page") => client
            .list_pages(&params)
            .await
            .unwrap_or_else(|e| fatal_err(&e)),
        _ => client
            .list_posts(&params)
            .await
            .unwrap_or_else(|e| fatal_err(&e)),
    };

    print_post_table(&posts, total);
}
