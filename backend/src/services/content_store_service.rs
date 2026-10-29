use crate::models::content::{ContentCollection, MediaItem};
use bson::doc;
use mongodb::Database;

const CONTENT_COLLECTION: &str = "content_collections";

pub async fn find_by_kind_and_id(
    kind: &str,
    id: &str,
    db: &Database,
) -> Option<ContentCollection> {
    let coll = db.collection::<ContentCollection>(CONTENT_COLLECTION);
    coll.find_one(doc! {"kind": kind, "id": id})
        .await
        .ok()
        .flatten()
}

pub async fn append_media(
    kind: &str,
    id: &str,
    media: MediaItem,
    db: &Database,
) -> bool {
    let coll = db.collection::<ContentCollection>(CONTENT_COLLECTION);
    let media_bson = match bson::to_bson(&media) {
        Ok(b) => b,
        Err(_) => return false,
    };
    let update = doc! {
        "$push": { "media": media_bson },
    };
    coll.update_one(doc! {"kind": kind, "id": id}, update)
        .await
        .map(|r| r.modified_count > 0)
        .unwrap_or(false)
}

pub async fn set_cover_url_if_empty(kind: &str, id: &str, cover_url: &str, db: &Database) -> bool {
    let coll = db.collection::<ContentCollection>(CONTENT_COLLECTION);
    let update = doc! {
        "$set": { "coverUrl": cover_url },
    };
    coll.update_one(
        doc! {
            "kind": kind,
            "id": id,
            "$or": [
                { "coverUrl": { "$exists": false } },
                { "coverUrl": bson::Bson::Null },
                { "coverUrl": "" },
            ],
        },
        update,
    )
    .await
    .map(|r| r.modified_count > 0)
    .unwrap_or(false)
}

pub async fn list_by_kind(kind: &str, db: &Database) -> Vec<ContentCollection> {
    let coll = db.collection::<ContentCollection>(CONTENT_COLLECTION);
    let mut cursor = coll.find(doc! {"kind": kind}).await.ok();
    let mut items = vec![];
    if let Some(mut cursor) = cursor {
        while cursor.advance().await.unwrap_or(false) {
            if let Ok(item) = cursor.deserialize_current() {
                items.push(item);
            }
        }
    }
    items
}

pub async fn insert(collection: ContentCollection, db: &Database) -> bool {
    let coll = db.collection::<ContentCollection>(CONTENT_COLLECTION);
    coll.insert_one(collection).await.is_ok()
}
