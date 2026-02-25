//! LinkedIn API client.

use anyhow::{bail, Result};
use serde_json::json;

/// Maximum character count for a LinkedIn post.
const MAX_BODY_LENGTH: usize = 3000;

/// LinkedIn visibility values.
pub fn map_visibility(visibility: &str) -> Result<&'static str> {
    match visibility.to_lowercase().as_str() {
        "public" => Ok("PUBLIC"),
        "connections" => Ok("CONNECTIONS"),
        _ => bail!(
            "Invalid LinkedIn visibility '{}'. Valid: public, connections",
            visibility
        ),
    }
}

/// Get the authenticated user's URN via /v2/userinfo.
pub fn get_user_urn(access_token: &str) -> Result<String> {
    let resp = ureq::get("https://api.linkedin.com/v2/userinfo")
        .set("Authorization", &format!("Bearer {}", access_token))
        .call()
        .map_err(|e| anyhow::anyhow!("LinkedIn userinfo request failed: {}", e))?;

    let body: serde_json::Value = resp.into_json()?;
    let sub = body["sub"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Missing 'sub' in userinfo response"))?;

    Ok(format!("urn:li:person:{}", sub))
}

/// Create a post on LinkedIn.
pub fn create_post(
    access_token: &str,
    author_urn: &str,
    body: &str,
    visibility: &str,
) -> Result<(String, String)> {
    // Validate body length
    let char_count = body.chars().count();
    if char_count > MAX_BODY_LENGTH {
        bail!(
            "Post body exceeds LinkedIn's {} character limit ({} characters)",
            MAX_BODY_LENGTH,
            char_count
        );
    }

    let li_visibility = map_visibility(visibility)?;

    let payload = json!({
        "author": author_urn,
        "lifecycleState": "PUBLISHED",
        "specificContent": {
            "com.linkedin.ugc.ShareContent": {
                "shareCommentary": {
                    "text": body
                },
                "shareMediaCategory": "NONE"
            }
        },
        "visibility": {
            "com.linkedin.ugc.MemberNetworkVisibility": li_visibility
        }
    });

    let resp = ureq::post("https://api.linkedin.com/rest/posts")
        .set("Authorization", &format!("Bearer {}", access_token))
        .set("LinkedIn-Version", "202401")
        .set("X-Restli-Protocol-Version", "2.0.0")
        .send_json(&payload);

    match resp {
        Ok(r) => {
            // LinkedIn returns the post ID in the x-restli-id header
            let post_id = r
                .header("x-restli-id")
                .unwrap_or("unknown")
                .to_string();
            let post_url = format!(
                "https://www.linkedin.com/feed/update/{}",
                post_id
            );
            Ok((post_id, post_url))
        }
        Err(ureq::Error::Status(status, resp)) => {
            let body = resp.into_string().unwrap_or_default();
            bail!(
                "LinkedIn API error (HTTP {}): {}",
                status,
                body
            );
        }
        Err(e) => bail!("LinkedIn API request failed: {}", e),
    }
}
