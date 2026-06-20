use crate::models::content::ContentCollection;
use bson::doc;
use mongodb::Database;

const CONTENT_COLLECTION: &str = "content_collections";

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
