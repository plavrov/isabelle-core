use isabelle_dm::data_model::item::Item;

pub trait Store {
	fn connect(&mut self, addr: &str);
	fn disconnect(&mut self);

	fn get_item(&mut self, collection: &str, id: u64) -> Option<Item>;
	fn set_item(&mut self, collection: &str, itm: &Item);
	fn del_item(&mut self, collection: &str, id: u64);
}
