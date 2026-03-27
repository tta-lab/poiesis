use serde::{Deserialize, Serialize};

/// Unified response type for both posts and pages
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Post {
    pub id: u64,
    pub title: PostTitle,
    pub content: PostContent,
    pub excerpt: Option<PostExcerpt>,
    pub slug: String,
    pub status: PostStatus,
    #[serde(rename = "type")]
    pub post_type: String,
    pub date: String,
    pub modified: String,
    pub link: String,
    pub author: u64,
    pub categories: Option<Vec<u64>>,
    pub tags: Option<Vec<u64>>,
    pub parent: Option<u64>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PostTitle {
    pub raw: String,
    pub rendered: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PostContent {
    pub raw: String,
    pub rendered: String,
    #[serde(default)]
    pub block_version: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PostExcerpt {
    pub raw: String,
    pub rendered: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PostStatus {
    Publish,
    Draft,
    Pending,
    Private,
    Trash,
    Custom(String),
}

impl std::fmt::Display for PostStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PostStatus::Publish => write!(f, "publish"),
            PostStatus::Draft => write!(f, "draft"),
            PostStatus::Pending => write!(f, "pending"),
            PostStatus::Private => write!(f, "private"),
            PostStatus::Trash => write!(f, "trash"),
            PostStatus::Custom(s) => write!(f, "{}", s),
        }
    }
}

impl<'de> Deserialize<'de> for PostStatus {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(match s.as_str() {
            "publish" => PostStatus::Publish,
            "draft" => PostStatus::Draft,
            "pending" => PostStatus::Pending,
            "private" => PostStatus::Private,
            "trash" => PostStatus::Trash,
            other => PostStatus::Custom(other.to_string()),
        })
    }
}

impl Serialize for PostStatus {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl PostStatus {
    pub fn parse(s: &str) -> Self {
        match s {
            "publish" => PostStatus::Publish,
            "draft" => PostStatus::Draft,
            "pending" => PostStatus::Pending,
            "private" => PostStatus::Private,
            "trash" => PostStatus::Trash,
            other => PostStatus::Custom(other.to_string()),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ListParams {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    pub search: Option<String>,
    pub status: Option<Vec<PostStatus>>,
    pub orderby: Option<String>,
    pub order: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct CreateParams {
    pub title: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<PostStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slug: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub excerpt: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub categories: Option<Vec<u64>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<u64>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent: Option<u64>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct UpdateParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<PostStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slug: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub excerpt: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub categories: Option<Vec<u64>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<u64>>,
}
