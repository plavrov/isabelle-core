extern crate serde_json;
use std::path::Path;

use crate::state::store::Store;
use isabelle_dm::data_model::item::*;
use log::{error, info};
use std::collections::HashMap;
use std::fs;

#[derive(Debug, Clone)]
pub struct StoreLocal {
    pub path: String,
    pub collections: HashMap<String, u64>,
    pub items: HashMap<u64, HashMap<u64, bool>>,
    pub items_count: HashMap<u64, u64>,
}

unsafe impl Send for StoreLocal {}

impl StoreLocal {
    pub fn new() -> Self {
        Self {
            path: "".to_string(),
            collections: HashMap::new(),
            items: HashMap::new(),
            items_count: HashMap::new(),
        }
    }
}

impl Store for StoreLocal {
    fn connect(&mut self, url: &str) {
        self.path = url.to_string();
        let collections = fs::read_dir(self.path.to_string() + "/collection").unwrap();
        for coll in collections {
            let idx = coll.as_ref().unwrap().file_name().into_string().unwrap();
            let new_col: HashMap<u64, bool> = HashMap::new();
            let coll_index = self.items.len().try_into().unwrap();
            self.items.insert(coll_index, new_col);
            self.collections.insert(idx.clone(), coll_index);
            info!("New collection {}", idx.clone());

            let cnt_str =
                std::fs::read_to_string(self.path.clone() + "/collection/" + &idx + "/cnt");
            if let Err(_e) = cnt_str {
                error!("Failed to read counter");
                continue;
            }

            let parsed = cnt_str.as_ref().unwrap().trim().parse::<u64>();
            if let Err(_e) = parsed {
                error!("Failed to parse counter {}", cnt_str.as_ref().unwrap());
                continue;
            }

            self.items_count
                .insert(self.collections[&idx], *parsed.as_ref().unwrap());
            info!(" - index: {}", self.collections[&idx]);
            info!(" - counter: {}", parsed.as_ref().unwrap());

            let data_files = fs::read_dir(self.path.to_string() + "/collection/" + &idx).unwrap();
            for data_file in data_files {
                let data_file_idx = data_file.as_ref().unwrap().file_name().into_string().unwrap();
                let tmp_path = self.path.to_string() + "/collection/" + &idx + "/" + &data_file_idx;
                if Path::new(&tmp_path).is_dir() {
                    let m = self.items.get_mut(&coll_index).unwrap();
                    (*m).insert(data_file_idx.parse::<u64>().unwrap(), true);
                    info!("{}: idx {}", &idx, &data_file_idx);
                }
            }
        }
    }

    fn disconnect(&mut self) {}

    fn get_all_items(&mut self, collection: &str) -> HashMap<u64, Item> {
        return self.get_items(collection, u64::MAX, u64::MAX, u64::MAX);
    }

    fn get_item(&mut self, collection: &str, id: u64) -> Option<Item> {
        let tmp_path = self.path.to_string() +
            "/collection/" + collection +
            "/" + &id.to_string() + "/data.js";
        if Path::new(&tmp_path).is_file() {
            let text = std::fs::read_to_string(tmp_path).unwrap();
            let itm: Item = serde_json::from_str(&text).unwrap();
            return Some(itm);
        }
        return None;
    }

