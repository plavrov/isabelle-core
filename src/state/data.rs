use crate::state::store_local::*;

pub struct Data {
    pub rw: StoreLocal,
    pub gc_path: String,
    pub py_path: String,
    pub data_path: String,
    pub public_url: String,
    pub port: u16,
}

impl Data {
    pub fn new() -> Self {
        Self {
            rw: StoreLocal::new(),

            gc_path: "".to_string(),
            py_path: "".to_string(),
            data_path: "".to_string(),
            public_url: "".to_string(),
            port: 8090,
        }
    }

    pub fn has_collection(&mut self, collection: &str) -> bool {
        return self.rw.collections.contains_key(collection);
    }
}
