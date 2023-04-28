use crate::data_model::mentee::*;
use crate::data_model::user::*;

pub struct ScheduleEntry {
    pub is_group: bool,
    pub mentees: Vec<Mentee>,
    pub users: Vec<User>,
}

unsafe impl Send for ScheduleEntry {}

impl ScheduleEntry {
    pub fn new() -> Self {
        Self {
            is_group: false,
            mentees: Vec::new(),
            users: Vec::new(),
        }
    }
}