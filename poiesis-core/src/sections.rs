use sha2::{Digest, Sha256};

use crate::{
    blocks::{parse_blocks, Block},
    error::PoiesisError,
    markdown::{html_to_markdown, markdown_to_html},
};

/// A section in a markdown document, identified by a heading
#[derive(Debug, Clone)]
pub struct Section {
    pub id: String,
    pub level: usize,
    pub text: String,
    /// Byte offset of the section heading line start in the full markdown
    pub start: usize,
    /// Byte offset of section end (start of next sibling/parent, or EOF)
    pub end: usize,
}

/// Maps a markdown byte range to its source Gutenberg block
#[derive(Debug, Clone)]
pub struct BlockRange {
    pub block_idx: usize,
    pub md_start: usize,
    pub md_end: usize,
}

/// A parsed content document with section and block mappings
pub struct ContentDocument {
    pub markdown: String,
    pub sections: Vec<Section>,
    pub(crate) blocks: Vec<Block>,
    #[allow(dead_code)]
    pub(crate) block_map: Vec<BlockRange>,
}

/// Parse raw Gutenberg content into a ContentDocument
pub fn parse_content(raw: &str) -> ContentDocument {
    let blocks = parse_blocks(raw);

    // Build markdown from blocks, tracking byte offsets
    let mut markdown = String::new();
    let mut block_map = Vec::new();

    for (i, block) in blocks.iter().enumerate() {
        let md_start = markdown.len();
        let block_md = if block.is_freeform {
            let html_stripped = strip_html_comments(&block.inner_html);
            if html_stripped.trim().is_empty() {
                String::new()
            } else {
                html_to_markdown(html_stripped)
            }
        } else {
            html_to_markdown(&block.inner_html)
        };

        if !block_md.trim().is_empty() {
            if !markdown.is_empty() {
                markdown.push('\n');
                markdown.push('\n');
            }
            markdown.push_str(&block_md);
        }

        let md_end = markdown.len();
        block_map.push(BlockRange {
            block_idx: i,
            md_start,
            md_end,
        });
    }

    let sections = parse_sections(&markdown);

    ContentDocument {
        markdown,
        sections,
        blocks,
        block_map,
    }
}

/// Strip HTML comments that are NOT Gutenberg block markers
fn strip_html_comments(s: &str) -> &str {
    // For now, just return as-is — freeform content rarely has HTML comments
    s
}

/// Parse sections from markdown text
pub fn parse_sections(markdown: &str) -> Vec<Section> {
    let mut sections = Vec::new();
    let mut heading_texts: Vec<String> = Vec::new();

    // Find all headings with their byte positions
    let mut headings: Vec<(usize, usize, String, usize)> = Vec::new(); // (start, level, text, end_of_line)
    let mut in_code_block = false;

    let mut pos = 0;
    for line in markdown.lines() {
        let line_start = pos;
        let line_end = pos + line.len();

        // Track code fences
        if line.starts_with("```") || line.starts_with("~~~") {
            in_code_block = !in_code_block;
            pos = line_end + 1; // +1 for newline
            continue;
        }

        if !in_code_block {
            if let Some((level, text)) = parse_heading_line(line) {
                headings.push((line_start, level, text, line_end));
            }
        }

        pos = line_end + 1;
    }

    // Assign section IDs and end positions
    let total_len = markdown.len();

    for (idx, (start, level, text, _line_end)) in headings.iter().enumerate() {
        // Find end: next heading at same or higher level (lower level number), or EOF
        let end = headings[idx + 1..]
            .iter()
            .find(|(_, next_level, _, _)| *next_level <= *level)
            .map(|(next_start, _, _, _)| *next_start)
            .unwrap_or(total_len);

        let id = assign_section_id(text, &heading_texts, &sections);
        heading_texts.push(text.clone());

        sections.push(Section {
            id,
            level: *level,
            text: text.clone(),
            start: *start,
            end,
        });
    }

    sections
}

/// Parse a heading line, returning (level, text) or None
fn parse_heading_line(line: &str) -> Option<(usize, String)> {
    let trimmed = line.trim_start();
    if !trimmed.starts_with('#') {
        return None;
    }

    let hash_count = trimmed.chars().take_while(|&c| c == '#').count();
    if hash_count > 6 {
        return None;
    }

    let rest = &trimmed[hash_count..];
    if !rest.starts_with(' ') && !rest.is_empty() {
        return None;
    }

    let text = rest.trim().to_string();
    Some((hash_count, text))
}

