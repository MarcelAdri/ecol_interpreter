use std::collections::BTreeMap;
use web_sys::js_sys;
use crate::interpreter::EcolMachine;
use crate::interpreter::helpers::{extract_variabele_naam, format_getal, get_sym_value, literal_to_string};
use crate::interpreter::interpreter::SYMBOLEN;
use crate::interpreter::program::{Line, LineInhoud, SprongDoel};
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
    pub(super) fn execute_met(&mut self, variabele_naam: &str, stap_expressie: &str, start_expressie: &str, stop_expressie: &str, volgende_regel: u16) -> Result<Option<()>, String> {
        let stap = self. solve_expression(stap_expressie)?;
        let start = self. solve_expression(start_expressie)?;
        let stop = self. solve_expression(stop_expressie)?;

        Ok(self.teller_nieuw(variabele_naam, stap, start, stop, volgende_regel)?)
    }
    pub(super) fn execute_naar(&mut self, lopend_programma: &BTreeMap<u16, LineInhoud>, sprong_doel: &SprongDoel, current: &u16) -> Result<Option<u16>, String> {
        match sprong_doel.regelnummer() {
            Some(regel) => {
                if lopend_programma.contains_key(&regel) {
                    if regel < *current {
                        self.rollback_program(lopend_programma, *current, regel);
                    }
                    Ok(Some(regel))
                } else {
                    Err(format!("FOUTMELDING in regel {}: Sprong naar niet gedefinieerde regel {}.", current, regel))
                }
            },
            None => { Ok(None) },
        }


    }
    pub(super)  fn execute_np(&self, output: &mut dyn FnMut(&str)) -> Result<String, String> {
        output("\x0C");
        Ok("".to_string())
    }
    pub(super) fn execute_nr(&mut self, aantal: &str) -> Result<String, String> {
        let mut reply = String::new();
        let number = self.solve_expression(aantal)?;

        if number > 0f32 {
            reply = format!("{}{}", self.lees_regel(), "\n".repeat(number as usize));
        }
        self.leeg_regel_buffer();
        Ok(reply)
    }
    pub(super) fn execute_rij(&mut self, start: &str, eind: &str, variabele_naam: &str) -> Result<String, String> {
        let begin = self.solve_expression(start)?;
        let einde = self.solve_expression(eind)?;
        self.var_reserveer_rij(variabele_naam, begin as usize, einde as usize)?;
        Ok("".to_string())
    }
    pub(super) fn execute_rijsym(&mut self, start: &str, eind: &str, variabele_naam: &str) -> Result<String, String> {
        let begin = self.solve_expression(start)?;
        let einde = self.solve_expression(eind)?;
        self.var_reserveer_rijsym(variabele_naam, begin as usize, einde as usize)?;
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
    pub(super) fn execute_spatie(&mut self, aantal: &str) -> Result<String, String> {
        let number = self.solve_expression(aantal)? as usize;
        if number > 80 {
            return Err(format!("SPATIE verwacht een aantal kleiner dan 80 (maximale regelgrootte). Aantal: {}", aantal));
        }
        let regel = format!("{: <width$}", "", width = number);
        self.naar_regel_buffer(&regel)?;
        Ok("".to_string())
    }
    pub(super) fn execute_start(&self, output: &mut dyn FnMut(&str)) -> Result<String, String> {
        let mut running_program = EcolMachine::new();
        let mut current = 0;
        let start_tijd = js_sys::Date::now();
        let mut stappen: u32 = 0;
        let programma = running_program.extract_functie_definities(self.programma())?;

        loop{
            stappen += 1;
            if stappen % 1000 == 0 && js_sys::Date::now() - start_tijd > 5000.0 {
                return Err("FOUTMELDING: Programma afgebroken na 5 seconden (mogelijke eindeloze lus).".to_string());
            }
            let Some((&regelnummer, current_regel)) = programma.range(current..).next() else {
                return Err("FOUTMELDING: Er zijn geen regels meer om uit te voeren. KLAAR niet aangetroffen.".to_string());
            };
            current = regelnummer + 1;

            match current_regel {
                LineInhoud::Als { vergelijking, dan, anders } => {
                    if running_program.parseer_vergelijking(vergelijking)? {
                        match running_program.execute_naar(&programma, dan, &current)? {
                            Some(regel) => current = regel,
                            None => {
                                break;
                            }
                        }
                    } else if let Some(anders_regel) = anders {
                        match running_program.execute_naar(&programma, anders_regel, &current)? {
                            Some(regel) => current = regel,
                            None => {
                                break;
                            }
                        }
                    }
                }
                LineInhoud::FunStart { .. } => {
                    return Err(format!("FOUTMELDING in regel {}: Functie definitie kan niet in een programma (interne fout).", regelnummer));
                }
                LineInhoud::FunEind { .. } => {
                    return Err(format!("FOUTMELDING in regel {}: Functie definitie kan niet in een programma (interne fout).", regelnummer));
                }
                LineInhoud::Help {} => {
                    return Err(format!("FOUTMELDING in regel {}: HELP mag niet in een programma (interpreter-besturing).", regelnummer));
                }
                LineInhoud::Herhaal {} => {
                    let Some( sprong) = running_program.teller_herhaal()? else { continue };
                    let doel = SprongDoel::Regel( sprong );
                    match running_program.execute_naar(&programma, &doel, &current)? {
                        Some(regel) => current = regel,
                        None => {
                            return Err("Interne FOUT bij HERHAAL-opdracht..".to_string());
                        }
                    }
                }
                LineInhoud::Klaar { } =>{
                    break;
                },
                LineInhoud::LegeRegel { } => {
                    continue;
                },
                LineInhoud::Lijst { } => {
                    return Err(format!("FOUTMELDING in regel {}: LIJST mag niet in een programma (interpreter-besturing).", regelnummer));
                },
                LineInhoud::Met { variabele_naam, stap_expressie, start_expressie, stop_expressie } => {
                    let Some((&volgende_regelnummer, _)) = &programma.range(current..).next() else {
                        return Err("FOUTMELDING: geen regels na MET-opdracht..".to_string());
                    };
                    let regel = running_program.execute_met(variabele_naam, stap_expressie, start_expressie, stop_expressie, volgende_regelnummer)
                        .map_err(|e| format!("FOUTMELDING in regel {}: {}", regelnummer, e))?;
                    match regel {
                        Some(_) => { continue; },
                        None => {
                            current = EcolMachine::teller_naar_herhaal(&programma, &current)?;
                            continue;
                        }
                    }

                },
                LineInhoud::Naar { sprong_doel } => {
                    match running_program.execute_naar(&programma, sprong_doel, &current)? {
                        Some(regel) => current = regel,
                        None => {
                            break;
                        },
                    }


                },
                LineInhoud::NP { } => {
                    let regel = running_program.execute_np( output )
                        .map_err(|e| format!("FOUTMELDING in regel {}: {}", regelnummer, e))?;
                    if !regel.is_empty() {
                        output(&regel);
                    }
                },
                LineInhoud::NR { aantal } => {
                    let regel = running_program.execute_nr( aantal )
                        .map_err(|e| format!("FOUTMELDING in regel {}: {}", regelnummer, e))?;
                    if !regel.is_empty() {
                        output(&regel);
                    }
                },
                LineInhoud::Rij { start, eind, variabele_naam } => {
                    let regel = running_program.execute_rij(start, eind, variabele_naam)
                        .map_err(|e| format!("FOUTMELDING in regel {}: {}", regelnummer, e))?;
                    if !regel.is_empty() {
                        output(&regel);
                    }
                },
                LineInhoud::Rijsym { start, eind, variabele_naam } => {
                    let regel = running_program.execute_rijsym(start, eind, variabele_naam)
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
                    let regel = running_program.execute_spatie(aantal)
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
                    let regel = running_program.execute_toekennen(variabele_naam, argument, expressie)
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

        Ok("Programma is normaal beëindigd.".to_string())
    }
    pub(super) fn execute_tekst(&mut self, expressie: &str) -> Result<String, String> {
        self.naar_regel_buffer(&literal_to_string(expressie)?)?;
        Ok("".to_string())
    }
    pub(super) fn execute_toekennen(&mut self, variabele_naam: &str, argument: &str, expressie: &str) -> Result<String, String> {
        //panic!(" Expressie: {}", expressie);
        let value = self.solve_expression(expressie)?;
        let arg = if argument.is_empty() {
            0usize
        } else {
            self.solve_expression(argument)? as usize
        };

        let mut waarde = self.var_lees_waarde(variabele_naam).unwrap_or(Waarde::NogNietBepaald).clone();
        let var_type_compleet = self.var_type_van(variabele_naam);
        let variabele_type: VariabeleType;

        match var_type_compleet {
            Some(var_type) => {
                variabele_type = var_type;
                match variabele_type {
                    VariabeleType::Getal => {
                        if arg != 0 {
                            return Err("Getalvariabele kan geen index hebben.".to_string());
                        }
                        waarde = Waarde::new_getal(value);
                    },
                    VariabeleType::Rij => {
                        if arg == 0 {
                            return Err("Geen index verwijzing bij RIJ-variabele.".to_string());
                        }
                        waarde.rij_set_value(value, arg)?;
                    },
                    VariabeleType::Rijsym => {
                        if arg == 0 {
                            return Err("Geen index verwijzing bij RIJSYM-variabele.".to_string());
                        }

                        waarde.rij_set_value(value, arg)?;
                    },
                    VariabeleType::Teller => {
                        if arg != 0 {
                            return Err("Teller-variabele kan geen index hebben.".to_string());
                        }

                        waarde.teller_schrijf_current(value)?;
                    },

                }
            },
            None => {
                if arg != 0 {
                    return Err(format!("RIJ-Variabele '{}' is niet gedefinieerd.", variabele_naam));
                }
                waarde = Waarde::new_getal(value);
            }
        }

        let _ = self.var_schrijf_waarde(variabele_naam, waarde)?;
        Ok("".to_string())
    }

    fn rollback_program(&mut self, lopend_programma: &BTreeMap<u16, LineInhoud>, current: u16, doel: u16) {
        let mut current_line = doel;

        loop {
            let Some((&regelnummer, current_regel)) = lopend_programma.range(current_line..current).next() else {
                return;
            };
            current_line = regelnummer + 1;

            match current_regel {
                LineInhoud::Rij { start, eind, variabele_naam } |
                LineInhoud::Rijsym { start, eind, variabele_naam } => {
                    self.var_wis(variabele_naam)
                },
                _ => { continue },
            }
        }
    }
}