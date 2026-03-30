# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

ECOL is a Dutch-language programming language interpreter compiled to WebAssembly, running in a web browser as a retro terminal interface.

## Build & Test Commands

```bash
# Build WASM package
wasm-pack build

# Build for web target
wasm-pack build --target web

# Run tests headless in Firefox
wasm-pack test --headless --firefox

# Run tests headless in Chrome
wasm-pack test --headless --chrome
```

There is no `cargo test` — all tests run via `wasm-pack test` because the crate targets WebAssembly.

## Architecture

### Entry Point
`src/lib.rs` — Rust `start()` function (marked `#[wasm_bindgen(start)]`) sets up the web UI: grabs DOM elements (`#history`, `#cursor-line`), attaches keyboard event listeners, and instantiates the interpreter.

### Module Layout

`src/interpreter/` is a module directory; `mod.rs` declares the submodules and re-exports `EcolMachine`.

| Bestand | Inhoud |
|---|---|
| `interpreter.rs` | `EcolMachine` struct + `execute()` + `VariabelenOpslag` + `RegelBuffer` |
| `opdrachten.rs` | `impl EcolMachine` — `execute_help/lijst/nr/schrijf/tekst/toekennen` |
| `expressies.rs` | `impl EcolMachine` — `solve_expression`, `bereken_expressie`, `bereken_operatoren`, `bereken_tussen_haakjes`, `vervang_variabelen/functies_in_expressie` |
| `functions.rs` | `impl EcolMachine` — `execute_function`, `execute_function_g/abs/wrtl/sin/cos/arctan/ln/log/exp/gok/gokc/ps` |
| `parsers.rs` | Vrije functies: `parseer_regel`, `parseer_argumenten`, `parseer_functie`, `parseer_variabele`, `parse_f32/i32/string` |
| `helpers.rs` | Vrije functies: `extract_regelnummer/keyword/variabele_naam/argumenten`, `geen_spaties_buiten_literals`, `is_alleen_keyword`, `is_geldig_wordt_teken`, `is_geldige_variabele_naam`, `syntaxis_foutmelding`, `verbijzonder_argumenten`, `first_word`, `heeft_geldige_variabele_syntax` |
| `program.rs` | `Programma` (BTreeMap), `Line`, `LineInhoud`, `Sleutelwoord`, `Operator`, `Functie` |
| `waarden.rs` | `Waarde` (Getal/f32), `VariabeleType`, `format_getal`, `haal_data`, `waarde_naar_expressie` |

### Core Components

**`EcolMachine`** (`interpreter.rs`) — The interpreter struct. Holds:
- `variabelen_opslag: VariabelenOpslag` — symbol table + data pool (HashMap → Vec<Waarde>)
- `regel_buffer: RegelBuffer` — output line buffer (flushed by `NR`)
- `programma: Programma` — stored program lines (BTreeMap<u16, LineInhoud>, max 999)
- `pub fn execute(&mut self, input: &str, output: &mut dyn FnMut(&str)) -> String` — main entry point called from JS per line; streaming output via callback

**`Programma`** (`program.rs`) — wraps `BTreeMap<u16, LineInhoud>`. Methods: `regel_toevoegen()` (insert/replace, returns feedback if replaced), `regel_verwijderen()` (remove, returns feedback).

**`Waarde`** (`waarden.rs`) — The value enum: `Getal(f32)`. Only numeric variables exist; no string type.

### Execution Flow

```
User types input → keydown Enter → JS calls machine.execute(line)
  → parseer_regel() (parsers.rs) parses into Line { regelnummer, inhoud }
  → if regelnummer == 0: execute immediately (opdrachten.rs)
  → if regelnummer > 0:  store in programma (Programma::regel_toevoegen)
  → execute_tekst / execute_schrijf / execute_toekennen call solve_expression (expressies.rs)
  → solve_expression()
      → geen_spaties_buiten_literals()           (helpers.rs)
      → vervang_variabelen_in_expressie(Getal)
      → vervang_functies_in_expressie()         → execute_function (functions.rs)
      → bereken_expressie()
  → result appended to #history div
```

### Language Features (verified against real ECOL program + ALGOL translation)

Currently implemented:
- `variabele := expressie` — keyword-less numeric assignment
- `TEKST := "..."` — voeg string-literal toe aan de regelbuffer
- `NR` — dump de regelbuffer naar het scherm en maak hem leeg
- `SCHRIJF (breedte, decimalen) := expressie` — numeric output to line buffer met opmaak (breedte en decimalen)
- `SCHRIJM := expressie` — wetenschappelijke notatie naar regelbuffer (bijv. `+0.12345E+4`)
- `SCHRIJFSYM (n)` — symbool nr. n (0–99) naar regelbuffer
- `SPATIE (n)` — n spaties naar regelbuffer
- `NR(n)` — regelbuffer dumpen + n regeleindes (standaard 1)
- `NP` — scherm wissen
- Numeric expressions: `+`, `-`, `*`, `/`, operator precedence, parentheses
- Numeric functions: `G`, `ABS`, `WRTL`, `SIN`, `COS`, `ARCTAN`, `LN`, `LOG`, `EXP`, `GOK`, `GOKC`, `PS`
- Random number generation: xorshift64 seeded from `Date::now()`, state stored in `EcolMachine.seed`

Not yet implemented:
- `variabele := LEES` — read input as a value
- Flow control: `NAAR`, `ALS`/`DAN`/`ANDERS`, `MET`/`HERHAAL`
- Arrays: `RIJ`, `ONDIN`, `BOVIN`
- `STOP` — runtime early-exit (differs from `KLAAR`: may appear in expressions/conditions)

Numbered program lines (1–999): `Programma` struct stores and retrieves lines; flow control not yet implemented.

### Variable Naming Rules

- Only numeric variables exist (`Waarde::Getal(f32)`); no string type
- Assignment is keyword-less: `variabele := expressie`
- Variable names: lowercase ASCII letters, digits, underscores; must not start with a digit
