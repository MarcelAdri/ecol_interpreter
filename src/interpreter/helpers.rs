use std::convert::TryFrom;
use crate::interpreter::program::{Line, LineInhoud, Operator, Sleutelwoord, SprongDoel, WORDT_TEKEN};
use crate::interpreter::vergelijkingen::Vergelijking;

pub(super) fn extract_argumenten(input: &str) -> Option<(Vec<String>, &str)> {
    let werkstring = input.trim_start();

    let (mut inhoud, rest) = werkstring.split_once(')')?;
    inhoud = inhoud.strip_prefix('(')?;

    let reply = inhoud.split(',').filter_map(|s| { let t = s.trim().to_string(); if t.is_empty() { None } else { Some(t) } }).collect();
    
    Some((reply, rest.trim_start()))
}
pub(super) fn extract_keyword(input: &str) -> Option<(Sleutelwoord, &str)> {
    let werkstring = input.trim_start();
    let resultaat: Sleutelwoord;

    if werkstring.chars().next().is_some_and(|c| c.is_ascii_lowercase()) {
        return Some((Sleutelwoord::TOEKENNEN, werkstring));
    }
    let position = werkstring.find(|c: char| !c.is_ascii_uppercase()).unwrap_or(werkstring.len());
    let (keyword_string, rest) = werkstring.split_at(position);

    if keyword_string.trim_start().starts_with("FUN") {
        if rest.trim_start().chars().next().is_some_and(|c| c.is_ascii_lowercase()) {
            resultaat = Sleutelwoord::FUNstart;
        } else {
            resultaat = Sleutelwoord::FUNeind;
        }
    } else {
        let Some(res) = Sleutelwoord::from_string(keyword_string.trim_start()) else { return None };
        resultaat = res;
    }


    Some((resultaat, rest.trim_start()))
}
pub(super) fn extract_regelnummer(input: &str) -> Result<(u16, &str, bool), String> {
    let restregel: &str;
    let regelnummer: u16;
    let nummer: &str;
    let is_alleen_regelnummer: bool = input.trim().chars().all(|c| c.is_ascii_digit());

    if is_alleen_regelnummer {
        nummer = input.trim();
        restregel = "";
    } else if let Some(positie) = input.find(|c: char| c.is_ascii_alphabetic() || c == ':') {
        let (gevonden_nummer, rest) = input.split_at(positie);
        nummer = gevonden_nummer;
        restregel = rest.trim_start();
    } else {
        nummer = "0";
        restregel = input.trim_start();
    }

    let Some(resultaat_parsing) = nummer.trim().parse::<u16>().ok() else { return Ok((0u16, input.trim_start(), is_alleen_regelnummer)) };
    regelnummer = resultaat_parsing;
    if regelnummer > 999u16 { return Err("Regelnummer mag niet groter zijn dan 999".to_string()) };


    Ok((regelnummer, restregel, is_alleen_regelnummer))
}
fn vind_top_level_komma(s: &str) -> Option<usize> {
    let mut diepte = 0usize;
    for (i, c) in s.char_indices() {
        match c {
            '(' => diepte += 1,
            ')' => diepte -= 1,
            ',' if diepte == 0 => return Some(i),
            _ => {}
        }
    }
    None
}
pub(super) fn extract_stap_expressie(input: &str) -> Result<(String, String), String> {
    let werkstring = geen_spaties(input);

    match vind_top_level_komma(&werkstring) {
        Some(p) => {
            let (stap_tekst, r) = werkstring.split_at(p);
            Ok((stap_tekst.to_string(), r[1..].to_string()))
        },
        None => Err("Ongeldige syntax voor MET bij de STAP-expressie.".to_string()),
    }
}
pub(super) fn extract_start_expressie(input: &str) -> Result<(String, String, String), String> {
    let werkstring = geen_spaties(input);

    match vind_top_level_komma(&werkstring) {
        Some(p) => {
            let (toekenning_string, r) = werkstring.split_at(p);
            let Some((naam, rest_na_variabele)) = extract_variabele_naam(toekenning_string) else { return Err("Ongeldige variabele-naam in MET.".to_string()) };
            let Some(rest_na_wordt_teken) = is_geldig_wordt_teken(rest_na_variabele) else { return Err("Ongeldig 'wordt'-teken in MET.".to_string()) };
            Ok((naam.to_string(), rest_na_wordt_teken.to_string(), r[1..].to_string()))
        },
        None => Err("Ongeldige syntax voor MET bij de START-expressie.".to_string()),
    }
}
pub(super) fn extract_variabele_naam(input: &str) -> Option<(&str, &str)> {
    let werkstring = input.trim_start();
    let position = werkstring.find(|c: char| !c.is_ascii_lowercase() && !c.is_ascii_digit() && c != '_').unwrap_or(werkstring.len());
    let (variabele_naam, rest) = werkstring.split_at(position);

    if is_geldige_variabele_naam(variabele_naam.trim()) {

        Some((variabele_naam.trim(), rest.trim_start()))
    } else {
        None
    }

}
pub(super) fn extract_als(input: &str) -> Option<(&str, &str)> {
    let werkstring = input.trim_start();
    let pos = werkstring.find("DAN");
    match pos {
        Some(positie) => {
            let vergelijking = &werkstring[..positie];
            let rest = &werkstring[positie + 3..];
            Some((vergelijking.trim(), rest.trim_start()))
        }
        None => None,
    }

}
pub(super) fn extract_dan(input: &str) -> Result<(SprongDoel, &str), String> {
    let werkstring = input.trim_start();
    let pos = werkstring.find("ANDERS");
    let dan_reply: SprongDoel;

    match pos {
        Some(positie) => {
            let dan = &werkstring[..positie];
            let rest = &werkstring[positie + 6..];

            dan_reply = SprongDoel::vul(&geen_spaties(dan))?;

            Ok((dan_reply, rest.trim_start()))
        }
        None => {
            let dan = &geen_spaties(werkstring);
            dan_reply = SprongDoel::vul(dan)?;

            Ok((dan_reply, ""))
        }
    }
}
pub(super) fn extract_anders(input: &str) -> Result<(SprongDoel, &str), String> {
    let werkstring = geen_spaties(input);

    let anders = SprongDoel::vul(&werkstring)?;
    
    Ok((anders, ""))
}
pub(super) fn first_word(input: &str) -> &str {
    for (i, c) in input.char_indices() {
        if Operator::is_operator_char(c) || c == '(' {
            return &input[..i];
        }
    }
    input
}
pub(super) fn format_getal(getal: f32, breedte: usize, decimalen: usize) -> Result<String, String> {
    let mut b = breedte;
    let mut d = decimalen;
    let x = getal;

    if breedte == 0 {
        b = 6;
        if x != x.trunc() {
            d = 4;
        } else {
            d = 0;
        }
    }

    let ruimte_voor_teken = if x < 0f32 { 1usize } else { 0usize };
    let b_totaal = if d == 0 { b + ruimte_voor_teken } else { b + d + ruimte_voor_teken + 1 };

    let reply = format!("{:breedte$.decimalen$}", x, breedte = b_totaal, decimalen = d);

    if reply.len() > b_totaal {
        return Err(format!("De opgegeven waarde {} past niet in de opgegeven breedte {}.", x, breedte))
    }

    Ok(reply)

}

