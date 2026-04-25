use std::collections::BTreeMap;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use web_sys::js_sys;
use crate::interpreter::{EcolMachine, LeesGeheugen};
use crate::interpreter::errors::EcolFout;
//use crate::interpreter::errors::EcolFout::{WachtOpLees, WachtOpLeessym};
use crate::interpreter::helpers::{format_getal, get_sym_value, literal_to_string};
use crate::interpreter::machine::SYMBOLEN;
use crate::interpreter::program::{Line, LineInhoud, SprongDoel, SubDef};
use crate::interpreter::waarden::{VariabeleType, Waarde};

type UitvoeringResultaat = Result<(Option<String>, Option<u16>, Option<WhatsNext>), EcolFout>;

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

impl EcolMachine {
    pub(super) fn execute_help(&self) -> Result<String, EcolFout> {
        if let Some(window) = web_sys::window() {
            let geopend = window.open_with_url_and_target(crate::interpreter::machine::HELP_PAGINA, "_blank");
            match geopend {
                Ok(_) => Ok("Zie het help-document in ander tabblad.".to_string()),
                Err(_) => Err(EcolFout::FoutMelding("Kon het help-document niet openen.".to_string())),
            }
        } else {
            Err(EcolFout::FoutMelding("Kon het help-document niet openen.".to_string()))
        }
    }

    pub(super) fn execute_bewaar(&self) -> Result<String, EcolFout> {
        let tekst = self.execute_lijst()?;
        if tekst.is_empty() {
            return Ok("Programma is leeg, niets op te slaan.".to_string());
        }
        let window = web_sys::window()
            .ok_or_else(|| EcolFout::FoutMelding("Geen browservenster.".to_string()))?;
        let document = window.document()
            .ok_or_else(|| EcolFout::FoutMelding("Geen document.".to_string()))?;

        let array = js_sys::Array::new();
        array.push(&JsValue::from_str(&tekst));
        let blob = web_sys::Blob::new_with_str_sequence(&JsValue::from(array))
            .map_err(|_| EcolFout::FoutMelding("Kon geen bestand aanmaken.".to_string()))?;
        let url = web_sys::Url::create_object_url_with_blob(&blob)
            .map_err(|_| EcolFout::FoutMelding("Kon geen URL aanmaken.".to_string()))?;

        let link = document.create_element("a")
            .map_err(|_| EcolFout::FoutMelding("Kon geen download-link aanmaken.".to_string()))?;
        link.set_attribute("href", &url).unwrap();
        link.set_attribute("download", "programma.ecol").unwrap();
        link.dyn_ref::<web_sys::HtmlElement>().unwrap().click();
        web_sys::Url::revoke_object_url(&url).unwrap();

        Ok("Programma opgeslagen als 'programma.ecol'.".to_string())
    }

    pub(super) fn execute_laad(&self) -> Result<(), EcolFout> {
        Err(EcolFout::WachtOpLaad)
    }

