extern crate getrandom;
extern crate lazy_static;
extern crate web_sys;
extern crate serde;
extern crate wasm_bindgen;
extern crate gloo_utils;
extern crate serde_json;
extern crate parking_lot;

use std::convert::TryInto;
use std::num::ParseIntError;

use self::serde_json::{to_string, from_str};
use self::gloo_utils::format::JsValueSerdeExt;
use self::wasm_bindgen::prelude::*;
use self::web_sys::*;
use self::lazy_static::*;
use self::getrandom::*;
use self::parking_lot::Mutex;


use crate::types::*;

lazy_static! {
    static ref LIST: Mutex<Vec<TodoItem>> = Mutex::new(Vec::new());
}

const STORAGE_KEY: &str = "todos";

pub fn u64_from_slice(slice: &[u8]) -> u64 {
    u64::from_ne_bytes(slice[..8].try_into().unwrap())
}


fn integer_part(repr: &str) -> Result<u64, ParseIntError> {
    match repr.split('.').next() {
        Some(x) => x.parse(),
        None => unreachable!()
    }
}

fn get_random_buf() -> Result<u64, getrandom::Error> {
    let mut buf = [0u8; 64];
    getrandom(&mut buf)?;

    Ok(u64_from_slice(&buf))
}

fn get_window() -> Window {
    web_sys::window().expect("no global `window` exists")
}

fn get_document() -> Document {
    get_window().document().expect("should have a document on window")
}

pub fn get_new_todo_button() -> Element {
    get_document().get_element_by_id("new-todo").expect("new todo button should be present")
}

fn get_todo_items_list() -> Element {
    get_document().get_element_by_id("todo-items").expect("should get the element")
}

fn get_doing_items_list() -> Element {
    get_document().get_element_by_id("doing-items").expect("should get the element")
}

fn get_done_items_list() -> Element {
    get_document().get_element_by_id("done-items").expect("should get the element")
}

fn find_item_index(item_id: u64) -> usize {
    LIST.lock().iter().position(|x| (*x).id == item_id).unwrap()
}


fn get_local_storage() -> Storage {
    get_window().local_storage().unwrap().unwrap()
}

pub fn update_state(item_id: u64, state: TodoItemState) {
    let index = find_item_index(item_id);
    let item = &mut LIST.lock()[index];
    item.state = state;

    unsafe {
        get_local_storage().set_item(STORAGE_KEY, to_string(&*LIST.data_ptr()).unwrap().as_str()).unwrap();
    }
}

pub fn delete_todo(item_id: u64) {
    let index = find_item_index(item_id);
    LIST.lock().remove(index);

    get_local_storage().set_item(STORAGE_KEY, to_string(&LIST.lock().clone()).unwrap().as_str()).unwrap();
}

pub fn on_new_todo_button_click() {
    let textarea = get_document().get_element_by_id("todo-textarea").unwrap().dyn_into::<web_sys::HtmlTextAreaElement>().unwrap();

    let textarea_val = textarea.value();

    if textarea_val.is_empty() {
        return;
    }
    
    new_todo(textarea_val.as_str());

    textarea.set_value("");

    textarea.focus().unwrap();
}

fn create_action_elements_container() -> Element {
    get_document().create_element("div").unwrap()
}


fn create_pass_to_doing_element(element: &Element) -> Element {
    let doing_button = get_document().create_element("i").unwrap();

    doing_button.set_attribute("class","bx bx-play").unwrap();
    
    {
        let cloned_element = element.clone();
        let element_id = integer_part(cloned_element.get_attribute("id").unwrap().as_str()).unwrap();
        let element_name = cloned_element.text_content().unwrap();

        let closure = Closure::<dyn FnMut(_)>::new(move |_: web_sys::MessageEvent| {
            cloned_element.remove();
            update_state(element_id, TodoItemState::DOING);
            get_doing_items_list().append_child(&create_todo(element_id, element_name.as_str(), TodoItemState::DOING)).unwrap();
            
        });
    
        doing_button.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref()).unwrap();
        
        closure.forget();
    }

    doing_button
}

