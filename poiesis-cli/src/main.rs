use clap::{Parser, Subcommand};

mod commands;
mod display;
mod util;

/// Poiesis — WordPress content management CLI
///
/// Configuration: ~/.config/poiesis/config.toml
///
///   [site]
///   url = "https://your-site.com"
///   username = "your-username"
///
/// Password: set POIESIS_PASSWORD environment variable
#[derive(Parser)]
#[command(name = "poi", version, about)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// List posts or pages
    List {
        /// Content type: post or page
        #[arg(long, value_name = "TYPE")]
        r#type: Option<String>,

        /// Filter by status: draft, publish, pending, private
        #[arg(long)]
        status: Option<String>,

        /// Search query
        #[arg(long)]
        search: Option<String>,

        /// Results per page (default: 20)
        #[arg(long, value_name = "N")]
        per_page: Option<u32>,

        /// Page number
        #[arg(long)]
        page: Option<u32>,
    },

    /// Find posts or pages by search query or slug substring
    Find {
        /// Search query (matches title and content via WP REST)
        query: Option<String>,

        /// Slug substring to match (client-side filter; paginates up to 1000 posts)
        #[arg(long, value_name = "SUBSTRING", allow_hyphen_values = true)]
        slug: Option<String>,

        /// Content type: post or page
        #[arg(long, value_name = "TYPE")]
        r#type: Option<String>,

        /// Filter by status: draft, publish, pending, private
        #[arg(long)]
        status: Option<String>,

        /// Results per page (default: 20; ignored when --slug is set)
        #[arg(long, value_name = "N")]
        per_page: Option<u32>,
    },

    /// Show post details and content
    Detail {
        /// Post or page ID
        id: String,

        /// Show heading tree only
        #[arg(long)]
        tree: bool,

        /// Show specific section content
        #[arg(long, value_name = "SID")]
        section: Option<String>,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Show post content (no metadata header)
    Content {
        /// Post or page ID
        id: String,

        /// Show specific section only
        #[arg(long, value_name = "SID")]
        section: Option<String>,
    },

    /// Create a new post or page (pipe markdown on stdin)
    Create {
        /// Post title (extracted from `# Heading` if not provided)
        #[arg(long)]
        title: Option<String>,

        /// Content type: post or page (default: post)
        #[arg(long, value_name = "TYPE")]
        r#type: Option<String>,

        /// Initial status: draft or publish (default: draft)
        #[arg(long)]
        status: Option<String>,
    },

    /// Edit post content or metadata
    Modify {
        /// Post or page ID
        id: String,

        /// Edit specific section only (pipe new body on stdin)
        #[arg(long, value_name = "SID")]
        section: Option<String>,

        /// Update post title
        #[arg(long)]
        title: Option<String>,

        /// Update post status
        #[arg(long)]
        status: Option<String>,

        /// Update post slug
        #[arg(long)]
        slug: Option<String>,
    },

    /// Delete a section or trash a post
    Delete {
        /// Post or page ID
        id: String,

        /// Delete a specific section only
        #[arg(long, value_name = "SID")]
        section: Option<String>,

        /// Confirm trashing the entire post
        #[arg(long)]
        force: bool,
    },

    /// Rename a section heading
    Rename {
        /// Post or page ID
        id: String,

        /// Section ID to rename
        #[arg(long, value_name = "SID")]
        section: String,

        /// New heading text
        new_name: String,
    },

    /// Insert content before or after a section (pipe markdown on stdin)
    Insert {
        /// Post or page ID
        id: String,

        /// Section ID for position reference
        #[arg(long, value_name = "SID")]
        section: String,

        /// Insert before the section
        #[arg(long)]
        before: bool,

        /// Insert after the section
        #[arg(long)]
        after: bool,
    },

    /// Append content to a post (pipe markdown on stdin)
    Append {
        /// Post or page ID
        id: String,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Command::List {
            r#type,
            status,
            search,
            per_page,
            page,
        } => {
            commands::list::run(r#type, status, search, per_page, page).await;
        }
        Command::Find {
            query,
            slug,
            r#type,
            status,
            per_page,
        } => {
            commands::find::run(query, slug, r#type, status, per_page).await;
        }
        Command::Detail {
            id,
            tree,
            section,
            json,
        } => {
            commands::detail::run(&id, tree, section, json).await;
        }
        Command::Content { id, section } => {
            commands::content::run(&id, section).await;
        }
        Command::Create {
            title,
            r#type,
            status,
        } => {
            commands::create::run(title, r#type, status).await;
        }
        Command::Modify {
            id,
            section,
            title,
            status,
            slug,
        } => {
            commands::modify::run(&id, section, title, status, slug).await;
        }
        Command::Delete { id, section, force } => {
            commands::delete::run(&id, section, force).await;
        }
        Command::Rename {
            id,
            section,
            new_name,
        } => {
            commands::rename::run(&id, &section, &new_name).await;
        }
        Command::Insert {
            id,
            section,
            before,
            after,
        } => {
            commands::insert::run(&id, &section, before, after).await;
        }
        Command::Append { id } => {
            commands::append::run(&id).await;
        }
    }
}
