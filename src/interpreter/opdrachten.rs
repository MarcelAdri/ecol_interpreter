use crate::interpreter::EcolMachine;
use crate::interpreter::program::Line;
use crate::interpreter::waarden::{haal_data, VariabeleType};

impl EcolMachine {
    pub(super) fn execute_help(&mut self) -> String {
        if let Some(window) = web_sys::window() {
            let _ = window.open_with_url_and_target(crate::interpreter::interpreter::HELP_PAGINA, "_blank");
        }

        "Zie het help-document in ander tabblad.".to_string()
    }
    pub(super) fn execute_lijst(&mut self) -> String {
        let mut reply = String::new();
        for (regelnummer, regel_inh) in self.programma() {
            let regel_inhoud = regel_inh.clone();
            reply.push_str(&Line::new(*regelnummer, regel_inhoud).genereer_regel());
            reply.push('\n');
        }
        reply
    }
    pub(super) fn execute_nr(&mut self) -> String {
        let reply = self.lees_regel();
        self.leeg_regel_buffer();
        reply
    }
    pub(super) fn execute_schrijf(&mut self, breedte: usize, decimalen: usize, expressie: &str) -> String {
        //TODO: formatting aan de hand van breedte en decimalen
        match self.solve_expression(expressie) {
            Ok(value) => {
                self.naar_regel_buffer(&haal_data(&value));
                "".to_string()
            }
            Err(error_message) => error_message,
        }
    }
    pub(super) fn execute_tekst(&mut self, expressie: &str) -> String {
        match self.solve_expression(expressie) {
            Ok(value) => {
                self.naar_regel_buffer(&haal_data(&value));
                "".to_string()
            }
            Err(error_message) => error_message,
        }
    }
    pub(super) fn execute_toekennen(&mut self, variabele_naam: &str, expressie: &str) -> String {
        match self.solve_expression(expressie) {
            Ok(value) => {
                match self.zet_waarde(variabele_naam, value){
                    Ok(_) => "".to_string(),
                    Err(error_message) => error_message
                }

            }
            Err(error_message) => {
                error_message
            }
        }
    }
}