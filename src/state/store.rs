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
use async_trait::async_trait;
use isabelle_dm::data_model::item::Item;
use isabelle_dm::data_model::list_result::ListResult;
use std::collections::HashMap;

#[async_trait]
pub trait Store {
    async fn connect(&mut self, addr: &str, altaddr: &str);
    async fn disconnect(&mut self);

    async fn get_collections(&mut self) -> Vec<String>;
    async fn get_item_ids(&mut self, collection: &str) -> HashMap<u64, bool>;

    async fn get_all_items(&mut self, collection: &str, sort_key: &str, filter: &str)
        -> ListResult;
    async fn get_item(&mut self, collection: &str, id: u64) -> Option<Item>;
    async fn get_items(
        &mut self,
        collection: &str,
        id_min: u64,
        id_max: u64,
        sort_key: &str,
        filter: &str,
        skip: u64,
        limit: u64,
    ) -> ListResult;

    async fn set_item(&mut self, collection: &str, itm: &Item, merge: bool);
    async fn del_item(&mut self, collection: &str, id: u64) -> bool;

    async fn get_credentials(&mut self) -> String;
    async fn get_pickle(&mut self) -> String;

    async fn get_internals(&mut self) -> Item;

    async fn get_settings(&mut self) -> Item;
    async fn set_settings(&mut self, itm: Item);
}