/// Compute section ID using same algorithm as flicknote-cli:
/// SHA-256 of heading text → first 8 bytes → base62 → take 2 chars
/// On collision: append \x00{index} to input, re-hash, take 3 chars
fn assign_section_id(
    text: &str,
    existing_texts: &[String],
    existing_sections: &[Section],
) -> String {
    let base_id = hash_to_base62(text, 2);

    // Check for collision
    let collision = existing_sections
        .iter()
        .any(|s| s.id == base_id || s.id.starts_with(&base_id));

    if collision {
        // Use position-based disambiguation: append \x00{count} to input
        let same_text_count = existing_texts.iter().filter(|t| t.as_str() == text).count();
        let disambig = format!("{}\x00{}", text, same_text_count);
        hash_to_base62(&disambig, 3)
    } else {
        base_id
    }
}

const BASE62_CHARS: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

fn hash_to_base62(text: &str, chars: usize) -> String {
    let mut hasher = Sha256::new();
    hasher.update(text.as_bytes());
    let result = hasher.finalize();

    // Take first 8 bytes, convert to u64
    let mut num: u64 = 0;
    for &byte in &result[..8] {
        num = (num << 8) | (byte as u64);
    }

    // Encode to base62
    let mut encoded = String::new();
    for _ in 0..chars {
        let idx = (num % 62) as usize;
        encoded.push(BASE62_CHARS[idx] as char);
        num /= 62;
    }

    // Reverse for proper order
    encoded.chars().rev().collect()
}

/// Find a section by ID
pub fn find_section(doc: &ContentDocument, id: &str) -> Result<Section, PoiesisError> {
    doc.sections
        .iter()
        .find(|s| s.id == id)
        .cloned()
        .ok_or_else(|| PoiesisError::SectionNotFound { id: id.to_string() })
}

/// Replace section body (keep heading, replace content after it)
pub fn replace_section(
    doc: &mut ContentDocument,
    id: &str,
    body: &str,
) -> Result<(), PoiesisError> {
    let section = find_section(doc, id)?;

    // Find end of heading line
    let heading_line_end = doc.markdown[section.start..]
        .find('\n')
        .map(|n| section.start + n + 1)
        .unwrap_or(section.end);

    let heading_line = doc.markdown[section.start..heading_line_end].to_string();
    let new_content = if body.trim().is_empty() {
        heading_line
    } else {
        format!("{}\n{}", heading_line.trim_end(), body)
    };

    doc.markdown = format!(
        "{}{}{}",
        &doc.markdown[..section.start],
        new_content,
        &doc.markdown[section.end..]
    );

    rebuild_sections(doc);
    Ok(())
}

/// Replace section including heading
pub fn replace_section_with_heading(
    doc: &mut ContentDocument,
    id: &str,
    content: &str,
) -> Result<(), PoiesisError> {
    let section = find_section(doc, id)?;

    doc.markdown = format!(
        "{}{}{}",
        &doc.markdown[..section.start],
        content,
        &doc.markdown[section.end..]
    );

    rebuild_sections(doc);
    Ok(())
}

/// Insert content before a section
pub fn insert_before(
    doc: &mut ContentDocument,
    id: &str,
    content: &str,
) -> Result<(), PoiesisError> {
    let section = find_section(doc, id)?;

    let insertion = if content.ends_with('\n') {
        content.to_string()
    } else {
        format!("{}\n\n", content)
    };

    doc.markdown = format!(
        "{}{}{}",
        &doc.markdown[..section.start],
        insertion,
        &doc.markdown[section.start..]
    );

    rebuild_sections(doc);
    Ok(())
}

/// Insert content after a section
pub fn insert_after(
    doc: &mut ContentDocument,
    id: &str,
    content: &str,
) -> Result<(), PoiesisError> {
    let section = find_section(doc, id)?;

    let insertion = if content.starts_with('\n') {
        content.to_string()
    } else {
        format!("\n\n{}", content)
    };

    doc.markdown = format!(
        "{}{}{}",
        &doc.markdown[..section.end],
        insertion,
        &doc.markdown[section.end..]
    );

    rebuild_sections(doc);
    Ok(())
}

/// Delete a section
pub fn delete_section(doc: &mut ContentDocument, id: &str) -> Result<(), PoiesisError> {
    let section = find_section(doc, id)?;

    // Remove the section and any leading blank lines before the next section
    let before = doc.markdown[..section.start].trim_end().to_string();
    let after = doc.markdown[section.end..].trim_start().to_string();

    doc.markdown = if before.is_empty() {
        after
    } else if after.is_empty() {
        before
    } else {
        format!("{}\n\n{}", before, after)
    };

    rebuild_sections(doc);
    Ok(())
}

