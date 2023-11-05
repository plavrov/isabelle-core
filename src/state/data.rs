use crate::handler::route::call_collection_read_hook;
use std::collections::HashMap;
use crate::state::store::Store;
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

    pub fn init_checks(&mut self) {
        let internals = self.rw.get_internals();
        let routes = internals
            .safe_strstr("collection_read_hook", &HashMap::new());
        let collections = self.rw.get_collections();
        for collection in &collections {
            let items = self.rw.get_item_ids(collection);
            for itm in items {
                let loaded_item_opt = self.rw.get_item(collection, itm.0);
                if loaded_item_opt.is_none() {
                    continue;
                }
                let mut loaded_item = loaded_item_opt.unwrap();
                let mut should_be_saved = false;
                for route in &routes {
                    if call_collection_read_hook(&route.1, collection, &mut loaded_item) {
                        should_be_saved = true;
                    }
                }
                if should_be_saved {
                    self.rw.set_item(collection, &loaded_item, false);
                }
            }
        }
    }
}
