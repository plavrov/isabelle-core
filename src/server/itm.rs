use crate::handler::route::*;
use crate::server::user_control::*;
use crate::state::state::*;
use crate::state::store::Store;
use actix_identity::Identity;
use actix_multipart::Multipart;
use actix_web::{web, HttpRequest, HttpResponse, Responder};
use futures_util::TryStreamExt;
use isabelle_dm::data_model::item::Item;
use isabelle_dm::data_model::list_query::ListQuery;
use isabelle_dm::data_model::merge_coll::MergeColl;
use isabelle_dm::data_model::process_result::ProcessResult;
use log::{error, info};
use serde_qs;
use std::collections::HashMap;
use std::ops::DerefMut;

pub async fn itm_edit(
    user: Identity,
    data: web::Data<State>,
    req: HttpRequest,
    mut payload: Multipart,
) -> HttpResponse {
    let mut srv = data.server.lock().unwrap();
    let usr = get_user(&mut srv, user.id().unwrap());

    let mc = serde_qs::from_str::<MergeColl>(&req.query_string()).unwrap();
    let mut itm = serde_qs::from_str::<Item>(&req.query_string()).unwrap();

    while let Ok(Some(mut field)) = payload.try_next().await {
        while let Ok(Some(chunk)) = field.try_next().await {
            let data = chunk;

            if field.name() == "item" {
                let v = &data.to_vec();
                let strv = std::str::from_utf8(v).unwrap_or("{}");
                let new_itm: Item = serde_json::from_str(strv).unwrap_or(Item::new());
                itm.merge(&new_itm);
            }
        }
    }
    /* call auth hooks */
    {
        let routes = srv
            .rw
            .get_internals()
            .safe_strstr("itm_auth_hook", &HashMap::new());
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

    if srv.has_collection(&mc.collection) {
        let srv_mut = srv.deref_mut();
        let mut itm_clone = itm.clone();

        let old_itm = srv_mut.rw.get_item(&mc.collection, itm.id);
        /* call pre edit ooks */
        {
            let routes = srv_mut
                .rw
                .get_internals()
                .safe_strstr("item_pre_edit_hook", &HashMap::new());
            for route in &routes {
                let parts: Vec<&str> = route.1.split(":").collect();
                if parts[0] == mc.collection {
                    let res = call_item_pre_edit_hook(
                        srv_mut,
                        parts[1],
                        &mc.collection,
                        old_itm.clone(),
                        &mut itm_clone,
                        false,
                    );
                    if !res.succeeded {
                        info!("Item pre edit hook failed: {}", parts[1]);
                        let s = serde_json::to_string(&res);
                        return HttpResponse::Ok().body(s.unwrap_or("{}".to_string()));
                    }
                }
            }
        }

        /* call hooks */
        if old_itm != None {
            let routes = srv_mut
                .rw
                .get_internals()
                .safe_strstr("item_post_edit_hook", &HashMap::new());
            for route in routes {
                let parts: Vec<&str> = route.1.split(":").collect();
                if parts[0] == mc.collection {
                    call_item_post_edit_hook(srv_mut, &parts[1], &mc.collection, itm.id, true);
                }
            }
        }

        srv_mut.rw.set_item(&mc.collection, &itm_clone, mc.merge);
        info!("Collection {} element {} set", mc.collection, itm.id);

        /* call hooks */
        {
            let routes = srv
                .rw
                .get_internals()
                .safe_strstr("item_post_edit_hook", &HashMap::new());
            for route in routes {
                let parts: Vec<&str> = route.1.split(":").collect();
                if parts[0] == mc.collection {
                    call_item_post_edit_hook(
                        srv.deref_mut(),
                        &parts[1],
                        &mc.collection,
                        itm.id,
                        false,
                    );
                }
            }
        }

        //write_data(&srv);
        return HttpResponse::Ok().body(
            serde_json::to_string(&ProcessResult {
                succeeded: true,
                error: "".to_string(),
            })
            .unwrap(),
        );
    } else {
        error!("Collection {} doesn't exist", mc.collection);
    }

    return HttpResponse::BadRequest().into();
}

pub async fn itm_del(user: Identity, data: web::Data<State>, req: HttpRequest) -> impl Responder {
    let mut srv = data.server.lock().unwrap();
    let usr = get_user(&mut srv, user.id().unwrap());

    let mc = serde_qs::from_str::<MergeColl>(&req.query_string()).unwrap();
    let itm = serde_qs::from_str::<Item>(&req.query_string()).unwrap();

    /* call auth hooks */
    {
        let routes = srv
            .rw
            .get_internals()
            .safe_strstr("itm_auth_hook", &HashMap::new());
        for route in routes {
            if !call_itm_auth_hook(&mut srv, &route.1, &usr, &mc.collection, itm.id, None, true) {
                return HttpResponse::Forbidden().into();
            }
        }
    }

    let srv_mut = srv.deref_mut();
    if srv_mut.has_collection(&mc.collection) {
        /* call hooks */
        {
            let routes = srv_mut
                .rw
                .get_internals()
                .safe_strstr("item_post_edit_hook", &HashMap::new());
            for route in routes {
                let parts: Vec<&str> = route.1.split(":").collect();
                if parts[0] == mc.collection {
                    call_item_post_edit_hook(srv_mut, &parts[1], &mc.collection, itm.id, true);
                }
            }
        }

        //let coll = srv_mut.itm.get_mut(&mc.collection).unwrap();
        if srv_mut.rw.del_item(&mc.collection, itm.id) {
            info!("Collection {} element {} removed", mc.collection, itm.id);
            //write_data(srv.deref_mut());
            return HttpResponse::Ok();
        }
    } else {
        error!("Collection {} doesn't exist", mc.collection);
    }

    return HttpResponse::BadRequest();
}

pub async fn itm_list(user: Identity, data: web::Data<State>, req: HttpRequest) -> HttpResponse {
    let mut srv = data.server.lock().unwrap();
    let usr = get_user(&mut srv, user.id().unwrap());

    let lq = serde_qs::from_str::<ListQuery>(&req.query_string()).unwrap();

    if !srv.has_collection(&lq.collection) {
        error!("Collection {} doesn't exist", lq.collection);
        return HttpResponse::BadRequest().into();
    }

    let mut map: HashMap<u64, Item> = HashMap::new();

    if lq.id != u64::MAX {
        let res = srv.rw.get_item(&lq.collection, lq.id);
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
        map = srv
            .rw
            .get_items(&lq.collection, lq.id_min, lq.id_max, lq.limit);
        info!(
            "Collection {} requested range {} - {} limit {}",
            lq.collection, lq.id_min, lq.id_max, lq.limit
        );
    } else if lq.id_list.len() > 0 {
        for id in lq.id_list {
            let res = srv.rw.get_item(&lq.collection, id);
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
            .rw
            .get_internals()
            .safe_strstr("itm_list_filter_hook", &HashMap::new());
        for route in routes {
            call_itm_list_filter_hook(
                &mut srv,
                &route.1,
                &usr,
                &lq.collection,
                &lq.context,
                &mut map,
            );
        }
    }

    HttpResponse::Ok().body(serde_json::to_string(&map).unwrap())
}
