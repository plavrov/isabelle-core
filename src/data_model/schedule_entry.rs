use serde::{Deserialize, Serialize};

use crate::data_model::mentee::*;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleEntry {
    pub is_group: bool,
    pub mentees: Vec<u64>,
    pub users: Vec<u64>,
    pub times: Vec<u64>,
}

unsafe impl Send for ScheduleEntry {}

impl ScheduleEntry {
    pub fn new() -> Self {
        Self {
            is_group: false,
            mentees: Vec::new(),
            users: Vec::new(),
            times: Vec::new(),
        }
    }
}