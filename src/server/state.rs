use crate::server::data::*;
use std::sync::Mutex;

pub struct State {
    pub server: Mutex<Data>,
}

impl State {
    pub fn new() -> Self {
        let srv = Data::new();
        Self {
            server: Mutex::new(srv),
        }
    }
}
