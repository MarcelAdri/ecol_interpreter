//! Kern van de ECOL-interpreter: parsing, uitvoering en toestandsbeheer.
//!
//! Exporteert [`EcolMachine`] als primaire ingang voor de WebAssembly-binding
//! en [`LeesGeheugen`] als sessiestatus voor de pauze-/hervat-flow bij
//! `LEES`, `LEESSYM` en `LAAD`.
mod machine;
mod waarden;
mod program;
mod helpers;
mod opdrachten;
mod expressies;
mod functions;
mod vergelijkingen;
mod errors;
mod leesgeheugen;

pub use machine::{EcolMachine};
pub use leesgeheugen::{LeesGeheugen};