extern crate serde_json;

use std::fs;
use std::path::Path;

use crate::server::data::*;
use crate::data_model::user::*;
use crate::data_model::mentee::*;
use crate::data_model::schedule_entry::*;
use log::{info};

use serde::Deserialize;


pub fn read_user(mut data: &mut Data, path: &str) {
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
            let user: User =
                serde_json::from_str(&text).unwrap();
            data.users.insert(idx.unwrap(), user);
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

    data.users_cnt = parsed.unwrap();
}

pub fn read_mentee(mut data: &mut Data, path: &str) {
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
            let mentee: Mentee =
                serde_json::from_str(&text).unwrap();
            data.mentees.insert(idx.unwrap(), mentee);
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

    data.mentee_cnt = parsed.unwrap();
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
pub fn read_data(path: &str) -> Data {
    let mut data = Data::new();

    let paths = fs::read_dir(path).unwrap();

    for path in paths {
        println!("Name: {}", path.unwrap().path().display())
    }
    data.schedule_entry_cnt = 5;

    read_user(&mut data, (path.to_string() + "/user").as_str());
    read_mentee(&mut data, (path.to_string() + "/mentee").as_str());
    read_schedule_entries(&mut data, (path.to_string() + "/schedule").as_str());
    return data;
}

pub fn write_user_data(data: &mut Data, path: &str) {
    let existing_paths = fs::read_dir(path.to_string() + "/user").unwrap();
    for ep in existing_paths {
        let epPath = ep.unwrap().path().display().to_string();
        if Path::new(&epPath).is_file() {
            std::fs::remove_file(&epPath);
        }
        else {
            std::fs::remove_dir_all(&epPath);
        }
    }

    for user in &data.users {
        let tmp_path = path.to_string() + "/user/" + &user.0.to_string();

        std::fs::create_dir(&tmp_path);

        let tmp_data_path = tmp_path.clone() + "/data.js";
        info!("User path: {}", tmp_data_path);
        let s = serde_json::to_string(&user.1);
        std::fs::write(tmp_data_path, s.unwrap());
    }
    std::fs::write(path.to_string() + "/user/cnt",
                   data.users_cnt.to_string());
}

pub fn write_mentee_data(data: &mut Data, path: &str) {
    let existing_paths = fs::read_dir(path.to_string() + "/mentee").unwrap();
    for ep in existing_paths {
        let epPath = ep.unwrap().path().display().to_string();
        if Path::new(&epPath).is_file() {
            std::fs::remove_file(&epPath);
        }
        else {
            std::fs::remove_dir_all(&epPath);
        }
    }

    for mentee in &data.mentees {
        let tmp_path = path.to_string() + "/mentee/" + &mentee.0.to_string();

        std::fs::create_dir(&tmp_path);

        let tmp_data_path = tmp_path.clone() + "/data.js";
        info!("Mentee path: {}", tmp_data_path);
        let s = serde_json::to_string(&mentee.1);
        std::fs::write(tmp_data_path, s.unwrap());
    }
    std::fs::write(path.to_string() + "/mentee/cnt",
                   data.mentee_cnt.to_string());
}

pub fn write_schedule_data(data: &mut Data, path: &str) {
    let existing_paths = fs::read_dir(path.to_string() + "/schedule").unwrap();
    for ep in existing_paths {
        let epPath = ep.unwrap().path().display().to_string();
        if Path::new(&epPath).is_file() {
            std::fs::remove_file(&epPath);
        }
        else {
            std::fs::remove_dir_all(&epPath);
        }
    }
    for sch in &data.schedule_entries {
        let tmp_path = path.to_string() + "/schedule/" + &sch.0.to_string();

        std::fs::create_dir(&tmp_path);

        let tmp_data_path = tmp_path.clone() + "/data.js";
        info!("schedule path: {}", tmp_data_path);
        let s = serde_json::to_string(&sch.1);
        std::fs::write(tmp_data_path, s.unwrap());
    }
    std::fs::write(path.to_string() + "/schedule/cnt",
                   data.schedule_entry_cnt.to_string());
}

pub fn write_data(data: &mut Data, path: &str) {
    write_user_data(data, path);
    write_mentee_data(data, path);
    write_schedule_data(data, path);
}