use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug, Serialize, Clone)]
pub struct Mentee {
    pub name: String,
}

unsafe impl Send for Mentee {}

impl Mentee {
    pub fn new() -> Self {
        Self {
            name: String::new(),
        }
    }
}