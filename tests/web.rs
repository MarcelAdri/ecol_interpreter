//! Test suite for the Web and headless browsers.

#![cfg(target_arch = "wasm32")]

extern crate wasm_bindgen_test;
use wasm_bindgen_test::*;
use ecol_interpreter::{EcolMachine, Waarde};

wasm_bindgen_test_configure!(run_in_browser);

// ---------------------------------------------------------------------------
// Hulpfuncties
// ---------------------------------------------------------------------------

fn eval_f32(expressie: &str) -> f32 {
    let mut machine = EcolMachine::new();
    match machine.solve_number_expression(expressie).unwrap() {
        Waarde::Getal(f) => f,
        _ => panic!("verwachtte Waarde::Float"),
    }
}

fn eval_f32_met_context(context: &[&str], expressie: &str) -> f32 {
    let mut machine = EcolMachine::new();
    for opdracht in context {
        machine.execute(opdracht);
    }
    match machine.solve_number_expression(expressie).unwrap() {
        Waarde::Getal(f) => f,
        _ => panic!("verwachtte Waarde::Float"),
    }
}

// ---------------------------------------------------------------------------
// Sanity-check
// ---------------------------------------------------------------------------

#[wasm_bindgen_test]
fn pass() {
    assert_eq!(1 + 1, 2);
}

// ---------------------------------------------------------------------------
// Groep 1 – Enkelvoudige getallen
// ---------------------------------------------------------------------------

#[wasm_bindgen_test]
fn getal_heel() {
    assert_eq!(eval_f32("5"), 5.0f32);
}

#[wasm_bindgen_test]
fn getal_decimaal() {
    assert_eq!(eval_f32("2.5"), 2.5f32);
}

#[wasm_bindgen_test]
fn getal_nul() {
    assert_eq!(eval_f32("0"), 0.0f32);
}

// ---------------------------------------------------------------------------
// Groep 2 – Basisoperaties
// ---------------------------------------------------------------------------

#[wasm_bindgen_test]
fn optellen() {
    assert_eq!(eval_f32("3 + 2"), 5.0f32);
}

#[wasm_bindgen_test]
fn aftrekken() {
    assert_eq!(eval_f32("10 - 3"), 7.0f32);
}

#[wasm_bindgen_test]
fn vermenigvuldigen() {
    assert_eq!(eval_f32("4 * 3"), 12.0f32);
}

#[wasm_bindgen_test]
fn delen() {
    assert_eq!(eval_f32("10 / 2"), 5.0f32);
}

#[wasm_bindgen_test]
fn delen_met_decimaal_resultaat() {
    assert_eq!(eval_f32("1 / 4"), 0.25f32);
}

#[wasm_bindgen_test]
fn optellen_decimalen() {
    assert_eq!(eval_f32("2.5 + 1.5"), 4.0f32);
}

// ---------------------------------------------------------------------------
// Groep 3 – Volgorde van bewerkingen (zonder haakjes)
// ---------------------------------------------------------------------------

#[wasm_bindgen_test]
fn vermenigvuldiging_voor_optelling() {
    assert_eq!(eval_f32("2 + 3 * 4"), 14.0f32);
}

#[wasm_bindgen_test]
fn deling_voor_aftrekking() {
    assert_eq!(eval_f32("10 - 6 / 2"), 7.0f32);
}

#[wasm_bindgen_test]
fn aftrekken_links_naar_rechts() {
    assert_eq!(eval_f32("10 - 3 - 2"), 5.0f32);
}

#[wasm_bindgen_test]
fn delen_links_naar_rechts() {
    assert_eq!(eval_f32("12 / 4 / 3"), 1.0f32);
}

#[wasm_bindgen_test]
fn gemengde_operaties() {
    assert_eq!(eval_f32("2 * 3 + 4 * 5"), 26.0f32);
}

// ---------------------------------------------------------------------------
// Groep 4 – Haakjes
// ---------------------------------------------------------------------------

#[wasm_bindgen_test]
fn haakjes_optelling_voor_vermenigvuldiging() {
    assert_eq!(eval_f32("(2 + 3) * 4"), 20.0f32);
}

#[wasm_bindgen_test]
fn haakjes_aftrekking_voor_vermenigvuldiging() {
    assert_eq!(eval_f32("(10 - 3) * 2"), 14.0f32);
}

#[wasm_bindgen_test]
fn geneste_haakjes() {
    assert_eq!(eval_f32("(2 + (3 * 4))"), 14.0f32);
}

#[wasm_bindgen_test]
fn haakjes_in_noemer() {
    assert_eq!(eval_f32("12 / (2 + 1)"), 4.0f32);
}

// ---------------------------------------------------------------------------
// Groep 5 – Variabelen in numerieke expressies
// ---------------------------------------------------------------------------

#[wasm_bindgen_test]
fn variabele_in_optelling() {
    assert_eq!(eval_f32_met_context(&["a := 5"], "a + 3"), 8.0f32);
}

#[wasm_bindgen_test]
fn twee_variabelen_vermenigvuldigen() {
    assert_eq!(eval_f32_met_context(&["a := 6", "b := 2"], "a * b"), 12.0f32);
}

#[wasm_bindgen_test]
fn variabele_in_haakjes() {
    assert_eq!(eval_f32_met_context(&["n := 4"], "(n + 1) * 3"), 15.0f32);
}

#[wasm_bindgen_test]
fn variabele_naar_variabele() {
    assert_eq!(eval_f32_met_context(&["a := 7"], "a"), 7.0f32);
}
