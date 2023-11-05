extern crate serde_json;
use std::path::Path;
use crate::handler::route::*;
use crate::state::collection::*;
use crate::state::data::*;
use isabelle_dm::data_model::item::*;
use log::info;
use std::collections::HashMap;
use std::fs;
use crate::state::store::Store;

#[derive(Debug, Clone)]
pub struct StoreLocal {
    pub path: String,
    pub collections: HashMap<String, u64>,
    pub items: HashMap<u64, HashMap<u64, bool>>,
}

unsafe impl Send for StoreLocal {

}

impl StoreLocal {
    pub fn new() -> Self {
        Self {
            path: "".to_string(),
            collections: HashMap::new(),
            items: HashMap::new(),
        }
    }
}

impl Store for StoreLocal {
    fn connect(&mut self, url: &str) {
        self.path = url.to_string();
        let collections = fs::read_dir(self.path.to_string() +
            "/collection").unwrap();
        for coll in collections {
            let idx = coll.as_ref().unwrap().file_name().into_string().unwrap();
            let new_col: HashMap<u64, bool> = HashMap::new();
            self.items.insert(self.items.len().try_into().unwrap(), new_col);
            self.collections.insert(idx.clone(),
                (self.items.len() - 1).try_into().unwrap());
            println!("New collection {}", idx.clone());
        }
    }

    fn disconnect(&mut self) {

    }

    fn get_item(&mut self, collection: &str, id: u64) -> Option<Item> {
        let tmp_path = self.path.to_string() + "/" + collection + "/" +
            &id.to_string();
        if Path::new(&tmp_path).is_file() {
            let text = std::fs::read_to_string(tmp_path).unwrap();
            let itm: Item = serde_json::from_str(&text).unwrap();
            return Some(itm);
        }
        return None;
    }

    fn set_item(&mut self, collection: &str, itm: &Item) {
        let tmp_path = self.path.to_string() + "/" + collection + "/" +
            &itm.id.to_string();

        std::fs::create_dir(&tmp_path).expect("Couldn't create directory");

        let tmp_data_path = tmp_path.clone() + "/data.js";
        let s = serde_json::to_string(&itm);
        std::fs::write(tmp_data_path, s.unwrap()).expect("Couldn't write item");

        let coll_id = self.collections[collection];
        if self.items.contains_key(&coll_id) {
            let coll = self.items.get_mut(&coll_id).unwrap();
            if coll.contains_key(&itm.id) {
                *(coll.get_mut(&itm.id).unwrap()) = true;
            } else {
                coll.insert(itm.id, true);
            }
        }
    }

    fn del_item(&mut self, collection: &str, id: u64) {
        let tmp_path = self.path.to_string() + "/" + collection + "/" +
            &id.to_string();
        let path = Path::new(&tmp_path);
        if path.exists() {
            let _res = std::fs::remove_dir_all(tmp_path);
        }
        let coll_id = self.collections[collection];
        if self.items.contains_key(&coll_id) {
            let coll = self.items.get_mut(&coll_id).unwrap();
            if coll.contains_key(&id) {
                coll.remove(&id);
            }
        }
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