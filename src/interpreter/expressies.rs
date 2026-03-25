use std::str::FromStr;
use crate::interpreter::EcolMachine;
use crate::interpreter::helpers::{first_word, geen_spaties_buiten_literals, is_geldige_variabele_naam};
use crate::interpreter::parsers::{parseer_functie, parseer_variabele};
use crate::interpreter::program::{Functie, Operator};
use crate::interpreter::waarden::{format_getal, haal_data, waarde_naar_expressie, EcolString, VariabeleType, Waarde};

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

        for o in Operator::operator_volgorde() {
            loop {
                let Some(operator_positie) = werk_expressie.find(o.to_char()) else {
                    break;
                };

                let links_pos = vind_operand_links(werk_expressie.as_str(), operator_positie);
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
            let deel_resultaat = self.solve_number_expression(deel_expressie)?;
            let vervanging = haal_data(&deel_resultaat);

            werk_expressie.replace_range(start..slot + 1, &vervanging);
        }

        if let Some(_) = werk_expressie.find('(') { return Err("Haak openen gevonden zonder haak sluiten".to_string()); }

        *expressie = werk_expressie;
        Ok(())
    }
    pub(super) fn expressie_type(&self, expressie: &str) -> VariabeleType {
        let werk_expressie = geen_spaties_buiten_literals(expressie);

        //1: string of getal?
        let mut is_string = false;
        let first_word = first_word(&werk_expressie);
        //1.1 eerste waarde string literal?
        if werk_expressie.starts_with('"') {
            is_string = true;
        }
        else if werk_expressie.starts_with('(') {
            is_string = false;
        }
        //1.2 eerste waarde een tekst variabele
        else if is_geldige_variabele_naam(first_word) {
            if let Some(variabele_type) = self.haal_variabele_type(first_word) {
                if variabele_type == VariabeleType::Tekst {
                    is_string = true;
                }
            }
        }
        //1.3 eerste waarde een string functie?
        else if Functie::is_string_functie(first_word) {
            is_string = true;
        }

        if is_string {
            VariabeleType::Tekst
        } else {
            VariabeleType::Getal
        }
    }
    pub(super) fn samenstellen_tekst_resultaat(&self, werk_expressie: &str) -> Result<String, String> {

        const ONGELDIGE_TEKST_EXPRESSIE: &str = "Ongeldige tekst-expressie";

        let mut tekst_resultaat = String::new();
        let mut in_quotes = false;
        let mut escape = false;
        let mut plus_seen = false;

        for c in werk_expressie.chars() {
            if !in_quotes {
                match c {
                    '"' => {
                        in_quotes = true;
                        if plus_seen {
                            plus_seen = false;
                        }
                    }
                    '+' if !plus_seen => {
                        plus_seen = true;
                    }
                    ' ' if plus_seen => {}
                    _ if plus_seen => return Err(ONGELDIGE_TEKST_EXPRESSIE.to_string()),
                    _ => {}
                }
            } else {
                if c == '\\' && !escape {
                    escape = true;
                    tekst_resultaat.push(c);
                    continue;
                }

                if escape {
                    escape = false;
                    tekst_resultaat.push(c);
                    continue;
                }

                if c == '"' {
                    in_quotes = false;
                } else {
                    tekst_resultaat.push(c);
                }
            }
        }

        if in_quotes || plus_seen {
            return Err(ONGELDIGE_TEKST_EXPRESSIE.to_string());
        }

        Ok(tekst_resultaat)
    }
    pub(super) fn solve_expression(&mut self, expression: &str) -> Result<Waarde, String> {
        let werk_expressie = geen_spaties_buiten_literals(expression);

        match self.expressie_type(&werk_expressie) {
            VariabeleType::Tekst => self.solve_string_expression(&werk_expressie),
            VariabeleType::Getal => self.solve_number_expression(&werk_expressie),
        }

    }
    pub(super) fn solve_number_expression(&mut self, expressie: &str) -> Result<Waarde, String> {
        let mut werk_expressie = geen_spaties_buiten_literals(expressie);
        self.vervang_variabelen_in_expressie(&mut werk_expressie, VariabeleType::Getal)?;
        self.vervang_functies_in_expressie(&mut werk_expressie, VariabeleType::Getal)?;
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
    pub(super) fn solve_string_expression(&mut self, expressie: &str) -> Result<Waarde, String> {
        let mut werk_expressie = expressie.to_string();

        self.vervang_variabelen_in_expressie(&mut werk_expressie, VariabeleType::Tekst)?;
        self.vervang_functies_in_expressie(&mut werk_expressie, VariabeleType::Tekst)?;

        let tekst_resultaat = self.samenstellen_tekst_resultaat(&werk_expressie)?;
        Ok(Waarde::Tekst(EcolString::new(tekst_resultaat)))
    }
    pub(super) fn vervang_functies_in_expressie(&mut self, werk_expressie: &mut String, variabele_type: VariabeleType) -> Result<(), String> {
        while let Some(werk_functie) = parseer_functie(werk_expressie) {
            let uitkomst = self.execute_function(&werk_functie)?;
            if uitkomst.type_van() != Some(variabele_type) {
                return Err(format!("Functie {:?} past niet in een expressie van het type {:?}", werk_functie.functie(), variabele_type));
            }
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