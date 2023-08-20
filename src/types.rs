extern crate serde;
use self::serde::{Serialize, Deserialize};
use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum TodoItemState {
    DONE,
    DOING,
    TODO
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TodoItem {
    pub id: u64,
    pub name: String,
    pub state: TodoItemState,
}
