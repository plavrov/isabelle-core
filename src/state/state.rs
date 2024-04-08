use std::cell::RefCell;
use std::sync::Arc;
use crate::state::data::*;

use parking_lot::ReentrantMutex;

pub struct State {
    pub server: Arc<ReentrantMutex<RefCell<Data>>>,
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
            server: Arc::new(ReentrantMutex::new(RefCell::new(srv))),
        }
    }
}

unsafe impl Send for State {}
