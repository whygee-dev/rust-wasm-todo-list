mod utils;
mod types;

extern crate wasm_bindgen;
use wasm_bindgen::prelude::*;
use wasm_bindgen::prelude::wasm_bindgen;
use utils::*;

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;


#[wasm_bindgen(start)]
pub fn main_js() -> Result<(), JsValue> {
    #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();

    init();

    {
        let closure = Closure::<dyn FnMut(_)>::new(move |_: web_sys::MessageEvent| {
            on_new_todo_button_click()
        });
    
        get_new_todo_button().add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;
        
        closure.forget();
    }
   

    Ok(())
}
