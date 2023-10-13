extern crate serde_json;

use isabelle_dm::data_model::all_settings::AllSettings;
use std::fs;
use std::path::Path;

use crate::server::data::*;
use isabelle_dm::data_model::item::*;
use isabelle_dm::data_model::schedule_entry::*;
use log::{info};

pub fn read_item(mut data: &mut Data, path: &str) {
    let paths = fs::read_dir(path).unwrap();

    for path in paths {
        let data_path = path.as_ref().unwrap().path().display().to_string() + "/data.js";
        let idx = path.as_ref().unwrap().file_name().into_string().unwrap().parse::<u64>();

        if let Err(_e) = idx {
            continue;
        }

        info!("Reading from {}", idx.clone().unwrap());

        if Path::new(&data_path).is_file() {
            let text = std::fs::read_to_string(data_path).unwrap();
            let itm: Item =
                serde_json::from_str(&text).unwrap();
            data.items.insert(idx.unwrap(), itm);
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

    data.items_cnt = parsed.unwrap();
}

pub fn read_schedule_entries(mut data: &mut Data, path: &str) {
    let paths = fs::read_dir(path).unwrap();

    for path in paths {
        let data_path = path.as_ref().unwrap().path().display().to_string() + "/data.js";
        let idx = path.as_ref().unwrap().file_name().into_string().unwrap().parse::<u64>();

        if let Err(_e) = idx {
            continue;
        }

        info!("Reading from {}", idx.clone().unwrap());

        if Path::new(&data_path).is_file() {
            let text = std::fs::read_to_string(data_path).unwrap();
            let sch: ScheduleEntry =
                serde_json::from_str(&text).unwrap();
            data.schedule_entries.insert(idx.unwrap(), sch);
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

    data.schedule_entry_cnt = parsed.unwrap();
}

pub fn read_settings_entries(mut data: &mut Data, path: &str) {
    let tmp_data_path = path.to_string() + "/settings.js";

    let read_data = std::fs::read_to_string(tmp_data_path);
    if let Err(_e) = read_data {
        return;
    }
    let text = read_data.unwrap();
    let settings: AllSettings = serde_json::from_str(&text).unwrap();
    data.settings = settings;
}

pub fn read_data(path: &str) -> Data {
    let mut data = Data::new();

    read_item(&mut data, (path.to_string() + "/item").as_str());
    read_schedule_entries(&mut data, (path.to_string() + "/schedule").as_str());
    read_settings_entries(&mut data, (path.to_string() + "/").as_str());
    return data;
}

pub fn write_item_data(data: &mut Data, path: &str) {
    let existing_paths = fs::read_dir(path.to_string() + "/item").unwrap();
    for ep in existing_paths {
        let ep_path = ep.unwrap().path().display().to_string();
        if Path::new(&ep_path).is_file() {
            std::fs::remove_file(&ep_path).expect("Couldn't remove file");
        }
        else {
            std::fs::remove_dir_all(&ep_path).expect("Couldn't remove directory");
        }
    }

    for item in &data.items {
        let tmp_path = path.to_string() + "/item/" + &item.0.to_string();

        std::fs::create_dir(&tmp_path).expect("Couldn't create directory");

        let tmp_data_path = tmp_path.clone() + "/data.js";
        info!("Item path: {}", tmp_data_path);
        let s = serde_json::to_string(&item.1);
        std::fs::write(tmp_data_path, s.unwrap()).expect("Couldn't write item");
    }
    std::fs::write(path.to_string() + "/item/cnt",
                   data.items_cnt.to_string()).expect("Couldn't write item counter");
}

pub fn write_schedule_data(data: &mut Data, path: &str) {
    let existing_paths = fs::read_dir(path.to_string() + "/schedule").unwrap();
    for ep in existing_paths {
        let ep_path = ep.unwrap().path().display().to_string();
        if Path::new(&ep_path).is_file() {
            std::fs::remove_file(&ep_path).expect("Couldn't remove file");
        }
        else {
            std::fs::remove_dir_all(&ep_path).expect("Couldn't remove directory");
        }
    }
    for sch in &data.schedule_entries {
        let tmp_path = path.to_string() + "/schedule/" + &sch.0.to_string();

        std::fs::create_dir(&tmp_path).expect("Couldn't create directory");

        let tmp_data_path = tmp_path.clone() + "/data.js";
        info!("schedule path: {}", tmp_data_path);
        let s = serde_json::to_string(&sch.1);
        std::fs::write(tmp_data_path, s.unwrap()).expect("Couldn't write to file");
    }
    std::fs::write(path.to_string() + "/schedule/cnt",
                   data.schedule_entry_cnt.to_string()).expect("Couldn't write schedule counter");
}

pub fn write_settings_data(data: &mut Data, path: &str) {
    let tmp_data_path = path.to_string() + "/settings.js";
    info!("settings path: {}", tmp_data_path);

    let s = serde_json::to_string(&data.settings);
    std::fs::write(tmp_data_path, s.unwrap()).expect("Couldn't write to file");
}

pub fn write_data(data: &mut Data) {
    write_item_data(data, &data.data_path.clone());
    write_schedule_data(data, &data.data_path.clone());
    write_settings_data(data, &data.data_path.clone());
}