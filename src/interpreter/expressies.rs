use std::str::FromStr;
use crate::interpreter::EcolMachine;
use crate::interpreter::helpers::{geen_spaties};
use crate::interpreter::parsers::{parseer_eigen_functie, parseer_functie, parseer_variabele};
use crate::interpreter::program::{Operator};
use crate::interpreter::functions::{Functie, FunctieNaam};
use crate::interpreter::waarden::{VariabeleType, Waarde};


impl EcolMachine {
    pub(super) fn bereken_expressie(&mut self, werk_expressie: &mut String) -> Result<(), String> {
        self.bereken_tussen_haakjes(werk_expressie)?;
        bereken_operatoren(werk_expressie)?;

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
            let deel_resultaat = self.solve_expression(deel_expressie)?.to_string();

            werk_expressie.replace_range(start..slot + 1, &deel_resultaat.trim());
        }

        if let Some(_) = werk_expressie.find('(') { return Err("Haak openen gevonden zonder haak sluiten".to_string()); }

        *expressie = werk_expressie;
        Ok(())
    }
    pub(super) fn solve_expression(&mut self, expressie: &str) -> Result<f32, String> {
        let mut werk_expressie = geen_spaties(expressie);
        self.vervang_functies_in_expressie(&mut werk_expressie)?;
        self.vervang_eigen_functies_in_expressie(&mut werk_expressie)?;
        self.vervang_variabelen_in_expressie(&mut werk_expressie)?;
        self.bereken_expressie(&mut werk_expressie)?;
        //panic!("expressie: {}", werk_expressie);
        let resultaat = f32::from_str(&werk_expressie);

        match resultaat {
            Ok(result) => {
                Ok(result)
            }
            Err(e) => {
                Err(format!("\"{}\" levert geen geldig getal: {}", expressie, e))
            }
        }

    }
    pub(super) fn vervang_functies_in_expressie(&mut self, werk_expressie: &mut String) -> Result<(), String> {

        while let Some(werk_functie) = parseer_functie(werk_expressie)? {
            let functie_naam = werk_functie.functie().clone();
            let functie: Functie;

            if functie_naam == FunctieNaam::ONDIN || functie_naam == FunctieNaam::BOVIN {
                let argumenten = werk_functie.argumenten();
                if argumenten.len() != 1 {
                    return Err(format!("Functie {} verwacht slechts één argument ({}).", functie_naam.to_string(), argumenten.len()));
                }
                functie = Functie::new(functie_naam, Vec::new(), &argumenten[0])?;
            } else {
                let mut argumenten_num: Vec<f32> = Vec::new();
                for argument in werk_functie.argumenten() {
                    let arg = self.solve_expression(argument)?;
                    argumenten_num.push(arg);
                }

                functie = Functie::new(functie_naam, argumenten_num, "")?;
            }


            let uitkomst = self.execute_function(&functie)?.to_string();

            werk_expressie.replace_range(werk_functie.start()..werk_functie.einde(), &uitkomst.trim());
        }

        Ok(())
    }
    pub(super) fn vervang_eigen_functies_in_expressie(&mut self, werk_expressie: &mut String) -> Result<(), String> {

        while let Some((naam, start, einde, argumenten)) = parseer_eigen_functie(self.haal_functie_register(), werk_expressie) {

            let verwachte_argumenten = match self.get_fundef_parameters(&naam) {
                None => return Err(format!("Interne fout: functie '{}' niet gevonden.", naam)),
                Some(params) if params.is_empty() => return Err(format!("Functie '{}' heeft geen parameters.", naam)),
                Some(params) => params,
            };
            let mut doel_argumenten: Vec<Waarde> = Vec::new();

            if verwachte_argumenten.len() != argumenten.len() {
                return Err(format!("Functie {} verwacht {} argumenten, maar {} argumenten zijn opgegeven.", naam, verwachte_argumenten.len(), argumenten.len()));
            }

            for index in 0..verwachte_argumenten.len() {
                let verwacht_argument = &verwachte_argumenten[index];
                let argument = &argumenten[index];
                if verwacht_argument.starts_with("RIJSYM"){
                    if self.var_type_van(argument) != Some(VariabeleType::Rijsym) {
                        return Err(format!("Argument {} van functie {} verwacht een RIJSYM, maar {} is opgegeven.", index + 1, naam, argument));
                    }
                    let Some(waarde) = self.var_lees_waarde(argument)  else { return Err("Variabele niet gevonden".to_string());};
                    doel_argumenten.push(waarde);
                } else if verwacht_argument.starts_with("RIJ") {
                    if self.var_type_van(argument) != Some(VariabeleType::Rij) {
                        return Err(format!("Argument {} van functie {} verwacht een RIJ, maar {} is opgegeven.", index + 1, naam, argument));
                    }
                    let Some(waarde) = self.var_lees_waarde(argument)  else { return Err("Variabele niet gevonden".to_string());};
                    doel_argumenten.push(waarde);
                } else {
                    doel_argumenten.push(Waarde::Getal(self.solve_expression(argument)?));
                }
            }

            let uitkomst = self.execute_eigen_functie(&naam, doel_argumenten)?.to_string();

            werk_expressie.replace_range(start..einde, &uitkomst.trim());
        }

        Ok(())
    }
    pub(super) fn vervang_variabelen_in_expressie(&mut self, werk_expressie: &mut String) -> Result<(), String> {
        while let Some(werk_variabele) = parseer_variabele(werk_expressie) {
            let Some(complete_waarde) = self.var_lees_waarde(&werk_variabele.variabele_naam())
                else { return Err("Variabele niet gevonden".to_string());};
            if let Some(var_typ) = complete_waarde.type_van() {
                if var_typ != VariabeleType::Getal && var_typ != VariabeleType::Rij && var_typ != VariabeleType::Rijsym && var_typ != VariabeleType::Teller {
                    return Err(format!("Variabele {:?} is type {:?} en kan niet herleid worden tot een waarde.", werk_variabele.variabele_naam(), var_typ));
                }
                let index_expressie = werk_variabele.index().unwrap_or("0");
                let positie = self.solve_expression(index_expressie)? as usize;

                let result = complete_waarde.haal_getal(positie)?.to_string();

                werk_expressie.replace_range(werk_variabele.start()..werk_variabele.einde(), &result.trim());

            } else {
                return Err(format!("Variabele {:?} is niet goed opgeslagen", werk_variabele.variabele_naam()));
            }

        }

        Ok(())
    }
}

pub(super) fn bereken_operatoren(expressie: &mut String) -> Result<(), String> {
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
                    let uitkomst = o.bereken(links, rechts)?.to_string();

                    werk_expressie.replace_range(
                        links_pos..rechts_pos,
                        &uitkomst,
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