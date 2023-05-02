use serde::{Deserialize, Serialize};

use crate::data_model::mentee::*;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleEntry {
    #[serde(default = "set_false")]
    pub is_group: bool,
    #[serde(default = "empty_u64_set")]
    pub mentees: Vec<u64>,
    #[serde(default = "empty_u64_set")]
    pub users: Vec<u64>,
    #[serde(default = "empty_u64_set")]
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

fn set_false() -> bool {
    return false;
}

fn empty_u64_set() -> Vec<u64> {
    return Vec::new();
}
