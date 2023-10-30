use isabelle_dm::data_model::process_result::ProcessResult;
use crate::handler::route::*;
use crate::state::collection::Collection;
use crate::state::state::*;
use crate::write_data;
use actix_identity::Identity;
use actix_web::{web, HttpRequest, HttpResponse, Responder};
use isabelle_dm::data_model::item::Item;
use isabelle_dm::data_model::list_query::ListQuery;
use isabelle_dm::data_model::merge_coll::MergeColl;
use log::{error, info};
use serde_qs;
use std::collections::HashMap;
use std::ops::Deref;
use std::ops::DerefMut;
use actix_multipart::Multipart;
use futures_util::TryStreamExt;
use crate::server::user_control::*;
use argon2::{
    password_hash::{
        rand_core::OsRng,
        PasswordHash, PasswordHasher, PasswordVerifier, SaltString
    },
    Argon2
};

pub async fn itm_edit(user: Identity,
                      data: web::Data<State>,
                      req: HttpRequest,
                      mut payload: Multipart) -> HttpResponse {
    let mut srv = data.server.lock().unwrap();
    let usr = get_user(srv.deref(), user.id().unwrap());

    let mc = serde_qs::from_str::<MergeColl>(&req.query_string()).unwrap();
    let mut itm = serde_qs::from_str::<Item>(&req.query_string()).unwrap();

    while let Ok(Some(mut field)) = payload.try_next().await {
        while let Ok(Some(chunk)) = field.try_next().await {
            let data = chunk;

            if field.name() == "item" {
                let new_itm : Item = serde_json::from_str(
                    std::str::from_utf8(&data.to_vec()).unwrap())
                    .unwrap();
                println!("Found item");
                itm.merge(&new_itm);
            }
        }
    }
    /* call auth hooks */
    {
        let routes = srv.internals.safe_strstr("itm_auth_hook", &HashMap::new());
        for route in routes {
            if !call_itm_auth_hook(
                &mut srv,
                &route.1,
                &usr,
                &mc.collection,
                itm.id,
                Some(itm.clone()),
                false,
            ) {
                return HttpResponse::Forbidden().into();
            }
        }
    }

    itm.normalize_negated();

    if srv.itm.contains_key(&mc.collection) {
        let coll = srv.itm.get_mut(&mc.collection).unwrap();
        let mut itm_clone = itm.clone();

        let old_itm = coll.get(itm.id);
        if mc.collection == "user" &&
           old_itm != None &&
           (itm.strs.contains_key("password") || itm.strs.contains_key("salt")) {
            error!("Can't edit password directly");
            return HttpResponse::Ok().body(
                    serde_json::to_string(&ProcessResult {
                        succeeded: false,
                        error: "Can't edit password directly".to_string(),
                    }).unwrap());
        }

        if mc.collection == "user" && old_itm.is_none() {
            /* Add salt when creating new user */
            let salt = SaltString::generate(&mut OsRng);
            itm_clone.set_str("salt", &salt.to_string());
        }

        if mc.collection == "user" &&
           old_itm != None &&
           itm.strs.contains_key("__password") &&
           itm.strs.contains_key("__new_password1") &&
           itm.strs.contains_key("__new_password2") {
            if old_itm.as_ref().unwrap().safe_str("password", "") !=
                 itm.safe_str("__password", "") ||
               itm.safe_str("__new_password1", "<bad1>") !=
                 itm.safe_str("__new_password2", "<bad2>") {
                error!("Password change challenge failed");
                return HttpResponse::Ok().body(
                    serde_json::to_string(&ProcessResult {
                        succeeded: false,
                        error: "Password change challenge failed".to_string(),
                    }).unwrap());
            }
            let new_pw = itm.safe_str("__new_password1", "");
            itm_clone.strs.remove("__password");
            itm_clone.strs.remove("__new_password1");
            itm_clone.strs.remove("__new_password2");
            itm_clone.set_str("password", &new_pw);
        }

        coll.set(itm.id, itm_clone, mc.merge);
        info!("Collection {} element {} set", mc.collection, itm.id);

        /* call hooks */
        {
            let routes = srv
                .internals
                .safe_strstr("collection_hook", &HashMap::new());
            for route in routes {
                let parts: Vec<&str> = route.1.split(":").collect();
                if parts[0] == mc.collection {
                    call_item_route(srv.deref_mut(), &parts[1], &mc.collection, itm.id, false);
                }
            }
        }

        write_data(&srv);
        return HttpResponse::Ok().body(
            serde_json::to_string(&ProcessResult {
                succeeded: true,
                error: "".to_string(),
            }).unwrap());
    } else {
        error!("Collection {} doesn't exist", mc.collection);
    }

    return HttpResponse::BadRequest().into();
}

