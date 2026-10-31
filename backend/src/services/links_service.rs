use bson::doc;
use mongodb::Database;
use serde::{Deserialize, Serialize};

const COLLECTION: &str = "site_links";
const DOC_ID: &str = "social";
const MAX_LINKS: usize = 12;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SocialLink {
    pub label: String,
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct LinksDoc {
    #[serde(rename = "_id")]
    id: String,
    links: Vec<SocialLink>,
}

pub fn clean_links(links: &[SocialLink]) -> Result<Vec<SocialLink>, &'static str> {
    if links.len() > MAX_LINKS {
        return Err("TOO_MANY");
    }
    let mut out = Vec::with_capacity(links.len());
    for link in links {
        let label = link.label.trim();
        let url = link.url.trim();
        if label.is_empty() || label.chars().count() > 40 {
            return Err("BAD_LABEL");
        }
        if !allowed_url(url) {
            return Err("BAD_URL");
        }
        out.push(SocialLink {
            label: label.to_string(),
            url: url.to_string(),
        });
    }
    Ok(out)
}

fn allowed_url(url: &str) -> bool {
    if url.chars().count() > 300 || url.contains(char::is_whitespace) {
        return false;
    }
    let Some(rest) = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))
    else {
        return false;
    };
    if rest.len() < 3 {
        return false;
    }
    // ponytail: @ in the host is userinfo. @ later is a normal social path.
    let host = rest.split('/').next().unwrap_or(rest);
    !host.contains('@')
}

pub async fn list(db: &Database) -> Vec<SocialLink> {
    let coll = db.collection::<LinksDoc>(COLLECTION);
    coll.find_one(doc! { "_id": DOC_ID })
        .await
        .ok()
        .flatten()
        .map(|doc| doc.links)
        .unwrap_or_default()
}

pub async fn replace(db: &Database, links: Vec<SocialLink>) -> Result<Vec<SocialLink>, &'static str> {
    let links = clean_links(&links)?;
    let coll = db.collection::<LinksDoc>(COLLECTION);
    let doc = LinksDoc {
        id: DOC_ID.to_string(),
        links: links.clone(),
    };
    coll.replace_one(doc! { "_id": DOC_ID }, &doc)
        .upsert(true)
        .await
        .map_err(|_| "SAVE_FAILED")?;
    Ok(links)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clean_links_keeps_http_urls_and_rejects_the_rest() {
        let ok = clean_links(&[SocialLink {
            label: " Instagram ".into(),
            url: "https://instagram.com/jheffmedia".into(),
        }])
        .unwrap();
        assert_eq!(ok[0].label, "Instagram");
        assert!(clean_links(&[SocialLink {
            label: "YouTube".into(),
            url: "https://www.youtube.com/@jheffmedia_".into(),
        }])
        .is_ok());
        assert!(clean_links(&[SocialLink {
            label: "X".into(),
            url: "javascript:alert(1)".into(),
        }])
        .is_err());
        assert!(clean_links(&[SocialLink {
            label: "X".into(),
            url: "https://user@evil.example/phish".into(),
        }])
        .is_err());
        let many = vec![
            SocialLink {
                label: "A".into(),
                url: "https://example.com".into(),
            };
            13
        ];
        assert_eq!(clean_links(&many), Err("TOO_MANY"));
    }
}
