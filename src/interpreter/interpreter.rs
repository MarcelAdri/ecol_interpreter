const HELP_PAGINA: &str = "ecol_syntaxis.html";

use std::collections::HashMap;
use std::f32;
use std::str::FromStr;
use super::waarden::{haal_data, waarde_naar_expressie, format_getal, VariabeleType, Waarde, EcolString};
use super::program::{Line, LineInhoud, Sleutelwoord, Operator, WORDT_TEKEN, Functie, Programma};

struct FunctieAanroep {
    functie: Functie,
    start: usize,
    einde: usize,
    argumenten: String,
}


impl FunctieAanroep {
    fn new(functie: Functie, start: usize, einde: usize, argumenten: String) -> Self {
        FunctieAanroep {
            functie,
            start,
            einde,
            argumenten,
        }
    }
}

struct VariabeleAanroep {
    variabele_naam: String,
    start: usize,
    einde: usize,
}
 impl VariabeleAanroep {
     fn new(variabele_naam: String, start: usize, einde: usize) -> Self {
         VariabeleAanroep {
             variabele_naam,
             start,
             einde,
         }
     }
 }
pub struct EcolMachine {
    symbolen: HashMap<String, usize>,
    data_pool: Vec<Waarde>,
    regel_buffer: String,
    programma: Programma,
}

impl EcolMachine {
    pub fn new() -> Self {
        EcolMachine {
            symbolen: HashMap::new(),
            data_pool: Vec::with_capacity(100),
            regel_buffer: String::new(),
            programma: Programma::new(),
        }
    }

    fn pak_of_maak_index(&mut self, naam: &str, variabele_type: Option<VariabeleType>) -> Result<usize, String> {
        use std::collections::hash_map::Entry;

        let volgende_vrije_index = self.data_pool.len();

        match self.symbolen.entry(naam.to_string()) {
            Entry::Vacant(entry) => {
                let var_type = variabele_type
                    .ok_or_else(|| format!("Variabele '{}' heeft geen type", naam))?;

                self.data_pool.push(Waarde::standaard_voor_type(var_type));
                entry.insert(volgende_vrije_index);
                Ok(volgende_vrije_index)
            }
            Entry::Occupied(entry) => Ok(*entry.get()),
        }
    }


    fn haal_variabele_type(&mut self, naam: &str) -> Option<VariabeleType> {
        if let Some(index) = self.symbolen.get(naam) {
            self.data_pool[*index].type_van()
        } else {
            None
        }
    }

    fn naar_regel_buffer(&mut self, regel: &str) {
        self.regel_buffer.push_str(regel);
    }
    pub fn execute(&mut self, input: &str) -> String {
        let reply: String;

        match parseer_regel(&input){
            Ok(regel) => {
                if regel.regelnummer == 0 {
                    match &regel.inhoud {
                        LineInhoud::Help { } => {
                            reply = self.execute_help();
                        },
                        LineInhoud::NR { } => {
                            reply = self.execute_nr();
                        },
                        LineInhoud::Schrijf { breedte, decimalen, expressie } => {
                            reply = self.execute_schrijf(*breedte, *decimalen, expressie);
                        },
                        LineInhoud::Tekst { expressie } => {
                            reply = self.execute_tekst(expressie);
                        },
                        LineInhoud::Toekennen {variabele_naam, expressie} => {
                            reply = self.execute_toekennen(variabele_naam, expressie);
                        },
                    }
                } else {
                    reply = self.programma.regel_toevoegen(regel);
                }
            }
            Err(e) => {
                return format!("Ongeldige invoer: {}" ,e);
            }
        }
       
        reply
    }

    fn execute_help(&mut self) -> String {
        if let Some(window) = web_sys::window() {
            let _ = window.open_with_url_and_target(HELP_PAGINA, "_blank");
        }

        "Zie het help-document in ander tabblad.".to_string()
    }

