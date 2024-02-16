use crate::state::store::*;
use log::info;

pub async fn merge_database(st1: &mut dyn Store, st2: &mut dyn Store) {
    let collections = st1.get_collections().await;
    for collection in &collections {
        info!("Merge collection: {}", &collection);
        let items = st1.get_all_items(collection, "id", "").await;
        for item in &items.map {
            info!("Setting {} item {}", &collection, &item.0);
            st2.set_item(&collection, &item.1, false).await;
        }
    }
}
