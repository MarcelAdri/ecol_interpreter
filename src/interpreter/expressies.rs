//! Evaluatie van numerieke ECOL-expressies.
//!
//! [`EcolMachine::solve_expression`] is de ingang: de expressie doorloopt een
//! vaste pijplijn — spaties verwijderen, ingebouwde functies vervangen, eigen
//! functies vervangen, variabelen vervangen, haakjes uitrekenen, operators
//! uitrekenen — totdat een kale `f32` overblijft.
use std::collections::HashMap;
use std::str::FromStr;
use crate::interpreter::{EcolMachine, LeesGeheugen};
use crate::interpreter::errors::{EcolFout, EcolFoutVariant};
use crate::interpreter::helpers::{geen_spaties, is_geldige_variabele_naam};
use crate::interpreter::program::{FunDef, Operator};
use crate::interpreter::functions::{Functie, FunctieNaam};
use crate::interpreter::waarden::{VariabeleAanroep, VariabeleType, Waarde};
#[derive(Debug, Clone, PartialEq)]
struct FunctieAanroep {
    functie: FunctieNaam,
    start: usize,
    einde: usize,
    argumenten: Vec<String>,
}
impl FunctieAanroep {
    fn new(functie: FunctieNaam, start: usize, einde: usize, argumenten: Vec<String>) -> Self {
        FunctieAanroep {
            functie,
            start,
            einde,
            argumenten
        }
    }
    fn functie(&self) -> &FunctieNaam {
        &self.functie
    }
    fn start(&self) -> usize {
        self.start
    }
    fn einde(&self) -> usize {
        self.einde
    }
    fn argumenten(&self) -> &Vec<String> {
        &self.argumenten
    }
}


impl EcolMachine {
    fn bereken_expressie(&mut self, werk_expressie: &mut String, lees_geheugen: &mut LeesGeheugen) -> Result<(), EcolFout> {
        self.bereken_tussen_haakjes(werk_expressie, lees_geheugen)?;
        bereken_operatoren(werk_expressie)?;

        Ok(())
    }