    fn execute_nr(&mut self) -> String {
        std::mem::take(&mut self.regel_buffer)
    }

    fn execute_schrijf(&mut self, breedte: usize, decimalen: usize, expressie: &str) -> String {
        //TODO: formatting aan de hand van breedte en decimalen
        match self.solve_expression(expressie) {
            Ok(value) => {
                self.naar_regel_buffer(&haal_data(&value));
                "".to_string()
            }
            Err(error_message) => error_message,
        }
    }
    fn execute_tekst(&mut self, expressie: &str) -> String {

        match self.solve_expression(expressie) {
            Ok(value) => {
                self.naar_regel_buffer(&haal_data(&value));
                "".to_string()
            }
            Err(error_message) => error_message,
        }
    }


    fn execute_toekennen(&mut self, variabele_naam: &str, expressie: &str) -> String {
        let mut var_type: VariabeleType;

        if let Some(variabeleType) = self.haal_variabele_type(variabele_naam) {
            var_type = variabeleType;
        } else {
            var_type = self.expressie_type(expressie);
        }

        let variabele_index = match self.pak_of_maak_index(variabele_naam, Some(var_type)) {
            Ok(index) => index,
            Err(e) => return e,
        };

        match self.solve_expression(expressie) {
            Ok(value) => {
                self.data_pool[variabele_index] = value;
                "".to_string()
            }
            Err(error_message) => {
                error_message
            }
        }
    }


