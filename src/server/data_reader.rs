use std::fs;
use crate::server::data::*;

pub fn read_user(mut data: &mut Data, path: &str) {
    data.users_cnt = 5;
}

pub fn read_data(path: &str) -> Data {
    let mut data = Data::new();

    let paths = fs::read_dir(path).unwrap();

    for path in paths {
        println!("Name: {}", path.unwrap().path().display())
    }
    data.schedule_entry_cnt = 5;

    read_user(&mut data, (path.to_string() + "/user").as_str());
    return data;
}