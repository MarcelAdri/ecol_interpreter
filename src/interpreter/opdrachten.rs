use std::collections::BTreeMap;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use web_sys::js_sys;
use crate::interpreter::{EcolMachine, LeesGeheugen};
use crate::interpreter::errors::{EcolFout, EcolFoutVariant, EcolSignaal};
use crate::interpreter::helpers::{get_sym_value};
use crate::interpreter::machine::SYMBOLEN;
use crate::interpreter::program::{Line, LineInhoud, SprongDoel, SubDef};
use crate::interpreter::waarden::{VariabeleType, Waarde};

const HELP_PAGINA: &str = concat!("ecol_syntaxis.html?v=", env!("CARGO_PKG_VERSION"));

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
    fn execute_help(&self) -> Result<String, EcolFout> {
        if let Some(window) = web_sys::window() {
            let geopend = window.open_with_url_and_target(HELP_PAGINA, "_blank");
            match geopend {
                Ok(_) => Ok("Zie het help-document in ander tabblad.".to_string()),
                Err(_) => Err(EcolFout::melding(EcolFoutVariant::HelpDocument)),
            }
        } else {
            Err(EcolFout::melding(EcolFoutVariant::HelpDocument))
        }
    }

    fn execute_bewaar(&self) -> Result<String, EcolFout> {
        let tekst = self.execute_lijst();
        if tekst.is_empty() {
            return Ok("Programma is leeg, niets op te slaan.".to_string());
        }
        let window = web_sys::window()
            .ok_or_else(|| EcolFout::melding(EcolFoutVariant::GeenBrowserVenster))?;
        let document = window.document()
            .ok_or_else(|| EcolFout::melding(EcolFoutVariant::GeenDocument))?;

        let array = js_sys::Array::new();
        array.push(&JsValue::from_str(&tekst));
        let blob = web_sys::Blob::new_with_str_sequence(&JsValue::from(array))
            .map_err(|_| EcolFout::melding(EcolFoutVariant::GeenBestand))?;
        let url = web_sys::Url::create_object_url_with_blob(&blob)
            .map_err(|_| EcolFout::melding(EcolFoutVariant::GeenUrl))?;

        let link = document.create_element("a")
            .map_err(|_| EcolFout::melding(EcolFoutVariant::GeenDownloadLink))?;
        link.set_attribute("href", &url).unwrap();
        link.set_attribute("download", "programma.ecol").unwrap();
        link.dyn_ref::<web_sys::HtmlElement>().unwrap().click();
        web_sys::Url::revoke_object_url(&url).unwrap();

        Ok("Programma opgeslagen als 'programma.ecol'.".to_string())
    }

    fn execute_laad(&self) -> Result<(), EcolFout> {
        Err(EcolFout::Signaal(EcolSignaal::WachtOpLaad))
    }

    fn execute_lijst(&self) -> String {
        let mut reply = String::new();
        for (regelnummer, regel_inh) in self.programma() {
            let regel_inhoud = regel_inh.clone();
            reply.push_str(&Line::new(*regelnummer, regel_inhoud).to_string());
            reply.push('\n');
        }
        reply
    }
    fn execute_met(&mut self, variabele_naam: &str, stap_expressie: &str, start_expressie: &str, stop_expressie: &str, volgende_regel: u16, lees_geheugen: &mut LeesGeheugen) -> Result<Option<()>, EcolFout> {
        let stap = self. solve_expression(stap_expressie, lees_geheugen)?;
        let start = self. solve_expression(start_expressie, lees_geheugen)?;
        let stop = self. solve_expression(stop_expressie, lees_geheugen)?;

        self.teller_nieuw(variabele_naam, stap, start, stop, volgende_regel)
    }
    fn execute_naar(&mut self, lopend_programma: &BTreeMap<u16, LineInhoud>, sprong_doel: &SprongDoel, current: &u16) -> Result<Option<u16>, EcolFout> {
        match sprong_doel.regelnummer() {
            Some(regel) => {
                if lopend_programma.contains_key(&regel) {
                    if regel < *current {
                        self.rollback_program(lopend_programma, *current, regel);
                    }
                    Ok(Some(regel))
                } else {
                    Err(EcolFout::melding(EcolFoutVariant::SprongFout(regel)).met_regel(*current))
                }
            },
            None => { Ok(None) },
        }


    }
    fn execute_np(&self, output: &mut dyn FnMut(&str)) -> String {
        output("\x0C");
        String::new()
    }
    fn execute_nr(&mut self, aantal: &str, lees_geheugen: &mut LeesGeheugen) -> Result<String, EcolFout> {
        let number = self.solve_expression(aantal, lees_geheugen)?;

        if !(1f32..=99f32).contains(&number) || number.fract() != 0f32 {
            return Err(EcolFout::melding(EcolFoutVariant::OngeldigAantal("NR".to_string(), 1f32, 99f32, number)));
        }
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let n = number as usize;  //veilig: gevalideerd 1-99 hierboven
        let reply = format!("{}{}", self.lees_regel(), "\n".repeat(n));

        self.leeg_regel_buffer();
        Ok(reply)
    }
    fn execute_rij(&mut self, start: &str, eind: &str, variabele_naam: &str, lees_geheugen: &mut LeesGeheugen) -> Result<String, EcolFout> {
        let begin = self.solve_expression(start, lees_geheugen)?;
        let einde = self.solve_expression(eind, lees_geheugen)?;
        if !(1f32..=9999f32).contains(&begin) || !(1f32..=9999f32).contains(&einde) {
            return Err(EcolFout::melding(EcolFoutVariant::GrenzenRijWaarde(begin, einde)));
        }
        if einde <= begin {
            return Err(EcolFout::melding(EcolFoutVariant::GrenzenVolgorde("RIJ".to_string(), begin, einde)));
        }
        if begin.fract() != 0f32 || einde.fract() != 0f32 {
            return Err(EcolFout::melding(EcolFoutVariant::GrenzenRijIntegers(begin, einde)));
        }
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let b = begin as usize; //veilig: gevalideerd 1-9999 hierboven

        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let e = einde as usize; //veilig: gevalideerd 1-9999 hierboven
        self.var_reserveer_rij(variabele_naam, b, e)?;
        Ok(String::new())
    }
    fn execute_rijsym(&mut self, start: &str, eind: &str, variabele_naam: &str, lees_geheugen: &mut LeesGeheugen) -> Result<String, EcolFout> {
        let begin = self.solve_expression(start, lees_geheugen)?;
        let einde = self.solve_expression(eind, lees_geheugen)?;
        if !(1f32..=9999f32).contains(&begin) || !(1f32..=9999f32).contains(&einde) {
            return Err(EcolFout::melding(EcolFoutVariant::GrenzenRijWaarde(begin, einde)));
        }
        if einde <= begin {
            return Err(EcolFout::melding(EcolFoutVariant::GrenzenVolgorde("RIJSYM".to_string(), begin, einde)));
        }
        if begin.fract() != 0f32 || einde.fract() != 0f32 {
            return Err(EcolFout::melding(EcolFoutVariant::GrenzenRijIntegers(begin, einde)));
        }
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let b = begin as usize; //veilig: gevalideerd 1-9999 hierboven
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let e = einde as usize; //veilig: gevalideerd 1-9999 hierboven
        self.var_reserveer_rijsym(variabele_naam, b, e)?;
        Ok(String::new())
    }

    fn execute_schrijf(&mut self, breedte: usize, decimalen: usize, expressie: &str, lees_geheugen: &mut LeesGeheugen) -> Result<String, EcolFout> {
        let value = self.solve_expression(expressie, lees_geheugen)?;

        self.naar_regel_buffer(&format_getal(value, breedte, decimalen)?)?;
        Ok(String::new())
    }
   fn execute_schrijfsym(&mut self, expressie: &str, lees_geheugen: &mut LeesGeheugen) -> Result<String, EcolFout> {
        let value = self.solve_expression(expressie, lees_geheugen)?;

        if !(0f32..=99f32).contains(&value) || value.fract() != 0f32 {
            return Err(EcolFout::melding(EcolFoutVariant::SymboolWaarde(value)));
        }
        let symbool_nummer = get_sym_value(&value)?;
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let n = symbool_nummer as usize; //veilig: gevalideerd 0-99 hierboven

        let Some(s) = SYMBOLEN[n] else {
            return Err(EcolFout::melding(EcolFoutVariant::SymboolBestaatNiet(symbool_nummer)));
        };
        let symbool: char = s;

        self.naar_regel_buffer(symbool.to_string().as_str())?;
        Ok(String::new())
    }
    fn execute_schrijm(&mut self, expressie: &str, lees_geheugen: &mut LeesGeheugen) -> Result<String, EcolFout> {
        let value = self.solve_expression(expressie, lees_geheugen)?;

        if value.is_nan() {
            return Err(EcolFout::melding(EcolFoutVariant::OngeldigGetal("bij uitkomst expressie".to_string())));
        }
        let s = format!("{value:+E}");

        let Some((mantisse_deel, exp_deel)) = s.split_once('E') else {
            return Err(EcolFout::melding(EcolFoutVariant::OnverwachtNotatieWaarde(s)));
        };
        let Ok(e) = exp_deel.parse::<i32>() else {
            return Err(EcolFout::melding(EcolFoutVariant::OnverwachtNotatieExponent(exp_deel.to_string())));
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

        Ok(String::new())
    }
    fn execute_spatie(&mut self, aantal: &str, lees_geheugen: &mut LeesGeheugen) -> Result<String, EcolFout> {
        let number = self.solve_expression(aantal, lees_geheugen)?;
        if number.fract() != 0f32 || !(1f32..=80f32).contains(&number) {
            return Err(EcolFout::melding(EcolFoutVariant::OngeldigAantal("SPATIE".to_string(), 1f32, 80f32, number)));
        }
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let n = number as usize; //veilig: gevalideerd 1-80 hierboven
        let regel = format!("{: <width$}", "", width = n);
        self.naar_regel_buffer(&regel)?;
        Ok(String::new())
    }
    pub(super) fn execute_start(&self, regel: Option<u16>, lees_geheugen: &mut LeesGeheugen, output: &mut dyn FnMut(&str)) -> Result<String, EcolFout> {
        let mut current = match regel {
            Some(r) => r - 1,
            None => 0,
        };
        let start_tijd = js_sys::Date::now();
        let mut stappen: u32 = 0;

        // Herstel opgeslagen toestand, of maak een verse machine aan.
        let (mut running_program, programma) = if let Some(toestand) = lees_geheugen.neem_lopende_toestand() { toestand } else {
            let mut m = EcolMachine::new();
            let mut p = m.extract_functie_definities(self.programma())?;
            p = m.extract_sub_definities(&p)?;
            (m, p)
        };

        loop {
            stappen += 1;
            if stappen.is_multiple_of(1000) && js_sys::Date::now() - start_tijd > 5000.0 {
                return Err(EcolFout::melding(EcolFoutVariant::ProgrammaAfgebroken));
            }

            let Some((&regel_num, current_reg)) = programma.range(current..).next() else {
                return Err(EcolFout::melding(EcolFoutVariant::GeenKlaar));
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
                Err(EcolFout::Signaal(s)) => {
                    return match s {
                        EcolSignaal::WachtOpLees(_) => {
                            lees_geheugen.sla_lopende_toestand_op(running_program, programma);
                            Err(EcolFout::Signaal(EcolSignaal::WachtOpLees(old_current)))
                        },
                        EcolSignaal::WachtOpLeessym(_) => {
                            lees_geheugen.sla_lopende_toestand_op(running_program, programma);
                            Err(EcolFout::Signaal(EcolSignaal::WachtOpLeessym(old_current)))
                        },
                        EcolSignaal::WachtOpLaad => {
                            Err(EcolFout::melding(EcolFoutVariant::LaadInProgramma))
                        }
                    }
                },
            };

            if let Some(next_line) = current_option { current = next_line; }

            match whatsnext {
                Some(WhatsNext::Break) => break,
                Some(WhatsNext::Continue) => { },
                None => { }
            }
        }

        Ok("Programma is normaal beëindigd.".to_string())
    }
    fn execute_tekst(&mut self, expressie: &str) -> Result<String, EcolFout> {
        self.naar_regel_buffer(&literal_to_string(expressie)?)?;
        Ok(String::new())
    }
    fn execute_toekennen(&mut self, variabele_naam: &str, argument: &str, expressie: &str, lees_geheugen: &mut LeesGeheugen) -> Result<String, EcolFout> {
        let value = self.solve_expression(expressie, lees_geheugen)?;

        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let arg = if argument.is_empty() {
            0usize
        } else {
            let arg_werk = self.solve_expression(argument, lees_geheugen)?;
            if arg_werk.fract() != 0f32 || !(1f32..=9999f32).contains(&arg_werk) {
                return Err(EcolFout::melding(EcolFoutVariant::OngeldigAantal("index van RIJ of RIJSYM".to_string(), 1f32, 9999f32, arg_werk)));
            }
            arg_werk as usize //veilig: gevalideerd 1-9999 hierboven
        };

        let mut waarde = self.var_lees_waarde(variabele_naam).unwrap_or(Waarde::NogNietBepaald).clone();
        let var_type_compleet = self.var_type_van(variabele_naam);
        let variabele_type: VariabeleType;

        if let Some(var_type) = var_type_compleet {
            variabele_type = var_type;
            match variabele_type {
                VariabeleType::Getal => {
                    if arg != 0 {
                        return Err(EcolFout::melding(EcolFoutVariant::WaardeZonderIndex(variabele_naam.to_string(), variabele_type)));
                    }
                    waarde = Waarde::new_getal(value);
                },
                VariabeleType::Rij | VariabeleType::Rijsym => {
                    if arg == 0 {
                        return Err(EcolFout::melding(EcolFoutVariant::WaardeMetIndex(variabele_naam.to_string(), variabele_type)));
                    }
                    waarde.rij_set_value(value, arg)?;
                },
                VariabeleType::Teller => {
                    if arg != 0 {
                        return Err(EcolFout::melding(EcolFoutVariant::WaardeZonderIndex(variabele_naam.to_string(), variabele_type)));
                    }

                    waarde.teller_schrijf_current(value)?;
                },

            }
        } else {
            if arg != 0 {
                return Err(EcolFout::melding(EcolFoutVariant::RijNietGedefinieerd(variabele_naam.to_string())));
            }
            waarde = Waarde::new_getal(value);
        }

        self.var_schrijf_waarde(variabele_naam, waarde)?;
        Ok(String::new())
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
                    self.var_wis(variabele_naam);
                },
                _ => { },
            }
        }
    }
    fn extract_sub_definities(&mut self, volledige_programma: &BTreeMap<u16,LineInhoud>) -> Result<BTreeMap<u16,LineInhoud>, EcolFout> {
        let mut nieuwe_programma: BTreeMap<u16, LineInhoud> = BTreeMap::new();
        let mut in_sub_definitie = false;
        let mut sub_def = SubDef::new();
        let mut naam_van_sub: &str = "";

        for (regelnummer, regel) in volledige_programma {
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
                    return Err(EcolFout::melding(EcolFoutVariant::NietInProgrammaOfFunctie("END".to_string())).met_regel(opdracht.regelnummer()-1));
                },
            }
        },
        LineInhoud::FunStart { .. } => {
            match context {
                Context::Direct => { no_reply },
                Context::Programma | Context::Subroutine => {
                    return Err(EcolFout::melding(EcolFoutVariant::NietInProgrammaOfSubroutine("FUN".to_string())).met_regel(opdracht.regelnummer()-1));
                },
                Context::Functie => {
                    return Err(EcolFout::melding(EcolFoutVariant::NietInFunctie("FUN".to_string())).met_regel(opdracht.regelnummer()-1));
                },

            }
        },
        LineInhoud::FunEind {  .. } => {
            match context {
                Context::Direct | Context::Functie => { no_reply },
                Context::Programma | Context::Subroutine => {
                    return Err(EcolFout::melding(EcolFoutVariant::NietInProgrammaOfSubroutine("FUN :=".to_string())).met_regel(opdracht.regelnummer()-1));
                },
            }
        },
        LineInhoud::GaSub { sub_naam, .. } => {
            match context {
                Context::Direct => { no_reply },
                Context::Programma | Context::Subroutine => {
                    if let Some(sub_def) = machine.lees_subregister(sub_naam).map(SubDef::clone) {
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
                                Some(WhatsNext::Continue) => {},
                                Some(WhatsNext::Break) => break,
                                None => {}
                            }
                        }
                        no_reply
                    }  else {
                        return Err(EcolFout::melding(EcolFoutVariant::OnbekendeSubroutine(sub_naam.to_string())).met_regel(opdracht.regelnummer() - 1));
                    }
                },
                Context::Functie => {
                    return Err(EcolFout::melding(EcolFoutVariant::NietVanuitFunctie).met_regel(opdracht.regelnummer() - 1));
                },

            }
        },
        LineInhoud::Bewaar { } => {
            match context {
                Context::Direct => (Some(machine.execute_bewaar()?), no_next_line, no_whats_next),
                Context::Programma | Context::Subroutine | Context::Functie => {
                    return Err(EcolFout::melding(EcolFoutVariant::NietInProgramma("BEWAAR".to_string())).met_regel(opdracht.regelnummer() - 1));
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
                    return Err(EcolFout::melding(EcolFoutVariant::NietInProgramma("LAAD".to_string())).met_regel(opdracht.regelnummer() - 1));
                }
            }
        },
        LineInhoud::Help { } => {
            match context {
                Context::Direct => { (Some(machine.execute_help()?), no_next_line, no_whats_next)  },
                Context::Programma | Context::Subroutine | Context::Functie => {
                    return Err(EcolFout::melding(EcolFoutVariant::NietInProgramma("HELP".to_string())).met_regel(opdracht.regelnummer() - 1));
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
                            return Err(EcolFout::melding(EcolFoutVariant::HerhaalIntern).met_regel(opdracht.regelnummer() - 1));
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
                    if machine.heeft_open_tellers() {
                        return Err(EcolFout::melding(EcolFoutVariant::MetZonderHerhaal).met_regel(opdracht.regelnummer() - 1));
                    }
                    (no_reply_string, no_next_line, Some(WhatsNext::Break))
                },
                Context::Subroutine | Context::Functie => {
                    return Err(EcolFout::melding(EcolFoutVariant::NietInSubroutineOfFunctie("KLAAR".to_string())).met_regel(opdracht.regelnummer() - 1));
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
                Context::Direct => { (Some(machine.execute_lijst()), no_next_line, no_whats_next)  },
                Context::Programma | Context::Subroutine | Context::Functie => {
                    return Err(EcolFout::melding(EcolFoutVariant::NietInProgramma("LIJST".to_string())).met_regel(opdracht.regelnummer() - 1));
                },

            }
        },
        LineInhoud::Met { variabele_naam, stap_expressie, start_expressie, stop_expressie, .. } => {
            match context {
                Context::Direct => { no_reply },
                Context::Programma | Context::Subroutine | Context::Functie => {
                    let Some((&volgende_regelnummer, _)) = programma.range(opdracht.regelnummer()..).next() else {
                        return Err(EcolFout::melding(EcolFoutVariant::MetZonderRegels).met_regel(opdracht.regelnummer() - 1));
                    };

                    let regel = machine.execute_met(variabele_naam, stap_expressie, start_expressie, stop_expressie, volgende_regelnummer, lees_geheugen)
                        .map_err(|e| e.met_regel(opdracht.regelnummer() - 1))?;
                    match regel {
                        Some(()) => { (no_reply_string, no_next_line, Some(WhatsNext::Continue)) },
                        None => {
                            (no_reply_string, Some(teller_naar_herhaal(programma, &opdracht.regelnummer())?), Some(WhatsNext::Continue))
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
                            return Err(EcolFout::melding(EcolFoutVariant::NietInSubroutineOfFunctie("STOP-functie".to_string())).met_regel(opdracht.regelnummer() - 1));
                        },
                    }
                },

            }
        },
        LineInhoud::NP { .. } => {
            match context {
                Context::Direct | Context::Programma | Context::Subroutine => {
                    (Some(machine.execute_np( output)), no_next_line, no_whats_next)
                },
                Context::Functie => {
                    return Err(EcolFout::melding(EcolFoutVariant::NietInFunctieUitvoer("NP".to_string())).met_regel(opdracht.regelnummer() - 1));
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
                    return Err(EcolFout::melding(EcolFoutVariant::NietInFunctieUitvoer("NR".to_string())).met_regel(opdracht.regelnummer() - 1));
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
                    return Err(EcolFout::melding(EcolFoutVariant::NietInFunctieUitvoer("SCHRIJF".to_string())).met_regel(opdracht.regelnummer() - 1));
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
                    return Err(EcolFout::melding(EcolFoutVariant::NietInFunctieUitvoer("SCHRIJFSYM".to_string())).met_regel(opdracht.regelnummer() - 1));
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
                    return Err(EcolFout::melding(EcolFoutVariant::NietInFunctieUitvoer("SCHRIJM".to_string())).met_regel(opdracht.regelnummer() - 1));
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
                    return Err(EcolFout::melding(EcolFoutVariant::NietInFunctieUitvoer("SPATIE".to_string())).met_regel(opdracht.regelnummer() - 1));
                },

            }
        },
        LineInhoud::Start { } => {
            match context {
                Context::Direct => { (Some(machine.execute_start(None, lees_geheugen, output)?), no_next_line, no_whats_next) },
                Context::Programma | Context::Subroutine | Context::Functie => {
                    return Err(EcolFout::melding(EcolFoutVariant::NietInProgramma("START".to_string())).met_regel(opdracht.regelnummer() - 1));
                },

            }
        }
        LineInhoud::Sub { .. } => {
            match context {
                Context::Direct => { no_reply },
                Context::Programma | Context::Subroutine | Context::Functie => {
                    return Err(EcolFout::melding(EcolFoutVariant::InSubroutine).met_regel(opdracht.regelnummer() - 1));
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
                    return Err(EcolFout::melding(EcolFoutVariant::NietInFunctieUitvoer("TEKST".to_string())).met_regel(opdracht.regelnummer() - 1));
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
                    Err(EcolFout::melding(EcolFoutVariant::VerwijderenOngenummerd))
                },
                Context::Programma | Context::Subroutine | Context::Functie => {
                    Err(EcolFout::melding(EcolFoutVariant::NietInProgramma("verwijderen".to_string())))
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
    if let Some(regel) = machine.execute_naar(programma, sprong_doel, regel)? {
        Ok((None, Some(regel), None))
    } else {
        if no_stop {
            return Err(EcolFout::melding(EcolFoutVariant::NietInFunctie("STOP-functie".to_string())).met_regel(*regel - 1));
        }
        Ok((None, None, Some(WhatsNext::Break)))
    }
}
fn format_getal(getal: f32, breedte: usize, decimalen: usize) -> Result<String, EcolFout> {
    let mut b = breedte;
    let mut d = decimalen;
    let x = getal;

    if breedte == 0 {
        b = 6;
        if x == x.trunc() {
            d = 0;
        } else {
            d = 4;
        }
    }

    let ruimte_voor_teken = usize::from(x < 0f32);
    let b_totaal = if d == 0 { b + ruimte_voor_teken } else { b + d + ruimte_voor_teken + 1 };

    let reply = format!("{x:b_totaal$.d$}");

    if reply.len() > b_totaal {
        return Err(EcolFout::melding(EcolFoutVariant::BreedteTeKlein(x, breedte)))
    }

    Ok(reply)

}
fn literal_to_string (literal: &str) -> Result<String, EcolFout> {
    let werk_string = literal.trim();

    if !werk_string.starts_with('"') {
        return Err(EcolFout::melding(EcolFoutVariant::GeenAanhalingOpen));
    }

    if !werk_string.ends_with('"') {
        return Err(EcolFout::melding(EcolFoutVariant::GeenAanhalingSluiten));
    }

    let inhoud = &werk_string[1..werk_string.len() - 1];

    if inhoud.contains('"') {
        return Err(EcolFout::melding(EcolFoutVariant::MeerTekstblokken));
    }

    Ok(inhoud.to_string())

}
fn teller_naar_herhaal(programma: &BTreeMap<u16, LineInhoud>, regel: &u16) -> Result<u16, EcolFout> {
    let mut current = *regel;
    let mut met_diepte = 0u16;
    loop{
        let Some((&regelnummer, current_regel)) = programma.range(current..).next() else {
            return Err(EcolFout::melding(EcolFoutVariant::MetZonderHerhaal));
        };
        current = regelnummer + 1;
        match current_regel {
            LineInhoud::Met { .. } => {
                met_diepte += 1;
            },
            LineInhoud::Herhaal { .. } => {
                if met_diepte == 0 {
                    return Ok(current)
                }
                met_diepte -= 1;
            },
            _ => {  },
        }
    }
}

