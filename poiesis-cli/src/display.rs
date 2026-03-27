use comfy_table::{Attribute, Cell, ContentArrangement, Table};
use poiesis_core::{parse_heading_line, sections::Section, Post};

/// Format a list of posts/pages as a table
pub fn print_post_table(posts: &[Post], total: Option<u64>) {
    let mut table = Table::new();
    table.set_content_arrangement(ContentArrangement::Dynamic);
    table.set_header(vec![
        Cell::new("ID").add_attribute(Attribute::Bold),
        Cell::new("Type").add_attribute(Attribute::Bold),
        Cell::new("Title").add_attribute(Attribute::Bold),
        Cell::new("Status").add_attribute(Attribute::Bold),
        Cell::new("Modified").add_attribute(Attribute::Bold),
    ]);

    for post in posts {
        let modified = post.modified.get(..10).unwrap_or(&post.modified);
        table.add_row(vec![
            Cell::new(post.id),
            Cell::new(&post.post_type),
            Cell::new(&post.title.raw),
            Cell::new(post.status.to_string()),
            Cell::new(modified),
        ]);
    }

    println!("{}", table);

    match total {
        Some(t) if t > posts.len() as u64 => {
            println!("(showing {} of {})", posts.len(), t);
        }
        _ => {
            println!("(showing {} of {})", posts.len(), posts.len());
        }
    }
}

/// Format a post's metadata header
pub fn print_post_header(post: &Post) {
    println!("Post {}: {} ({})", post.id, post.title.raw, post.status);
    println!("Type:     {}", post.post_type);
    println!("Slug:     {}", post.slug);
    println!("Link:     {}", post.link);
    println!(
        "Modified: {}",
        post.modified.get(..10).unwrap_or(&post.modified)
    );
    println!();
}

/// Print a heading tree
pub fn print_heading_tree(post: &Post, sections: &[Section]) {
    println!("Post {}: {} ({})", post.id, post.title.raw, post.status);

    if sections.is_empty() {
        println!("  (no headings)");
        return;
    }

    print_sections_tree(
        sections,
        sections.iter().map(|s| s.level).min().unwrap_or(1),
    );
}

fn print_sections_tree(sections: &[Section], base_level: usize) {
    for (i, section) in sections.iter().enumerate() {
        let relative_depth = section.level.saturating_sub(base_level);
        let indent = "   ".repeat(relative_depth);
        let is_last = i == sections.len() - 1 || sections[i + 1].level <= section.level;
        let hashes = "#".repeat(section.level);

        if relative_depth == 0 {
            let branch = if i == sections.len() - 1 { "└─" } else { "├─" };
            println!("{}[{}] {} {}", branch, section.id, hashes, section.text);
        } else {
            let branch = if is_last { "└─" } else { "├─" };
            println!(
                "   {}{}[{}] {} {}",
                indent, branch, section.id, hashes, section.text
            );
        }
    }
}

/// Print content with section IDs inline after headings
pub fn print_content_with_ids(markdown: &str, sections: &[Section]) {
    for line in markdown.lines() {
        if let Some((level, text)) = parse_heading_line(line) {
            // Find matching section ID
            let id = sections
                .iter()
                .find(|s| s.level == level && s.text == text)
                .map(|s| s.id.as_str())
                .unwrap_or("?");
            let hashes = "#".repeat(level);
            println!("{} {} [{}]", hashes, text, id);
        } else {
            println!("{}", line);
        }
    }
}