pub(super) fn geen_spaties(input: &str) -> String {
    input.chars().filter(|c| !c.is_whitespace()).collect::<String>()
}
pub(super) fn get_sym_value(getal: &f32) -> Result<u8, String> {
    if getal.is_nan() || getal < &0.0 || getal > &99.0 {
        return Err(format!("Waarde {} is ongeldig (xxxSYM verwacht 0–99).", getal));
    }
    Ok(*getal as u8)
}
pub(super) fn grens_bewaking (getal: &f32, alleen_positieve_getallen: bool, alleen_hele_getallen: bool) -> Result<f32, String> {
    if getal.fract() != 0.0 && alleen_hele_getallen {
        return Err("LN functie kan alleen hele getallen accepteren.".to_string());
    }
    if *getal <= 0f32 && alleen_positieve_getallen {
        return Err(format!("Getal moet positief zijn, maar is {}", getal));
    }

    Ok(*getal)
}
pub(super) fn heeft_geldige_variabele_syntax(naam: &str) -> bool {
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
pub(super) fn is_alleen_keyword(input: &str, regelnummer: u16, keyword: Sleutelwoord) -> Result<Line, String> {
    if input.trim() == keyword.to_string() {
        Ok(Line::new(regelnummer, LineInhoud::from_sleutelwoord(keyword)?))
    } else {
        Err(syntaxis_foutmelding(keyword.to_string()))
    }
}
pub(super) fn is_geldig_wordt_teken(input: &str) -> Option<&str> {
    let rest = input.strip_prefix(WORDT_TEKEN)?;
    Some(rest.trim_start())
}
pub(super) fn is_geldige_variabele_naam(naam: &str) -> bool {
    heeft_geldige_variabele_syntax(naam)


}
pub(super) fn literal_to_string (literal: &str) -> Result<String, String> {
    let werk_string = literal.trim();

    if !werk_string.starts_with('"') {
        return Err("Tekstblok moet beginnen met een aanhalingsteken.".to_string());
    }

    if !werk_string.ends_with('"') {
        return Err("Tekstblok moet eindigen met een aanhalingsteken.".to_string());
    }

    let inhoud = &werk_string[1..werk_string.len() - 1];

    if inhoud.contains('"') {
        return Err("Slechts één aaneengesloten tekstblok toegestaan.".to_string());
    }

    Ok(inhoud.to_string())

}
pub(super) fn result_to_string (result: Result<String, String>) -> String {
    match result {
        Ok(value) => format!("{}", value),
        Err(err) => format!("{}", err),
    }
}
pub(super) fn syntaxis_foutmelding(input: &str) -> String {
    format!("Onjuiste syntax voor sleutelwoord {}.", input)
}
pub(super) fn verbijzonder_argumenten(werk_expressie: &str) -> String {
    let mut argumenten = "".to_string();

    if let Some(i) = werk_expressie.find('(') {
        let inhoud = &werk_expressie[i + 1..];
        if let Some(inhoud) = inhoud.strip_suffix(')') {
            argumenten.push_str(inhoud);
        }
    }
    argumenten
}