    fn expressie_type(&mut self, expressie: &str) -> VariabeleType {
        let mut werk_expressie = geen_spaties_buiten_literals(expressie);

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

    fn solve_expression(&mut self, expression: &str) -> Result<Waarde, String> {
        let mut werk_expressie = geen_spaties_buiten_literals(expression);

        match self.expressie_type(&werk_expressie) {
            VariabeleType::Tekst => self.solve_string_expression(&werk_expressie),
            VariabeleType::Getal => self.solve_number_expression(&werk_expressie),
        }

    }



    fn solve_string_expression(&mut self, expressie: &str) -> Result<Waarde, String> {
        let mut werk_expressie = expressie.to_string();

        self.vervang_variabelen_in_expressie(&mut werk_expressie, VariabeleType::Tekst)?;
        self.vervang_functies_in_expressie(&mut werk_expressie, VariabeleType::Tekst)?;

        let tekst_resultaat = self.samenstellen_tekst_resultaat(&werk_expressie)?;
        Ok(Waarde::Tekst(EcolString::new(tekst_resultaat)))
    }

    pub fn solve_number_expression(&mut self, expressie: &str) -> Result<Waarde, String> {
        let mut werk_expressie = geen_spaties_buiten_literals(expressie);
        self.vervang_variabelen_in_expressie(&mut werk_expressie, VariabeleType::Getal)?;
        self.vervang_functies_in_expressie(&mut werk_expressie, VariabeleType::Getal)?;
        self.bereken_expressie(&mut werk_expressie)?;

        let resultaat = f32::from_str(&werk_expressie);

        match resultaat {
            Ok(result) => {
                return Ok(Waarde::Getal(result));
            }
            Err(e) => {
                return Err(format!("Fout bij parsen van nummer: {}", e));
            }
        }

    }


    fn vervang_variabelen_in_expressie(&mut self, werk_expressie: &mut String, variabele_type: VariabeleType) -> Result<(), String> {
        while let Some(werk_variabele) = parseer_variabele(werk_expressie) {
            let idx = self.pak_of_maak_index(&werk_variabele.variabele_naam, Some(variabele_type))?;
            let mut complete_waarde = &self.data_pool[idx];
            if let Some(var_typ) = complete_waarde.type_van() {
                if var_typ == variabele_type {
                    let result = waarde_naar_expressie(&complete_waarde);

                    werk_expressie.replace_range(werk_variabele.start..werk_variabele.einde, &result);
                } else {
                    return Err(format!("Variabele {:?} is niet van het type {:?}", werk_variabele.variabele_naam, variabele_type));
                }
            } else {
                return Err(format!("Variabele {:?} is niet goed opgeslagen", werk_variabele.variabele_naam));
            }

        }

        Ok(())
    }


    fn vervang_functies_in_expressie(&mut self, werk_expressie: &mut String, variabele_type: VariabeleType) -> Result<(), String> {
        while let Some(werk_functie) = parseer_functie(werk_expressie) {
            let uitkomst = self.execute_function(&werk_functie)?;
            if uitkomst.type_van() != Some(variabele_type) {
                return Err(format!("Functie {:?} past niet in een expressie van het type {:?}", werk_functie.functie, variabele_type));
            }
            let result = waarde_naar_expressie(&uitkomst);

            werk_expressie.replace_range(werk_functie.start..werk_functie.einde, &result);
        }

        Ok(())
    }

    fn bereken_expressie(&mut self, werk_expressie: &mut String) -> Result<(), String> {
        self.bereken_tussen_haakjes(werk_expressie)?;
        self.bereken_operatoren(werk_expressie)?;

        Ok(())
    }

    fn bereken_tussen_haakjes(&mut self, expressie: &mut String) -> Result<(), String> {
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


    fn bereken_operatoren(&mut self, expressie: &mut String) -> Result<(), String> {
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

    fn execute_function(&mut self, werk_functie: &FunctieAanroep) -> Result<Waarde, String> {
        let uitkomst = match werk_functie.functie {
            Functie::LinksString => self.execute_function_links(werk_functie.argumenten.as_str()),
            Functie::RechtsString => self.execute_function_rechts(werk_functie.argumenten.as_str()),
            Functie::MiddenString => self.execute_function_midden(werk_functie.argumenten.as_str()),
            Functie::INT => self.execute_function_int(werk_functie.argumenten.as_str()),
        };

        uitkomst.map_err(|e| format!("Fout bij uitvoeren van functie: {:?}: {}", werk_functie.functie, e))
    }


    fn samenstellen_tekst_resultaat(&self, werk_expressie: &str) -> Result<String, String> {

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

    fn execute_function_int(&mut self, argumenten: &str) -> Result<Waarde, String> {
        let arguments = parseer_argumenten(argumenten, 1)?;
        let getal = self.solve_number_expression(&arguments[0])?;
        let werk_getal = haal_data(&getal).parse::<f32>().map_err(|_| "Ongeldig getal".to_string())?;
        let reply = werk_getal.trunc();

        Ok(Waarde::Getal(reply))
    }

    fn execute_function_links(&mut self, argumenten: &str) -> Result<Waarde, String> {
        let arguments = parseer_argumenten(argumenten, 2)?;
        let tekst = self.solve_string_expression(&arguments[0])?;
        let lengte = self.lees_integer_argument(&arguments[1])?;
        let mut reply = String::new();

        let werk_tekst = haal_data(&tekst);
        reply = werk_tekst.chars().take(lengte).collect::<String>();

        Ok(Waarde::Tekst(EcolString::new(reply)))
    }

    fn execute_function_midden(&mut self, argumenten: &str) -> Result<Waarde, String> {
        let arguments = parseer_argumenten(argumenten, 3)?;
        let tekst = self.solve_string_expression(&arguments[0])?;
        let start = self.lees_integer_argument(&arguments[1])?;
        let lengte = self.lees_integer_argument(&arguments[2])?;
        let mut reply = String::new();
        let werk_tekst = haal_data(&tekst);

        reply = werk_tekst
            .chars()
            .skip(start.saturating_sub(1))
            .take(lengte)
            .collect::<String>();

        Ok(Waarde::Tekst(EcolString::new(reply)))
    }


    fn execute_function_rechts(&mut self, argumenten: &str) -> Result<Waarde, String> {
        let arguments = parseer_argumenten(argumenten, 2)?;
        let tekst = self.solve_string_expression(&arguments[0])?;
        let lengte = self.lees_integer_argument(&arguments[1])?;
        let mut reply = String::new();

        let werk_tekst = haal_data(&tekst);
        let totaal = werk_tekst.chars().count();
        reply = werk_tekst
            .chars()
            .skip(totaal.saturating_sub(lengte))
            .collect::<String>();

        Ok(Waarde::Tekst(EcolString::new(reply)))

    }

    fn lees_getal_variabele_argument(&mut self, argument: &str) -> Result<Waarde, String> {
        if !(self.haal_variabele_type(argument) == Some(VariabeleType::Getal))
        {
            return Err("Argument moet een getal zijn".to_string());
        }

        let idx = self.pak_of_maak_index(argument, Some(VariabeleType::Getal))?;

        Ok(self.data_pool[idx].clone())
    }
    fn lees_integer_argument(&mut self, argument: &str) -> Result<usize, String> {
        if is_geldige_variabele_naam(argument) {

            let complete_waarde = self.lees_getal_variabele_argument(argument)?;
            Ok(parse_i32(&haal_data(&complete_waarde)) as usize)
        } else {
            argument
                .parse::<usize>()
                .map_err(|_| "Ongeldige integerwaarde".to_string())
        }
    }

    fn lees_getal_argument(&mut self, argument: &str) -> Result<f32, String> {
        if is_geldige_variabele_naam(argument) {

            let complete_waarde = self.lees_getal_variabele_argument(argument)?;
            Ok(parse_f32(&haal_data(&complete_waarde)))
        } else {
            argument
                .parse::<f32>()
                .map_err(|_| "Ongeldig getal".to_string())
        }
    }


}





fn verbijzonder_argumenten(werk_expressie: &str) -> String {
    let mut argumenten = "".to_string();

    if let Some(i) = werk_expressie.find('(') {
        let inhoud = &werk_expressie[i + 1..];
        if let Some(inhoud) = inhoud.strip_suffix(')') {
            argumenten.push_str(inhoud);
        }
    }
    argumenten
}


fn parseer_functie(expressie: &str) -> Option<FunctieAanroep> {
    let mut stack: Vec<(Functie, usize, usize)> = Vec::new();
    let mut naam_start: Option<usize> = None;
    let mut tussen_quotes = false;

    for (i, c) in expressie.char_indices() {
        if c == '"' {
            tussen_quotes = !tussen_quotes;
            naam_start = None;
            continue;
        }

        if tussen_quotes {
            continue;
        }

        if c.is_ascii_alphabetic() {
            if naam_start.is_none() {
                naam_start = Some(i);
            }
            continue;
        }

        if c.is_ascii_alphanumeric() || c == '_' {
            continue;
        }

        if c == '(' {

            if let Some(start) = naam_start {
                let naam = &expressie[start..i];
                if let Some(functie) = Functie::from_str(naam) {
                    stack.push((functie, start, i));
                }
            }
            naam_start = None;
            continue;
        }

        if c == ')' {
            if let Some((functie, start, _open_index)) = stack.pop() {
                let volledige_aanroep = &expressie[start..=i];
                let argumenten = verbijzonder_argumenten(volledige_aanroep);
                return Some(FunctieAanroep::new(functie, start, i + 1, argumenten));
            }
            naam_start = None;
            continue;
        }

        naam_start = None;
    }

    None
}


fn parseer_variabele(expressie: &str) -> Option<VariabeleAanroep> {
    let mut naam_start: Option<usize> = None;
    let mut tussen_quotes = false;

    for (i, c) in expressie.char_indices() {
        if c == '"' {
            tussen_quotes = !tussen_quotes;
            naam_start = None;
            continue;
        }

        if tussen_quotes {
            continue;
        }


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
                    return Some(VariabeleAanroep::new(naam.to_string(), start, i));
                }
                naam_start = None;
            }
        }

    }
    if let Some(start) = naam_start {
        let naam = &expressie[start..];
        if is_geldige_variabele_naam(naam) {
            return Some(VariabeleAanroep::new(naam.to_string(), start, expressie.len()));
        }
    }

    None
}

fn parseer_regel(input: &str) -> Result<Line, String> {
    let (regelnummer, rest_na_regelnummer) = extract_regelnummer(input)?;

    let Some((keyword, rest_na_keyword)) = extract_keyword(rest_na_regelnummer) else { return Err("Onbekend of geen keyword.".to_string()) };

    match keyword {
        Sleutelwoord::HELP => {
            is_alleen_keyword(rest_na_regelnummer, regelnummer, keyword)
        },
        Sleutelwoord::NR => {
            is_alleen_keyword(rest_na_regelnummer, regelnummer, keyword)
        },
        Sleutelwoord::TOEKENNEN => {
            let Some((variabele_naam, rest_na_variabele)) = extract_variabele_naam(rest_na_regelnummer) else { return Err("Variabele naam ontbreekt.".to_string()) };
            let Some(rest_na_wordt_teken) = is_geldig_wordt_teken(rest_na_variabele) else { return Err("Ongeldig 'wordt'-teken.".to_string()) };
            let expressie = rest_na_wordt_teken.to_string();
            let variabele_naam = variabele_naam.to_string();
            Ok(Line::new(regelnummer, LineInhoud::Toekennen{ variabele_naam, expressie }))
        },
        Sleutelwoord::TEKST => {
            let Some(rest_na_wordt_teken) = is_geldig_wordt_teken(rest_na_keyword) else { return Err("Ongeldig 'wordt'-teken.".to_string()) };
            let expressie = rest_na_wordt_teken.to_string();
            Ok(Line::new(regelnummer, LineInhoud::Tekst{ expressie }))
        },
        Sleutelwoord::SCHRIJF => {
            let Some((argumenten, rest_na_argumenten)) = extract_argumenten(rest_na_keyword) else { return Err("Ongeldige argumenten bij SCHRIJF.".to_string()) };
            if argumenten.len() != 2 { return Err("SCHRIJF verwacht precies twee argumenten.".to_string()) }
            let breedte = argumenten[0];
            let decimalen = argumenten[1];

            let Some(rest_na_wordt_teken) = is_geldig_wordt_teken(rest_na_argumenten) else { return Err("Ongeldig 'wordt'-teken.".to_string()) };

            let expressie = rest_na_wordt_teken.to_string();
            Ok(Line::new(regelnummer, LineInhoud::Schrijf{ breedte, decimalen, expressie }))
        },
    }

}

fn parseer_argumenten(argumenten: &str, aantal_argumenten: usize) -> Result<Vec<String>, String> {
    let mut result = Vec::new();
    let mut tussen_quotes = false;
    let mut escape = false;
    let mut start = 0;

    for (i, c) in argumenten.char_indices() {
        if escape {
            escape = false;
            continue;
        }

        if tussen_quotes && c == '\\' {
            escape = true;
            continue;
        }

        if c == '"' {
            tussen_quotes = !tussen_quotes;
            continue;
        }

        if !tussen_quotes && c == ',' {
            result.push(argumenten[start..i].trim().to_string());
            start = i + c.len_utf8();
        }
    }

    if escape || tussen_quotes {
        return Err("Ongeldige argumentlijst: quotes zijn niet correct afgesloten".to_string());
    }

    if argumenten.is_empty() {
        if aantal_argumenten == 0 {
            return Ok(Vec::new());
        }
        return Err(format!(
            "Verkeerd aantal argumenten: verwacht {}, kreeg 0",
            aantal_argumenten
        ));
    }

    result.push(argumenten[start..].trim().to_string());

    if result.len() != aantal_argumenten {
        return Err(format!(
            "Verkeerd aantal argumenten: verwacht {}, kreeg {}",
            aantal_argumenten,
            result.len()
        ));
    }

    Ok(result)
}

/// ```rust
/// Parses the given string token into a 32-bit signed integer (`i32`).
///
/// The function attempts to parse the input in the following order:
/// 1. Tries to directly parse the input string as an `i32`.
/// 2. If that fails, it attempts to parse the input string as a `f32`
///    and converts the resulting value to an `i32` using truncation.
/// 3. If both attempts fail, returns `0` as the default fallback value.
///
/// # Arguments
///
/// * `token` - A string slice (`&str`) representing the value to parse.
///
/// # Returns
///
/// * An `i32` value parsed from the input string, or `0` if parsing fails.
///
/// # Examples
///
/// ```
/// let result = parse_i32("42");
/// assert_eq!(result, 42);
///
/// let result = parse_i32("42.7");
/// assert_eq!(result, 42);
///
/// let result = parse_i32("abc");
/// assert_eq!(result, 0);
/// ```
/// fn
fn parse_i32(token: &str) -> i32 {
    if let Ok(value) = token.parse::<i32>() {
        value
    } else if let Ok(value) = token.parse::<f32>() {
        value as i32
    } else {
        0
    }
}

/// ```rust
/// Attempts to parse a string slice (`&str`) into a `f32` (32-bit floating-point number).
///
/// The function follows these steps:
/// 1. It first tries to directly parse the input string into a `f32`.
/// 2. If the above parsing fails, it then tries to parse the string into an `i32`
///    (32-bit integer) and converts the result into a `f32`.
/// 3. If both parsing attempts fail, it defaults to returning `0.0`.
///
/// # Arguments
///
/// * `token` - A string slice that represents the input to be parsed.
///
/// # Returns
///
/// * A `f32` value parsed from the input string if successful.
/// * Returns `0.0` if the string cannot be parsed into either a `f32` or `i32`.
///
/// # Examples
///
/// ```
/// let num = parse_f32("3.14");
/// assert_eq!(num, 3.14);
///
/// let num = parse_f32("42");
/// assert_eq!(num, 42.0);
///
/// let num = parse_f32("invalid");
/// assert_eq!(num, 0.0);
/// ```
/// ```
fn parse_f32(token: &str) -> f32 {
    if let Ok(value) = token.parse::<f32>() {
        value
    } else if let Ok(value) = token.parse::<i32>() {
        value as f32
    } else {
        0.0
    }
}

/// ```rust
/// Parses a given string literal into an `EcolString`.
///
/// # Arguments
///
/// * `token` - A string slice (`&str`) representing the string literal to be parsed.
///
/// # Returns
///
/// A `Result` enum:
/// - `Ok(EcolString)` if the parsing is successful.
/// - `Err(String)` if there is an error during parsing, containing a description of the error.
///
/// # Example
///
/// ```rust
/// let token = "example";
/// match parse_string(token) {
///     Ok(ecol_string) => println!("Parsed successfully: {:?}", ecol_string),
///     Err(err) => eprintln!("Parsing failed: {}", err),
/// }
/// ```
///
/// # Errors
///
/// Returns an `Err` if `EcolString::from_literal` fails to parse the provided `token`.
///
/// # Note
///
/// This function relies on the implementation of `EcolString::from_literal`.
/// ```
fn parse_string(token: &str) -> Result<EcolString, String> {
    EcolString::from_literal(token)

}


/// ```rust
/// Checks whether the given string is a valid variable name.
///
/// This function verifies if the provided `naam` satisfies the following conditions:
/// 1. The name adheres to the syntax rules for valid variable names, determined by the
///    `heeft_geldige_variabele_syntax` function.
/// 2. The name is neither a reserved function name (`Functie::is_functie`) nor a reserved
///    keyword (`Sleutelwoord::is_sleutelwoord`).
///
/// # Parameters
/// - `naam`: A string slice representing the name to be validated.
///
/// # Returns
/// - `true` if the provided name is valid and not reserved.
/// - `false` otherwise.
///
/// # Example
/// ```rust
/// assert!(is_geldige_variabele_naam("my_variable"));
/// assert!(!is_geldige_variabele_naam("fn")); // `fn` is a reserved keyword
/// assert!(!is_geldige_variabele_naam("123abc")); // Invalid syntax
/// ```
/// ```
pub fn is_geldige_variabele_naam(naam: &str) -> bool {
    heeft_geldige_variabele_syntax(naam)


}

fn heeft_geldige_variabele_syntax(naam: &str) -> bool {
    let mut chars = naam.chars();

    let Some(eerste) = chars.next() else {
        return false;
    };

    if !eerste.is_ascii_lowercase() {
        return false;
    }

    let rest: Vec<char> = chars.collect();

    if rest.is_empty() {
        return true;
    }

    for (i, &c) in rest.iter().enumerate() {
        let is_laatste = i == rest.len() - 1;

        if is_laatste {
            if !(c.is_ascii_lowercase() || c.is_ascii_digit()) {
                return false;
            }
        } else if !(c.is_ascii_lowercase() || c == '_' || c.is_ascii_digit()) {
            return false;
        }
    }

    true
}

/// ```rust
/// Returns the first word from the given input string, stopping at the first
/// occurrence of an operator character or an open parenthesis.
///
/// This function iterates over the characters in the input string along with
/// their byte indices. When it encounters a character that is either an operator
/// character (as determined by `Operator::is_operator_char`) or an open parenthesis
/// `(`, it slices the string up to (but not including) that character and returns it.
/// If no such character is found, the entire input string is returned.
///
/// # Arguments
///
/// * `input` - A string slice (`&str`) from which the first word will be extracted.
///
/// # Returns
///
/// * A string slice containing the first word from the input string.
///
/// # Examples
///
/// ```
/// struct Operator;
/// impl Operator {
///     fn is_operator_char(c: char) -> bool {
///         "+-*/".contains(c)
///     }
/// }
///
/// let input = "hello+world";
/// let result = first_word(input);
/// assert_eq!(result, "hello");
///
/// let input = "(test+data)";
/// let result = first_word(input);
/// assert_eq!(result, "");
///
/// let input = "no_operator_here";
/// let result = first_word(input);
/// assert_eq!(result, "no_operator_here");
/// ```
///
/// # Note
///
/// Ensure that the `Operator` struct and its `is_operator_char` method are defined,
/// as they are prerequisites for this function to work as expected.
/// ```
fn first_word(input: &str) -> &str {
    for (i, c) in input.char_indices() {
        if Operator::is_operator_char(c) || c == '(' {
            return &input[..i];
        }
    }
    input
}

/// ```rust
/// Removes spaces from a given input string, except for spaces that are inside string literals.
/// A string literal is defined as text enclosed in double quotes (`"`) in the input.
///
/// - Spaces outside of string literals are entirely removed.
/// - Escape sequences (e.g., `\"`) within string literals are properly handled, ensuring
///   quotes inside strings are not interpreted incorrectly.
///
/// # Parameters
/// - `input`: A string slice (`&str`) representing the input text to process.
///
/// # Returns
/// A new `String` with spaces removed unless they appear inside string literals.
///
/// # Examples
/// ```
/// let input = r#" This is "a test" string with " multiple spaces " outside "#;
/// let result = geen_spaties_buiten_literals(input);
/// assert_eq!(result, r#"Thisis"a test"stringwith" multiple spaces "outside"#);
/// ```
///
/// ```
/// let input = r#"" Keep spaces   inside", but remove   outside "#;
/// let result = geen_spaties_buiten_literals(input);
/// assert_eq!(result, r#"" Keep spaces   inside",butremoveoutside"#);
/// ```
/// fn
fn geen_spaties_buiten_literals(input: &str) -> String {
    let mut result = String::new();
    let mut in_string = false;
    let mut escape = false;

    for c in input.chars() {
        if !in_string {
            if c == '"' {
                in_string = true;
                result.push(c);
                continue;
            }
            if c.is_whitespace() {
                continue;
            }
            result.push(c);
        } else {
            if escape {
                escape = false;
                result.push(c);
                continue;
            }
            if c == '\\' {
                escape = true;
                result.push(c);
                continue;
            }
            if c == '"' {
                in_string = false;
                result.push(c);
                continue;

            }
            result.push(c);
        }
    }
    result
}

fn syntaxis_foutmelding(input: &str) -> String {
    format!("Onjuiste syntax voor sleutelwoord {}.", input)
}
fn is_alleen_keyword(input: &str, regelnummer: u16, keyword: Sleutelwoord) -> Result<Line, String> {
    if input.trim() == keyword.to_string() {
        Ok(Line::new(regelnummer, LineInhoud::from_sleutelwoord(keyword)))
    } else {
        Err(syntaxis_foutmelding(keyword.to_string()))
    }
}
fn extract_regelnummer(input: &str) -> Result<(u16, &str), String> {
    let restregel: &str;
    let regelnummer: u16;

    if let Some(positie) = input.find(|c: char| c.is_ascii_alphabetic()) {
        let (nummer, rest) = input.split_at(positie);
        restregel = rest.trim_start();
        let Some(rnummer) = nummer.trim().parse::<u16>().ok() else { return Ok((0u16, input.trim_start())) };
        regelnummer = rnummer;
        if regelnummer > 999u16 { return Err("Regelnummer mag niet groter zijn dan 999".to_string()) };
    } else {
        regelnummer = 0u16;
        restregel = input.trim_start();
    }

    Ok((regelnummer, restregel))
}
fn extract_keyword(input: &str) -> Option<(Sleutelwoord, &str)> {
    let mut werkstring = input.trim_start();
    if werkstring.chars().next().is_some_and(|c| c.is_ascii_lowercase()) {
        return Some((Sleutelwoord::TOEKENNEN, werkstring));
    }
    let position = werkstring.find(|c: char| !c.is_ascii_uppercase()).unwrap_or(werkstring.len());
    let (keyword_string, rest) = werkstring.split_at(position);

    let Some(resultaat) = Sleutelwoord::from_string(keyword_string.trim_start()) else { return None };

    Some((resultaat, rest.trim_start()))
}
fn extract_variabele_naam(input: &str) -> Option<(&str, &str)> {
    let mut werkstring = input.trim_start();
    let position = werkstring.find(|c: char| !c.is_ascii_lowercase() && !c.is_ascii_digit() && c != '_').unwrap_or(werkstring.len());
    let (variabele_naam, rest) = werkstring.split_at(position);


    if is_geldige_variabele_naam(variabele_naam.trim()) {
        Some((variabele_naam.trim(), rest.trim_start()))
    } else {
        None
    }

}
fn extract_argumenten(input: &str) -> Option<(Vec<usize>, &str)> {
    let werkstring = input.trim_start();

    let (mut inhoud, rest) = werkstring.split_once(')')?;
    inhoud = inhoud.strip_prefix('(')?;

    let reply = inhoud
        .trim_start()
        .split(',')
        .map(|part| part.trim())
        .filter(|part| !part.is_empty())
        .map(|token| token
            .parse::<usize>().ok()).collect::<Option<Vec<usize>>>()?;


    Some((reply, rest.trim_start()))
}

fn is_geldig_wordt_teken(input: &str) -> Option<&str> {
    let rest = input.strip_prefix(WORDT_TEKEN)?;
    Some(rest.trim_start())
}


