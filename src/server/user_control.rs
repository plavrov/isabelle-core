use isabelle_dm::data_model::item::Item;

pub fn get_user(srv: &crate::state::data::Data, login: String) -> Option<Item> {
    for item in &srv.items {
        if item.1.fields.contains_key("login")
            && item.1.fields["login"] == login
            && item.1.bool_params.contains_key("is_human")
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
    return user.unwrap().safe_bool(&("role_is_".to_owned() + role), false);
}