    pub(super) fn execute_lijst(&self) -> Result<String, EcolFout> {
        let mut reply = String::new();
        for (regelnummer, regel_inh) in self.programma() {
            let regel_inhoud = regel_inh.clone();
            reply.push_str(&Line::new(*regelnummer, regel_inhoud).genereer_regel());
            reply.push('\n');
        }
        Ok(reply)
    }
    pub(super) fn execute_met(&mut self, variabele_naam: &str, stap_expressie: &str, start_expressie: &str, stop_expressie: &str, volgende_regel: u16, lees_geheugen: &mut LeesGeheugen) -> Result<Option<()>, EcolFout> {
        let stap = self. solve_expression(stap_expressie, lees_geheugen)?;
        let start = self. solve_expression(start_expressie, lees_geheugen)?;
        let stop = self. solve_expression(stop_expressie, lees_geheugen)?;

        self.teller_nieuw(variabele_naam, stap, start, stop, volgende_regel)
    }
    pub(super) fn execute_naar(&mut self, lopend_programma: &BTreeMap<u16, LineInhoud>, sprong_doel: &SprongDoel, current: &u16) -> Result<Option<u16>, EcolFout> {
        match sprong_doel.regelnummer() {
            Some(regel) => {
                if lopend_programma.contains_key(&regel) {
                    if regel < *current {
                        self.rollback_program(lopend_programma, *current, regel);
                    }
                    Ok(Some(regel))
                } else {
                    Err(EcolFout::FoutMelding(format!("FOUTMELDING in regel {}: Sprong naar niet gedefinieerde regel {}.", current, regel)))
                }
            },
            None => { Ok(None) },
        }


    }
    pub(super)  fn execute_np(&self, output: &mut dyn FnMut(&str)) -> Result<String, EcolFout> {
        output("\x0C");
        Ok("".to_string())
    }
    pub(super) fn execute_nr(&mut self, aantal: &str, lees_geheugen: &mut LeesGeheugen) -> Result<String, EcolFout> {
        let mut reply = String::new();
        let number = self.solve_expression(aantal, lees_geheugen)?;

        if number > 0f32 {
            reply = format!("{}{}", self.lees_regel(), "\n".repeat(number as usize));
        }
        self.leeg_regel_buffer();
        Ok(reply)
    }
    pub(super) fn execute_rij(&mut self, start: &str, eind: &str, variabele_naam: &str, lees_geheugen: &mut LeesGeheugen) -> Result<String, EcolFout> {
        let begin = self.solve_expression(start, lees_geheugen)?;
        let einde = self.solve_expression(eind, lees_geheugen)?;
        self.var_reserveer_rij(variabele_naam, begin as usize, einde as usize)?;
        Ok("".to_string())
    }
    pub(super) fn execute_rijsym(&mut self, start: &str, eind: &str, variabele_naam: &str, lees_geheugen: &mut LeesGeheugen) -> Result<String, EcolFout> {
        let begin = self.solve_expression(start, lees_geheugen)?;
        let einde = self.solve_expression(eind, lees_geheugen)?;
        self.var_reserveer_rijsym(variabele_naam, begin as usize, einde as usize)?;
        Ok("".to_string())
    }