    fn bereken_tussen_haakjes(&mut self, expressie: &mut String, lees_geheugen: &mut LeesGeheugen) -> Result<(), EcolFout> {
        let mut werk_expressie = expressie.clone();

        loop {
            let Some(slot) = werk_expressie.find(')') else {
                break;
            };

            let Some(start) = werk_expressie[..slot].rfind('(') else {
                return Err(EcolFout::melding(EcolFoutVariant::GeenHaakOpenen));
            };

            let deel_expressie = &werk_expressie[start + 1..slot];
            let deel_resultaat = self.solve_expression(deel_expressie, lees_geheugen)?.to_string();

            werk_expressie.replace_range(start..=slot, deel_resultaat.trim());
        }

        if werk_expressie.find('(').is_some() { return Err(EcolFout::melding(EcolFoutVariant::GeenHaakSluiten)); }

        *expressie = werk_expressie;
        Ok(())
    }
    pub(super) fn solve_expression(&mut self, expressie: &str, lees_geheugen: &mut LeesGeheugen) -> Result<f32, EcolFout> {
        let mut werk_expressie = geen_spaties(expressie);
        self.vervang_functies_in_expressie(&mut werk_expressie, lees_geheugen)?;
        self.vervang_eigen_functies_in_expressie(&mut werk_expressie, lees_geheugen)?;
        self.vervang_variabelen_in_expressie(&mut werk_expressie, lees_geheugen)?;
        self.bereken_expressie(&mut werk_expressie, lees_geheugen)?;
        
        let resultaat = f32::from_str(&werk_expressie);

        match resultaat {
            Ok(result) => {
                if result.is_infinite() || result.is_nan() {
                    return Err(EcolFout::melding(EcolFoutVariant::GetalBuitenGrens(expressie.to_string())));
                }
                controleer_precisie(result, &werk_expressie)?;
                Ok(result)
            }
            Err(_) => {
                Err(EcolFout::melding(EcolFoutVariant::OngeldigGetal(expressie.to_string())))
            }
        }

    }
    fn vervang_functies_in_expressie(&mut self, werk_expressie: &mut String, lees_geheugen: &mut LeesGeheugen) -> Result<(), EcolFout> {

        while let Some(w) = parseer_functie(werk_expressie)? {
            let werk_functie: FunctieAanroep = w;
            let functie_naam = werk_functie.functie().clone();

            let functie: Functie = if functie_naam == FunctieNaam::ONDIN || functie_naam == FunctieNaam::BOVIN {
                let argumenten = werk_functie.argumenten();
                if argumenten.len() != 1 {
                    return Err(EcolFout::melding(EcolFoutVariant::VerkeerdAantalArgumenten(functie_naam.to_string(), 1, argumenten.len())));
                }
                Functie::new(&functie_naam, &[], &argumenten[0])?
            } else {
                let mut argumenten_num: Vec<f32> = Vec::new();
                for argument in werk_functie.argumenten() {
                    let arg = self.solve_expression(argument, lees_geheugen)?;
                    argumenten_num.push(arg);
                }

                Functie::new(&functie_naam, &argumenten_num, "")?
            };


            let uitkomst = self.execute_function(lees_geheugen, &functie)?.to_string();

            werk_expressie.replace_range(werk_functie.start()..werk_functie.einde(), uitkomst.trim());
        }

        Ok(())
    }
    fn vervang_eigen_functies_in_expressie(&mut self, werk_expressie: &mut String, lees_geheugen: &mut LeesGeheugen) -> Result<(), EcolFout> {

        while let Some((naam, start, einde, argumenten)) = parseer_eigen_functie(self.haal_functie_register(), werk_expressie) {

            let verwachte_argumenten = match self.get_fundef_parameters(&naam) {
                None => return Err(EcolFout::melding(EcolFoutVariant::GeenFunctie(naam))),
                Some(params) if params.is_empty() => return Err(EcolFout::melding(EcolFoutVariant::GeenArgumenten(naam))),
                Some(params) => params,
            };
            let mut doel_argumenten: Vec<Waarde> = Vec::new();

            if verwachte_argumenten.len() != argumenten.len() {
                return Err(EcolFout::melding(EcolFoutVariant::VerkeerdAantalArgumenten(naam, verwachte_argumenten.len(), argumenten.len())));
            }

            for (index, (verwacht_argument, argument)) in verwachte_argumenten.iter()
                .zip(argumenten.iter())
                .enumerate()
            {
                if verwacht_argument.starts_with("RIJSYM"){
                    if self.var_type_van(argument) != Some(VariabeleType::Rijsym) {
                        return Err(EcolFout::melding(EcolFoutVariant::VerkeerdArgument(index + 1, naam, "RIJSYM".to_string(), argument.clone())));
                    }
                    let Some(waarde) = self.var_lees_waarde(argument)  else { return Err(EcolFout::melding(EcolFoutVariant::GeenVariabele(argument.clone())));};
                    doel_argumenten.push(waarde);
                } else if verwacht_argument.starts_with("RIJ") {
                    if self.var_type_van(argument) != Some(VariabeleType::Rij) {
                        return Err(EcolFout::melding(EcolFoutVariant::VerkeerdArgument(index + 1, naam, "RIJ".to_string(), argument.clone())));
                    }
                    let Some(waarde) = self.var_lees_waarde(argument)  else { return Err(EcolFout::melding(EcolFoutVariant::GeenVariabele(argument.clone())));};
                    doel_argumenten.push(waarde);
                } else {
                    doel_argumenten.push(Waarde::Getal(self.solve_expression(argument, lees_geheugen)?));
                }
            }

            let uitkomst = self.execute_eigen_functie(&naam, &doel_argumenten, lees_geheugen)?.to_string();

            werk_expressie.replace_range(start..einde, uitkomst.trim());
        }

        Ok(())
    }
    fn vervang_variabelen_in_expressie(&mut self, werk_expressie: &mut String, lees_geheugen: &mut LeesGeheugen) -> Result<(), EcolFout> {
        while let Some(werk_variabele) = parseer_variabele(werk_expressie) {
            let Some(complete_waarde) = self.var_lees_waarde(werk_variabele.variabele_naam())
                else { return Err(EcolFout::melding(EcolFoutVariant::GeenVariabele(werk_variabele.variabele_naam().to_string())));};
            if let Some(var_typ) = complete_waarde.type_van() {
                if var_typ != VariabeleType::Getal && var_typ != VariabeleType::Rij && var_typ != VariabeleType::Rijsym && var_typ != VariabeleType::Teller {
                    return Err(EcolFout::melding(EcolFoutVariant::GeenWaardeMogelijk(werk_variabele.variabele_naam().to_string(), var_typ)));
                }
                let index_expressie = werk_variabele.index().unwrap_or("0");
                let p = self.solve_expression(index_expressie, lees_geheugen)?;
                let pos = f32_naar_usize(p)?;
                
                if (var_typ == VariabeleType::Getal || var_typ == VariabeleType::Teller) && p != 0f32 {
                    return Err(EcolFout::melding(EcolFoutVariant::WaardeZonderIndex(werk_variabele.variabele_naam().to_string(), var_typ)));
                }
                if var_typ == VariabeleType::Rij || var_typ == VariabeleType::Rijsym {
                    let (b, e) = complete_waarde.rij_haal_grenswaarden();
                    if pos < b || pos > e {
                        return Err(EcolFout::melding(EcolFoutVariant::OngeldigeIndex(pos, b, e)));
                    }
                }
                let result = complete_waarde.haal_getal(pos)?.to_string();

                werk_expressie.replace_range(werk_variabele.start()..werk_variabele.einde(), result.trim());

            } else {
                return Err(EcolFout::melding(EcolFoutVariant::GeenVariabele(werk_variabele.variabele_naam().to_string())));
            }

        }

        Ok(())
    }
}

