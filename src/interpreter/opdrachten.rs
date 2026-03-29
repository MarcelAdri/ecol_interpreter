use crate::interpreter::EcolMachine;
use crate::interpreter::helpers::{literal_to_string};
use crate::interpreter::program::{Line, LineInhoud};

impl EcolMachine {
    pub(super) fn execute_help(&self) -> Result<String, String> {
        if let Some(window) = web_sys::window() {
            let geopend = window.open_with_url_and_target(crate::interpreter::interpreter::HELP_PAGINA, "_blank");
            match geopend {
                Ok(_) => Ok("Zie het help-document in ander tabblad.".to_string()),
                Err(_) => Err("Kon het help-document niet openen.".to_string()),
            }
        } else {
            Err("Kon het help-document niet openen.".to_string())
        }
    }
    pub(super) fn execute_lijst(&self) -> Result<String, String> {
        let mut reply = String::new();
        for (regelnummer, regel_inh) in self.programma() {
            let regel_inhoud = regel_inh.clone();
            reply.push_str(&Line::new(*regelnummer, regel_inhoud).genereer_regel());
            reply.push('\n');
        }
        Ok(reply)
    }
    pub(super) fn execute_nr(&mut self) -> Result<String, String> {
        let reply = self.lees_regel();
        self.leeg_regel_buffer();
        Ok(format!("{}\n", reply))
    }
    pub(super) fn execute_schrijf(&mut self, breedte: usize, decimalen: usize, expressie: &str) -> Result<String, String> {
        let value = self.solve_expression(expressie)?;

        self.naar_regel_buffer(&value.format_getal(breedte, decimalen)?)?;
        Ok("".to_string())
    }
    pub(super) fn execute_start(&self, output: &mut dyn FnMut(&str)) -> Result<String, String> {
        let mut running_program = EcolMachine::new();
        let mut current = 0;

        loop{
            let Some((&regelnummer, current_regel)) = self.programma().range(current..).next() else {
                return Err("FOUTMELDING: Er zijn geen regels meer om uit te voeren. KLAAR niet aangetroffen.".to_string());
            };
            current = regelnummer + 1;

            match current_regel {
                LineInhoud::Help {} => {
                    return Err(format!("FOUTMELDING in regel {}: HELP mag niet in een programma (interpreter-besturing).", regelnummer));
                }
                LineInhoud::Klaar { } =>{
                    break;
                },
                LineInhoud::Lijst { } => {
                    return Err(format!("FOUTMELDING in regel {}: LIJST mag niet in een programma (interpreter-besturing).", regelnummer));
                },
                LineInhoud::NR { } => {
                    let regel = running_program.execute_nr()
                        .map_err(|e| format!("FOUTMELDING in regel {}: {}", regelnummer, e))?;
                    if !regel.is_empty() {
                        output(&regel);
                    }
                },
                LineInhoud::Schrijf { breedte, decimalen, expressie } => {
                    let regel = running_program.execute_schrijf(*breedte, *decimalen, expressie)
                        .map_err(|e| format!("FOUTMELDING in regel {}: {}", regelnummer, e))?;
                    if !regel.is_empty() {
                        output(&regel);
                    }
                },
                LineInhoud::Start { } => {
                    return Err(format!("FOUTMELDING in regel {}: START mag niet in een programma (interpreter-besturing).", regelnummer));
                },
                LineInhoud::Tekst { expressie } => {
                    let regel = running_program.execute_tekst(expressie)
                        .map_err(|e| format!("FOUTMELDING in regel {}: {}", regelnummer, e))?;
                    if !regel.is_empty() {
                        output(&regel);
                    }
                },
                LineInhoud::Toekennen {variabele_naam, expressie} => {
                    let regel = running_program.execute_toekennen(variabele_naam, expressie)
                        .map_err(|e| format!("FOUTMELDING in regel {}: {}", regelnummer, e))?;
                    if !regel.is_empty() {
                        output(&regel);
                    }
                },
                LineInhoud::Verwijderen { } => {
                    //Kan niet voorkomen in een programma
                },
            }
        }

        Ok("Programma heeft succesvol gelopen.".to_string())
    }
    pub(super) fn execute_tekst(&mut self, expressie: &str) -> Result<String, String> {
        self.naar_regel_buffer(&literal_to_string(expressie)?)?;
        Ok("".to_string())
    }
    pub(super) fn execute_toekennen(&mut self, variabele_naam: &str, expressie: &str) -> Result<String, String> {
        let value = self.solve_expression(expressie)?;
        let _ = self.zet_waarde(variabele_naam, value)?;
        Ok("".to_string())
    }
}