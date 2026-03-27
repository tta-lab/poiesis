use regex::Regex;

/// A parsed Gutenberg block
#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    /// Block type, e.g. "paragraph", "heading", "list"
    pub block_type: String,
    /// Raw JSON attributes string (e.g. `{"level":2}`)
    pub attrs_json: Option<String>,
    /// HTML content between block comment delimiters
    pub inner_html: String,
    /// Nested blocks (e.g. list-item inside list)
    pub inner_blocks: Vec<Block>,
    /// Raw source for non-block content between blocks
    pub is_freeform: bool,
}

impl Block {
    fn new_freeform(html: &str) -> Self {
        Block {
            block_type: String::new(),
            attrs_json: None,
            inner_html: html.to_string(),
            inner_blocks: vec![],
            is_freeform: true,
        }
    }
}

/// Parse raw Gutenberg content into blocks
pub fn parse_blocks(raw: &str) -> Vec<Block> {
    if raw.is_empty() {
        return vec![];
    }

    parse_blocks_from(raw)
}

fn parse_blocks_from(input: &str) -> Vec<Block> {
    let open_re =
        Regex::new(r"<!--\s+wp:([a-z0-9/-]+)(\s+(\{[^}]*(?:\{[^}]*\}[^}]*)*\}))?\s+(/)?\s*-->")
            .unwrap();

    let mut blocks = Vec::new();
    let mut pos = 0;

    while pos < input.len() {
        // Find next block comment
        let search_from = &input[pos..];
        if let Some(open_match) = open_re.find(search_from) {
            let open_start = pos + open_match.start();
            let open_end = pos + open_match.end();

            // Any content before this block comment is freeform
            if open_start > pos {
                let freeform = &input[pos..open_start];
                if !freeform.trim().is_empty() {
                    blocks.push(Block::new_freeform(freeform));
                }
            }

            let full_match = open_match.as_str();
            let caps = open_re.captures(full_match).unwrap();
            let block_type = caps.get(1).unwrap().as_str().to_string();
            let attrs_json = caps.get(3).map(|m| m.as_str().to_string());
            let is_self_closing = caps.get(4).is_some();

            if is_self_closing {
                blocks.push(Block {
                    block_type,
                    attrs_json,
                    inner_html: String::new(),
                    inner_blocks: vec![],
                    is_freeform: false,
                });
                pos = open_end;
            } else {
                // Find the matching close comment
                let close_pattern = format!("<!-- /wp:{} -->", block_type);
                let rest = &input[open_end..];
                if let Some(close_pos) = find_close(rest, &block_type) {
                    let inner_raw = &rest[..close_pos];
                    let close_end = open_end + close_pos + close_pattern.len();

                    // Recursively parse inner content
                    let inner_blocks = parse_blocks_from(inner_raw);
                    // If inner blocks exist, inner_html is the full raw inner content
                    // If no sub-blocks, inner_html is just the raw HTML
                    let inner_html = inner_raw.to_string();

                    blocks.push(Block {
                        block_type,
                        attrs_json,
                        inner_html,
                        inner_blocks,
                        is_freeform: false,
                    });
                    pos = close_end;
                } else {
                    // Malformed: no closing comment — treat as freeform
                    let remaining = &input[pos..];
                    blocks.push(Block::new_freeform(remaining));
                    break;
                }
            }
        } else {
            // No more block comments — rest is freeform
            let remaining = &input[pos..];
            if !remaining.trim().is_empty() {
                blocks.push(Block::new_freeform(remaining));
            }
            break;
        }
    }

    blocks
}

/// Find the position of the matching close comment for a block type,
/// accounting for nested same-type blocks
fn find_close(input: &str, block_type: &str) -> Option<usize> {
    let open_pattern = format!("<!-- wp:{}", block_type);
    let close_pattern = format!("<!-- /wp:{} -->", block_type);

    let mut depth = 0i32;
    let mut search_pos = 0;

    loop {
        let rest = &input[search_pos..];
        let next_open = rest.find(&open_pattern).map(|p| (p, true));
        let next_close = rest.find(&close_pattern).map(|p| (p, false));

        let next = match (next_open, next_close) {
            (Some(o), Some(c)) => {
                if o.0 < c.0 {
                    Some(o)
                } else {
                    Some(c)
                }
            }
            (Some(o), None) => Some(o),
            (None, Some(c)) => Some(c),
            (None, None) => None,
        };

        match next {
            None => return None,
            Some((offset, is_open)) => {
                if is_open {
                    // Check it's actually a block open (not something else starting with wp:type)
                    let after = &rest[offset + open_pattern.len()..];
                    // Must be followed by space, { or -->
                    if after.starts_with(' ') || after.starts_with('{') || after.starts_with(" /") {
                        depth += 1;
                        search_pos += offset + open_pattern.len();
                    } else {
                        search_pos += offset + 1;
                    }
                } else {
                    if depth == 0 {
                        return Some(search_pos + offset);
                    }
                    depth -= 1;
                    search_pos += offset + close_pattern.len();
                }
            }
        }
    }
}

