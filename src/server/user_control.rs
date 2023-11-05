use crate::state::store::Store;
use isabelle_dm::data_model::item::Item;
use log::info;

pub fn get_user(srv: &mut crate::state::data::Data, login: String) -> Option<Item> {
    let users = srv.rw.get_all_items("user");
    info!("Users: {}", users.len());
    for item in &users {
        if item.1.strs.contains_key("login") && item.1.strs["login"] == login {
            return Some(item.1.clone());
        }
        if item.1.strs.contains_key("email") && item.1.strs["email"] == login {
            return Some(item.1.clone());
        }
    }

    return None;
}

pub fn check_role(srv: &mut crate::state::data::Data, user: &Option<Item>, role: &str) -> bool {
    let role_is = srv
        .rw
        .get_internals()
        .safe_str("user_role_prefix", "role_is_");
    if user.is_none() {
        return false;
    }
    return user
        .as_ref()
        .unwrap()
        .safe_bool(&(role_is.to_owned() + role), false);
}

pub fn clear_otp(srv: &mut crate::state::data::Data, login: String) {
    let users = srv.rw.get_all_items("user");
    for item in &users {
        if item.1.strs.contains_key("login")
            && item.1.strs["login"] == login
            && item.1.strs.contains_key("email")
            && item.1.strs["email"] == login
        {
            let mut itm = item.1.clone();
            itm.set_str("otp", "");
            srv.rw.set_item("user", &itm, false);
            return;
        }
    }
}