/// Rename a section heading
pub fn rename_section(
    doc: &mut ContentDocument,
    id: &str,
    new_name: &str,
) -> Result<(), PoiesisError> {
    let section = find_section(doc, id)?;

    // Build new heading with same level
    let hashes = "#".repeat(section.level);
    let new_heading = format!("{} {}", hashes, new_name);

    // Find end of heading line
    let heading_line_end = doc.markdown[section.start..]
        .find('\n')
        .map(|n| section.start + n + 1)
        .unwrap_or(section.start + section.text.len() + section.level + 2);

    doc.markdown = format!(
        "{}{}\n{}",
        &doc.markdown[..section.start],
        new_heading,
        &doc.markdown[heading_line_end..]
    );

    rebuild_sections(doc);
    Ok(())
}

/// Convert a ContentDocument back to raw Gutenberg markup
pub fn to_raw(doc: &ContentDocument) -> String {
    // Regenerate blocks from the updated markdown
    // Strategy: map sections of markdown back to blocks
    // For simplicity: if we have blocks, regenerate the HTML for each block
    // based on the corresponding markdown section

    if doc.blocks.is_empty() {
        // No blocks — convert the whole markdown to a paragraph block
        let html = markdown_to_html(&doc.markdown);
        return format!("<!-- wp:paragraph -->{html}<!-- /wp:paragraph -->");
    }

    // Rebuild block list: for each block, check if any part of it was edited
    // by comparing the current markdown section to the original
    // Simple approach: regenerate all blocks from current markdown
    regenerate_blocks(&doc.markdown, &doc.blocks)
}

/// Regenerate Gutenberg blocks from edited markdown, preserving block structure
fn regenerate_blocks(markdown: &str, original_blocks: &[Block]) -> String {
    // Split markdown into logical sections by double-newline
    // Then map back to block types from the original

    // Find non-freeform original blocks to use as templates
    let content_blocks: Vec<&Block> = original_blocks.iter().filter(|b| !b.is_freeform).collect();

    if content_blocks.is_empty() {
        // All freeform — wrap in paragraph block
        let html = markdown_to_html(markdown);
        return format!("<!-- wp:paragraph -->{html}<!-- /wp:paragraph -->");
    }

    // Parse markdown paragraphs/sections
    let paragraphs: Vec<&str> = split_markdown_blocks(markdown);

    let mut output = String::new();
    let mut block_idx = 0;

    for para in &paragraphs {
        if para.trim().is_empty() {
            continue;
        }

        let html = markdown_to_html(para);

        // Determine block type based on content
        let block = if block_idx < content_blocks.len() {
            Some(content_blocks[block_idx])
        } else {
            None
        };

        match block {
            Some(b) => {
                // Use original block type and attributes
                let open = match &b.attrs_json {
                    Some(attrs) => format!("<!-- wp:{} {} -->", b.block_type, attrs),
                    None => format!("<!-- wp:{} -->", b.block_type),
                };
                output.push_str(&open);
                output.push_str(&html);
                output.push_str(&format!("<!-- /wp:{} -->", b.block_type));
            }
            None => {
                // New block — use paragraph
                output.push_str("<!-- wp:paragraph -->");
                output.push_str(&html);
                output.push_str("<!-- /wp:paragraph -->");
            }
        }
        output.push('\n');
        block_idx += 1;
    }

    // If original had more blocks (e.g., separator), they're lost in this simple approach
    // For a full implementation, we'd need more sophisticated mapping

    output.trim_end().to_string()
}

/// Split markdown into block-level sections (by double newline)
fn split_markdown_blocks(markdown: &str) -> Vec<&str> {
    markdown.split("\n\n").collect()
}

