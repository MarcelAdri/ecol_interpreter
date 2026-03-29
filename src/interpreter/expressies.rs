use std::str::FromStr;
use crate::interpreter::EcolMachine;
use crate::interpreter::helpers::{first_word, geen_spaties_buiten_literals, is_geldige_variabele_naam};
use crate::interpreter::parsers::{parseer_functie, parseer_variabele};
use crate::interpreter::program::{Operator};
use crate::interpreter::functions::{Functie};
use crate::interpreter::waarden::{format_getal, haal_data, waarde_naar_expressie, VariabeleType, Waarde};

impl EcolMachine {
    pub(super) fn bereken_expressie(&mut self, werk_expressie: &mut String) -> Result<(), String> {
        self.bereken_tussen_haakjes(werk_expressie)?;
        self.bereken_operatoren(werk_expressie)?;

        Ok(())
    }
    pub(super) fn bereken_operatoren(&mut self, expressie: &mut String) -> Result<(), String> {
        fn vind_operand_links(expr: &str, op_pos: usize) -> usize {
            let bytes = expr.as_bytes();
            let mut i = op_pos;

            while i > 0 {
                let c = bytes[i - 1] as char;

                if c.is_ascii_digit() || c == '.' {
                    i -= 1;
                    continue;
                }

                if c == '-' {
                    let prev = if i >= 2 { Some(bytes[i - 2] as char) } else { None };

                    let unary = match prev {
                        None => true,
                        Some(p) => p.is_ascii_whitespace() || Operator::is_operator_char(p),
                    };

                    if unary {
                        i -= 1;
                    }
                }

                break;
            }

            i
        }

        fn vind_operand_rechts(expr: &str, op_pos: usize) -> usize {
            let bytes = expr.as_bytes();
            let mut i = op_pos + 1;

            while i < expr.len() {
                let c = bytes[i] as char;
                if i == op_pos + 1 && c == '-' {
                    i += 1;
                    continue;
                }
                if c.is_ascii_digit() || c == '.' {
                    i += 1;
                } else {
                    break;
                }
            }

            i
        }

        let mut werk_expressie = expressie.to_string();
        let mut search_from = 0;

        for o in Operator::operator_volgorde() {
            loop {
                let Some(rel_pos) = werk_expressie[search_from..].find(o.to_char()) else { break; };
                let operator_positie = search_from + rel_pos;

                let links_pos = vind_operand_links(werk_expressie.as_str(), operator_positie);

                if links_pos == operator_positie {
                    // Unaire min — geen linkeroperand, sla deze positie over
                    search_from = operator_positie + 1;
                    continue;
                }

                let rechts_pos = vind_operand_rechts(werk_expressie.as_str(), operator_positie);

                let links_deel = &werk_expressie[links_pos..operator_positie];
                let rechts_deel = &werk_expressie[operator_positie + 1..rechts_pos];

                let links_poging = f32::from_str(links_deel);
                let rechts_poging = f32::from_str(rechts_deel);

                match (links_poging, rechts_poging) {
                    (Ok(links), Ok(rechts)) => {
                        let uitkomst = o.bereken(links, rechts)?;

                        werk_expressie.replace_range(
                            links_pos..rechts_pos,
                            &format_getal(uitkomst),
                        );
                    }
                    _ => return Err("Ongeldige tekens in numerieke expressie".to_string()),
                }
                search_from = 0;
            }
        }

        *expressie = werk_expressie;
        Ok(())
    }
    pub(super) fn bereken_tussen_haakjes(&mut self, expressie: &mut String) -> Result<(), String> {
        let mut werk_expressie = expressie.to_string();

        loop {
            let Some(slot) = werk_expressie.find(')') else {
                break;
            };

            let Some(start) = werk_expressie[..slot].rfind('(') else {
                return Err("Haak sluiten gevonden zonder haak openen".to_string());
            };

            let deel_expressie = &werk_expressie[start + 1..slot];
            let deel_resultaat = self.solve_expression(deel_expressie)?;
            let vervanging = haal_data(&deel_resultaat);

            werk_expressie.replace_range(start..slot + 1, &vervanging);
        }

        if let Some(_) = werk_expressie.find('(') { return Err("Haak openen gevonden zonder haak sluiten".to_string()); }

        *expressie = werk_expressie;
        Ok(())
    }
    pub(super) fn solve_expression(&mut self, expressie: &str) -> Result<Waarde, String> {
        let mut werk_expressie = geen_spaties_buiten_literals(expressie);
        self.vervang_variabelen_in_expressie(&mut werk_expressie, VariabeleType::Getal)?;
        self.vervang_functies_in_expressie(&mut werk_expressie)?;
        self.bereken_expressie(&mut werk_expressie)?;

        let resultaat = f32::from_str(&werk_expressie);

        match resultaat {
            Ok(result) => {
                Ok(Waarde::Getal(result))
            }
            Err(e) => {
                Err(format!("Fout bij parsen van nummer: {}", e))
            }
        }

    }
    pub(super) fn solve_string_expression(&mut self, expressie: &str) -> Result<String, String> {
        Ok(expressie.to_string())
    }
    pub(super) fn vervang_functies_in_expressie(&mut self, werk_expressie: &mut String) -> Result<(), String> {

        while let Some(werk_functie) = parseer_functie(werk_expressie)? {
            let mut argumenten_num: Vec<f32> = Vec::new();
            for argument in werk_functie.argumenten() {
                let arg = self.solve_expression(argument)?.haal_getal();
                argumenten_num.push(arg);
            }
            let functie_naam = werk_functie.functie().clone();
            let functie = &Functie::new(functie_naam, argumenten_num, "")?;

            let uitkomst = self.execute_function(functie)?;

            let result = waarde_naar_expressie(&uitkomst);

            werk_expressie.replace_range(werk_functie.start()..werk_functie.einde(), &result);
        }

        Ok(())
    }
    pub(super) fn vervang_variabelen_in_expressie(&mut self, werk_expressie: &mut String, variabele_type: VariabeleType) -> Result<(), String> {
        while let Some(werk_variabele) = parseer_variabele(werk_expressie) {
            let Some(complete_waarde) = self.pak_of_maak_waarde(&werk_variabele.variabele_naam()) else { return Err("Variabele niet gevonden".to_string());};
            if let Some(var_typ) = complete_waarde.type_van() {
                if var_typ == variabele_type {
                    let result = waarde_naar_expressie(&complete_waarde);

                    werk_expressie.replace_range(werk_variabele.start()..werk_variabele.einde(), &result);
                } else {
                    return Err(format!("Variabele {:?} is niet van het type {:?}", werk_variabele.variabele_naam(), variabele_type));
                }
            } else {
                return Err(format!("Variabele {:?} is niet goed opgeslagen", werk_variabele.variabele_naam()));
            }

        }

        Ok(())
    }
}