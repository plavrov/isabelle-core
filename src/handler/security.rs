use std::collections::HashMap;
use crate::notif::email::send_email;
use crate::server::user_control::check_role;
use crate::state::store::Store;
use crate::util::crypto::get_new_salt;
use crate::util::crypto::get_password_hash;
use crate::util::crypto::verify_password;
use isabelle_dm::data_model::item::Item;
use isabelle_dm::data_model::process_result::ProcessResult;
use log::{error, info};

pub async fn security_check_unique_login_email(
    srv: &mut crate::state::data::Data,
    _user: &Option<Item>,
    _collection: &str,
    _old_itm: Option<Item>,
    itm: &mut Item,
    del: bool,
    merge: bool,
) -> ProcessResult {
    let mut itm_upd = if _old_itm != None {
        _old_itm.unwrap()
    } else {
        Item::new()
    };
    if merge {
        itm_upd.merge(itm);
    } else {
        itm_upd = itm.clone();
    }
    if del {
        return ProcessResult {
            succeeded: true,
            error: "".to_string(),
        };
    }
    let email = itm_upd.safe_str("email", "").to_lowercase();
    let login = itm.safe_str("login", "").to_lowercase();

    if email == "" {
        return ProcessResult {
            succeeded: false,
            error: "E-Mail must not be empty".to_string(),
        };
    }

    let users = srv.rw.get_all_items("user", "id").await;
    for usr in &users.map {
        if *usr.0 != itm.id {
            if login != "" && login == usr.1.safe_str("login", "").to_lowercase() {
                return ProcessResult {
                    succeeded: false,
                    error: "Login mustn't match already existing one".to_string(),
                };
            }
            if email == usr.1.safe_str("email", "").to_lowercase() {
                return ProcessResult {
                    succeeded: false,
                    error: "E-Mail mustn't match already existing one".to_string(),
                };
            }
        }
    }

    return ProcessResult {
        succeeded: true,
        error: "".to_string(),
    };
}

pub async fn security_password_challenge_pre_edit_hook(
    srv: &mut crate::state::data::Data,
    user: &Option<Item>,
    collection: &str,
    old_itm: Option<Item>,
    itm: &mut Item,
    del: bool,
    _merge: bool,
) -> ProcessResult {
    let mut salt: String = "<empty salt>".to_string();
    let is_admin = check_role(srv, &user, "admin").await;

    if del {
        return ProcessResult {
            succeeded: true,
            error: "".to_string(),
        };
    }

    if collection == "user"
        && old_itm != None
        && (itm.strs.contains_key("password") || itm.strs.contains_key("salt"))
    {
        error!("Can't edit password directly");
        return ProcessResult {
            succeeded: false,
            error: "Can't edit password directly".to_string(),
        };
    }

    if collection == "user" {
        if old_itm.is_none() {
            /* Add salt when creating new user */
            salt = get_new_salt();
            itm.set_str("salt", &salt);
        } else {
            salt = old_itm.as_ref().unwrap().safe_str("salt", "<empty salt>");
        }
    }

    if collection == "user"
        && old_itm != None
        && itm.strs.contains_key("__password")
        && itm.strs.contains_key("__new_password1")
        && itm.strs.contains_key("__new_password2")
    {
        let old_pw_hash = old_itm.as_ref().unwrap().safe_str("password", "");
        let old_otp = old_itm.as_ref().unwrap().safe_str("otp", "");
        let old_checked_pw = itm.safe_str("__password", "");
        if !is_admin && old_checked_pw == "" {
            error!("Old password is empty");
            return ProcessResult {
                succeeded: false,
                error: "Old password is empty".to_string(),
            };
        }
        let res = is_admin
            || verify_password(&old_checked_pw, &old_pw_hash)
            || (old_otp != "" && old_otp == old_checked_pw);
        if !res
            || itm.safe_str("__new_password1", "<bad1>")
                != itm.safe_str("__new_password2", "<bad2>")
        {
            error!("Password change challenge failed");
            return ProcessResult {
                succeeded: false,
                error: "Password change challenge failed".to_string(),
            };
        }
        let new_pw = itm.safe_str("__new_password1", "");
        itm.strs.remove("__password");
        itm.strs.remove("__new_password1");
        itm.strs.remove("__new_password2");
        itm.strs.remove("otp");

        let pw_hash = get_password_hash(&new_pw, &salt);
        if itm.strs.contains_key("otp") {
            itm.strs.remove("otp");
        }
        itm.set_str("password", &pw_hash);
    }
    return ProcessResult {
        succeeded: true,
        error: "".to_string(),
    };
}