fn create_pass_to_done_element(element: &Element) -> Element {
    let done_button = get_document().create_element("i").unwrap();

    done_button.set_attribute("class","bx bx-check").unwrap();
    
    {
        let cloned_element = element.clone();
        let element_id = integer_part(cloned_element.get_attribute("id").unwrap().as_str()).unwrap();
        let element_name = cloned_element.text_content().unwrap();

        let closure = Closure::<dyn FnMut(_)>::new(move |_: web_sys::MessageEvent| {
            cloned_element.remove();
            update_state(element_id, TodoItemState::DONE);
            get_done_items_list().append_child(&create_todo(element_id, element_name.as_str(), TodoItemState::DONE)).unwrap();
            
        });
    
        done_button.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref()).unwrap();
        
        closure.forget();
    }

    done_button
}

fn create_delete_button(element: &Element) -> Element {
    let delete_button = get_document().create_element("i").unwrap();

    delete_button.set_attribute("class","bx bx-trash").unwrap();

    {
        let cloned_element = element.clone();
        let element_id = integer_part(cloned_element.get_attribute("id").unwrap().as_str()).unwrap();

        let closure = Closure::<dyn FnMut(_)>::new(move |_: web_sys::MessageEvent| {
            cloned_element.remove();
            delete_todo(element_id);
        });

        delete_button.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref()).unwrap();
            
        closure.forget();
    }

    delete_button
}

fn create_todo(id: u64, name: &str, state: TodoItemState) -> Element {
    let li = get_document().create_element("li").unwrap();
    li.set_text_content(Some(name));
    li.set_attribute("id", id.to_string().as_str()).unwrap();

    let actions_element = create_action_elements_container();

    actions_element.append_child(&create_delete_button(&li)).unwrap();

    match state {
        TodoItemState::TODO => {actions_element.append_child(&create_pass_to_doing_element(&li)).unwrap();},
        TodoItemState::DOING => {actions_element.append_child(&create_pass_to_done_element(&li)).unwrap();},
        _ => {}
    }

    li.append_child(&actions_element).unwrap();
    
    li

}

pub fn new_todo(
    name: &str
) -> () {
    let id = get_random_buf().unwrap();

    LIST.lock().push(TodoItem {
        id,
        name: name.to_string(),
        state: TodoItemState::TODO
    });
   
    get_todo_items_list().append_child(&create_todo(id, name, TodoItemState::TODO)).unwrap();

    get_local_storage().set_item(STORAGE_KEY, to_string(&LIST.lock().clone()).unwrap().as_str()).unwrap();
}

pub fn init() {
    let raw_items_in_storage = match get_local_storage().get_item(STORAGE_KEY) {
        Ok(option) => match option {
            Some(value) => value,
            None => "[]".to_string()
        }
        Err(_) => "[]".to_string()
    };

    let items_in_storage: Result<Vec<TodoItem>, _> = from_str(raw_items_in_storage.as_str());
    let items_in_storage_unwrapped = items_in_storage.unwrap();

    LIST.lock().extend(items_in_storage_unwrapped.iter().cloned());

    render()
}

pub fn render() {
    for item in &LIST.lock().clone() {
        let li = create_todo(item.id, item.name.as_str(), item.state.clone());

        match item.state {
            TodoItemState::TODO => {get_todo_items_list().append_child(&li).unwrap();},
            TodoItemState::DOING => {get_doing_items_list().append_child(&li).unwrap();},
            TodoItemState::DONE => {get_done_items_list().append_child(&li).unwrap();},
        }
    }
}


#[wasm_bindgen]
pub fn get_list() -> JsValue {
    let list = LIST.lock().clone();

    JsValue::from_serde(&list).unwrap()
}