/// Serialize blocks back to raw Gutenberg format
pub fn serialize_blocks(blocks: &[Block]) -> String {
    let mut out = String::new();
    for block in blocks {
        if block.is_freeform {
            out.push_str(&block.inner_html);
        } else {
            // Open comment
            match &block.attrs_json {
                Some(attrs) => {
                    out.push_str(&format!("<!-- wp:{} {} -->", block.block_type, attrs));
                }
                None => {
                    out.push_str(&format!("<!-- wp:{} -->", block.block_type));
                }
            }
            // Inner content
            out.push_str(&block.inner_html);
            // Close comment
            out.push_str(&format!("<!-- /wp:{} -->", block.block_type));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_empty_content() {
        let blocks = parse_blocks("");
        assert!(blocks.is_empty());
    }

    #[test]
    fn test_parse_simple_paragraph() {
        let raw = "<!-- wp:paragraph --><p>Hello world</p><!-- /wp:paragraph -->";
        let blocks = parse_blocks(raw);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].block_type, "paragraph");
        assert_eq!(blocks[0].attrs_json, None);
        assert_eq!(blocks[0].inner_html, "<p>Hello world</p>");
    }

    #[test]
    fn test_parse_heading_block() {
        let raw = r#"<!-- wp:heading {"level":2} --><h2>My Heading</h2><!-- /wp:heading -->"#;
        let blocks = parse_blocks(raw);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].block_type, "heading");
        assert_eq!(blocks[0].attrs_json, Some(r#"{"level":2}"#.to_string()));
        assert_eq!(blocks[0].inner_html, "<h2>My Heading</h2>");
    }

    #[test]
    fn test_parse_separator() {
        let raw = "<!-- wp:separator /-->";
        let blocks = parse_blocks(raw);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].block_type, "separator");
        assert!(blocks[0].inner_html.is_empty());
    }

    #[test]
    fn test_parse_block_with_plugin_attrs() {
        let raw = r#"<!-- wp:paragraph {"TrpContentRestriction":{"content":""}} --><p>Text</p><!-- /wp:paragraph -->"#;
        let blocks = parse_blocks(raw);
        assert_eq!(blocks.len(), 1);
        assert!(blocks[0]
            .attrs_json
            .as_ref()
            .unwrap()
            .contains("TrpContentRestriction"));
    }

    #[test]
    fn test_parse_block_name_with_hyphens() {
        let raw = "<!-- wp:core-embed --><div>embed</div><!-- /wp:core-embed -->";
        let blocks = parse_blocks(raw);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].block_type, "core-embed");
    }

    #[test]
    fn test_parse_empty_attrs_object() {
        let raw = "<!-- wp:paragraph {} --><p>text</p><!-- /wp:paragraph -->";
        let blocks = parse_blocks(raw);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].attrs_json, Some("{}".to_string()));
    }

    #[test]
    fn test_parse_malformed_block() {
        // Unclosed block — treat as freeform
        let raw = "<!-- wp:paragraph --><p>Unclosed block";
        let blocks = parse_blocks(raw);
        assert_eq!(blocks.len(), 1);
        assert!(blocks[0].is_freeform);
    }

    #[test]
    fn test_round_trip_simple() {
        let raw = "<!-- wp:paragraph --><p>Hello</p><!-- /wp:paragraph -->";
        let blocks = parse_blocks(raw);
        let serialized = serialize_blocks(&blocks);
        assert_eq!(serialized, raw);
    }

    #[test]
    fn test_round_trip_with_attrs() {
        let raw = r#"<!-- wp:heading {"level":2} --><h2>Title</h2><!-- /wp:heading -->"#;
        let blocks = parse_blocks(raw);
        let serialized = serialize_blocks(&blocks);
        assert_eq!(serialized, raw);
    }

    #[test]
    fn test_round_trip_multiple_blocks() {
        // Round-trip preserves content modulo inter-block whitespace normalization
        let raw = concat!(
            "<!-- wp:heading {\"level\":2} --><h2>Title</h2><!-- /wp:heading -->",
            "<!-- wp:paragraph --><p>Body text here.</p><!-- /wp:paragraph -->",
        );
        let blocks = parse_blocks(raw);
        let serialized = serialize_blocks(&blocks);
        assert_eq!(serialized, raw);

        // Verify that content with inter-block whitespace round-trips to same content blocks
        let raw_with_ws = concat!(
            "<!-- wp:heading {\"level\":2} --><h2>Title</h2><!-- /wp:heading -->",
            "\n",
            "<!-- wp:paragraph --><p>Body text here.</p><!-- /wp:paragraph -->",
        );
        let blocks_ws = parse_blocks(raw_with_ws);
        assert_eq!(blocks_ws.iter().filter(|b| !b.is_freeform).count(), 2);
        let h = blocks_ws
            .iter()
            .find(|b| b.block_type == "heading")
            .unwrap();
        assert_eq!(h.inner_html, "<h2>Title</h2>");
    }

    #[test]
    fn test_parse_list_with_nested_items() {
        let raw = concat!(
            "<!-- wp:list -->",
            "<ul>",
            "<!-- wp:list-item --><li>Item 1</li><!-- /wp:list-item -->",
            "<!-- wp:list-item --><li>Item 2</li><!-- /wp:list-item -->",
            "</ul>",
            "<!-- /wp:list -->",
        );
        let blocks = parse_blocks(raw);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].block_type, "list");
        // Inner blocks should include list-item blocks
        let inner_blocks = &blocks[0].inner_blocks;
        let list_items: Vec<_> = inner_blocks
            .iter()
            .filter(|b| b.block_type == "list-item")
            .collect();
        assert_eq!(list_items.len(), 2);
    }
}