pub async fn security_collection_read_hook(collection: &str, itm: &mut Item) -> bool {
    if collection == "user" {
        if !itm.strs.contains_key("salt") {
            let salt = get_new_salt();
            itm.set_str("salt", &salt);
            info!("There is no salt for user {}, created new", itm.id);
            if itm.strs.contains_key("password") {
                let pw_old = itm.safe_str("password", "");
                let hash = get_password_hash(&pw_old, &salt);
                itm.set_str("password", &hash);
                info!("Rehashed password for user {}", itm.id);
            }
            return true;
        }
    }
    return false;
}

pub async fn security_otp_send_email(srv: &mut crate::state::data::Data, itm: Item) {
    let email = itm.safe_str("email", "");
    let otp = itm.safe_str("otp", "");
    if email == "" || otp == "" {
        return;
    }

    send_email(
        srv,
        &email,
        "Your login code",
        &format!("Enter this as password: {}", otp),
    )
    .await;
}

pub async fn security_itm_filter_hook(
    mut srv: &mut crate::state::data::Data,
    user: &Option<Item>,
    collection: &str,
    context: &str,
    map: &mut HashMap<u64, Item>,
) {
    let mut list = true;
    let is_admin = check_role(&mut srv, &user, "admin").await;

    if is_admin && collection != "user" {
        return;
    }

    if context == "full" {
        list = false;
    }

    let mut short_map: HashMap<u64, Item> = HashMap::new();
    if user.is_none() {
        *map = short_map;
        return;
    }

    info!(
        "Checking collection {} user id {}",
        collection,
        user.as_ref().unwrap().id
    );
    if list {
        for el in &mut *map {
            if collection == "user" {
                if *el.0 == user.as_ref().unwrap().id || is_admin {
                    let mut itm = Item::new();
                    itm.id = *el.0;
                    itm.strs
                        .insert("name".to_string(), el.1.safe_str("name", ""));
                    if *el.0 == user.as_ref().unwrap().id || is_admin {
                        itm.strs
                            .insert("phone".to_string(), el.1.safe_str("phone", ""));
                        itm.bools.insert(
                            "has_insurance".to_string(),
                            el.1.safe_bool("has_insurance", false),
                        );
                    }
                    itm.bools.insert(
                        "role_is_active".to_string(),
                        el.1.safe_bool("role_is_active", false),
                    );
                    itm.bools.insert(
                        "role_is_admin".to_string(),
                        el.1.safe_bool("role_is_admin", false),
                    );
                    short_map.insert(*el.0, itm);
                }
            } else {
                let mut itm = Item::new();
                itm.id = *el.0;
                itm.strs
                    .insert("name".to_string(), el.1.safe_str("name", ""));
                short_map.insert(*el.0, itm);
            }
        }
    } else {
        if collection == "user" {
            for el in &mut *map {
                if *el.0 != user.as_ref().unwrap().id && !is_admin {
                    /* nothing */
                } else {
                    let mut itm = el.1.clone();
                    if itm.strs.contains_key("salt") {
                        itm.strs.remove("salt");
                    }
                    if itm.strs.contains_key("password") {
                        itm.strs.remove("password");
                    }
                    short_map.insert(*el.0, itm);
                }
            }
        } else {
            short_map = map.clone();
        }
    }
    *map = short_map;
}

