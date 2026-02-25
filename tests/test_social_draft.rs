//! Draft parsing edge cases (D1–D9).

mod common;

use corky::social::draft::{DraftStatus, SocialDraft, SocialDraftMeta};
use corky::social::platform::Platform;

// D1: Valid draft with all fields
#[test]
fn d1_valid_draft_all_fields() {
    let content = r#"---
platform: linkedin
author: btakita
visibility: public
status: ready
tags:
  - rust
  - ai
---
Hello LinkedIn! This is my post.
"#;

    let draft = SocialDraft::parse(content).unwrap();
    assert_eq!(draft.meta.platform, Platform::LinkedIn);
    assert_eq!(draft.meta.author, "btakita");
    assert_eq!(draft.meta.visibility, "public");
    assert_eq!(draft.meta.status, DraftStatus::Ready);
    assert_eq!(draft.meta.tags, vec!["rust", "ai"]);
    assert!(draft.body.contains("Hello LinkedIn!"));
}

// D2: Missing required field (platform)
#[test]
fn d2_missing_platform() {
    let content = r#"---
author: btakita
---
Some body
"#;

    let result = SocialDraft::parse(content);
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("platform") || err.contains("missing field"),
        "Expected platform missing error, got: {}",
        err
    );
}

// D3: Missing required field (author)
#[test]
fn d3_missing_author() {
    let content = r#"---
platform: linkedin
---
Some body
"#;

    let result = SocialDraft::parse(content);
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("author") || err.contains("missing field"),
        "Expected author missing error, got: {}",
        err
    );
}

// D4: Unknown platform
#[test]
fn d4_unknown_platform() {
    let content = r#"---
platform: myspace
author: btakita
---
Some body
"#;

    let result = SocialDraft::parse(content);
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("unknown variant") || err.contains("myspace"),
        "Expected unknown platform error, got: {}",
        err
    );
}

// D5: Invalid status value
#[test]
fn d5_invalid_status() {
    let content = r#"---
platform: linkedin
author: btakita
status: pending
---
Some body
"#;

    let result = SocialDraft::parse(content);
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("unknown variant") || err.contains("pending"),
        "Expected invalid status error, got: {}",
        err
    );
}

// D6: No YAML frontmatter delimiters
#[test]
fn d6_no_frontmatter() {
    let content = "Just plain text without frontmatter.";

    let result = SocialDraft::parse(content);
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("---"),
        "Expected missing delimiter error, got: {}",
        err
    );
}

// D7: Empty body after frontmatter
#[test]
fn d7_empty_body() {
    let content = r#"---
platform: linkedin
author: btakita
---
"#;

    let draft = SocialDraft::parse(content).unwrap();
    assert!(draft.body.is_empty() || draft.body.trim().is_empty());
}

// D8: Render/parse round-trip
#[test]
fn d8_render_parse_roundtrip() {
    let meta = SocialDraftMeta {
        platform: Platform::LinkedIn,
        author: "btakita".to_string(),
        visibility: "public".to_string(),
        status: DraftStatus::Ready,
        tags: vec!["rust".to_string(), "ai".to_string()],
        scheduled_at: None,
        published_at: None,
        post_id: None,
        post_url: None,
    };

    let original = SocialDraft::new(meta, "Test body content.\n".to_string());
    let rendered = original.render().unwrap();
    let parsed = SocialDraft::parse(&rendered).unwrap();

    assert_eq!(parsed.meta.platform, original.meta.platform);
    assert_eq!(parsed.meta.author, original.meta.author);
    assert_eq!(parsed.meta.visibility, original.meta.visibility);
    assert_eq!(parsed.meta.status, original.meta.status);
    assert_eq!(parsed.meta.tags, original.meta.tags);
    assert!(parsed.body.contains("Test body content."));
}

// D9: Malformed YAML
#[test]
fn d9_malformed_yaml() {
    let content = r#"---
platform: linkedin
author: [invalid yaml here
---
Some body
"#;

    let result = SocialDraft::parse(content);
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(!err.is_empty(), "Should have parse error details");
}

// Additional: BOM handling
#[test]
fn parse_with_bom() {
    let content = "\u{feff}---\nplatform: linkedin\nauthor: btakita\n---\nBody\n";
    let draft = SocialDraft::parse(content).unwrap();
    assert_eq!(draft.meta.platform, Platform::LinkedIn);
}

// Additional: Default status is draft
#[test]
fn default_status_is_draft() {
    let content = r#"---
platform: linkedin
author: btakita
---
Body
"#;

    let draft = SocialDraft::parse(content).unwrap();
    assert_eq!(draft.meta.status, DraftStatus::Draft);
}

// Additional: Default visibility is public
#[test]
fn default_visibility_is_public() {
    let content = r#"---
platform: linkedin
author: btakita
---
Body
"#;

    let draft = SocialDraft::parse(content).unwrap();
    assert_eq!(draft.meta.visibility, "public");
}
