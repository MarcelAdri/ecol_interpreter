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

**`Waarde`** (`src/interpreter/waarden.rs`) — The value enum: `Float(f32)`, `Tekst(EcolString)`. Variable type is determined by the assignment keyword, not the variable name.

**`program.rs`** — Tokenization structs/enums (`Operator`, `Sleutelwoord`). Contains `lexer()` and `parseer_regel()` which convert raw input strings into parsed command structures.

### Execution Flow

```
User types input → keydown Enter → JS calls machine.execute(line)
  → lexer() tokenizes
  → parseer_regel() parses into command type (TEKST, NR, SCHRIJF, HELP)
  → solve_expression() dispatches to string or numeric evaluator
  → solve_string_expression()
      → vervang_variabelen_in_tekst_expressie()  (substitute variable values)
      → vervang_functies_in_tekst_expressie()    (run string functions)
      → samenstellen_tekst_resultaat()           (assemble quoted segments)
  → result appended to #history div
```

### Language Features

Currently implemented:
- `TEKST naam := expressie` — text variable assignment (repeat keyword on reassignment)
- `NR naam := expressie` — numeric variable assignment (repeat keyword on reassignment)
- `SCHRIJF expressie` — print to terminal
- String concatenation with `+`
- String functions: `LINKS$(str, n)`, `RECHTS$(str, n)`, `MIDDEN$(str, start, len)` (1-indexed)

Not yet implemented:
- Numeric expression evaluation (parsing exists, evaluation returns error)
- Flow control, line numbers, arrays, INPUT

### Variable Naming Rules

- Every valid ECOL line begins with a keyword — there is no keyword-less syntax
- Variable names: lowercase ASCII letters, digits, underscores; must not start with a digit
- Type is determined by the keyword (`TEKST` or `NR`), not by any suffix
- No `$` or `%` suffixes
