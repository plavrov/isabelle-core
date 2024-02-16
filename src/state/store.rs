use async_trait::async_trait;
use isabelle_dm::data_model::item::Item;
use isabelle_dm::data_model::list_result::ListResult;
use std::collections::HashMap;

#[async_trait]
pub trait Store {
    async fn connect(&mut self, addr: &str, altaddr: &str);
    async fn disconnect(&mut self);

    async fn get_collections(&mut self) -> Vec<String>;
    async fn get_item_ids(&mut self, collection: &str) -> HashMap<u64, bool>;

    async fn get_all_items(&mut self, collection: &str, sort_key: &str, filter: &str)
        -> ListResult;
    async fn get_item(&mut self, collection: &str, id: u64) -> Option<Item>;
    async fn get_items(
        &mut self,
        collection: &str,
        id_min: u64,
        id_max: u64,
        sort_key: &str,
        filter: &str,
        skip: u64,
        limit: u64,
    ) -> ListResult;

    async fn set_item(&mut self, collection: &str, itm: &Item, merge: bool);
    async fn del_item(&mut self, collection: &str, id: u64) -> bool;

    async fn get_credentials(&mut self) -> String;
    async fn get_pickle(&mut self) -> String;

    async fn get_internals(&mut self) -> Item;

    async fn get_settings(&mut self) -> Item;
    async fn set_settings(&mut self, itm: Item);
}
