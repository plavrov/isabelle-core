use futures_util::TryStreamExt;
use isabelle_dm::data_model::list_result::ListResult;
extern crate serde_json;

use crate::state::store::Store;
use async_trait::async_trait;
use isabelle_dm::data_model::item::*;
use log::info;

use mongodb::{
    bson::doc, options::CreateCollectionOptions, options::FindOptions, Client, Collection,
    IndexModel,
};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct StoreMongo {
    pub path: String,
    pub local_path: String,
    pub collections: HashMap<String, u64>,
    pub items: HashMap<u64, HashMap<u64, bool>>,
    pub items_count: HashMap<u64, u64>,

    pub client: Option<mongodb::Client>,
}

unsafe impl Send for StoreMongo {}

impl StoreMongo {
    pub fn new() -> Self {
        Self {
            path: "".to_string(),
            local_path: "".to_string(),
            collections: HashMap::new(),
            items: HashMap::new(),
            items_count: HashMap::new(),
            client: None,
        }
    }

    pub async fn do_conn(&mut self) -> bool {
        if self.client.is_none() {
            let client = Client::with_uri_str(&self.path).await;
            match client {
                Ok(cl) => {
                    self.client = Some(cl);
                }
                Err(_err) => {
                    self.client = None;
                    return false;
                }
            };
        }

        return true;
    }
}

#[async_trait]
impl Store for StoreMongo {
    async fn connect(&mut self, url: &str, alturl: &str) {
        self.path = url.to_string();
        self.local_path = alturl.to_string();
        let res = self.do_conn().await;
        if res {
            info!("Connected!");
            let internals = self.get_internals().await;
            let collections = internals.safe_strstr("collections", &HashMap::new());
            info!("Collections: {}", collections.len());
            let db = self.client.as_ref().unwrap().database("isabelle");
            for coll_name in collections {
                info!("Create collection {}", &coll_name.1);
                db.create_collection(&coll_name.1, CreateCollectionOptions::default())
                    .await
                    .unwrap();
                let coll: Collection<Item> = db.collection(&coll_name.1);
                let index: IndexModel = IndexModel::builder().keys(doc! { "id": 1 }).build();
                let _result = coll.create_index(index, None).await;

                let coll_idx = self.collections.len().try_into().unwrap();
                self.collections.insert(coll_name.1.to_string(), coll_idx);

                let mut map: HashMap<u64, bool> = HashMap::new();
                let filter = doc! {}; // An empty filter matches all documents
                let options = FindOptions::default();

                // Find documents in the collection
                let mut cursor = coll.find(filter, options).await.unwrap();
                let mut count = 0;
                while let Some(doc) = cursor.try_next().await.unwrap() {
                    map.insert(doc.id, true);
                    count = std::cmp::max(count, doc.id);
                }

                self.items.insert(coll_idx, map);
                self.items_count.insert(coll_idx, count);
            }
        } else {
            info!("Not connected");
        }
    }

    async fn disconnect(&mut self) {}

    async fn get_collections(&mut self) -> Vec<String> {
        let colls = self
            .client
            .as_ref()
            .unwrap()
            .database("isabelle")
            .list_collection_names(None)
            .await
            .unwrap();
        let mut lst: Vec<String> = Vec::new();

        for coll in &colls {
            lst.push(coll.clone());
        }

        return lst;
    }

    async fn get_item_ids(&mut self, collection: &str) -> HashMap<u64, bool> {
        if !self.collections.contains_key(collection) {
            return HashMap::new();
        }

        let coll_id = self.collections[collection];
        return self.items[&coll_id].clone();
    }

    async fn get_all_items(&mut self, collection: &str) -> ListResult {
        return self
            .get_items(collection, u64::MAX, u64::MAX, u64::MAX, u64::MAX)
            .await;
    }

    async fn get_item(&mut self, collection: &str, id: u64) -> Option<Item> {
        let coll = self
            .client
            .as_ref()
            .unwrap()
            .database("isabelle")
            .collection(collection);
        let filter = doc! {
            "id": id as i64,
        };

        let result = coll.find_one(filter, None).await;

        match result {
            Ok(r) => {
                if r.is_none() {
                    return None;
                }
                return Some(r.unwrap());
            }
            Err(_e) => {}
        };
        return None;
    }

