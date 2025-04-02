use activitypub_federation::kinds::collection::OrderedCollectionType;
use serde::{Deserialize, Serialize};

/// Empty placeholder outbox used for Person, Instance, which dont implement a proper outbox yet.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct EmptyOutbox {
    r#type: OrderedCollectionType,
    id: String,
    ordered_items: Vec<()>,
    total_items: i32,
}

impl EmptyOutbox {
    pub(crate) fn new(id: String) -> EmptyOutbox {
        EmptyOutbox {
            r#type: OrderedCollectionType::OrderedCollection,
            id,
            ordered_items: vec![],
            total_items: 0,
        }
    }
}
