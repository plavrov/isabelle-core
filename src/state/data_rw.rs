extern crate serde_json;
use crate::handler::route::*;
use std::collections::HashMap;
use crate::state::collection::*;
use crate::state::data::*;
use isabelle_dm::data_model::item::*;
use log::info;
use std::fs;

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

    let collection_routes = data.internals.safe_strstr("collection_read_hook", &HashMap::new());
    let collections = fs::read_dir(path.to_string() + "/collection").unwrap();
    for coll in collections {
        let idx = coll.as_ref().unwrap().file_name().into_string().unwrap();
        let mut new_col = Collection::new();
        new_col.read_fs(&(path.to_string() + "/collection/" + &idx), &idx);
        for collection_route in &collection_routes {
            call_collection_read_hook(&collection_route.1,
                                      &idx,
                                      & mut new_col);
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
