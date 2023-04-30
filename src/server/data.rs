use crate::data_model::schedule_entry::ScheduleEntry;
use crate::data_model::mentee::*;
use crate::data_model::user::*;

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Data {
    pub users_cnt: u64,
    pub users: HashMap<u64, User>,

    pub mentee_cnt: u64,
    pub mentees: HashMap<u64, Mentee>,

    pub schedule_entry_cnt: u64,
    pub schedule_entries: HashMap<u64, ScheduleEntry>,
    pub schedule_entry_times: HashMap<u64, Vec<u64>>
}

impl Data {
    pub fn new() -> Self {
        Self {
            users_cnt: 0,
            users: HashMap::new(),

            mentee_cnt: 0,
            mentees: HashMap::new(),

            schedule_entry_cnt: 0,
            schedule_entries: HashMap::new(),

            schedule_entry_times: HashMap::new(),
        }
    }
}
