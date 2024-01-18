use crate::state::store::Store;
use isabelle_dm::data_model::item::Item;
use crate::server::user_control::*;
use log::info;

pub async fn intranet_itm_auth_hook(
    mut srv: &mut crate::state::data::Data,
    user: &Option<Item>,
    collection: &str,
    id: u64,
    _new_item: Option<Item>,
    del: bool,
) -> bool {
    if check_role(&mut srv, &user, "admin").await {
        return true;
    }

    info!(
        "Checking collection {} user id {}",
        collection,
        user.as_ref().unwrap().id
    );

    if collection == "diary_record" {
        let mut accept = true;
        let itm = srv.rw.get_item("diary_record", id).await;

        if !itm.is_none()
            && itm.unwrap().safe_id("user", u64::MAX) != user.as_ref().unwrap().id
        {
            accept = false;
        }

        return accept;
    } else if collection == "project" {
        return false;
    } else if collection == "user" {
        if del {
            return false;
        }
        let itm = srv.rw.get_item("user", id).await;
        if !itm.is_none() && itm.unwrap().id == user.as_ref().unwrap().id {
            return true;
        }
        return false;
    }

    return false;
}
