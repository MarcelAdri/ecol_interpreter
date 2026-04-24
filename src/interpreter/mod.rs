mod interpreter;
mod waarden;
mod program;
mod parsers;
mod helpers;
mod opdrachten;
mod expressies;
mod functions;
mod vergelijkingen;
mod errors;
mod leesgeheugen;

pub use interpreter::{EcolMachine};
pub use leesgeheugen::{LeesGeheugen};