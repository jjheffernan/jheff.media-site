use crate::models::content::{ContentCollection, MediaItem, UpdateContentCollection};
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

pub async fn set_cover_url(kind: &str, id: &str, cover_url: &str, db: &Database) -> bool {
    let coll = db.collection::<ContentCollection>(CONTENT_COLLECTION);
    coll.update_one(
        doc! { "kind": kind, "id": id },
        doc! { "$set": { "coverUrl": cover_url } },
    )
    .await
    .map(|r| r.matched_count > 0)
    .unwrap_or(false)
}

pub async fn update_metadata(
    kind: &str,
    id: &str,
    patch: UpdateContentCollection,
    db: &Database,
) -> bool {
    let mut set = doc! {};
    if let Some(title) = patch.title {
        set.insert("title", title);
    }
    if let Some(summary) = patch.summary {
        set.insert("summary", summary);
    }
    if let Some(status) = patch.status {
        set.insert("status", status);
    }
    if let Some(date) = patch.date {
        set.insert("date", date);
    }
    if let Some(location) = patch.location {
        set.insert("location", location);
    }
    if set.is_empty() {
        return find_by_kind_and_id(kind, id, db).await.is_some();
    }
    let coll = db.collection::<ContentCollection>(CONTENT_COLLECTION);
    coll.update_one(doc! {"kind": kind, "id": id}, doc! { "$set": set })
        .await
        .map(|r| r.matched_count > 0)
        .unwrap_or(false)
}

pub async fn delete_collection(kind: &str, id: &str, db: &Database) -> Option<ContentCollection> {
    let coll = db.collection::<ContentCollection>(CONTENT_COLLECTION);
    coll.find_one_and_delete(doc! {"kind": kind, "id": id})
        .await
        .ok()
        .flatten()
}

pub async fn remove_media(
    kind: &str,
    id: &str,
    asset_id: &str,
    db: &Database,
) -> Option<MediaItem> {
    let Some(mut collection) = find_by_kind_and_id(kind, id, db).await else {
        return None;
    };
    let idx = collection.media.iter().position(|m| m.id == asset_id)?;
    let removed = collection.media.remove(idx);
    let cover_cleared = collection
        .cover_url
        .as_ref()
        .map(|u| u.contains(asset_id))
        .unwrap_or(false);
    if cover_cleared {
        collection.cover_url = None;
    }
    let coll = db.collection::<ContentCollection>(CONTENT_COLLECTION);
    let ok = coll
        .replace_one(doc! {"kind": kind, "id": id}, &collection)
        .await
        .map(|r| r.matched_count > 0)
        .unwrap_or(false);
    if ok {
        Some(removed)
    } else {
        None
    }
}

pub async fn reorder_media(
    kind: &str,
    id: &str,
    media_ids: &[String],
    db: &Database,
) -> bool {
    let Some(mut collection) = find_by_kind_and_id(kind, id, db).await else {
        return false;
    };
    // ponytail: ordered ids first; leftovers keep relative order at end
    let mut remaining = collection.media;
    let mut ordered = Vec::with_capacity(remaining.len());
    for mid in media_ids {
        if let Some(pos) = remaining.iter().position(|m| &m.id == mid) {
            ordered.push(remaining.remove(pos));
        }
    }
    ordered.append(&mut remaining);
    collection.media = ordered;
    let coll = db.collection::<ContentCollection>(CONTENT_COLLECTION);
    coll.replace_one(doc! {"kind": kind, "id": id}, &collection)
        .await
        .map(|r| r.matched_count > 0)
        .unwrap_or(false)
}

pub async fn list_by_kind(kind: &str, db: &Database) -> Vec<ContentCollection> {
    let coll = db.collection::<ContentCollection>(CONTENT_COLLECTION);
    let cursor = coll.find(doc! {"kind": kind}).await.ok();
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
