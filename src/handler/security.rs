use crate::notif::email::send_email;
use crate::state::store::Store;
use crate::util::crypto::get_new_salt;
use crate::util::crypto::get_password_hash;
use crate::util::crypto::verify_password;
use isabelle_dm::data_model::item::Item;
use isabelle_dm::data_model::process_result::ProcessResult;
use log::{error, info};

pub async fn security_check_unique_login_email(
    srv: &mut crate::state::data::Data,
    _collection: &str,
    _old_itm: Option<Item>,
    itm: &mut Item,
    del: bool,
) -> ProcessResult {
    if del {
        return ProcessResult {
            succeeded: true,
            error: "".to_string(),
        };
    }
    let email = itm.safe_str("email", "");
    let login = itm.safe_str("login", "");

    if email == "" {
        return ProcessResult {
            succeeded: false,
            error: "E-Mail must not be empty".to_string(),
        };
    }

    let users = srv.rw.get_all_items("user").await;
    for usr in &users.map {
        if *usr.0 != itm.id {
            if login != "" && login == usr.1.safe_str("login", "") {
                return ProcessResult {
                    succeeded: false,
                    error: "Login mustn't match already existing one".to_string(),
                };
            }
            if email == usr.1.safe_str("email", "") {
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
    _srv: &mut crate::state::data::Data,
    collection: &str,
    old_itm: Option<Item>,
    itm: &mut Item,
    del: bool,
) -> ProcessResult {
    let mut salt: String = "<empty salt>".to_string();

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
        let res = verify_password(&old_checked_pw, &old_pw_hash)
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
