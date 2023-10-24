use isabelle_dm::data_model::item::Item;

pub fn get_user(srv: &crate::state::data::Data, login: String) -> Option<Item> {
    for item in srv.itm["user"].get_all() {
        if item.1.strs.contains_key("login")
            && item.1.strs["login"] == login
        {
            return Some(item.1.clone());
        }
    }

    return None;
}

pub fn check_role(user: Option<Item>, role: &str) -> bool {
    if user == None {
        return false;
    }
    return user
        .unwrap()
        .safe_bool(&("role_is_".to_owned() + role), false);
}
