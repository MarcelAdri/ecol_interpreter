use wasm_bindgen::prelude::*;
use std::rc::Rc;
use std::cell::RefCell;

mod interpreter;
mod utils;

pub use interpreter::EcolMachine;

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    utils::set_panic_hook();

    let window = web_sys::window().expect("no window");
    let document = window.document().expect("no doc");
    let machine = Rc::new(RefCell::new(interpreter::EcolMachine::new()));

    let history = document.get_element_by_id("history").unwrap();
    let cursor_line = document.get_element_by_id("cursor-line").unwrap().dyn_into::<web_sys::HtmlElement>()?;

    history.set_inner_html(&format!(
        "ECOL INTERPRETER v{}<br/>Typ 'HELP' voor instructies.",
        env!("CARGO_PKG_VERSION")
    ));

    // --- CLOSURE 1: ENTER TOETS ---
    let m = machine.clone();
    let hist = history.clone();
    let cur_keydown = cursor_line.clone();
    let win_keydown = window.clone();

    let hist_cb = hist.clone();          // ← extra clone voor de output-callback
    let win_cb = win_keydown.clone();

    let keydown_closure = Closure::wrap(Box::new(move |event: web_sys::KeyboardEvent| {
        if event.key() == "Enter" {
            event.prevent_default();
            let command = cur_keydown.inner_text();
            if command.trim().is_empty() { return; }

            // Prompt-echo eerst tonen, zodat callback-output daaronder verschijnt
            let oude_hist = hist.inner_html();
            hist.set_inner_html(&format!("{}<br/>ECOL > {}<br/>", oude_hist, command));

            let resultaat = m.borrow_mut().execute(&command, &mut |regel| {
                if regel == "\x0C" {
                    hist_cb.set_inner_html((&format!(
                        "ECOL INTERPRETER v{}<br/>Typ 'HELP' voor instructies.",
                        env!("CARGO_PKG_VERSION")
                    )));   // scherm leegmaken
                } else {
                    let regel_html = regel.replace('\n', "<br/>");
                    let oude = hist_cb.inner_html();
                    hist_cb.set_inner_html(&format!("{}{}", oude, regel_html));
                }
                win_cb.scroll_to_with_x_and_y(0.0, 1_000_000.0);
            });

            if !resultaat.is_empty() {
                let resultaat_html = resultaat.replace('\n', "<br/>");
                let oude = hist.inner_html();
                hist.set_inner_html(&format!("{}{}<br/>", oude, resultaat_html));
            }
            cur_keydown.set_inner_text("");
            win_keydown.scroll_to_with_x_and_y(0.0, 1000000.0);
        }
    }) as Box<dyn FnMut(web_sys::KeyboardEvent)>);

    cursor_line.add_event_listener_with_callback("keydown", keydown_closure.as_ref().unchecked_ref())?;
    keydown_closure.forget();

    // --- CLOSURE 2: INPUT FILTER (ZONDER HOOFDLETTERS) ---
    let cur_input_filter = cursor_line.clone();

    let input_closure = Closure::wrap(Box::new(move |_e: web_sys::InputEvent| {
        let text = cur_input_filter.inner_text();

        // Geen omzetting meer naar hoofdletters:
        // de invoer blijft gewoon zoals de gebruiker die typt.
        if text.is_empty() {
            // optioneel: hier kun je extra logica zetten
        }
    }) as Box<dyn FnMut(web_sys::InputEvent)>);

    cursor_line.add_event_listener_with_callback("input", input_closure.as_ref().unchecked_ref())?;
    input_closure.forget();

    Ok(())
}