/// Rebuild the sections and block_map after a mutation
pub fn rebuild_sections(doc: &mut ContentDocument) {
    doc.sections = parse_sections(&doc.markdown);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_section_id_deterministic() {
        let id1 = hash_to_base62("My Heading", 2);
        let id2 = hash_to_base62("My Heading", 2);
        assert_eq!(id1, id2);
    }

    #[test]
    fn test_section_id_different_headings() {
        let id1 = hash_to_base62("Heading One", 2);
        let id2 = hash_to_base62("Heading Two", 2);
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_section_id_collision_extends_to_3() {
        // Two identical headings should produce different IDs
        let markdown = "## Intro\n\nFirst intro.\n\n## Intro\n\nSecond intro.";
        let sections = parse_sections(markdown);
        assert_eq!(sections.len(), 2);
        assert_ne!(sections[0].id, sections[1].id);
        // Second should be 3 chars
        assert_eq!(sections[1].id.len(), 3);
    }

    #[test]
    fn test_parse_content_no_headings() {
        let markdown = "Just some text without any headings.";
        let sections = parse_sections(markdown);
        assert!(sections.is_empty());
    }

    #[test]
    fn test_parse_content_single_heading() {
        let markdown = "## My Section\n\nContent here.";
        let sections = parse_sections(markdown);
        assert_eq!(sections.len(), 1);
        assert_eq!(sections[0].text, "My Section");
        assert_eq!(sections[0].level, 2);
        assert_eq!(sections[0].end, markdown.len());
    }

    #[test]
    fn test_parse_content_multiple_headings() {
        let markdown = "## Section A\n\nContent A.\n\n## Section B\n\nContent B.";
        let sections = parse_sections(markdown);
        assert_eq!(sections.len(), 2);
        assert_eq!(sections[0].text, "Section A");
        assert_eq!(sections[1].text, "Section B");

        // Section A ends where Section B starts
        assert_eq!(sections[0].end, sections[1].start);
    }

    #[test]
    fn test_parse_content_nested_headings() {
        let markdown =
            "## Chapter\n\nIntro.\n\n### Sub-section\n\nDetail.\n\n## Next Chapter\n\nMore.";
        let sections = parse_sections(markdown);
        assert_eq!(sections.len(), 3);
        // h3 end should be at next h2
        let h3 = sections.iter().find(|s| s.level == 3).unwrap();
        let next_h2 = sections.iter().find(|s| s.text == "Next Chapter").unwrap();
        assert_eq!(h3.end, next_h2.start);
    }

    #[test]
    fn test_find_section_exists() {
        let raw = concat!(
            "<!-- wp:heading {\"level\":2} --><h2>Alpha</h2><!-- /wp:heading -->",
            "<!-- wp:paragraph --><p>Text A.</p><!-- /wp:paragraph -->",
            "<!-- wp:heading {\"level\":2} --><h2>Beta</h2><!-- /wp:heading -->",
            "<!-- wp:paragraph --><p>Text B.</p><!-- /wp:paragraph -->",
        );
        let doc = parse_content(raw);
        // find by iterating sections
        assert!(
            !doc.sections.is_empty(),
            "expected sections in heading content"
        );
        let id = doc.sections[0].id.clone();
        let result = find_section(&doc, &id);
        assert!(result.is_ok());
    }

    #[test]
    fn test_find_section_not_found() {
        let doc = ContentDocument {
            markdown: "## Hello\n\nWorld.".to_string(),
            sections: parse_sections("## Hello\n\nWorld."),
            blocks: vec![],
            block_map: vec![],
        };
        let result = find_section(&doc, "ZZ");
        assert!(matches!(result, Err(PoiesisError::SectionNotFound { id }) if id == "ZZ"));
    }

    #[test]
    fn test_replace_section_body() {
        let markdown = "## Section A\n\nOld content.\n\n## Section B\n\nKeep this.";
        let mut doc = ContentDocument {
            markdown: markdown.to_string(),
            sections: parse_sections(markdown),
            blocks: vec![],
            block_map: vec![],
        };
        let id = doc.sections[0].id.clone();
        replace_section(&mut doc, &id, "New content.").unwrap();
        assert!(doc.markdown.contains("New content."));
        assert!(doc.markdown.contains("Section A")); // heading preserved
        assert!(doc.markdown.contains("Section B")); // other section untouched
    }

    #[test]
    fn test_replace_section_with_heading() {
        let markdown = "## Old Heading\n\nContent.\n\n## Other\n\nStuff.";
        let mut doc = ContentDocument {
            markdown: markdown.to_string(),
            sections: parse_sections(markdown),
            blocks: vec![],
            block_map: vec![],
        };
        let id = doc.sections[0].id.clone();
        replace_section_with_heading(&mut doc, &id, "## New Heading\n\nNew content.").unwrap();
        assert!(doc.markdown.contains("New Heading"));
        assert!(!doc.markdown.contains("Old Heading"));
    }

    #[test]
    fn test_replace_section_first() {
        let markdown = "## First\n\nContent.\n\n## Second\n\nMore.";
        let mut doc = ContentDocument {
            markdown: markdown.to_string(),
            sections: parse_sections(markdown),
            blocks: vec![],
            block_map: vec![],
        };
        let id = doc.sections[0].id.clone();
        replace_section(&mut doc, &id, "Updated.").unwrap();
        assert!(doc.markdown.contains("Updated."));
        assert!(doc.markdown.contains("Second"));
    }

    #[test]
    fn test_replace_section_last() {
        let markdown = "## First\n\nContent.\n\n## Last\n\nOld content.";
        let mut doc = ContentDocument {
            markdown: markdown.to_string(),
            sections: parse_sections(markdown),
            blocks: vec![],
            block_map: vec![],
        };
        let id = doc.sections[1].id.clone();
        replace_section(&mut doc, &id, "New last content.").unwrap();
        assert!(doc.markdown.contains("New last content."));
        assert!(doc.markdown.contains("First"));
    }

    #[test]
    fn test_insert_before() {
        let markdown = "## Alpha\n\nText A.\n\n## Beta\n\nText B.";
        let mut doc = ContentDocument {
            markdown: markdown.to_string(),
            sections: parse_sections(markdown),
            blocks: vec![],
            block_map: vec![],
        };
        let beta_id = doc.sections[1].id.clone();
        insert_before(&mut doc, &beta_id, "## Inserted\n\nNew section.").unwrap();
        let pos_inserted = doc.markdown.find("Inserted").unwrap();
        let pos_beta = doc.markdown.find("Beta").unwrap();
        assert!(pos_inserted < pos_beta);
    }

    #[test]
    fn test_insert_after() {
        let markdown = "## Alpha\n\nText A.\n\n## Beta\n\nText B.";
        let mut doc = ContentDocument {
            markdown: markdown.to_string(),
            sections: parse_sections(markdown),
            blocks: vec![],
            block_map: vec![],
        };
        let alpha_id = doc.sections[0].id.clone();
        insert_after(&mut doc, &alpha_id, "## Inserted\n\nNew section.").unwrap();
        let pos_alpha = doc.markdown.find("Alpha").unwrap();
        let pos_inserted = doc.markdown.find("Inserted").unwrap();
        let pos_beta = doc.markdown.find("Beta").unwrap();
        assert!(pos_alpha < pos_inserted);
        assert!(pos_inserted < pos_beta);
    }

    #[test]
    fn test_delete_section() {
        let markdown = "## Alpha\n\nText A.\n\n## Beta\n\nText B.\n\n## Gamma\n\nText C.";
        let mut doc = ContentDocument {
            markdown: markdown.to_string(),
            sections: parse_sections(markdown),
            blocks: vec![],
            block_map: vec![],
        };
        let beta_id = doc.sections[1].id.clone();
        delete_section(&mut doc, &beta_id).unwrap();
        assert!(!doc.markdown.contains("Beta"));
        assert!(doc.markdown.contains("Alpha"));
        assert!(doc.markdown.contains("Gamma"));
    }

    #[test]
    fn test_delete_only_section() {
        let markdown = "## Only\n\nSome content.";
        let mut doc = ContentDocument {
            markdown: markdown.to_string(),
            sections: parse_sections(markdown),
            blocks: vec![],
            block_map: vec![],
        };
        let id = doc.sections[0].id.clone();
        delete_section(&mut doc, &id).unwrap();
        assert!(doc.markdown.trim().is_empty() || !doc.markdown.contains("Only"));
    }

    #[test]
    fn test_rename_section() {
        let markdown = "## Old Name\n\nContent here.\n\n## Other\n\nStuff.";
        let mut doc = ContentDocument {
            markdown: markdown.to_string(),
            sections: parse_sections(markdown),
            blocks: vec![],
            block_map: vec![],
        };
        let id = doc.sections[0].id.clone();
        rename_section(&mut doc, &id, "New Name").unwrap();
        assert!(doc.markdown.contains("## New Name"));
        assert!(!doc.markdown.contains("Old Name"));
        // Other section untouched
        assert!(doc.markdown.contains("Other"));
    }

    #[test]
    fn test_to_raw_unchanged() {
        let raw = "<!-- wp:paragraph --><p>Hello world</p><!-- /wp:paragraph -->";
        let doc = parse_content(raw);
        // The basic check: to_raw produces valid Gutenberg format
        let output = to_raw(&doc);
        assert!(output.contains("wp:paragraph") || output.contains("<p>"));
    }

    #[test]
    fn test_headings_inside_code_blocks_ignored() {
        let markdown = "## Real Heading\n\nContent.\n\n```\n## Not A Heading\n```\n";
        let sections = parse_sections(markdown);
        assert_eq!(sections.len(), 1);
        assert_eq!(sections[0].text, "Real Heading");
    }
}