fn bereken_operatoren(expressie: &mut String) -> Result<(), EcolFout> {
    let mut werk_expressie = expressie.clone();
    
    for groep in Operator::operator_prioriteiten() {
        loop {
            let mut gevonden: Option<(usize, Operator)> = None;
            for (i, c) in werk_expressie.char_indices() {
                if let Some(o) = Operator::from_char(c) {
                    if groep.contains(&o) {
                        let links_pos = vind_operand_links(&werk_expressie, i);
                        if links_pos < i {
                            gevonden = Some((i, o));
                            break;
                        }
                    }
                }
            }

            let Some((operator_positie, o)) = gevonden else { break; };

            let links_pos = vind_operand_links(&werk_expressie, operator_positie);
            let rechts_pos = vind_operand_rechts(&werk_expressie, operator_positie);
            let links_deel = &werk_expressie[links_pos..operator_positie];
            let rechts_deel = &werk_expressie[operator_positie + 1..rechts_pos];
            let links_poging = f32::from_str(links_deel);
            let rechts_poging = f32::from_str(rechts_deel);

            match (links_poging, rechts_poging) {
                (Ok(links), Ok(rechts)) => {
                    controleer_precisie(links, links_deel)?;
                    controleer_precisie(rechts, rechts_deel)?;
                    let uitkomst = o.bereken(links, rechts)?;
                    if uitkomst.is_infinite() || uitkomst.is_nan() {
                        return Err(EcolFout::melding(EcolFoutVariant::GetalBuitenGrens(format!("{links_deel} {o} {rechts_deel}"))));
                    }
                    werk_expressie.replace_range(links_pos..rechts_pos, &uitkomst.to_string());
                }
                _ => return Err(EcolFout::melding(EcolFoutVariant::OngeldigeTekens(format!("{links_deel} {o} {rechts_deel}")))),
            }
        }
    }


    

    *expressie = werk_expressie;
    Ok(())
}

