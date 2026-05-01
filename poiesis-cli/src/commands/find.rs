use poiesis_core::{Config, ListParams, Post, PostStatus, WpClient};

use crate::display::print_post_table;
use crate::util::fatal_err;

/// When scanning by slug substring (no native WP REST support),
/// paginate up to this many pages of `SLUG_SCAN_PER_PAGE` each.
/// 10 × 100 = 1000 posts is a sensible upper bound for ad-hoc lookups.
const MAX_PAGES_FOR_SLUG_SCAN: u32 = 10;
const SLUG_SCAN_PER_PAGE: u32 = 100;

pub async fn run(
    query: Option<String>,
    slug: Option<String>,
    post_type: Option<String>,
    status: Option<String>,
    per_page: Option<u32>,
) {
    if query.is_none() && slug.is_none() {
        eprintln!("error: provide a search query (positional) or --slug <substring>");
        std::process::exit(2);
    }

    let config = Config::load().unwrap_or_else(|e| fatal_err(&e));
    let client = WpClient::new(&config).unwrap_or_else(|e| fatal_err(&e));

    let want_pages = matches!(post_type.as_deref(), Some("page"));
    let status_filter = status.as_deref().map(|s| vec![PostStatus::parse(s)]);

    if let Some(slug_substr) = slug.as_deref() {
        // Slug substring: paginate, client-side filter.
        // Optional `query` further narrows the WP-side search before filtering.
        let mut matched: Vec<Post> = Vec::new();
        let mut hit_cap = false;
        for p in 1..=MAX_PAGES_FOR_SLUG_SCAN {
            let params = ListParams {
                page: Some(p),
                per_page: Some(SLUG_SCAN_PER_PAGE),
                search: query.clone(),
                status: status_filter.clone(),
                ..Default::default()
            };
            let (posts, _total) = if want_pages {
                client
                    .list_pages(&params)
                    .await
                    .unwrap_or_else(|e| fatal_err(&e))
            } else {
                client
                    .list_posts(&params)
                    .await
                    .unwrap_or_else(|e| fatal_err(&e))
            };
            let count_this_page = posts.len();
            for post in posts {
                if post.slug.contains(slug_substr) {
                    matched.push(post);
                }
            }
            if count_this_page < SLUG_SCAN_PER_PAGE as usize {
                break; // no more results
            }
            if p == MAX_PAGES_FOR_SLUG_SCAN {
                hit_cap = true;
            }
        }
        if hit_cap {
            eprintln!(
                "warning: scanned {} posts (page cap), more may exist; narrow with a search query",
                MAX_PAGES_FOR_SLUG_SCAN * SLUG_SCAN_PER_PAGE
            );
        }
        let total = matched.len() as u64;
        print_post_table(&matched, Some(total));
    } else {
        // Pure search-by-query (positional). Mirrors `list --search` but with
        // a sharper command name and a positional argument.
        let params = ListParams {
            per_page,
            search: query,
            status: status_filter,
            ..Default::default()
        };
        let (posts, total) = if want_pages {
            client
                .list_pages(&params)
                .await
                .unwrap_or_else(|e| fatal_err(&e))
        } else {
            client
                .list_posts(&params)
                .await
                .unwrap_or_else(|e| fatal_err(&e))
        };
        print_post_table(&posts, total);
    }
}
