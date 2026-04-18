use std::collections::BTreeMap;
use web_sys::js_sys;
use crate::interpreter::EcolMachine;
use crate::interpreter::helpers::{format_getal, get_sym_value, literal_to_string, result_to_string};
use crate::interpreter::interpreter::SYMBOLEN;
use crate::interpreter::program::{Line, LineInhoud, SprongDoel};
use crate::interpreter::program::LineInhoud::Verwijderen;
use crate::interpreter::waarden::{VariabeleType, Waarde};

pub(super) enum Context {
    Direct,
    Programma,
    Subroutine,
    Functie,
}
#[derive(Debug, Clone, PartialEq)]
pub(super) enum WhatsNext {
    Continue,
    Break,
}
pub(super) struct SubDef {
    regels: BTreeMap<u16, LineInhoud>
}
impl SubDef {
    fn new() -> Self {
        Self {
            regels: BTreeMap::new(),
        }
    }
    fn get_sub_def(&self) -> &BTreeMap<u16, LineInhoud> {
        &self.regels
    }
    pub(crate) fn clone(&self) -> SubDef {
        let mut nieuw = SubDef::new();
        for (regelnummer, regel_inhoud) in self.regels.iter() {
            nieuw.regels.insert(*regelnummer, regel_inhoud.clone());
        }
        nieuw
    }
}
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
        let mut programma = running_program.extract_functie_definities(self.programma())?;
        programma = running_program.extract_sub_definities(&programma)?;
        let mut regelnummer: u16;
        let mut current_regel: &LineInhoud;
        let mut whatsnext: Option<WhatsNext> = None;

        loop {
            stappen += 1;
            if stappen % 1000 == 0 && js_sys::Date::now() - start_tijd > 5000.0 {
                return Err("FOUTMELDING: Programma afgebroken na 5 seconden (mogelijke eindeloze lus).".to_string());
            }

            let Some((&regelnum, current_reg)) = programma.range(current..).next() else {
                return Err("FOUTMELDING: Er zijn geen regels meer om uit te voeren. KLAAR niet aangetroffen.".to_string());
            };
            regelnummer = regelnum;
            current_regel = current_reg;

            current = regelnummer + 1;
            let line = Line::new(current, current_regel.clone());
            let current_option: Option<u16>;

            (_, current_option, whatsnext) = execute_all(&line, &mut running_program, &programma, Context::Programma, output)?;
            match current_option {
                Some(next_line) => {
                    current = next_line;
                },
                None => { }
            }
            match whatsnext {
                Some(WhatsNext::Break) => break,
                Some(WhatsNext::Continue) => continue,
                None => { }
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
    pub(super) fn extract_sub_definities(&mut self, volledige_programma: &BTreeMap<u16,LineInhoud>) -> Result<BTreeMap<u16,LineInhoud>, String> {
        let mut nieuwe_programma: BTreeMap<u16, LineInhoud> = BTreeMap::new();
        let mut in_sub_definitie = false;
        let mut subdef = SubDef::new();
        let mut naam_van_sub: &str = "";

        for (regelnummer, regel) in volledige_programma.iter() {
            match regel {
                LineInhoud::Sub { sub_naam} => {
                    in_sub_definitie = true;
                    naam_van_sub = sub_naam;

                    subdef = SubDef::new();
                }
                LineInhoud::End { } => {
                    if in_sub_definitie {
                        subdef.regels.insert(*regelnummer, regel.clone());
                        self.schrijf_subregister(naam_van_sub, subdef.clone())?;
                        naam_van_sub = "";
                        in_sub_definitie = false;
                    } else {
                        nieuwe_programma.insert(*regelnummer, regel.clone());
                    }
                }
                _ => {
                    if in_sub_definitie {
                        subdef.regels.insert(*regelnummer, regel.clone());
                    } else {
                        nieuwe_programma.insert(*regelnummer, regel.clone());
                    }
                }
            }
        }

        Ok(nieuwe_programma)
    }


}
pub(super) fn execute_all (
    opdracht: &Line,
    machine: &mut EcolMachine,
    programma: &BTreeMap<u16, LineInhoud>,
    context: Context,
    output: &mut dyn FnMut(&str)) -> Result<(Option<String>, Option<u16>, Option<WhatsNext>), String> {
    let no_reply_string: Option<String> = None;
    let no_whats_next: Option<WhatsNext> = None;
    let no_next_line: Option<u16> = None;
    let no_reply = (None, None, None);

    let (reply, nextline, whats_next) = match opdracht.inhoud() {
        LineInhoud::Als { vergelijking, dan, anders } => {
            match context {
                Context::Direct => { no_reply },
                Context::Programma  => {
                    if machine.parseer_vergelijking(vergelijking)? {
                        match machine.execute_naar(programma, dan, &opdracht.regelnummer())? {
                            Some(regel) => {
                                (no_reply_string, Some(regel), no_whats_next)
                            },
                            None => {
                                (no_reply_string, no_next_line, Some(WhatsNext::Break))
                            }
                        }
                    } else if let Some(anders_regel) = anders {
                        match machine.execute_naar(programma, anders_regel, &opdracht.regelnummer())? {
                            Some(regel) => {
                                (no_reply_string, Some(regel), no_whats_next)
                            },
                            None => {
                                (no_reply_string, no_next_line, Some(WhatsNext::Break))
                            }
                        }
                    } else {
                        no_reply
                    }
                },
                Context::Subroutine | Context::Functie => {
                    if machine.parseer_vergelijking(vergelijking)? {
                        match machine.execute_naar(programma, dan, &opdracht.regelnummer())? {
                            Some(regel) => {
                                (no_reply_string, Some(regel), no_whats_next)
                            },
                            None => {
                                return Err(format!("FOUTMELDING in regel {}: STOP-functie is niet geldig in functie-definitie.", opdracht.regelnummer() - 1));
                            }
                        }
                    } else if let Some(anders_regel) = anders {
                        match machine.execute_naar(programma, anders_regel, &opdracht.regelnummer())? {
                            Some(regel) => {
                                (no_reply_string, Some(regel), no_whats_next)
                            },
                            None => {
                                return Err(format!("FOUTMELDING in regel {}: STOP-functie is niet geldig in functie-definitie.", opdracht.regelnummer() - 1));
                            }
                        }
                    } else {
                        no_reply
                    }
                },
            }
        },
        LineInhoud::End {} => {
            match context {
                Context::Direct | Context::Subroutine => { no_reply },
                Context::Programma | Context::Functie => {
                    return Err(format!("FOUTMELDING in regel {}: END kan niet voorkomen in een programma of FUNctie (interne fout).", opdracht.regelnummer()-1));
                },
            }
        },
        LineInhoud::FunStart { .. } => {
            match context {
                Context::Direct => { no_reply },
                Context::Programma | Context::Subroutine => {
                    return Err(format!("FOUTMELDING in regel {}: Functie kan niet in een programma (interne fout).", opdracht.regelnummer()-1));
                },
                Context::Functie => {
                    return Err(format!("FOUTMELDING in regel {}: Functie kan niet in een een andere functie definitie.", opdracht.regelnummer()-1));
                },

            }
        },
        LineInhoud::FunEind { expressie } => {
            match context {
                Context::Direct | Context::Functie => { no_reply },
                Context::Programma | Context::Subroutine => { return Err(format!("FOUTMELDING in regel {}: Functie definitie kan niet in een programma (interne fout).", opdracht.regelnummer()-1)); },
            }
        },
        LineInhoud::GaSub { sub_naam } => {
            match context {
                Context::Direct => { no_reply },
                Context::Programma | Context::Subroutine => {
                    if let Some(subdef) = machine.lees_subregister(sub_naam).map(|s| s.clone()) {
                        machine.start_sub(sub_naam, opdracht.regelnummer())?;

                        let subroutine = subdef.regels;
                        let mut sub_current = 0u16;
                        loop {
                            let Some((&regelnum, current_reg)) = subroutine.range(sub_current..).next() else {
                                break;
                            };
                            sub_current = regelnum + 1;
                            let sub_line= Line::new(sub_current, current_reg.clone());
                            let (_, sub_next_line, whats_next) = execute_all(&sub_line, machine, &subroutine, Context::Subroutine, output)?;
                            match sub_next_line {
                                Some(next_line) => {
                                    sub_current = next_line;
                                },
                                None => { }
                            }

                            match whats_next {
                                Some(WhatsNext::Continue) => continue,
                                Some(WhatsNext::Break) => break,
                                None => {}
                            }
                        }
                        no_reply
                    }  else {
                        return Err(format!("FOUTMELDING in regel {}: Onbekende subroutine '{}'.", opdracht.regelnummer(), sub_naam));
                    }
                },
                Context::Functie => {
                    return Err(format!("FOUTMELDING in regel {}: Vanuit een functie kan een subroutine niet aangeroepen worden.", opdracht.regelnummer()-1));
                },

            }
        },
        LineInhoud::Help { } => {
            match context {
                Context::Direct => { (Some(machine.execute_help()?), no_next_line, no_whats_next)  },
                Context::Programma | Context::Subroutine | Context::Functie => {
                    return Err(format!("FOUTMELDING in regel {}: HELP mag niet in een programma (interpreter-besturing).", opdracht.regelnummer()-1));
                }

            }
        },
        LineInhoud::Herhaal { } => {
            match context {
                Context::Direct => { no_reply },
                Context::Programma | Context::Subroutine | Context::Functie => {
                    let mut next_line = no_next_line;
                    let sprongdoel = machine.teller_herhaal()?;
                    match sprongdoel {
                        Some(sprong) => {
                            let doel = SprongDoel::Regel( sprong );
                            match machine.execute_naar(programma, &doel, &opdracht.regelnummer())? {
                                Some(regel) => next_line = Some(regel),
                                None => {
                                    return Err("Interne FOUT bij HERHAAL-opdracht..".to_string());
                                }
                            }
                        },
                        None => { }
                    }
                    (no_reply_string, next_line, no_whats_next)
                }

            }
        },
        LineInhoud::Klaar { } => {
            match context {
                Context::Direct => { no_reply },
                Context::Programma => {
                    (no_reply_string, no_next_line, Some(WhatsNext::Break))
                },
                Context::Subroutine | Context::Functie => {
                    return Err(format!("FOUTMELDING in regel {}: KLAAR mag niet in een subroutine of een FUNctie staan.", opdracht.regelnummer()));
                },

            }
        },
        LineInhoud::LegeRegel { } => {
            match context {
                Context::Direct => { no_reply },
                Context::Programma | Context::Subroutine | Context::Functie => {
                    (no_reply_string, no_next_line, Some(WhatsNext::Continue))
                },

            }
        },
        LineInhoud::Lijst { } => {
            match context {
                Context::Direct => { (Some(machine.execute_lijst()?), no_next_line, no_whats_next)  },
                Context::Programma | Context::Subroutine | Context::Functie => {
                    return Err(format!("FOUTMELDING in regel {}: LIJST mag niet in een programma (interpreter-besturing).", opdracht.regelnummer()));
                },

            }
        },
        LineInhoud::Met { variabele_naam, stap_expressie, start_expressie, stop_expressie } => {
            match context {
                Context::Direct => { no_reply },
                Context::Programma | Context::Subroutine | Context::Functie => {
                    let Some((&volgende_regelnummer, _)) = programma.range(opdracht.regelnummer()..).next() else {
                        return Err(format!("FOUTMELDING in regel {}: FOUTMELDING: geen regels na MET-opdracht.", opdracht.regelnummer()));
                    };

                    let regel = machine.execute_met(variabele_naam, stap_expressie, start_expressie, stop_expressie, volgende_regelnummer)
                        .map_err(|e| format!("FOUTMELDING in regel {}: {}", opdracht.regelnummer(), e))?;
                    match regel {
                        Some(_) => { (no_reply_string, no_next_line, Some(WhatsNext::Continue)) },
                        None => {
                            (no_reply_string, Some(EcolMachine::teller_naar_herhaal(programma, &opdracht.regelnummer())?), Some(WhatsNext::Continue))
                        }
                    }
                },

            }
        },
        LineInhoud::Naar {sprong_doel } => {
            match context {
                Context::Direct => { no_reply },
                Context::Programma  => {
                    match machine.execute_naar(programma, sprong_doel, &opdracht.regelnummer())? {
                        Some(regel) => {
                            (no_reply_string, Some(regel), no_whats_next)
                        },
                        None => {
                            (no_reply_string, no_next_line, Some(WhatsNext::Break))
                        },
                    }
                },
                Context::Functie | Context::Subroutine => {
                    match machine.execute_naar(programma, sprong_doel, &opdracht.regelnummer())? {
                        Some(regel) => {
                            (no_reply_string, Some(regel), no_whats_next)
                        },
                        None => {
                            return Err(format!("FOUTMELDING in regel {}: STOP-functie is niet geldig in  subroutine of functie-definitie.", opdracht.regelnummer()-1));
                        },
                    }
                },

            }
        },
        LineInhoud::NP { } => {
            match context {
                Context::Direct => {
                    (Some(machine.execute_np( output)?), no_next_line, no_whats_next)
                },
                Context::Programma | Context::Subroutine => {
                    (Some(machine.execute_np( output)
                        .map_err(|e| format!("FOUTMELDING in regel {}: {}", opdracht.regelnummer() -1, e))?), no_next_line, no_whats_next)
                },
                Context::Functie => {
                    return Err(format!("FOUTMELDING in regel {}: NP kan niet in een FUN definitie (geen uitvoer-apparaat).", opdracht.regelnummer()-1));
                },

            }
        },
        LineInhoud::NR { aantal } => {
            match context {
                Context::Direct => { (Some(machine.execute_nr( &aantal )?), no_next_line, no_whats_next) },
                Context::Programma | Context::Subroutine => {
                    output(&machine.execute_nr( &aantal )
                        .map_err(|e| format!("FOUTMELDING in regel {}: {}", opdracht.regelnummer() -1, e))?);
                    no_reply
                },
                Context::Functie => {
                    return Err(format!("FOUTMELDING in regel {}: NR kan niet in een FUN definitie (geen uitvoer-apparaat).", opdracht.regelnummer()-1));
                },

            }
        },
        LineInhoud::Rij { start, eind, variabele_naam } => {
            match context {
                Context::Direct => {
                    _ = machine.execute_rij(start, eind, variabele_naam)?;
                    no_reply
                },
                Context::Programma | Context::Subroutine | Context::Functie => {
                    _ = machine.execute_rij(&start, &eind, &variabele_naam)
                        .map_err(|e| format!("FOUTMELDING in regel {}: {}", opdracht.regelnummer() -1, e))?;
                    no_reply
                },

            }
        },
        LineInhoud::Rijsym { start, eind, variabele_naam } => {
            match context {
                Context::Direct => {
                    _ = machine.execute_rijsym(start, eind, variabele_naam)?;
                    no_reply
                },
                Context::Programma | Context::Subroutine | Context::Functie => {
                    _ = machine.execute_rijsym(&start, &eind, &variabele_naam)
                        .map_err(|e| format!("FOUTMELDING in regel {}: {}", opdracht.regelnummer() -1, e))?;
                    no_reply
                },

            }
        },
        LineInhoud::Schrijf { breedte, decimalen, expressie } => {
            match context {
                Context::Direct => {
                    _ = machine.execute_schrijf(*breedte, *decimalen, expressie)?;
                    no_reply
                },
                Context::Programma | Context::Subroutine => {
                    _ = machine.execute_schrijf(*breedte, *decimalen, expressie)
                        .map_err(|e| format!("FOUTMELDING in regel {}: {}", opdracht.regelnummer() -1, e))?;
                    no_reply
                },
                Context::Functie => {
                    return Err(format!("FOUTMELDING in regel {}: SCHRIJF kan niet in een FUN definitie (geen uitvoer-apparaat).", opdracht.regelnummer()-1));
                },

            }
        },
        LineInhoud::Schrijfsym { expressie } => {
            match context {
                Context::Direct => {
                    _ = machine.execute_schrijfsym(expressie)?;
                    no_reply
                },
                Context::Programma | Context::Subroutine => {
                    _ = machine.execute_schrijfsym(expressie)
                        .map_err(|e| format!("FOUTMELDING in regel {}: {}", opdracht.regelnummer() -1, e))?;
                    no_reply
                },
                Context::Functie => {
                    return Err(format!("FOUTMELDING in regel {}: SCHRIJFSYM kan niet in een FUN definitie (geen uitvoer-apparaat).", opdracht.regelnummer()-1));
                },

            }
        },
        LineInhoud::Schrijm { expressie } => {
            match context {
                Context::Direct => {
                    _ = machine.execute_schrijm(expressie)?;
                    no_reply
                },
                Context::Programma | Context::Subroutine => {
                    _ = machine.execute_schrijm(expressie)
                        .map_err(|e| format!("FOUTMELDING in regel {}: {}", opdracht.regelnummer() -1, e))?;
                    no_reply
                },
                Context::Functie => {
                    return Err(format!("FOUTMELDING in regel {}: SCHRIJM kan niet in een FUN definitie (geen uitvoer-apparaat).", opdracht.regelnummer()-1));
                },

            }
        },
        LineInhoud::Spatie { aantal } => {
            match context {
                Context::Direct => {
                    _ = machine.execute_spatie(aantal)?;
                    no_reply
                },
                Context::Programma | Context::Subroutine => {
                    _ = machine.execute_spatie(aantal)
                        .map_err(|e| format!("FOUTMELDING in regel {}: {}", opdracht.regelnummer() -1, e))?;
                    no_reply
                },
                Context::Functie => {
                    return Err(format!("FOUTMELDING in regel {}: SPATIE kan niet in een FUN definitie (geen uitvoer-apparaat).", opdracht.regelnummer()-1));
                },

            }
        },
        LineInhoud::Start { } => {
            match context {
                Context::Direct => { (Some(machine.execute_start(output)?), no_next_line, no_whats_next) },
                Context::Programma | Context::Subroutine | Context::Functie => {
                    return Err(format!("FOUTMELDING in regel {}: START mag niet in een programma (interpreter-besturing).", opdracht.regelnummer()-1));
                },

            }
        }
        LineInhoud::Sub { .. } => {
            match context {
                Context::Direct => { no_reply },
                Context::Programma | Context::Subroutine | Context::Functie => {
                    return Err(format!("FOUTMELDING in regel {}: fout bij verwerking subroutine (interne fout).", opdracht.regelnummer()-1));
                },
            }
        }
        LineInhoud::Tekst { expressie } => {
            match context {
                Context::Direct => {
                    _ = machine.execute_tekst(expressie)?;
                    no_reply
                },
                Context::Programma | Context::Subroutine => {
                    _ = machine.execute_tekst(expressie)
                        .map_err(|e| format!("FOUTMELDING in regel {}: {}", opdracht.regelnummer() -1, e))?;
                    no_reply
                },
                Context::Functie => {
                    return Err(format!("FOUTMELDING in regel {}: TEKST kan niet in een FUN definitie (geen uitvoer-apparaat).", opdracht.regelnummer()-1));
                },

            }
        },
        LineInhoud::Toekennen {variabele_naam, argument,  expressie} => {
            match context {
                Context::Direct => {
                    _ = machine.execute_toekennen(&variabele_naam, &argument, &expressie)?;
                    no_reply
                },
                Context::Programma | Context::Subroutine | Context::Functie => {
                    _ = machine.execute_toekennen(&variabele_naam, &argument, &expressie)
                        .map_err(|e| format!("FOUTMELDING in regel {}: {}", opdracht.regelnummer() -1, e))?;
                    no_reply
                },

            }
        },
        LineInhoud::Verwijderen { } => {
            match context {
                Context::Direct => {
                    return Err("Verwijderen van een ongenummerde regel is niet mogelijk.".to_string());
                },
                Context::Programma | Context::Subroutine | Context::Functie => {
                    return Err("Verwijderen van een regel kan niet voorkomen in een programma (interne fout).".to_string());
                },

            }
        },
    };



    Ok((reply, nextline, whats_next))
}