pub async fn itm_del(user: Identity, data: web::Data<State>, req: HttpRequest) -> impl Responder {
    let mut srv = data.server.lock().unwrap();
    let usr = get_user(srv.deref(), user.id().unwrap());

    let mc = serde_qs::from_str::<MergeColl>(&req.query_string()).unwrap();
    let itm = serde_qs::from_str::<Item>(&req.query_string()).unwrap();

    /* call auth hooks */
    {
        let routes = srv.internals.safe_strstr("itm_auth_hook", &HashMap::new());
        for route in routes {
            if !call_itm_auth_hook(&mut srv, &route.1, &usr, &mc.collection, itm.id, None, true) {
                return HttpResponse::Forbidden().into();
            }
        }
    }

    if srv.itm.contains_key(&mc.collection) {
        let coll = srv.itm.get_mut(&mc.collection).unwrap();
        if coll.del(itm.id) {
            info!("Collection {} element {} removed", mc.collection, itm.id);
            write_data(srv.deref_mut());
            return HttpResponse::Ok();
        }
    } else {
        error!("Collection {} doesn't exist", mc.collection);
    }

    return HttpResponse::BadRequest();
}

pub async fn itm_list(user: Identity, data: web::Data<State>, req: HttpRequest) -> HttpResponse {
    let srv = data.server.lock().unwrap();
    let usr = get_user(srv.deref(), user.id().unwrap());

    let lq = serde_qs::from_str::<ListQuery>(&req.query_string()).unwrap();

    if !srv.itm.contains_key(&lq.collection) {
        error!("Collection {} doesn't exist", lq.collection);
        return HttpResponse::BadRequest().into();
    }

    let coll: &Collection = &srv.itm[&lq.collection];
    let mut map: HashMap<u64, Item> = HashMap::new();

    if lq.id != u64::MAX {
        let res = coll.get(lq.id);
        if res == None {
            error!(
                "Collection {} requested element {} doesn't exist",
                lq.collection, lq.id
            );
            return HttpResponse::BadRequest().into();
        }

        if lq.limit == u64::MAX || lq.limit >= 1 {
            map.insert(lq.id, res.unwrap());
            info!(
                "Collection {} requested element {} limit {}",
                lq.collection, lq.id, lq.limit
            );
        }
    } else if lq.id_min != u64::MAX || lq.id_max != u64::MAX {
        map = coll.get_range(lq.id_min, lq.id_max, lq.limit);
        info!(
            "Collection {} requested range {} - {} limit {}",
            lq.collection, lq.id_min, lq.id_max, lq.limit
        );
    } else if lq.id_list.len() > 0 {
        for id in lq.id_list {
            let res = coll.get(id);
            if res != None {
                map.insert(id, res.unwrap());
            }
        }
        info!("Collection {} requested list of IDs", lq.collection);
    } else {
        info!("Collection {} unknown filter", lq.collection);
    }

    /* itm filter hooks */
    {
        let routes = srv
            .internals
            .safe_strstr("itm_list_filter_hook", &HashMap::new());
        for route in routes {
            call_itm_list_filter_hook(&srv, &route.1, &usr, &lq.collection, &lq.context, &mut map);
        }
    }

    HttpResponse::Ok().body(serde_json::to_string(&map).unwrap())
}
