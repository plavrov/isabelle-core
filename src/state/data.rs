use crate::state::collection::*;
use isabelle_dm::data_model::item::*;
use std::collections::HashMap;
use crate::state::store_local::StoreLocal;

#[derive(Debug, Clone)]
pub struct Data {
    pub itm_cnt: HashMap<String, u64>,
    pub itm: HashMap<String, Collection>,

    pub settings: Item,
    pub internals: Item,

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
            itm_cnt: HashMap::new(),
            itm: HashMap::new(),

            settings: Item::new(),
            internals: Item::new(),

            rw: StoreLocal::new(),

            gc_path: "".to_string(),
            py_path: "".to_string(),
            data_path: "".to_string(),
            public_url: "".to_string(),
            port: 8090,
        }
    }
}
