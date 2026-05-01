# Poiesis

A Rust CLI for reading and editing WordPress content as markdown. Flicknote-style interface — tree navigation, section IDs, section-level editing — with the WordPress REST API as the backend.

## Setup

### Configuration

Create `~/.config/poiesis/config.toml`:

```toml
[site]
url = "https://your-site.com"
username = "your-username"
```

Set the `POIESIS_PASSWORD` environment variable to your WordPress application password:

```bash
export POIESIS_PASSWORD="xxxx xxxx xxxx xxxx xxxx xxxx"
```

### Install

```bash
cargo install --path poiesis-cli
```

## Usage

### List posts and pages

```bash
poi list                           # list recent posts
poi list --type page               # list pages
poi list --status draft            # list drafts
poi list --search "keyword"        # search
poi list --per-page 50             # more results
```

### Find posts and pages

```bash
poi find "keyword"                       # WP-side search by title/content
poi find "keyword" --type page           # search pages
poi find "keyword" --status draft        # filter by status

poi find --slug "-51634-"                # client-side filter on slug substring
poi find "佛说" --slug "-51574-"          # narrow with query first, then slug filter
```

`--slug` performs a client-side scan (WP REST has no native slug-substring filter),
paginating up to 1000 posts. Combine with a positional query to narrow the scan
on larger sites.

### View content

```bash
poi detail 42                      # full post with section IDs
poi detail 42 --tree               # heading tree only
poi detail 42 --section Fb         # single section
poi detail 42 --json               # raw JSON

poi content 42                     # content only, with section IDs inline
poi content 42 --section Fb        # single section content
```

### Edit content

```bash
# Replace section body
echo "Updated content." | poi modify 42 --section Fb

# Replace full content
cat updated.md | poi modify 42

# Metadata only
poi modify 42 --status publish
poi modify 42 --title "New Title"
poi modify 42 --slug "new-slug"
poi modify 42 --status draft --title "WIP"
```

### Create posts

```bash
# Title extracted from first # heading
echo "# My Post\n\nContent here." | poi create

# Explicit title
echo "Content." | poi create --title "My Post"

# Create a page
echo "# About\n\nWe are..." | poi create --type page --status publish
```

### Section operations

```bash
# Insert before/after section (--section is required, --before/--after picks position)
echo "## New Section\n\nContent." | poi insert 42 --section Fb --before
echo "## New Section\n\nContent." | poi insert 42 --section Fb --after

# Append to end
echo "## Final Notes\n\nFinal content." | poi append 42

# Rename heading
poi rename 42 --section Fb "New Heading Name"

# Delete section
poi delete 42 --section Fb

# Trash entire post (requires --force)
poi delete 42 --force
```

## Section IDs

Section IDs are 2-character base62 identifiers derived from heading text via SHA-256. They're stable as long as heading text doesn't change.

Find section IDs with `poi detail <id> --tree` or inline with `poi content <id>`.

## Architecture

```
poiesis/
├── poiesis-core/    # REST API client, block parser, markdown conversion, section system
└── poiesis-cli/     # CLI commands using clap
```

- **REST client**: `reqwest` with Basic auth using WordPress application passwords
- **Block parser**: Regex-based Gutenberg block comment parser with round-trip fidelity
- **Markdown**: `htmd` (HTML→markdown) and `pulldown-cmark` (markdown→HTML)
- **Sections**: SHA-256 based IDs, same algorithm as flicknote-cli

## License

MIT
