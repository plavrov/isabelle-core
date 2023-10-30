use isabelle_dm::data_model::item::Item;
use crate::util::crypto::get_new_salt;
use crate::state::collection::Collection;
use crate::util::crypto::get_password_hash;
use log::info;

pub fn security_collection_read_hook(collection: &str, new_col: & mut Collection) {
    if collection == "user" {
        let mut replace : Vec<Item> = Vec::new();
        for pair in &new_col.items {
            let mut new_itm = pair.1.clone();
            if !pair.1.strs.contains_key("salt") {
                let salt = get_new_salt();
                new_itm.set_str("salt", &salt);
                info!("There is no salt for user {}, created new", pair.0);
                if pair.1.strs.contains_key("password") {
                    let pw_old = pair.1.safe_str("password", "");
                    let hash = get_password_hash(&pw_old, &salt);
                    new_itm.set_str("password", &hash);
                    info!("Rehashed password for user {}", pair.0);
                }
                replace.push(new_itm);
            }
        }
        for itm in replace {
            new_col.set(itm.id, itm, false);
        }
    }
}