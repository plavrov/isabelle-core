use crate::data_model::mentee::*;
use crate::data_model::user::*;

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Data {
    pub users_cnt: u64,
    pub users: HashMap<u64, User>,

    pub mentee_cnt: u64,
    pub mentees: HashMap<u64, Mentee>,
}

impl Data {
    pub fn new() -> Self {
        Self {
            users_cnt: 0,
            users: HashMap::new(),

            mentee_cnt: 0,
            mentees: HashMap::new(),
        }
    }
}
