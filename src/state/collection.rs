extern crate serde_json;

use log::info;

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use isabelle_dm::data_model::item::*;

#[derive(Debug, Clone)]
pub struct Collection {
    pub name: String,
    pub count: u64,
    pub items: HashMap<u64, Item>,
    pub settings: Item,
}

impl Collection {
    pub fn new() -> Self {
        Self {
            name: "".to_string(),
            count: 0,
            items: HashMap::new(),
            settings: Item::new(),
        }
    }

    pub fn get(&self, id: u64) -> Option<Item> {
        if self.items.contains_key(&id) {
            return Some(self.items[&id].clone());
        }
        return None;
    }

    pub fn set(&mut self, id: u64, itm: Item, merge: bool) -> u64 {
        let mut ret_id = id;
        if id != u64::MAX && self.items.contains_key(&ret_id) {
            let m = self.items.get_mut(&ret_id).unwrap();
            if merge {
                m.merge(&itm);
            } else {
                *m = itm;
            }
        } else {
            let mut added_itm = itm;
            if ret_id == u64::MAX {
                self.count += 1;
                ret_id = self.count;
                added_itm.id = ret_id;
            }
            self.items.insert(ret_id, added_itm);
        }
        ret_id
    }

    pub fn del(&mut self, id: u64) -> bool {
        if id != u64::MAX && self.items.contains_key(&id) {
            self.items.remove(&id);
            return true;
        }
        return false;
    }

    pub fn get_all(&self) -> &HashMap<u64, Item> {
        return &self.items;
    }

    pub fn get_range(&self, id_min: u64, id_max: u64, limit: u64) -> HashMap<u64, Item> {
        let mut res: HashMap<u64, Item> = HashMap::new();
        let mut eff_id_min = id_min;
        let eff_id_max = id_max;
        let mut count = 0;

        if eff_id_min == u64::MAX {
            eff_id_min = 0;
        }

        for itm in &self.items {
            if itm.0 >= &eff_id_min && itm.0 <= &eff_id_max {
                res.insert(*itm.0, self.items[&itm.0].clone());
                count += 1;
            }
            if limit == count {
                return res;
            }
        }

        return res;
    }

    pub fn read_fs(&mut self, path: &str, name: &str) {
        let paths = fs::read_dir(path).unwrap();

        for path in paths {
            let data_path = path.as_ref().unwrap().path().display().to_string() + "/data.js";
            let idx = path
                .as_ref()
                .unwrap()
                .file_name()
                .into_string()
                .unwrap()
                .parse::<u64>();

            if let Err(_e) = idx {
                continue;
            }

            info!("Reading {}/{}", name, idx.clone().unwrap());

            if Path::new(&data_path).is_file() {
                let text = std::fs::read_to_string(data_path).unwrap();
                let itm: Item = serde_json::from_str(&text).unwrap();
                self.items.insert(idx.unwrap(), itm);
            }
        }

        let cnt_str = std::fs::read_to_string(path.to_string() + "/cnt");
        if let Err(_e) = cnt_str {
            return;
        }

        let parsed = cnt_str.unwrap().parse::<u64>();
        if let Err(_e) = parsed {
            return;
        }

        self.name = name.to_string();
        self.count = parsed.unwrap();

        let setting_text = std::fs::read_to_string(path.to_owned() + "/settings.js");
        match setting_text {
            Ok(file) => self.settings = serde_json::from_str(&file).unwrap(),
            Err(_error) => {}
        }

        info!("Collection {} has {} items", self.name, self.count);
    }

    pub fn write_fs(&self, path: &str) {
        let existing_paths = fs::read_dir(path.to_string()).unwrap();
        for ep in existing_paths {
            let ep_path = ep.unwrap().path().display().to_string();
            if Path::new(&ep_path).is_file() {
                std::fs::remove_file(&ep_path).expect("Couldn't remove file");
            } else {
                std::fs::remove_dir_all(&ep_path).expect("Couldn't remove directory");
            }
        }

        for item in &self.items {
            let tmp_path = path.to_string() + "/" + &item.0.to_string();

            std::fs::create_dir(&tmp_path).expect("Couldn't create directory");

            let tmp_data_path = tmp_path.clone() + "/data.js";
            info!("Item path: {}", tmp_data_path);
            let s = serde_json::to_string(&item.1);
            std::fs::write(tmp_data_path, s.unwrap()).expect("Couldn't write item");
        }
        std::fs::write(path.to_string() + "/cnt", self.count.to_string())
            .expect("Couldn't write item counter");

        let setting_str = serde_json::to_string(&self.settings);
        std::fs::write(path.to_string() + "/settings.js", setting_str.unwrap())
            .expect("Couldn't write settings");
    }
}
