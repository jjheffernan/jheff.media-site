use crate::{
    error::{ServiceError, ServiceResult},
    models::media::{
        PrintCheckoutInput, PrintOrder, PrintOrderItem, PrintOrderView, PrintSelection,
        PrintSelectionInput,
    },
    utils::auth::AuthUser,
};
use actix_web::http::StatusCode;
use bson::{doc, oid::ObjectId, DateTime};
use mongodb::Database;

const PRINTS_COLLECTION: &str = "print_selections";
const ORDERS_COLLECTION: &str = "print_orders";

pub async fn list_for_user(auth: &AuthUser, db: &Database) -> Vec<PrintSelection> {
    let coll = db.collection::<PrintSelection>(PRINTS_COLLECTION);
    let mut items = vec![];
    if let Ok(mut cursor) = coll.find(doc! {"userId": auth.id}).await {
        while cursor.advance().await.unwrap_or(false) {
            if let Ok(item) = cursor.deserialize_current() {
                items.push(item);
            }
        }
    }
    items
}

pub async fn add(
    auth: &AuthUser,
    input: PrintSelectionInput,
    db: &Database,
) -> ServiceResult<PrintSelection> {
    let record = PrintSelection {
        id: None,
        user_id: auth.id,
        item_id: input.item_id,
        source: input.source,
        title: input.title,
        thumbnail_url: input.thumbnail_url,
        media_url: input.media_url,
        collection_id: input.collection_id,
        collection_kind: input.collection_kind,
        collection_title: input.collection_title,
        created_at: DateTime::now(),
    };

    let coll = db.collection::<PrintSelection>(PRINTS_COLLECTION);
    let result = coll
        .insert_one(&record)
        .await
        .map_err(|e| ServiceError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(PrintSelection {
        id: result.inserted_id.as_object_id(),
        ..record
    })
}

pub async fn remove(auth: &AuthUser, selection_id: &str, db: &Database) -> ServiceResult<()> {
    let oid = ObjectId::parse_str(selection_id)
        .map_err(|_| ServiceError::new(StatusCode::BAD_REQUEST, "INVALID_ID"))?;

    let coll = db.collection::<PrintSelection>(PRINTS_COLLECTION);
    coll.delete_one(doc! {"_id": oid, "userId": auth.id})
        .await
        .map_err(|e| ServiceError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(())
}

fn order_item(selection: &PrintSelection) -> PrintOrderItem {
    PrintOrderItem {
        item_id: selection.item_id.clone(),
        source: selection.source.clone(),
        title: selection.title.clone(),
        thumbnail_url: selection.thumbnail_url.clone(),
        media_url: selection.media_url.clone(),
        collection_id: selection.collection_id.clone(),
        collection_kind: selection.collection_kind.clone(),
        collection_title: selection.collection_title.clone(),
    }
}

fn unit_price_cents(size: &str) -> Option<i64> {
    match size {
        "8x10" => Some(2500),
        "11x14" => Some(4500),
        "16x20" => Some(7500),
        _ => None,
    }
}

fn order_view(order: PrintOrder) -> PrintOrderView {
    PrintOrderView {
        id: order.id.map(|oid| oid.to_hex()).unwrap_or_default(),
        status: order.status,
        note: order.note,
        item_count: order.items.len(),
        created_at: order
            .created_at
            .try_to_rfc3339_string()
            .unwrap_or_else(|_| order.created_at.timestamp_millis().to_string()),
        size: order.size,
        unit_price_cents: order.unit_price_cents,
        total_cents: order.total_cents,
        payment_status: order.payment_status,
    }
}

pub async fn list_orders(auth: &AuthUser, db: &Database) -> Vec<PrintOrderView> {
    let coll = db.collection::<PrintOrder>(ORDERS_COLLECTION);
    let mut orders = vec![];
    if let Ok(mut cursor) = coll.find(doc! {"userId": auth.id}).await {
        while cursor.advance().await.unwrap_or(false) {
            if let Ok(order) = cursor.deserialize_current() {
                orders.push(order);
            }
        }
    }
    orders.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    orders.into_iter().map(order_view).collect()
}

pub async fn checkout(
    auth: &AuthUser,
    input: PrintCheckoutInput,
    db: &Database,
) -> ServiceResult<PrintOrderView> {
    let size = input.size.trim();
    let Some(unit_price_cents) = unit_price_cents(size) else {
        return Err(ServiceError::new(StatusCode::BAD_REQUEST, "UNKNOWN_SIZE"));
    };

    let selections = list_for_user(auth, db).await;
    if selections.is_empty() {
        return Err(ServiceError::new(StatusCode::BAD_REQUEST, "NO_SELECTIONS"));
    }

    let note = input
        .note
        .map(|note| note.trim().to_string())
        .filter(|note| !note.is_empty())
        .map(|note| note.chars().take(500).collect());

    let record = PrintOrder {
        id: None,
        user_id: auth.id,
        status: "requested".into(),
        note,
        size: size.to_string(),
        unit_price_cents,
        total_cents: unit_price_cents * selections.len() as i64,
        payment_status: "unpaid".into(),
        items: selections.iter().map(order_item).collect(),
        created_at: DateTime::now(),
    };

    let coll = db.collection::<PrintOrder>(ORDERS_COLLECTION);
    let result = coll
        .insert_one(&record)
        .await
        .map_err(|e| ServiceError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(order_view(PrintOrder {
        id: result.inserted_id.as_object_id(),
        ..record
    }))
}

#[cfg(test)]
mod tests {
    use super::unit_price_cents;
    use crate::models::media::PrintOrder;
    use bson::{doc, oid::ObjectId, DateTime};

    #[test]
    fn fixed_sizes_are_integer_cents() {
        assert_eq!(unit_price_cents("8x10"), Some(2500));
        assert_eq!(unit_price_cents("11x14"), Some(4500));
        assert_eq!(unit_price_cents("16x20"), Some(7500));
        assert_eq!(unit_price_cents("4x6"), None);
        assert_eq!(unit_price_cents("8x10").unwrap() * 2, 5000);
    }

    #[test]
    fn old_orders_default_missing_price_fields() {
        let order: PrintOrder = bson::from_document(doc! {
            "userId": ObjectId::new(),
            "status": "requested",
            "items": [],
            "createdAt": DateTime::now(),
        })
        .unwrap();
        assert_eq!(order.size, "");
        assert_eq!(order.unit_price_cents, 0);
        assert_eq!(order.total_cents, 0);
        assert_eq!(order.payment_status, "unpaid");
    }
}
