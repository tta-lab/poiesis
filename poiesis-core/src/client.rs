use base64::Engine;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde::Deserialize;

use crate::{
    config::Config,
    error::PoiesisError,
    types::{CreateParams, ListParams, Post, UpdateParams},
};

/// WP REST API client. Does NOT derive Debug (auth header redacted manually).
pub struct WpClient {
    http: reqwest::Client,
    base_url: String,
}

impl std::fmt::Debug for WpClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WpClient")
            .field("base_url", &self.base_url)
            .finish()
    }
}

#[derive(Deserialize)]
struct WpErrorResponse {
    code: String,
    message: String,
    #[serde(default)]
    data: Option<WpErrorData>,
}

#[derive(Deserialize)]
struct WpErrorData {
    status: Option<u16>,
}

impl WpClient {
    pub fn new(config: &Config) -> Result<Self, PoiesisError> {
        let credentials = format!("{}:{}", config.username, config.password);
        let encoded = base64::engine::general_purpose::STANDARD.encode(credentials.as_bytes());
        let auth_header = format!("Basic {}", encoded);

        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&auth_header)
                .map_err(|_| PoiesisError::Auth("invalid auth header".to_string()))?,
        );

        let http = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .map_err(PoiesisError::Http)?;

        Ok(WpClient {
            http,
            base_url: format!("{}/wp-json/wp/v2", config.url),
        })
    }

    // --- Posts ---

    pub async fn list_posts(
        &self,
        params: &ListParams,
    ) -> Result<(Vec<Post>, Option<u64>), PoiesisError> {
        self.list("/posts", params).await
    }

    pub async fn get_post(&self, id: u64) -> Result<Post, PoiesisError> {
        self.get("/posts", id).await
    }

    pub async fn create_post(&self, params: &CreateParams) -> Result<Post, PoiesisError> {
        self.create("/posts", params).await
    }

    pub async fn update_post(&self, id: u64, params: &UpdateParams) -> Result<Post, PoiesisError> {
        self.update("/posts", id, params).await
    }

    pub async fn delete_post(&self, id: u64) -> Result<(), PoiesisError> {
        self.delete("/posts", id).await
    }

    // --- Pages ---

    pub async fn list_pages(
        &self,
        params: &ListParams,
    ) -> Result<(Vec<Post>, Option<u64>), PoiesisError> {
        self.list("/pages", params).await
    }

    pub async fn get_page(&self, id: u64) -> Result<Post, PoiesisError> {
        self.get("/pages", id).await
    }

    pub async fn create_page(&self, params: &CreateParams) -> Result<Post, PoiesisError> {
        self.create("/pages", params).await
    }

    pub async fn update_page(&self, id: u64, params: &UpdateParams) -> Result<Post, PoiesisError> {
        self.update("/pages", id, params).await
    }

    pub async fn delete_page(&self, id: u64) -> Result<(), PoiesisError> {
        self.delete("/pages", id).await
    }

    // --- Internal implementation ---

    async fn list(
        &self,
        endpoint: &str,
        params: &ListParams,
    ) -> Result<(Vec<Post>, Option<u64>), PoiesisError> {
        let url = format!("{}{}", self.base_url, endpoint);
        let mut query: Vec<(&str, String)> = vec![("context", "edit".to_string())];

        if let Some(page) = params.page {
            query.push(("page", page.to_string()));
        }
        if let Some(per_page) = params.per_page {
            query.push(("per_page", per_page.to_string()));
        }
        if let Some(ref search) = params.search {
            query.push(("search", search.clone()));
        }
        if let Some(ref statuses) = params.status {
            let status_str: Vec<String> = statuses.iter().map(|s| s.to_string()).collect();
            query.push(("status", status_str.join(",")));
        }
        if let Some(ref orderby) = params.orderby {
            query.push(("orderby", orderby.clone()));
        }
        if let Some(ref order) = params.order {
            query.push(("order", order.clone()));
        }

        let resp = self.http.get(&url).query(&query).send().await?;
        let status = resp.status();

        // Extract total count from X-WP-Total header before consuming response
        let total = resp
            .headers()
            .get("X-WP-Total")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<u64>().ok());

        if status.is_success() {
            let posts: Vec<Post> = resp.json().await?;
            return Ok((posts, total));
        }

        let body = resp.text().await.unwrap_or_default();
        if let Ok(wp_err) = serde_json::from_str::<WpErrorResponse>(&body) {
            let http_status = wp_err
                .data
                .as_ref()
                .and_then(|d| d.status)
                .unwrap_or(status.as_u16());
            return Err(PoiesisError::WpApi {
                code: wp_err.code,
                message: wp_err.message,
                status: http_status,
            });
        }

        Err(PoiesisError::WpApi {
            code: "unknown".to_string(),
            message: format!("HTTP {}", status),
            status: status.as_u16(),
        })
    }

    async fn get(&self, endpoint: &str, id: u64) -> Result<Post, PoiesisError> {
        let url = format!("{}{}/{}", self.base_url, endpoint, id);
        let resp = self
            .http
            .get(&url)
            .query(&[("context", "edit")])
            .send()
            .await?;
        self.handle_response(resp).await
    }

    async fn create(&self, endpoint: &str, params: &CreateParams) -> Result<Post, PoiesisError> {
        let url = format!("{}{}", self.base_url, endpoint);
        let resp = self
            .http
            .post(&url)
            .header(CONTENT_TYPE, "application/json")
            .json(params)
            .send()
            .await?;
        self.handle_response(resp).await
    }

    async fn update(
        &self,
        endpoint: &str,
        id: u64,
        params: &UpdateParams,
    ) -> Result<Post, PoiesisError> {
        let url = format!("{}{}/{}", self.base_url, endpoint, id);
        let resp = self
            .http
            .post(&url)
            .header(CONTENT_TYPE, "application/json")
            .json(params)
            .send()
            .await?;
        self.handle_response(resp).await
    }

    async fn delete(&self, endpoint: &str, id: u64) -> Result<(), PoiesisError> {
        let url = format!("{}{}/{}", self.base_url, endpoint, id);
        let resp = self
            .http
            .delete(&url)
            .query(&[("force", "true")])
            .send()
            .await?;

        let status = resp.status();
        if status.is_success() {
            return Ok(());
        }

        let body = resp.text().await.unwrap_or_default();
        if let Ok(wp_err) = serde_json::from_str::<WpErrorResponse>(&body) {
            let http_status = wp_err
                .data
                .as_ref()
                .and_then(|d| d.status)
                .unwrap_or(status.as_u16());
            return Err(PoiesisError::WpApi {
                code: wp_err.code,
                message: wp_err.message,
                status: http_status,
            });
        }

        Err(PoiesisError::WpApi {
            code: "unknown".to_string(),
            message: format!("HTTP {}", status),
            status: status.as_u16(),
        })
    }

    async fn handle_response<T>(&self, resp: reqwest::Response) -> Result<T, PoiesisError>
    where
        T: serde::de::DeserializeOwned,
    {
        let status = resp.status();

        if status.is_success() {
            let value: T = resp.json().await?;
            return Ok(value);
        }

        let body = resp.text().await.unwrap_or_default();
        if let Ok(wp_err) = serde_json::from_str::<WpErrorResponse>(&body) {
            let http_status = wp_err
                .data
                .as_ref()
                .and_then(|d| d.status)
                .unwrap_or(status.as_u16());
            return Err(PoiesisError::WpApi {
                code: wp_err.code,
                message: wp_err.message,
                status: http_status,
            });
        }

        Err(PoiesisError::WpApi {
            code: "unknown".to_string(),
            message: format!("HTTP {}", status),
            status: status.as_u16(),
        })
    }

    /// Validate a post ID string — must be positive integer
    pub fn validate_post_id(id_str: &str) -> Result<u64, PoiesisError> {
        id_str
            .parse::<u64>()
            .ok()
            .filter(|&n| n > 0)
            .ok_or_else(|| PoiesisError::InvalidPostId(id_str.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::PostStatus;
    use wiremock::matchers::{header, method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn make_post_json(id: u64, title: &str) -> serde_json::Value {
        serde_json::json!({
            "id": id,
            "title": { "raw": title, "rendered": title },
            "content": { "raw": "<!-- wp:paragraph --><p>Test</p><!-- /wp:paragraph -->", "rendered": "<p>Test</p>", "block_version": 1 },
            "excerpt": { "raw": "", "rendered": "" },
            "slug": "test-post",
            "status": "publish",
            "type": "post",
            "date": "2026-01-01T00:00:00",
            "modified": "2026-01-01T00:00:00",
            "link": "https://example.org/test",
            "author": 1,
            "categories": [],
            "tags": []
        })
    }

    fn make_config(server: &MockServer) -> Config {
        Config {
            url: server.uri(),
            username: "neil".to_string(),
            password: "test-password".to_string(),
        }
    }

    #[tokio::test]
    async fn test_list_posts() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/wp-json/wp/v2/posts"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
                make_post_json(1, "Post One"),
                make_post_json(2, "Post Two"),
            ])))
            .mount(&server)
            .await;

        let client = WpClient::new(&make_config(&server)).unwrap();
        let (posts, _total) = client.list_posts(&ListParams::default()).await.unwrap();
        assert_eq!(posts.len(), 2);
        assert_eq!(posts[0].id, 1);
        assert_eq!(posts[1].title.raw, "Post Two");
    }

    #[tokio::test]
    async fn test_get_post_with_raw_content() {
        let server = MockServer::start().await;
        let post_json = make_post_json(42, "My Post");
        Mock::given(method("GET"))
            .and(path("/wp-json/wp/v2/posts/42"))
            .and(query_param("context", "edit"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&post_json))
            .mount(&server)
            .await;

        let client = WpClient::new(&make_config(&server)).unwrap();
        let post = client.get_post(42).await.unwrap();
        assert_eq!(post.id, 42);
        assert!(post.content.raw.contains("wp:paragraph"));
    }

    #[tokio::test]
    async fn test_create_post() {
        let server = MockServer::start().await;
        let response = make_post_json(100, "New Post");
        Mock::given(method("POST"))
            .and(path("/wp-json/wp/v2/posts"))
            .respond_with(ResponseTemplate::new(201).set_body_json(&response))
            .mount(&server)
            .await;

        let client = WpClient::new(&make_config(&server)).unwrap();
        let params = CreateParams {
            title: "New Post".to_string(),
            content: "<!-- wp:paragraph --><p>Hello</p><!-- /wp:paragraph -->".to_string(),
            status: Some(PostStatus::Draft),
            ..Default::default()
        };
        let post = client.create_post(&params).await.unwrap();
        assert_eq!(post.id, 100);
    }

    #[tokio::test]
    async fn test_update_post() {
        let server = MockServer::start().await;
        let response = make_post_json(42, "Updated Post");
        Mock::given(method("POST"))
            .and(path("/wp-json/wp/v2/posts/42"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&response))
            .mount(&server)
            .await;

        let client = WpClient::new(&make_config(&server)).unwrap();
        let params = UpdateParams {
            title: Some("Updated Post".to_string()),
            ..Default::default()
        };
        let post = client.update_post(42, &params).await.unwrap();
        assert_eq!(post.title.raw, "Updated Post");
    }

    #[tokio::test]
    async fn test_delete_post() {
        let server = MockServer::start().await;
        Mock::given(method("DELETE"))
            .and(path("/wp-json/wp/v2/posts/42"))
            .and(query_param("force", "true"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "deleted": true,
                "previous": make_post_json(42, "Deleted Post")
            })))
            .mount(&server)
            .await;

        let client = WpClient::new(&make_config(&server)).unwrap();
        // delete returns Ok(()) — we just check no error
        let result = client.delete_post(42).await;
        // The response is 200 but body may not parse as Post — need custom handling
        // Since delete() just checks status.is_success(), this should be Ok
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_list_pages() {
        let server = MockServer::start().await;
        let mut page_json = make_post_json(5, "About Page");
        page_json["type"] = serde_json::json!("page");
        Mock::given(method("GET"))
            .and(path("/wp-json/wp/v2/pages"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([page_json])))
            .mount(&server)
            .await;

        let client = WpClient::new(&make_config(&server)).unwrap();
        let (pages, _total) = client.list_pages(&ListParams::default()).await.unwrap();
        assert_eq!(pages.len(), 1);
        assert_eq!(pages[0].post_type, "page");
    }

    #[tokio::test]
    async fn test_auth_header_sent() {
        let server = MockServer::start().await;
        let credentials = base64::engine::general_purpose::STANDARD.encode("neil:test-password");
        let expected_auth = format!("Basic {}", credentials);

        Mock::given(method("GET"))
            .and(path("/wp-json/wp/v2/posts"))
            .and(header("authorization", expected_auth.as_str()))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([])))
            .mount(&server)
            .await;

        let client = WpClient::new(&make_config(&server)).unwrap();
        let result = client.list_posts(&ListParams::default()).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_wp_error_response() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/wp-json/wp/v2/posts/99"))
            .respond_with(ResponseTemplate::new(403).set_body_json(serde_json::json!({
                "code": "rest_forbidden",
                "message": "Sorry, you are not allowed to do that.",
                "data": { "status": 403 }
            })))
            .mount(&server)
            .await;

        let client = WpClient::new(&make_config(&server)).unwrap();
        let err = client.get_post(99).await.unwrap_err();
        assert!(matches!(err, PoiesisError::WpApi { code, .. } if code == "rest_forbidden"));
    }

    #[tokio::test]
    async fn test_404_post_not_found() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/wp-json/wp/v2/posts/9999"))
            .respond_with(ResponseTemplate::new(404).set_body_json(serde_json::json!({
                "code": "rest_post_invalid_id",
                "message": "Invalid post ID.",
                "data": { "status": 404 }
            })))
            .mount(&server)
            .await;

        let client = WpClient::new(&make_config(&server)).unwrap();
        let err = client.get_post(9999).await.unwrap_err();
        assert!(matches!(err, PoiesisError::WpApi { status, .. } if status == 404));
    }

    #[tokio::test]
    async fn test_list_with_params() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/wp-json/wp/v2/posts"))
            .and(query_param("search", "hello"))
            .and(query_param("per_page", "5"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([])))
            .mount(&server)
            .await;

        let client = WpClient::new(&make_config(&server)).unwrap();
        let params = ListParams {
            search: Some("hello".to_string()),
            per_page: Some(5),
            ..Default::default()
        };
        let result = client.list_posts(&params).await;
        assert!(result.is_ok());
        let (posts, _total) = result.unwrap();
        let _ = posts;
    }

    #[test]
    fn test_invalid_post_id_rejected() {
        let err = WpClient::validate_post_id("abc").unwrap_err();
        assert!(matches!(err, PoiesisError::InvalidPostId(_)));

        let err = WpClient::validate_post_id("0").unwrap_err();
        assert!(matches!(err, PoiesisError::InvalidPostId(_)));

        let ok = WpClient::validate_post_id("42").unwrap();
        assert_eq!(ok, 42);
    }
}