    fn get_items(
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

        info!("Getting {} in range {} - {} limit {}", &collection, eff_id_min, eff_id_max, limit);
        for itm in itms {
            if itm.0 >= eff_id_min && itm.0 <= eff_id_max {
                let new_item = self.get_item(collection, itm.0);
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

    fn set_item(&mut self, collection: &str, itm: &Item, merge: bool) {
        let old_itm = self.get_item(collection, itm.id);
        let mut new_itm = itm.clone();
        if !old_itm.is_none() && merge {
            new_itm = old_itm.unwrap().clone();
            new_itm.merge(itm);
        }
        let tmp_path =
            self.path.to_string() + "/collection/" + collection + "/" + &new_itm.id.to_string();

        std::fs::create_dir(&tmp_path).expect("Couldn't create directory");

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

    fn del_item(&mut self, collection: &str, id: u64) -> bool {
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

    fn get_credentials(&mut self) -> String {
        return self.path.clone() + "/credentials.json";
    }

    fn get_pickle(&mut self) -> String {
        return self.path.clone() + "/token.pickle";
    }

    fn get_internals(&mut self) -> Item {
        let tmp_data_path = self.path.clone() + "/internals.js";

        let read_data = std::fs::read_to_string(tmp_data_path);
        if let Err(_e) = read_data {
            return Item::new();
        }
        let text = read_data.unwrap();
        let itm: Item = serde_json::from_str(&text).unwrap();
        return itm;
    }

    fn get_settings(&mut self) -> Item {
        let tmp_data_path = self.path.clone() + "/settings.js";

        let read_data = std::fs::read_to_string(tmp_data_path);
        if let Err(_e) = read_data {
            return Item::new();
        }
        let text = read_data.unwrap();
        let itm: Item = serde_json::from_str(&text).unwrap();
        return itm;
    }

    fn set_settings(&mut self, itm: Item) {
        let tmp_data_path = self.path.clone() + "/settings.js";
        let s = serde_json::to_string(&itm);
        std::fs::write(tmp_data_path, s.unwrap()).expect("Couldn't write item");
    }
}

/*

pub fn get_credentials_json(srv: &crate::state::data::Data) -> String {
    return srv.data_path.clone() + "/credentials.json";
}

pub fn get_pickle(srv: &crate::state::data::Data) -> String {
    return srv.data_path.clone() + "/token.pickle";
}

pub fn read_settings_entries(mut data: &mut Data, path: &str) {
    let tmp_data_path = path.to_string() + "/settings.js";

    let read_data = std::fs::read_to_string(tmp_data_path);
    if let Err(_e) = read_data {
        return;
    }
    let text = read_data.unwrap();
    let settings: Item = serde_json::from_str(&text).unwrap();
    data.settings = settings;
}

pub fn read_internals_entries(mut data: &mut Data, path: &str) {
    let tmp_data_path = path.to_string() + "/internals.js";

    let read_data = std::fs::read_to_string(tmp_data_path);
    if let Err(_e) = read_data {
        return;
    }
    let text = read_data.unwrap();
    let settings: Item = serde_json::from_str(&text).unwrap();
    data.internals = settings;
}

pub fn read_data(path: &str) -> Data {
    let mut data = Data::new();

    read_internals_entries(&mut data, (path.to_string() + "/").as_str());
    read_settings_entries(&mut data, (path.to_string() + "/").as_str());

    let collection_routes = data
        .internals
        .safe_strstr("collection_read_hook", &HashMap::new());
    let collections = fs::read_dir(path.to_string() + "/collection").unwrap();
    for coll in collections {
        let idx = coll.as_ref().unwrap().file_name().into_string().unwrap();
        let mut new_col = Collection::new();
        new_col.read_fs(&(path.to_string() + "/collection/" + &idx), &idx);
        for collection_route in &collection_routes {
            call_collection_read_hook(&collection_route.1, &idx, &mut new_col);
        }
        data.itm.insert(idx, new_col);
    }

    return data;
}

pub fn write_settings_data(data: &Data, path: &str) {
    let tmp_data_path = path.to_string() + "/settings.js";
    info!("settings path: {}", tmp_data_path);

    let s = serde_json::to_string(&data.settings);
    std::fs::write(tmp_data_path, s.unwrap()).expect("Couldn't write to file");

    if data.settings.strs.contains_key("site_name") {
        let tmp_name_path = path.to_string() + "/site_name.txt";
        std::fs::write(tmp_name_path, &data.settings.strs["site_name"])
            .expect("Couldn't write to file");
    }
}

pub fn write_data(data: &Data) {
    for coll in &data.itm {
        if &coll.1.name == "" {
            info!("Unknown collection");
            continue;
        }
        coll.1
            .write_fs(&(data.data_path.clone() + "/collection/" + &coll.1.name));
    }
    write_settings_data(data, &data.data_path.clone());
}
*/
