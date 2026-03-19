use wasm_bindgen::prelude::*;
//use web_sys::{HtmlElement, KeyboardEvent, InputEvent, Range, Selection}; // HtmlInputElement mag weg als je die niet meer gebruikt
use std::rc::Rc;
use std::cell::RefCell;

mod interpreter;
mod utils;

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    utils::set_panic_hook();

    let window = web_sys::window().expect("no window");
    let document = window.document().expect("no doc");
    let machine = Rc::new(RefCell::new(interpreter::EcolMachine::new()));

    let history = document.get_element_by_id("history").unwrap();
    let cursor_line = document.get_element_by_id("cursor-line").unwrap().dyn_into::<web_sys::HtmlElement>()?;

    // --- CLOSURE 1: ENTER TOETS ---
    let m = machine.clone();
    let hist = history.clone();
    let cur_keydown = cursor_line.clone();
    let win_keydown = window.clone();

    let keydown_closure = Closure::wrap(Box::new(move |event: web_sys::KeyboardEvent| {
        if event.key() == "Enter" {
            event.prevent_default();
            let command = cur_keydown.inner_text();
            if command.trim().is_empty() { return; }

            let resultaat = m.borrow_mut().execute(&command);
            let oude_hist = hist.inner_html();
            hist.set_inner_html(&format!("{}<br/>ECOL > {}<br/>{}", oude_hist, command, resultaat));
            cur_keydown.set_inner_text("");
            win_keydown.scroll_to_with_x_and_y(0.0, 1000000.0);
        }
    }) as Box<dyn FnMut(web_sys::KeyboardEvent)>);

    cursor_line.add_event_listener_with_callback("keydown", keydown_closure.as_ref().unchecked_ref())?;
    keydown_closure.forget();

    // --- CLOSURE 2: INPUT FILTER (HOOFDLETTERS) ---
    let cur_input_filter = cursor_line.clone();
    let doc_input_filter = document.clone();

    let input_closure = Closure::wrap(Box::new(move |_e: web_sys::InputEvent| {
        let text = cur_input_filter.inner_text();
        let mut new_text = String::new();
        let mut inside_quotes = false;

        for c in text.chars() {
            if c == '"' {
                inside_quotes = !inside_quotes;
                new_text.push(c);
            } else if inside_quotes {
                new_text.push(c);
            } else {
                new_text.push_str(&c.to_uppercase().to_string());
            }
        }

        if text != new_text {
            cur_input_filter.set_inner_text(&new_text);

            // De "Hamer-methode" voor de cursor:
            if let Ok(Some(selection)) = doc_input_filter.get_selection() {
                let range = web_sys::Range::new().unwrap();

                // 1. Selecteer de inhoud van onze typ-regel
                range.select_node_contents(&cur_input_filter).unwrap();

                // 2. In plaats van collapse op de range, gebruiken we de SELECTION
                // Dit dwingt de browser om het focus-punt naar het einde van de selectie te verplaatsen
                let _ = selection.remove_all_ranges();
                let _ = selection.add_range(&range);
                let _ = selection.collapse_to_end();
            }
        }

    }) as Box<dyn FnMut(web_sys::InputEvent)>);

    cursor_line.add_event_listener_with_callback("input", input_closure.as_ref().unchecked_ref())?;
    input_closure.forget();

    Ok(())
}