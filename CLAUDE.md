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

### Core Components

**`EcolMachine`** (`src/interpreter/interpreter.rs`) — The interpreter struct. Holds:
- `symbolen: HashMap<String, usize>` — symbol table mapping variable name → index in data pool
- `data: Vec<Waarde>` — data pool of all stored values
- `execute(&mut self, command: &str) -> String` — main entry point called from JS per line

**`Waarde`** (`src/interpreter/waarden.rs`) — The value enum: `Float(f32)`, `Tekst(EcolString)`. Variable type is inferred from the expression, not declared by keyword.

**`program.rs`** — Tokenization structs/enums (`Operator`, `Sleutelwoord`). Contains `lexer()` and `parseer_regel()` which convert raw input strings into parsed command structures.

### Execution Flow

```
User types input → keydown Enter → JS calls machine.execute(line)
  → lexer() tokenizes
  → parseer_regel() parses into command type
  → solve_expression() dispatches to string or numeric evaluator
  → solve_string_expression()
      → vervang_variabelen_in_tekst_expressie()  (substitute variable values)
      → vervang_functies_in_tekst_expressie()    (run string functions)
      → samenstellen_tekst_resultaat()           (assemble quoted segments)
  → result appended to #history div
```

### Language Features (verified against real ECOL program + ALGOL translation)

Currently implemented:
- `variabele := expressie` — keyword-less assignment
- `TEKST := expressie` — voeg tekst toe aan de regelbuffer
- `NR` — dump de regelbuffer naar het scherm en maak hem leeg
- String concatenation with `+`
- String functions: `LINKS$(str, n)`, `RECHTS$(str, n)`, `MIDDEN$(str, start, len)` (1-indexed)

Verified ECOL syntax (not yet implemented):
- `SCHRIJF (breedte, decimalen) := expressie` — formatted numeric output
- `variabele := LEES` — read input as a value (LEES appears on the right-hand side)
- `RIJ (start:einde) naam` — 1D array declaration; no 2D arrays
- `array(index) := expressie` — array element assignment
- `ALS voorwaarde DAN regelnr ANDERS regelnr` — jumps to line numbers
- `NAAR regelnr` — unconditional jump
- `MET stap, var := begin, einde` ... `HERHAAL var` — counted loop

Not yet implemented:
- Numeric expression evaluation
- Flow control, line numbers, arrays

### Variable Naming Rules

- Assignment is keyword-less: `variabele := expressie`
- Variable names: lowercase ASCII letters, digits, underscores; must not start with a digit
- Type is inferred from the expression (no suffixes, no type keywords)
