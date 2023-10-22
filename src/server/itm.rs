use crate::state::collection::Collection;
use std::collections::HashMap;
use isabelle_dm::data_model::item::Item;
use isabelle_dm::data_model::merge_coll::MergeColl;
use isabelle_dm::data_model::list_query::ListQuery;
use std::ops::Deref;
use serde_qs;
use actix_identity::Identity;
use actix_web::{web, HttpRequest, HttpResponse, Responder};
use crate::state::state::*;
use log::{error, info};

use crate::server::user_control::*;

pub async fn itm_edit(
    user: Identity,
    data: web::Data<State>,
    req: HttpRequest,
) -> impl Responder {
    let mut srv = data.server.lock().unwrap();
    let usr = get_user(srv.deref(), user.id().unwrap());

    if check_role(usr, "admin") {
        return HttpResponse::Forbidden();
    }

    let mc = serde_qs::from_str::<MergeColl>(&req.query_string()).unwrap();
    let itm = serde_qs::from_str::<Item>(&req.query_string()).unwrap();

    if srv.itm.contains_key(&mc.collection) {
        let coll = srv.itm.get_mut(&mc.collection).unwrap();
        coll.set(itm.id, itm.clone(), mc.merge);
        info!("Collection {} element {} set", mc.collection, itm.id);
        return HttpResponse::Ok();
    } else {
        error!("Collection {} doesn't exist", mc.collection);
    }

    return HttpResponse::BadRequest();
}

pub async fn itm_del(
    user: Identity,
    data: web::Data<State>,
    req: HttpRequest,
) -> impl Responder {
    let mut srv = data.server.lock().unwrap();
    let usr = get_user(srv.deref(), user.id().unwrap());

    if check_role(usr, "admin") {
        return HttpResponse::Forbidden();
    }

    let mc = serde_qs::from_str::<MergeColl>(&req.query_string()).unwrap();
    let itm = serde_qs::from_str::<Item>(&req.query_string()).unwrap();

    if srv.itm.contains_key(&mc.collection) {
        let coll = srv.itm.get_mut(&mc.collection).unwrap();
        if coll.del(itm.id) {
            info!("Collection {} element {} removed", mc.collection, itm.id);
            return HttpResponse::Ok();
        }
    } else {
        error!("Collection {} doesn't exist", mc.collection);
    }

    return HttpResponse::BadRequest();
}

pub async fn itm_list(
    user: Identity,
    data: web::Data<State>,
    req: HttpRequest,
) -> HttpResponse {
    let srv = data.server.lock().unwrap();
    let usr = get_user(srv.deref(), user.id().unwrap());

    if check_role(usr, "admin") {
        return HttpResponse::Unauthorized().into();
    }

    let lq = serde_qs::from_str::<ListQuery>(&req.query_string()).unwrap();

    if !srv.itm.contains_key(&lq.collection) {
        error!("Collection {} doesn't exist", lq.collection);
        return HttpResponse::BadRequest().into();
    }

    let coll : &Collection = &srv.itm[&lq.collection];
    let mut map : HashMap<u64, Item> = HashMap::new();

    if lq.id != u64::MAX {
        let res = coll.get(lq.id);
        if res == None {
            error!("Collection {} requested element {} doesn't exist", lq.collection, lq.id);
            return HttpResponse::BadRequest().into();
        }

        if lq.limit == u64::MAX || lq.limit >= 1 {
            map.insert(lq.id, res.unwrap());
            info!("Collection {} requested element {} limit {}", lq.collection,
                  lq.id, lq.limit);
        }
    } else if lq.id_min != u64::MAX || lq.id_max != u64::MAX {
        map = coll.get_range(lq.id_min, lq.id_max, lq.limit);
        info!("Collection {} requested range {} - {} limit {}", lq.collection,
              lq.id_min, lq.id_max, lq.limit);
    } else {
        error!("Collection {} unknown filter", lq.collection);
        return HttpResponse::BadRequest().into();
    }

    HttpResponse::Ok().body(serde_json::to_string(&map).unwrap())
}
