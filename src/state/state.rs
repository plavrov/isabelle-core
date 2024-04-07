use std::sync::Arc;
use crate::state::data::*;
use std::sync::Mutex;

pub struct State {
    pub server: Arc<Mutex<Data>>,
}

impl Clone for State {
    // Define the clone method
    fn clone(&self) -> Self {
        // Create a new instance with the same value
        State {
            server: self.server.clone(),
        }
    }
}

impl State {
    pub fn new() -> Self {
        let srv = Data::new();
        Self {
            server: Arc::new(Mutex::new(srv)),
        }
    }
}

unsafe impl Send for State {}
