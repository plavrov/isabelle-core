/*
 * Isabelle project
 *
 * Copyright 2023-2024 Maxim Menshikov
 *
 * Permission is hereby granted, free of charge, to any person obtaining
 * a copy of this software and associated documentation files (the “Software”),
 * to deal in the Software without restriction, including without limitation
 * the rights to use, copy, modify, merge, publish, distribute, sublicense,
 * and/or sell copies of the Software, and to permit persons to whom the
 * Software is furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included
 * in all copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED “AS IS”, WITHOUT WARRANTY OF ANY KIND, EXPRESS
 * OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
 * FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
 * DEALINGS IN THE SOFTWARE.
 */
use isabelle_plugin_api::api::*;
use isabelle_plugin_api::plugin_pool::PluginPool;
use crate::handler::route::call_collection_read_hook;
use crate::state::store::Store;
use crate::state::store_local::*;
use crate::state::store_mongo::*;
use std::collections::HashMap;

pub struct Data {
    pub file_rw: StoreLocal,
    pub rw: StoreMongo,
    pub gc_path: String,
    pub py_path: String,
    pub data_path: String,
    pub public_url: String,
    pub port: u16,
    pub plugin_pool: PluginPool,
    pub plugin_api: PluginApi,

    pub item_pre_edit_hook: HashMap<String, IsabelleRouteItemPreEditHook>,
    pub item_post_edit_hook: HashMap<String, IsabelleRouteItemPostEditHook>,
    pub item_auth_hook: HashMap<String, IsabelleRouteItemAuthHook>,
    pub item_list_filter_hook: HashMap<String, IsabelleRouteItemListFilterHook>,
    pub url_hook: HashMap<String, IsabelleRouteUrlHook>,
    pub unprotected_url_hook: HashMap<String, IsabelleRouteUnprotectedUrlHook>,
    pub unprotected_url_post_hook: HashMap<String, IsabelleRouteUnprotectedUrlPostHook>,
    pub collection_read_hook: HashMap<String, IsabelleRouteCollectionReadHook>,
    pub call_otp_hook: HashMap<String, IsabelleRouteCallOtpHook>,
}

impl Data {
    pub fn new() -> Self {
        Self {
            file_rw: StoreLocal::new(),
            rw: StoreMongo::new(),

            gc_path: "".to_string(),
            py_path: "".to_string(),
            data_path: "".to_string(),
            public_url: "".to_string(),
            port: 8090,
            plugin_pool: PluginPool {},
            plugin_api: PluginApi::new(),

            item_pre_edit_hook: HashMap::new(),
            item_post_edit_hook: HashMap::new(),
            item_auth_hook: HashMap::new(),
            item_list_filter_hook: HashMap::new(),
            url_hook: HashMap::new(),
            unprotected_url_hook: HashMap::new(),
            unprotected_url_post_hook: HashMap::new(),
            collection_read_hook: HashMap::new(),
            call_otp_hook: HashMap::new(),
        }
    }

    pub fn has_collection(&mut self, collection: &str) -> bool {
        return self.rw.collections.contains_key(collection);
    }

    pub async fn init_checks(&mut self) {
        let internals = self.rw.get_internals().await;
        let routes = internals.safe_strstr("collection_read_hook", &HashMap::new());
        let collections = self.rw.get_collections().await;
        for collection in &collections {
            let items = self.rw.get_item_ids(collection).await;
            for itm in items {
                let loaded_item_opt = self.rw.get_item(collection, itm.0).await;
                if loaded_item_opt.is_none() {
                    continue;
                }
                let mut loaded_item = loaded_item_opt.unwrap();
                let mut should_be_saved = false;
                for route in &routes {
                    if call_collection_read_hook(self, &route.1, collection, &mut loaded_item).await {
                        should_be_saved = true;
                    }
                }
                if should_be_saved {
                    self.rw.set_item(collection, &loaded_item, false).await;
                }
            }
        }
    }
}