    pub(super) fn execute_schrijf(&mut self, breedte: usize, decimalen: usize, expressie: &str, lees_geheugen: &mut LeesGeheugen) -> Result<String, EcolFout> {
        let value = self.solve_expression(expressie, lees_geheugen)?;

        self.naar_regel_buffer(&format_getal(value, breedte, decimalen)?)?;
        Ok("".to_string())
    }
    pub(super) fn execute_schrijfsym(&mut self, expressie: &str, lees_geheugen: &mut LeesGeheugen) -> Result<String, EcolFout> {
        let value = self.solve_expression(expressie, lees_geheugen)?;

        let symbool_nummer = get_sym_value(&value)? as usize;

        let Some(s) = SYMBOLEN[symbool_nummer] else {
            return Err(EcolFout::FoutMelding(format!("Symboolwaarde {} is niet gedefinieerd.", symbool_nummer)));
        };
        let symbool: char = s;

        self.naar_regel_buffer(symbool.to_string().as_str())?;
        Ok("".to_string())
    }
    pub(super) fn execute_schrijm(&mut self, expressie: &str, lees_geheugen: &mut LeesGeheugen) -> Result<String, EcolFout> {
        let value = self.solve_expression(expressie, lees_geheugen)?;

        if value.is_nan() {
            return Err(EcolFout::FoutMelding("FOUTMELDING: Expressie levert ongeldige waarde op.".to_string()));
        }
        let s = format!("{:+E}", value);

        let Some((mantisse_deel, exp_deel)) = s.split_once('E') else {
            return Err(EcolFout::FoutMelding(format!("FOUTMELDING: Onverwachte notatie voor waarde: {}", s)));
        };
        let Ok(e) = exp_deel.parse::<i32>() else {
            return Err(EcolFout::FoutMelding(format!("FOUTMELDING: Onverwachte notatie voor exponent: {}", exp_deel)));
        };
        let exp = e + 1;
        let (teken, cijfers) = if let Some(cijfers) = mantisse_deel.strip_prefix('-') {
            ("-", cijfers)
        } else {
            ("+", mantisse_deel.trim_start_matches('+'))
        };

        let ecol_mantisse = format!("0.{}", cijfers.replace('.', ""));

        let exp_teken = if exp >= 0 { "+" } else { "-" };

        self.naar_regel_buffer(&format!("{}{}E{}{}", teken, ecol_mantisse, exp_teken, exp.unsigned_abs()))?;

        Ok("".to_string())
    }
    pub(super) fn execute_spatie(&mut self, aantal: &str, lees_geheugen: &mut LeesGeheugen) -> Result<String, EcolFout> {
        let number = self.solve_expression(aantal, lees_geheugen)? as usize;
        if number > 80 {
            return Err(EcolFout::FoutMelding(format!("SPATIE verwacht een aantal kleiner dan 80 (maximale regelgrootte). Aantal: {}", aantal)));
        }
        let regel = format!("{: <width$}", "", width = number);
        self.naar_regel_buffer(&regel)?;
        Ok("".to_string())
    }
    pub(super) fn execute_start(&self, regel: Option<u16>, lees_geheugen: &mut LeesGeheugen, output: &mut dyn FnMut(&str)) -> Result<String, EcolFout> {
        let mut current = match regel {
            Some(r) => r - 1,
            None => 0,
        };
        let start_tijd = js_sys::Date::now();
        let mut stappen: u32 = 0;

        // Herstel opgeslagen toestand, of maak een verse machine aan.
        let (mut running_program, programma) = match lees_geheugen.neem_lopende_toestand() {
            Some(toestand) => toestand,
            None => {
                let mut m = EcolMachine::new();
                let mut p = m.extract_functie_definities(self.programma())?;
                p = m.extract_sub_definities(&p)?;
                (m, p)
            }
        };

        loop {
            stappen += 1;
            if stappen.is_multiple_of(1000) && js_sys::Date::now() - start_tijd > 5000.0 {
                return Err(EcolFout::FoutMelding("FOUTMELDING: Programma afgebroken na 5 seconden (mogelijke eindeloze lus).".to_string()));
            }

            let Some((&regel_num, current_reg)) = programma.range(current..).next() else {
                return Err(EcolFout::FoutMelding("FOUTMELDING: Er zijn geen regels meer om uit te voeren. KLAAR niet aangetroffen.".to_string()));
            };
            let regelnummer = regel_num;
            let old_current = regelnummer;
            current = regelnummer + 1;
            let line = Line::new(current, current_reg.clone());

            let (_, current_option, whatsnext) =  match execute_all(&line, &mut running_program, &programma, Context::Programma, lees_geheugen, output) {
                Ok((st, co, wh)) => {
                    (st, co, wh)
                },
                Err(EcolFout::FoutMelding(s)) => {
                    return Err(EcolFout::FoutMelding(s));
                },
                Err(EcolFout::WachtOpLees(_)) => {
                    lees_geheugen.sla_lopende_toestand_op(running_program, programma);
                    return Err(EcolFout::WachtOpLees(old_current));
                }
                Err(EcolFout::WachtOpLeessym(_)) => {
                    lees_geheugen.sla_lopende_toestand_op(running_program, programma);
                    return Err(EcolFout::WachtOpLeessym(old_current));
                }
                Err(EcolFout::WachtOpLaad) => {
                    return Err(EcolFout::FoutMelding("Interne fout: LAAD in programma.".to_string()));
                }
            };

            if let Some(next_line) = current_option { current = next_line; }

            match whatsnext {
                Some(WhatsNext::Break) => break,
                Some(WhatsNext::Continue) => continue,
                None => { }
            }
        }

        Ok("Programma is normaal beëindigd.".to_string())
    }
    pub(super) fn execute_tekst(&mut self, expressie: &str) -> Result<String, EcolFout> {
        self.naar_regel_buffer(&literal_to_string(expressie)?)?;
        Ok("".to_string())
    }
    pub(super) fn execute_toekennen(&mut self, variabele_naam: &str, argument: &str, expressie: &str, lees_geheugen: &mut LeesGeheugen) -> Result<String, EcolFout> {
        let value = self.solve_expression(expressie, lees_geheugen)?;
        let arg = if argument.is_empty() {
            0usize
        } else {
            self.solve_expression(argument, lees_geheugen)? as usize
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
                            return Err(EcolFout::FoutMelding("Getalvariabele kan geen index hebben.".to_string()));
                        }
                        waarde = Waarde::new_getal(value);
                    },
                    VariabeleType::Rij => {
                        if arg == 0 {
                            return Err(EcolFout::FoutMelding("Geen index verwijzing bij RIJ-variabele.".to_string()));
                        }
                        waarde.rij_set_value(value, arg)?;
                    },
                    VariabeleType::Rijsym => {
                        if arg == 0 {
                            return Err(EcolFout::FoutMelding("Geen index verwijzing bij RIJSYM-variabele.".to_string()));
                        }

                        waarde.rij_set_value(value, arg)?;
                    },
                    VariabeleType::Teller => {
                        if arg != 0 {
                            return Err(EcolFout::FoutMelding("Teller-variabele kan geen index hebben.".to_string()));
                        }

                        waarde.teller_schrijf_current(value)?;
                    },

                }
            },
            None => {
                if arg != 0 {
                    return Err(EcolFout::FoutMelding(format!("RIJ-Variabele '{}' is niet gedefinieerd.", variabele_naam)));
                }
                waarde = Waarde::new_getal(value);
            }
        }

        self.var_schrijf_waarde(variabele_naam, waarde)?;
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
                LineInhoud::Rij { variabele_naam, .. } |
                LineInhoud::Rijsym { variabele_naam, .. } => {
                    self.var_wis(variabele_naam)
                },
                _ => { continue },
            }
        }
    }
    pub(super) fn extract_sub_definities(&mut self, volledige_programma: &BTreeMap<u16,LineInhoud>) -> Result<BTreeMap<u16,LineInhoud>, EcolFout> {
        let mut nieuwe_programma: BTreeMap<u16, LineInhoud> = BTreeMap::new();
        let mut in_sub_definitie = false;
        let mut sub_def = SubDef::new();
        let mut naam_van_sub: &str = "";

        for (regelnummer, regel) in volledige_programma.iter() {
            match regel {
                LineInhoud::Sub { sub_naam, ..} => {
                    in_sub_definitie = true;
                    naam_van_sub = sub_naam;

                    sub_def = SubDef::new();
                }
                LineInhoud::End { .. } => {
                    if in_sub_definitie {
                        sub_def.regels_insert(*regelnummer, regel);
                        self.schrijf_subregister(naam_van_sub, sub_def.clone())?;
                        naam_van_sub = "";
                        in_sub_definitie = false;
                    } else {
                        nieuwe_programma.insert(*regelnummer, regel.clone());
                    }
                }
                _ => {
                    if in_sub_definitie {
                        sub_def.regels_insert(*regelnummer, regel);
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
    lees_geheugen: &mut LeesGeheugen,
    output: &mut dyn FnMut(&str)) -> UitvoeringResultaat {
    let no_reply_string: Option<String> = None;
    let no_whats_next: Option<WhatsNext> = None;
    let no_next_line: Option<u16> = None;
    let no_reply = (None, None, None);

    let (reply, nextline, whats_next) = match opdracht.inhoud() {
        LineInhoud::Als { vergelijking, dan, anders, .. } => {
            match context {
                Context::Direct => { no_reply },
                Context::Programma  => {
                    if machine.parseer_vergelijking(vergelijking, lees_geheugen)? {
                        doe_naar(machine, programma, dan, &opdracht.regelnummer(), false)?
                    } else if let Some(anders_regel) = anders {
                        doe_naar(machine, programma, anders_regel, &opdracht.regelnummer(), false)?
                    } else {
                        no_reply
                    }
                },
                Context::Subroutine | Context::Functie => {
                    if machine.parseer_vergelijking(vergelijking, lees_geheugen)? {
                        doe_naar(machine, programma, dan, &opdracht.regelnummer(), true)?
                    } else if let Some(anders_regel) = anders {
                        doe_naar(machine, programma, anders_regel, &opdracht.regelnummer(), true)?
                    } else {
                        no_reply
                    }
                },
            }
        },
        LineInhoud::End { .. } => {
            match context {
                Context::Direct | Context::Subroutine => { no_reply },
                Context::Programma | Context::Functie => {
                    return Err(EcolFout::FoutMelding(format!("FOUTMELDING in regel {}: END kan niet voorkomen in een programma of FUNctie (interne fout).", opdracht.regelnummer()-1)));
                },
            }
        },
        LineInhoud::FunStart { .. } => {
            match context {
                Context::Direct => { no_reply },
                Context::Programma | Context::Subroutine => {
                    return Err(EcolFout::FoutMelding(format!("FOUTMELDING in regel {}: Functie kan niet in een programma (interne fout).", opdracht.regelnummer()-1)));
                },
                Context::Functie => {
                    return Err(EcolFout::FoutMelding(format!("FOUTMELDING in regel {}: Functie kan niet in een een andere functie definitie.", opdracht.regelnummer()-1)));
                },

            }
        },
        LineInhoud::FunEind {  .. } => {
            match context {
                Context::Direct | Context::Functie => { no_reply },
                Context::Programma | Context::Subroutine => {
                    return Err(EcolFout::FoutMelding(format!("FOUTMELDING in regel {}: Functie definitie kan niet in een programma (interne fout).", opdracht.regelnummer()-1)));
                },
            }
        },
        LineInhoud::GaSub { sub_naam, .. } => {
            match context {
                Context::Direct => { no_reply },
                Context::Programma | Context::Subroutine => {
                    if let Some(sub_def) = machine.lees_subregister(sub_naam).map(|s| s.clone()) {
                        machine.start_sub(sub_naam, opdracht.regelnummer())?;

                        let subroutine = sub_def.regels();
                        let mut sub_current = 0u16;
                        loop {
                            let Some((&regel_num, current_reg)) = subroutine.range(sub_current..).next() else {
                                break;
                            };
                            sub_current = regel_num + 1;
                            let sub_line= Line::new(sub_current, current_reg.clone());
                            let (_, sub_next_line, whats_next) = execute_all(&sub_line, machine, subroutine, Context::Subroutine, lees_geheugen, output)?;

                            if let Some(next_line) = sub_next_line {
                                sub_current = next_line;
                            }

                            match whats_next {
                                Some(WhatsNext::Continue) => continue,
                                Some(WhatsNext::Break) => break,
                                None => {}
                            }
                        }
                        no_reply
                    }  else {
                        return Err(EcolFout::FoutMelding(format!("FOUTMELDING in regel {}: Onbekende subroutine '{}'.", opdracht.regelnummer(), sub_naam)));
                    }
                },
                Context::Functie => {
                    return Err(EcolFout::FoutMelding(format!("FOUTMELDING in regel {}: Vanuit een functie kan een subroutine niet aangeroepen worden.", opdracht.regelnummer()-1)));
                },

            }
        },
        LineInhoud::Bewaar { } => {
            match context {
                Context::Direct => (Some(machine.execute_bewaar()?), no_next_line, no_whats_next),
                Context::Programma | Context::Subroutine | Context::Functie => {
                    return Err(EcolFout::FoutMelding(format!("FOUTMELDING in regel {}: BEWAAR mag niet in een programma (interpreter-besturing).", opdracht.regelnummer()-1)));
                }
            }
        },
        LineInhoud::Laad { } => {
            match context {
                Context::Direct => {
                    machine.execute_laad()?;
                    no_reply
                },
                Context::Programma | Context::Subroutine | Context::Functie => {
                    return Err(EcolFout::FoutMelding(format!("FOUTMELDING in regel {}: LAAD mag niet in een programma (interpreter-besturing).", opdracht.regelnummer()-1)));
                }
            }
        },
        LineInhoud::Help { } => {
            match context {
                Context::Direct => { (Some(machine.execute_help()?), no_next_line, no_whats_next)  },
                Context::Programma | Context::Subroutine | Context::Functie => {
                    return Err(EcolFout::FoutMelding(format!("FOUTMELDING in regel {}: HELP mag niet in een programma (interpreter-besturing).", opdracht.regelnummer()-1)));
                }

            }
        },
        LineInhoud::Herhaal { .. } => {
            match context {
                Context::Direct => { no_reply },
                Context::Programma | Context::Subroutine | Context::Functie => {
                    let sprong_doel = machine.teller_herhaal()?;
                    let next_line = if let Some(sprong) = sprong_doel {
                        let doel = SprongDoel::Regel( sprong );
                        let resultaat = machine.execute_naar(programma, &doel, &opdracht.regelnummer())?;
                        if resultaat.is_none() {
                            return Err(EcolFout::FoutMelding("Interne FOUT bij HERHAAL-opdracht..".to_string()));
                        }
                        resultaat
                    } else {
                        no_next_line
                    };

                    (no_reply_string, next_line, no_whats_next)
                }

            }
        },
        LineInhoud::Klaar { .. } => {
            match context {
                Context::Direct => { no_reply },
                Context::Programma => {
                    (no_reply_string, no_next_line, Some(WhatsNext::Break))
                },
                Context::Subroutine | Context::Functie => {
                    return Err(EcolFout::FoutMelding(format!("FOUTMELDING in regel {}: KLAAR mag niet in een subroutine of een FUNctie staan.", opdracht.regelnummer())));
                },

            }
        },
        LineInhoud::LegeRegel { .. } => {
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
                    return Err(EcolFout::FoutMelding(format!("FOUTMELDING in regel {}: LIJST mag niet in een programma (interpreter-besturing).", opdracht.regelnummer())));
                },

            }
        },
        LineInhoud::Met { variabele_naam, stap_expressie, start_expressie, stop_expressie, .. } => {
            match context {
                Context::Direct => { no_reply },
                Context::Programma | Context::Subroutine | Context::Functie => {
                    let Some((&volgende_regelnummer, _)) = programma.range(opdracht.regelnummer()..).next() else {
                        return Err(EcolFout::FoutMelding(format!("FOUTMELDING in regel {}: FOUTMELDING: geen regels na MET-opdracht.", opdracht.regelnummer())));
                    };

                    let regel = machine.execute_met(variabele_naam, stap_expressie, start_expressie, stop_expressie, volgende_regelnummer, lees_geheugen)
                        .map_err(|e| e.met_regel(opdracht.regelnummer()))?;
                    match regel {
                        Some(_) => { (no_reply_string, no_next_line, Some(WhatsNext::Continue)) },
                        None => {
                            (no_reply_string, Some(EcolMachine::teller_naar_herhaal(programma, &opdracht.regelnummer())?), Some(WhatsNext::Continue))
                        }
                    }
                },

            }
        },
        LineInhoud::Naar {sprong_doel, .. } => {
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
                            return Err(EcolFout::FoutMelding(format!("FOUTMELDING in regel {}: STOP-functie is niet geldig in  subroutine of functie-definitie.", opdracht.regelnummer()-1)));
                        },
                    }
                },

            }
        },
        LineInhoud::NP { .. } => {
            match context {
                Context::Direct => {
                    (Some(machine.execute_np( output)?), no_next_line, no_whats_next)
                },
                Context::Programma | Context::Subroutine => {
                    (Some(machine.execute_np(output)
                        .map_err(|e| e.met_regel(opdracht.regelnummer() - 1))?), no_next_line, no_whats_next)
                },
                Context::Functie => {
                    return Err(EcolFout::FoutMelding(format!("FOUTMELDING in regel {}: NP kan niet in een FUN definitie (geen uitvoer-apparaat).", opdracht.regelnummer()-1)));
                },

            }
        },
        LineInhoud::NR { aantal, .. } => {
            match context {
                Context::Direct => { (Some(machine.execute_nr(aantal, lees_geheugen)?), no_next_line, no_whats_next) },
                Context::Programma | Context::Subroutine => {
                    output(&machine.execute_nr(aantal, lees_geheugen)
                        .map_err(|e| e.met_regel(opdracht.regelnummer() - 1))?);
                    no_reply
                },
                Context::Functie => {
                    return Err(EcolFout::FoutMelding(format!("FOUTMELDING in regel {}: NR kan niet in een FUN definitie (geen uitvoer-apparaat).", opdracht.regelnummer()-1)));
                },

            }
        },
        LineInhoud::Rij { start, eind, variabele_naam, .. } => {
            match context {
                Context::Direct => {
                    _ = machine.execute_rij(start, eind, variabele_naam, lees_geheugen)?;
                    no_reply
                },
                Context::Programma | Context::Subroutine | Context::Functie => {
                    _ = machine.execute_rij(start, eind, variabele_naam, lees_geheugen)
                        .map_err(|e| e.met_regel(opdracht.regelnummer() - 1))?;
                    no_reply
                },

            }
        },
        LineInhoud::Rijsym { start, eind, variabele_naam, .. } => {
            match context {
                Context::Direct => {
                    _ = machine.execute_rijsym(start, eind, variabele_naam, lees_geheugen)?;
                    no_reply
                },
                Context::Programma | Context::Subroutine | Context::Functie => {
                    _ = machine.execute_rijsym(start, eind, variabele_naam, lees_geheugen)
                        .map_err(|e| e.met_regel(opdracht.regelnummer() - 1))?;
                    no_reply
                },

            }
        },
        LineInhoud::Schrijf { breedte, decimalen, expressie, .. } => {
            match context {
                Context::Direct => {
                    _ = machine.execute_schrijf(*breedte, *decimalen, expressie, lees_geheugen)?;
                    no_reply
                },
                Context::Programma | Context::Subroutine => {
                    _ = machine.execute_schrijf(*breedte, *decimalen, expressie, lees_geheugen)
                        .map_err(|e| e.met_regel(opdracht.regelnummer() - 1))?;
                    no_reply
                },
                Context::Functie => {
                    return Err(EcolFout::FoutMelding(format!("FOUTMELDING in regel {}: SCHRIJF kan niet in een FUN definitie (geen uitvoer-apparaat).", opdracht.regelnummer()-1)));
                },

            }
        },
        LineInhoud::Schrijfsym { expressie, .. } => {
            match context {
                Context::Direct => {
                    _ = machine.execute_schrijfsym(expressie, lees_geheugen)?;
                    no_reply
                },
                Context::Programma | Context::Subroutine => {
                    _ = machine.execute_schrijfsym(expressie, lees_geheugen)
                        .map_err(|e| e.met_regel(opdracht.regelnummer() - 1))?;
                    no_reply
                },
                Context::Functie => {
                    return Err(EcolFout::FoutMelding(format!("FOUTMELDING in regel {}: SCHRIJFSYM kan niet in een FUN definitie (geen uitvoer-apparaat).", opdracht.regelnummer()-1)));
                },

            }
        },
        LineInhoud::Schrijm { expressie, .. } => {
            match context {
                Context::Direct => {
                    _ = machine.execute_schrijm(expressie, lees_geheugen)?;
                    no_reply
                },
                Context::Programma | Context::Subroutine => {
                    _ = machine.execute_schrijm(expressie, lees_geheugen)
                        .map_err(|e| e.met_regel(opdracht.regelnummer() - 1))?;
                    no_reply
                },
                Context::Functie => {
                    return Err(EcolFout::FoutMelding(format!("FOUTMELDING in regel {}: SCHRIJM kan niet in een FUN definitie (geen uitvoer-apparaat).", opdracht.regelnummer()-1)));
                },

            }
        },
        LineInhoud::Spatie { aantal, .. } => {
            match context {
                Context::Direct => {
                    _ = machine.execute_spatie(aantal, lees_geheugen)?;
                    no_reply
                },
                Context::Programma | Context::Subroutine => {
                    _ = machine.execute_spatie(aantal, lees_geheugen)
                        .map_err(|e| e.met_regel(opdracht.regelnummer() - 1))?;
                    no_reply
                },
                Context::Functie => {
                    return Err(EcolFout::FoutMelding(format!("FOUTMELDING in regel {}: SPATIE kan niet in een FUN definitie (geen uitvoer-apparaat).", opdracht.regelnummer()-1)));
                },

            }
        },
        LineInhoud::Start { } => {
            match context {
                Context::Direct => { (Some(machine.execute_start(None, lees_geheugen, output)?), no_next_line, no_whats_next) },
                Context::Programma | Context::Subroutine | Context::Functie => {
                    return Err(EcolFout::FoutMelding(format!("FOUTMELDING in regel {}: START mag niet in een programma (interpreter-besturing).", opdracht.regelnummer()-1)));
                },

            }
        }
        LineInhoud::Sub { .. } => {
            match context {
                Context::Direct => { no_reply },
                Context::Programma | Context::Subroutine | Context::Functie => {
                    return Err(EcolFout::FoutMelding(format!("FOUTMELDING in regel {}: fout bij verwerking subroutine (interne fout).", opdracht.regelnummer()-1)));
                },
            }
        }
        LineInhoud::Tekst { expressie, .. } => {
            match context {
                Context::Direct => {
                    _ = machine.execute_tekst(expressie)?;
                    no_reply
                },
                Context::Programma | Context::Subroutine => {
                    _ = machine.execute_tekst(expressie)
                        .map_err(|e| e.met_regel(opdracht.regelnummer() - 1))?;
                    no_reply
                },
                Context::Functie => {
                    return Err(EcolFout::FoutMelding(format!("FOUTMELDING in regel {}: TEKST kan niet in een FUN definitie (geen uitvoer-apparaat).", opdracht.regelnummer()-1)));
                },

            }
        },
        LineInhoud::Toekennen {variabele_naam, argument,  expressie, ..} => {
            match context {
                Context::Direct => {
                    _ = machine.execute_toekennen(variabele_naam, argument, expressie, lees_geheugen)?;
                    no_reply
                },
                Context::Programma | Context::Subroutine | Context::Functie => {
                    _ = machine.execute_toekennen(variabele_naam, argument, expressie, lees_geheugen)
                        .map_err(|e| e.met_regel(opdracht.regelnummer() - 1))?;
                    no_reply
                },

            }
        },
        LineInhoud::Verwijderen { } => {
            return match context {
                Context::Direct => {
                    Err(EcolFout::FoutMelding("Verwijderen van een ongenummerde regel is niet mogelijk.".to_string()))
                },
                Context::Programma | Context::Subroutine | Context::Functie => {
                    Err(EcolFout::FoutMelding("Verwijderen van een regel kan niet voorkomen in een programma (interne fout).".to_string()))
                },

            }
        },
    };



    Ok((reply, nextline, whats_next))
}

fn doe_naar(machine: &mut EcolMachine
            ,programma: &BTreeMap<u16, LineInhoud>
            ,sprong_doel: &SprongDoel
            ,regel: &u16
,no_stop: bool) -> UitvoeringResultaat {
    match machine.execute_naar(programma, sprong_doel, regel)? {
        Some(regel) => {
            Ok((None, Some(regel), None))
        },
        None => {
            if no_stop {
                return Err(EcolFout::FoutMelding(format!("FOUTMELDING in regel {}: STOP-functie is niet geldig in functie-definitie.", regel - 1)));
            }
            Ok((None, None, Some(WhatsNext::Break)))
        }
    }
}