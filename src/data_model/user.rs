use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug, Serialize, Clone)]
pub struct User {
    pub firstname: String,
    pub surname: String,
    pub phone: String,
}

unsafe impl Send for User {}

impl User {
    pub fn new() -> Self {
        Self {
            firstname: String::new(),
            surname: String::new(),
            phone: String::new(),
        }
    }
}