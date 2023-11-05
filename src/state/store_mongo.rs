extern crate serde_json;
use std::path::Path;

use crate::state::store::Store;
use isabelle_dm::data_model::item::*;
use log::{error, info};
use std::collections::HashMap;
use std::fs;
use mongodb::{ bson::doc, Client, Collection };
use mongodb::bson::Document;
use async_trait::async_trait;

#[derive(Debug, Clone)]
pub struct StoreMongo {
    pub path: String,
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
    async fn connect(&mut self, url: &str) {
        self.path = url.to_string();
        let res = self.do_conn().await;
        if res {
            info!("Connected");
        } else {
            info!("Not connected");
        }
    }

    async fn disconnect(&mut self) {}

    async fn get_collections(&mut self) -> Vec<String> {
        let my_coll: Collection<Item> =
            self.client.as_ref().unwrap()
            .database("user")
            .collection("restaurants");
        let _result = my_coll.find_one(
            doc! { "name": "Tompkins Square Bagels" },
            None
        ).await;
        let mut lst: Vec<String> = Vec::new();

        for coll in &self.collections {
            lst.push(coll.0.clone());
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

    async fn get_all_items(&mut self, collection: &str) -> HashMap<u64, Item> {
        return self.get_items(collection, u64::MAX, u64::MAX, u64::MAX).await;
    }

    async fn get_item(&mut self, collection: &str, id: u64) -> Option<Item> {
        let tmp_path = self.path.to_string()
            + "/collection/"
            + collection
            + "/"
            + &id.to_string()
            + "/data.js";
        if Path::new(&tmp_path).is_file() {
            let text = std::fs::read_to_string(tmp_path).unwrap();
            let itm: Item = serde_json::from_str(&text).unwrap();
            return Some(itm);
        }
        return None;
    }

    async fn get_items(
        &mut self,
        collection: &str,
        id_min: u64,
        id_max: u64,
        limit: u64,
    ) -> HashMap<u64, Item> {
        let mut map: HashMap<u64, Item> = HashMap::new();
        let itms = self
            .items
            .get_mut(&self.collections[collection])
            .unwrap()
            .clone();
        let mut eff_id_min = id_min;
        let eff_id_max = id_max;
        let mut count = 0;

        if eff_id_min == u64::MAX {
            eff_id_min = 0;
        }

        info!(
            "Getting {} in range {} - {} limit {}",
            &collection, eff_id_min, eff_id_max, limit
        );
        for itm in itms {
            if itm.0 >= eff_id_min && itm.0 <= eff_id_max {
                let new_item = self.get_item(collection, itm.0).await;
                if !new_item.is_none() {
                    map.insert(itm.0, new_item.unwrap());
                    count = count + 1;
                    if count >= limit {
                        break;
                    }
                }
            }
        }
        info!(" - result: {} items", count);

        return map;
    }

    async fn set_item(&mut self, collection: &str, itm: &Item, merge: bool) {
        let old_itm = self.get_item(collection, itm.id).await;
        let mut new_itm = itm.clone();
        if !old_itm.is_none() && merge {
            new_itm = old_itm.unwrap().clone();
            new_itm.merge(itm);
        }
        let tmp_path =
            self.path.to_string() + "/collection/" + collection + "/" + &new_itm.id.to_string();

        let _dir_create_err = std::fs::create_dir(&tmp_path);

        let tmp_data_path = tmp_path.clone() + "/data.js";
        let s = serde_json::to_string(&new_itm);
        std::fs::write(tmp_data_path, s.unwrap()).expect("Couldn't write item");

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
                if new_itm.id >= *cnt {
                    *cnt = new_itm.id + 1;
                    let _res = std::fs::write(
                        self.path.to_string() + "/collection/" + collection + "/cnt",
                        (new_itm.id + 1).to_string(),
                    );
                }
            } else {
                self.items_count.insert(coll_id, new_itm.id + 1);
                let _res = std::fs::write(
                    self.path.to_string() + "/collection/" + collection + "/cnt",
                    (new_itm.id + 1).to_string(),
                );
            }
        }
    }

    async fn del_item(&mut self, collection: &str, id: u64) -> bool {
        let tmp_path = self.path.to_string() + "/" + collection + "/" + &id.to_string();
        let path = Path::new(&tmp_path);
        if path.exists() {
            let _res = std::fs::remove_dir_all(tmp_path);
        }
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
        return self.path.clone() + "/credentials.json";
    }

    async fn get_pickle(&mut self) -> String {
        return self.path.clone() + "/token.pickle";
    }

    async fn get_internals(&mut self) -> Item {
        let tmp_data_path = self.path.clone() + "/internals.js";

        let read_data = std::fs::read_to_string(tmp_data_path);
        if let Err(_e) = read_data {
            return Item::new();
        }
        let text = read_data.unwrap();
        let itm: Item = serde_json::from_str(&text).unwrap();
        return itm;
    }

    async fn get_settings(&mut self) -> Item {
        let tmp_data_path = self.path.clone() + "/settings.js";

        let read_data = std::fs::read_to_string(tmp_data_path);
        if let Err(_e) = read_data {
            return Item::new();
        }
        let text = read_data.unwrap();
        let itm: Item = serde_json::from_str(&text).unwrap();
        return itm;
    }

    async fn set_settings(&mut self, itm: Item) {
        let tmp_data_path = self.path.clone() + "/settings.js";
        let s = serde_json::to_string(&itm);
        std::fs::write(tmp_data_path, s.unwrap()).expect("Couldn't write item");
    }
}
