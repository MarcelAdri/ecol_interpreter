use crate::interpreter::EcolMachine;
use crate::interpreter::helpers::{extract_variabele_naam, format_getal, get_sym_value, literal_to_string};
use crate::interpreter::interpreter::SYMBOLEN;
use crate::interpreter::program::{Line, LineInhoud};
use crate::interpreter::waarden::{VariabeleType, Waarde};

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
    pub(super)  fn execute_np(&self, output: &mut dyn FnMut(&str)) -> Result<String, String> {
        output("\x0C");
        Ok("".to_string())
    }
    pub(super) fn execute_nr(&mut self, aantal: usize) -> Result<String, String> {
        let mut reply = String::new();

        if aantal > 0 {
            reply = format!("{}{}", self.lees_regel(), "\n".repeat(aantal));
        }
        self.leeg_regel_buffer();
        Ok(reply)
    }
    pub(super) fn execute_rij(&mut self, start: usize, eind: usize, variabele_naam: &str) -> Result<String, String> {
        self.var_reserveer_rij(variabele_naam, start, eind)?;
        Ok("".to_string())
    }
    pub(super) fn execute_rijsym(&mut self, start: usize, eind: usize, variabele_naam: &str) -> Result<String, String> {
        self.var_reserveer_rijsym(variabele_naam, start, eind)?;
        Ok("".to_string())
    }

    pub(super) fn execute_schrijf(&mut self, breedte: usize, decimalen: usize, expressie: &str) -> Result<String, String> {
        let value = self.solve_expression(expressie)?;

        self.naar_regel_buffer(&format_getal(value, breedte, decimalen)?)?;
        Ok("".to_string())
    }
    pub(super) fn execute_schrijfsym(&mut self, expressie: &str) -> Result<String, String> {
        let value = self.solve_expression(expressie)?;

        let symbool_nummer = get_sym_value(&value)? as usize;

        let Some(symbool) = SYMBOLEN[symbool_nummer] else {
            return Err(format!("Symboolwaarde {} is niet gedefinieerd.", symbool_nummer));
        };


        self.naar_regel_buffer(symbool.to_string().as_str())?;
        Ok("".to_string())
    }
    pub(super) fn execute_schrijm(&mut self, expressie: &str) -> Result<String, String> {
        let value = self.solve_expression(expressie)?;

        if value.is_nan() {
            return Err("FOUTMELDING: Expressie levert ongeldige waarde op.".to_string());
        }
        let s = format!("{:+E}", value);

        let (mantisse_deel, exp_deel) = s.split_once('E').unwrap();
        let exp: i32 = exp_deel.parse::<i32>().unwrap() + 1;

        let (teken, cijfers) = if mantisse_deel.starts_with('-') {
            ("-", &mantisse_deel[1..])
        } else {
            ("+", &mantisse_deel[1..])
        };

        let ecol_mantisse = format!("0.{}", cijfers.replace('.', ""));

        let exp_teken = if exp >= 0 { "+" } else { "-" };

        self.naar_regel_buffer(&format!("{}{}E{}{}", teken, ecol_mantisse, exp_teken, exp.unsigned_abs()))?;

        Ok("".to_string())
    }
    pub(super) fn execute_spatie(&mut self, aantal: usize) -> Result<String, String> {
        if aantal > 80 {
            return Err(format!("SPATIE verwacht een aantal kleiner dan 80 (maximale regelgrootte). Aantal: {}", aantal));
        }
        let regel = format!("{: <width$}", "", width = aantal);
        self.naar_regel_buffer(&regel)?;
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
                LineInhoud::NP { } => {
                    let regel = running_program.execute_np( output )
                        .map_err(|e| format!("FOUTMELDING in regel {}: {}", regelnummer, e))?;
                    if !regel.is_empty() {
                        output(&regel);
                    }
                },
                LineInhoud::NR { aantal } => {
                    let regel = running_program.execute_nr( *aantal )
                        .map_err(|e| format!("FOUTMELDING in regel {}: {}", regelnummer, e))?;
                    if !regel.is_empty() {
                        output(&regel);
                    }
                },
                LineInhoud::Rij { start, eind, variabele_naam } => {
                    let regel = running_program.execute_rij(*start, *eind, variabele_naam)
                        .map_err(|e| format!("FOUTMELDING in regel {}: {}", regelnummer, e))?;
                    if !regel.is_empty() {
                        output(&regel);
                    }
                },
                LineInhoud::Rijsym { start, eind, variabele_naam } => {
                    let regel = running_program.execute_rijsym(*start, *eind, variabele_naam)
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
                LineInhoud::Schrijfsym { expressie } => {
                    let regel = running_program.execute_schrijfsym(expressie)
                        .map_err(|e| format!("FOUTMELDING in regel {}: {}", regelnummer, e))?;
                    if !regel.is_empty() {
                        output(&regel);
                    }
                },
                LineInhoud::Schrijm { expressie } => {
                    let regel = running_program.execute_schrijm(expressie)
                        .map_err(|e| format!("FOUTMELDING in regel {}: {}", regelnummer, e))?;
                    if !regel.is_empty() {
                        output(&regel);
                    }
                },
                LineInhoud::Spatie { aantal } => {
                    let regel = running_program.execute_spatie(*aantal)
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
                LineInhoud::Toekennen {variabele_naam, argument, expressie} => {
                    let regel = running_program.execute_toekennen(variabele_naam, *argument, expressie)
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
    pub(super) fn execute_toekennen(&mut self, variabele_naam: &str, argument: usize, expressie: &str) -> Result<String, String> {
        let value = self.solve_expression(expressie)?;
        let mut waarde = self.var_lees_waarde(variabele_naam).unwrap_or(Waarde::NogNietBepaald).clone();
        let var_type_compleet = self.var_type_van(variabele_naam);
        let variabele_type: VariabeleType;

        match var_type_compleet {
            Some(var_type) => {
                variabele_type = var_type;
                match variabele_type {
                    VariabeleType::Getal => {
                        if argument != 0 {
                            return Err("Getalvariabele kan geen index hebben.".to_string());
                        }
                        waarde = Waarde::new_getal(value);
                    },
                    VariabeleType::Rij => {
                        if argument == 0 {
                            return Err("Geen index verwijzing bij RIJ-variabele.".to_string());
                        }
                        waarde.rij_set_value(value, argument)?;
                    },
                    VariabeleType::Rijsym => {
                        if argument == 0 {
                            return Err("Geen index verwijzing bij RIJSYM-variabele.".to_string());
                        }

                        waarde.rij_set_value(value, argument)?;
                    }
                }
            },
            None => {
                if argument != 0 {
                    return Err(format!("RIJ-Variabele '{}' is niet gedefinieerd.", variabele_naam));
                }
                waarde = Waarde::new_getal(value);
            }
        }

        let _ = self.var_schrijf_waarde(variabele_naam, waarde)?;
        Ok("".to_string())
    }
}