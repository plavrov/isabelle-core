use isabelle_dm::data_model::item::Item;
use std::collections::HashMap;

pub trait Store {
	fn connect(&mut self, addr: &str);
	fn disconnect(&mut self);

	fn get_item(&mut self, collection: &str, id: u64) -> Option<Item>;
	fn get_items(&mut self, collection: &str, id_min: u64, id_max: u64, limit: u64) -> HashMap<u64, Item>;

	fn set_item(&mut self, collection: &str, itm: &Item);
	fn del_item(&mut self, collection: &str, id: u64);

	fn get_credentials(&mut self) -> String;
	fn get_pickle(&mut self) -> String;

	fn get_internals(&mut self) -> Item;

	fn get_settings(&mut self) -> Item;
	fn set_settings(&mut self, itm: Item);
}