    async fn get_items(
        &mut self,
        collection: &str,
        id_min: u64,
        id_max: u64,
        skip: u64,
        limit: u64,
    ) -> ListResult {
        let mut lr = ListResult {
            map: HashMap::new(),
            total_count: 0,
        };
        let itms = self
            .items
            .get_mut(&self.collections[collection])
            .unwrap()
            .clone();
        let mut eff_id_min = id_min;
        let eff_id_max = id_max;
        let mut count = 0;
        let mut eff_skip = skip;

        if eff_skip == u64::MAX {
            eff_skip = 0;
        }

        if eff_id_min == u64::MAX {
            eff_id_min = 0;
        }

        info!(
            "Getting {} in range {} - {} limit {}",
            &collection, eff_id_min, eff_id_max, limit
        );
        for itm in &itms {
            if itm.0 >= &eff_id_min && itm.0 <= &eff_id_max {
                let new_item = self.get_item(collection, *itm.0).await;
                if !new_item.is_none() {
                    if count >= eff_skip {
                        lr.map.insert(*itm.0, new_item.unwrap());
                    }
                    count = count + 1;
                    if count >= eff_skip && (count - eff_skip) >= limit {
                        break;
                    }
                }
            }
        }

        lr.total_count = itms.len() as u64;

        info!(" - result: {} items", count);
        return lr;
    }

    async fn set_item(&mut self, collection: &str, exp_itm: &Item, merge: bool) {
        let mut itm = exp_itm.clone();
        if itm.id == u64::MAX {
            let coll_id = self.collections[collection];
            if self.items.contains_key(&coll_id) {
                itm.id = self.items_count[&coll_id] + 1;
            }
        }

        let old_itm = if itm.id != u64::MAX {
            self.get_item(collection, itm.id).await
        } else {
            None
        };
        let mut new_itm = itm.clone();
        if !old_itm.is_none() && merge {
            new_itm = old_itm.as_ref().unwrap().clone();
            new_itm.merge(&itm);
        }

        let coll: Collection<Item> = self
            .client
            .as_ref()
            .unwrap()
            .database("isabelle")
            .collection(collection);
        let filter = doc! {
            "id": itm.id as i64,
        };

        if old_itm.as_ref().is_none() {
            let _res = coll.insert_one(itm.clone(), None).await;
        } else {
            let _res = coll.replace_one(filter, itm.clone(), None).await;
        }

        let coll_id = self.collections[collection];
        if self.items.contains_key(&coll_id) {
            let coll = self.items.get_mut(&coll_id).unwrap();
            if coll.contains_key(&new_itm.id) {
                *(coll.get_mut(&new_itm.id).unwrap()) = true;
            } else {
                coll.insert(new_itm.id, true);
            }
            if self.items_count.contains_key(&coll_id) {
                let cnt = self.items_count.get_mut(&coll_id).unwrap();
                if new_itm.id > *cnt {
                    *cnt = new_itm.id;
                }
            } else {
                self.items_count.insert(coll_id, new_itm.id + 1);
            }
        }
    }

    async fn del_item(&mut self, collection: &str, id: u64) -> bool {
        let coll: Collection<Item> = self
            .client
            .as_ref()
            .unwrap()
            .database("isabelle")
            .collection(collection);
        let filter = doc! {
            "id": id as i64,
        };

        let _res = coll.delete_one(filter, None).await;

        let coll_id = self.collections[collection];
        if self.items.contains_key(&coll_id) {
            let coll = self.items.get_mut(&coll_id).unwrap();
            if coll.contains_key(&id) {
                coll.remove(&id);
                return true;
            }
        }
        return false;
    }

    async fn get_credentials(&mut self) -> String {
        return self.local_path.clone() + "/credentials.json";
    }

    async fn get_pickle(&mut self) -> String {
        return self.local_path.clone() + "/token.pickle";
    }

    async fn get_internals(&mut self) -> Item {
        let tmp_data_path = self.local_path.clone() + "/internals.js";

        let read_data = std::fs::read_to_string(tmp_data_path);
        if let Err(_e) = read_data {
            return Item::new();
        }
        let text = read_data.unwrap();
        let itm: Item = serde_json::from_str(&text).unwrap();
        return itm;
    }

    async fn get_settings(&mut self) -> Item {
        let tmp_data_path = self.local_path.clone() + "/settings.js";

        let read_data = std::fs::read_to_string(tmp_data_path);
        if let Err(_e) = read_data {
            return Item::new();
        }
        let text = read_data.unwrap();
        let itm: Item = serde_json::from_str(&text).unwrap();
        return itm;
    }

    async fn set_settings(&mut self, itm: Item) {
        let tmp_data_path = self.local_path.clone() + "/settings.js";
        let s = serde_json::to_string(&itm);
        std::fs::write(tmp_data_path, s.unwrap()).expect("Couldn't write item");
    }
}
