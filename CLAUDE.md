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

**`Waarde`** (`src/interpreter/waarden.rs`) — The value enum: `Integer(i32)`, `Float(f32)`, `Tekst(EcolString)`. Variable type is determined by name suffix: `$` = text, `%` = integer, no suffix = float.

**`program.rs`** — Tokenization structs/enums (`Lexeem`, `Operator`, `Sleutelwoord`). Contains `lexer()` and `parseer_regel()` which convert raw input strings into parsed command structures.

### Execution Flow

```
User types input → keydown Enter → JS calls machine.execute(line)
  → lexer() tokenizes
  → parseer_regel() parses into command type (ZET or SCHRIJF)
  → solve_expression() dispatches to string or numeric evaluator
  → solve_string_expression()
      → vervang_variabelen_in_tekst_expressie()  (substitute variable values)
      → vervang_functies_in_tekst_expressie()    (run string functions)
      → samenstellen_tekst_resultaat()           (assemble quoted segments)
  → result appended to #history div
```

### Language Features

Currently implemented:
- `ZET VAR$ := expression` — variable assignment
- `SCHRIJF expression` — print to terminal
- String concatenation with `+`
- String functions: `LINKS$(str, n)`, `RECHTS$(str, n)`, `MIDDEN$(str, start, len)` (1-indexed)
- Escape sequences in strings: `\"`, `\\`, `\n`, `\r`, `\t`, `\0`

Not yet implemented:
- Numeric expression evaluation (parsing exists, evaluation returns error)
- Flow control, line numbers, arrays, INPUT

### Variable Naming Rules

- Must start with an uppercase ASCII letter
- Can contain uppercase letters, digits, underscores
- Suffix determines type: `NAAM$` (text), `TELLER%` (integer), `PRIJS` (float)
