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

use crate::server::user_control::*;

pub async fn itm_edit(user: Identity, data: web::Data<State>, req: HttpRequest) -> impl Responder {
    let mut srv = data.server.lock().unwrap();
    let usr = get_user(srv.deref(), user.id().unwrap());

    let mc = serde_qs::from_str::<MergeColl>(&req.query_string()).unwrap();
    let mut itm = serde_qs::from_str::<Item>(&req.query_string()).unwrap();

    /* call auth hooks */
    {
        let routes = srv.internals.safe_strstr("itm_auth_hook", &HashMap::new());
        for route in routes {
            if !call_itm_auth_hook(&mut srv, &route.1, &usr, &mc.collection, itm.id, false) {
                return HttpResponse::Forbidden().into();
            }
        }
    }

    itm.normalize_negated();

    if srv.itm.contains_key(&mc.collection) {
        let coll = srv.itm.get_mut(&mc.collection).unwrap();
        coll.set(itm.id, itm.clone(), mc.merge);
        info!("Collection {} element {} set", mc.collection, itm.id);

        /* call hooks */
        {
            let routes = srv.internals.safe_strstr("collection_hook", &HashMap::new());
            for route in routes {
                let parts: Vec<&str> = route.1.split(":").collect();
                if parts[0] == mc.collection {
                    call_item_route(srv.deref_mut(), &parts[1], &mc.collection, itm.id, false);
                }
            }
        }

        write_data(&srv);
        return HttpResponse::Ok();
    } else {
        error!("Collection {} doesn't exist", mc.collection);
    }

    return HttpResponse::BadRequest();
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
            if !call_itm_auth_hook(&mut srv, &route.1, &usr, &mc.collection, itm.id, true) {
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
        let routes = srv.internals.safe_strstr("itm_list_filter_hook", &HashMap::new());
        for route in routes {
            call_itm_list_filter_hook(&srv, &route.1, &usr, &lq.collection, &lq.context, &mut map);
        }
    }

    HttpResponse::Ok().body(serde_json::to_string(&map).unwrap())
}