fn controleer_precisie(getal: f32, source: &str) -> Result<(), EcolFout> {
    if let Ok(als_f64) = f64::from_str(source) {
        if (f64::from(getal) - als_f64).abs() > 0.5 {
            return Err(EcolFout::melding(EcolFoutVariant::PrecisieVerlies(source.to_string())));
        }
    }
    Ok(())
}
fn f32_naar_usize(x: f32) -> Result<usize, EcolFout> {
    if !x.is_finite() {
        return Err(EcolFout::melding(EcolFoutVariant::OngeldigGetal("bij bepaling index".to_string())));
    }
    if x < 0f32 {
        return Err(EcolFout::melding(EcolFoutVariant::GeenPositiefGetal(x)));
    }
    if x.fract() != 0f32 {
        return Err(EcolFout::melding(EcolFoutVariant::GeenGeheelGetal(x)));
    }
    #[allow(clippy::cast_possible_truncation)]
    #[allow(clippy::cast_sign_loss)]
    Ok(x as usize) //Veilig, want hierboven gevalideerd dat het een geldig, positief geheel getal is
}
fn haal_index_expressie(expressie: &str, naam_einde: usize) -> (Option<String>, usize) {
    let rest = &expressie[naam_einde..];
    if !rest.trim_start().starts_with('(') {
        return (None, naam_einde);
    }
    let rest_getrimd = rest.trim_start();  // begint op '('
    match vind_sluitende_haak(rest_getrimd) {
        Ok(rel_sluit) => {
            let inhoud = rest_getrimd[1..rel_sluit].trim().to_string();
            let abs_sluit = naam_einde + (rest.len() - rest_getrimd.len()) + rel_sluit;
            (Some(inhoud), abs_sluit + 1)
        }
        Err(_) => (None, naam_einde),
    }
}
fn parseer_eigen_functie(functie_register: &HashMap<String, FunDef>, expressie: &str) -> Option<(String, usize, usize, Vec<String>)> {
    let mut naam_start: Option<usize> = None;

    for (i, c) in expressie.char_indices() {

        match naam_start {
            None => {
                if c.is_ascii_lowercase() {
                    naam_start = Some(i);
                }
            }
            Some(start) => {
                if c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' {
                    continue;
                }

                let naam = &expressie[start..i];
                if functie_register.contains_key(naam) {
                    let (arg_str, einde) = haal_index_expressie(expressie, i);
                    let argumenten = match arg_str {
                        Some(s) => splits_argumenten(&s),
                        None => Vec::new(),
                    };
                    return Some((naam.to_string(), start, einde, argumenten));
                }
                naam_start = None;
            }
        }

    }
    if let Some(start) = naam_start {
        let naam = &expressie[start..];
        if functie_register.contains_key(naam) {
            return Some((naam.to_string(), start, expressie.len(), Vec::new()));
        }
    }

    None
}
fn parseer_functie(expressie: &str) -> Result<Option<FunctieAanroep>, EcolFout> {
    // 1+2. Zoek de eerste hoofdletterreeks die geen operator is
    let mut zoek_vanaf = 0;
    let (start_naam, einde_naam) = loop {
        let Some(rel) = expressie[zoek_vanaf..].find(|c: char| c.is_ascii_uppercase()) else { return Ok(None) };
        let start = zoek_vanaf + rel;
        let einde = expressie[start..]
            .find(|c: char| !c.is_ascii_uppercase())
            .map_or(expressie.len(), |p| start + p);
        if Operator::is_operator_string(&expressie[start..einde]) {
            zoek_vanaf = einde;
            continue;
        }
        break (start, einde);
    };

    let functienaam = &expressie[start_naam..einde_naam];
    let Some(functie) = FunctieNaam::from_str(functienaam) else {
        return Err(EcolFout::melding(EcolFoutVariant::OngeldigeFunctieNaam(functienaam.to_string())));
    };

    // 3. Geen argumenten: direct klaar
    if functie.verwacht_argumenten() == 0 && !functie.verwacht_string_argument() {
        return Ok(Some(FunctieAanroep::new(functie, start_naam, einde_naam, vec![])));
    }

    // 4. Zoek openingshaakje
    let rest = expressie[einde_naam..].trim_start();
    let rest_offset = expressie.len() - rest.len(); // absolute byte-offset van `rest` in `expressie`
    if !rest.starts_with('(') {
        return Err(EcolFout::melding(EcolFoutVariant::GeenArgumenten(functienaam.to_string())));
    }
    // 5. Zoek bijbehorend sluithaakje (haakjes-diepte meetellen)
    let rel_sluit = vind_sluitende_haak(rest)?;

    // 6. Parseer argumenten
    let argumenten_str = &rest[1..rel_sluit];
    let argumenten = splits_argumenten(argumenten_str);

    if argumenten.len() != functie.verwacht_argumenten() && !functie.verwacht_string_argument() {
        return Err(EcolFout::melding(EcolFoutVariant::VerkeerdAantalArgumenten(
            functienaam.to_string(),
            functie.verwacht_argumenten(),
            argumenten.len()
        )));
    } else if functie.verwacht_string_argument()  && argumenten.len() != 1 {
        return Err(EcolFout::melding(EcolFoutVariant::GeenStringArgument(
            functienaam.to_string(),
            argumenten.len()
        )));
    }

    Ok(Some(FunctieAanroep::new(functie, start_naam, rest_offset + rel_sluit + 1, argumenten)))
}
fn parseer_variabele(expressie: &str) -> Option<VariabeleAanroep> {
    let mut naam_start: Option<usize> = None;

    for (i, c) in expressie.char_indices() {

        match naam_start {
            None => {
                if c.is_ascii_lowercase() {
                    naam_start = Some(i);
                }
            }
            Some(start) => {
                if c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' {
                    continue;
                }

                let naam = &expressie[start..i];
                if is_geldige_variabele_naam(naam) {
                    let (index, einde) = haal_index_expressie(expressie, i);
                    return Some(VariabeleAanroep::new(naam.to_string(), start, einde, index));
                }
                naam_start = None;
            }
        }

    }
    if let Some(start) = naam_start {
        let naam = &expressie[start..];
        if is_geldige_variabele_naam(naam) {
            return Some(VariabeleAanroep::new(naam.to_string(), start, expressie.len(), None));
        }
    }

    None
}
fn splits_argumenten(s: &str) -> Vec<String> {
    if s.trim().is_empty() {
        return vec![];
    }

    let mut argumenten = Vec::new();
    let mut diepte = 0usize;
    let mut start = 0;

    for (i, c) in s.char_indices() {
        match c {
            '(' => diepte += 1,
            ')' => diepte -= 1,
            ',' if diepte == 0 => {
                argumenten.push(s[start..i].trim().to_string());
                start = i + 1;
            }
            _ => {}
        }
    }

    argumenten.push(s[start..].trim().to_string());
    argumenten
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
    let mut einde = expr[op_pos + 1..].len();

    for (i, c) in expr[op_pos + 1..].char_indices() {
        if i == 0 && c == '-' {
            continue;
        }
        if c.is_ascii_digit() || c == '.' {
            continue;
        }
        einde = i;
        break;
    };

    einde + op_pos + 1
}
/// Zoekt de `)` die hoort bij de `(` op `open`-positie in `s`.
fn vind_sluitende_haak(s: &str) -> Result<usize, EcolFout> {
    // s begint op de '('
    let mut diepte = 0usize;
    for (i, c) in s.char_indices() {
        match c {
            '(' => diepte += 1,
            ')' => {
                diepte -= 1;
                if diepte == 0 { return Ok(i); }
            }
            _ => {}
        }
    }
    Err(EcolFout::melding(EcolFoutVariant::GeenHaakSluiten))
}