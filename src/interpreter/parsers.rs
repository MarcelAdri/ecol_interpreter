use crate::interpreter::helpers::{extract_argumenten, extract_keyword, extract_regelnummer, extract_variabele_naam, is_alleen_keyword, is_geldig_wordt_teken, is_geldige_variabele_naam, verbijzonder_argumenten};
use crate::interpreter::program::{Functie, FunctieAanroep, Line, LineInhoud, Sleutelwoord};
use crate::interpreter::waarden::{EcolString, VariabeleAanroep};

pub(super) fn parseer_argumenten(argumenten: &str, aantal_argumenten: usize) -> Result<Vec<String>, String> {
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
        return Err("Ongeldige argumentenlijst: quotes zijn niet correct afgesloten".to_string());
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
pub(super) fn parse_f32(token: &str) -> f32 {
    if let Ok(value) = token.parse::<f32>() {
        value
    } else if let Ok(value) = token.parse::<i32>() {
        value as f32
    } else {
        0.0
    }
}
pub(super) fn parseer_functie(expressie: &str) -> Option<FunctieAanroep> {
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
pub(super) fn parse_i32(token: &str) -> i32 {
    if let Ok(value) = token.parse::<i32>() {
        value
    } else if let Ok(value) = token.parse::<f32>() {
        value as i32
    } else {
        0
    }
}
pub(super) fn parseer_regel(input: &str) -> Result<Line, String> {
    let (regelnummer, rest_na_regelnummer, is_alleen_regelnummer) = extract_regelnummer(input)?;
    if is_alleen_regelnummer {
        return Ok(Line::new(regelnummer, LineInhoud::Verwijderen {}));
    }

    let Some((keyword, rest_na_keyword)) = extract_keyword(rest_na_regelnummer) else { return Err("Onbekend of geen keyword.".to_string()) };

    match keyword {
        Sleutelwoord::HELP => {
            if regelnummer == 0 {
                is_alleen_keyword(rest_na_regelnummer, regelnummer, keyword)
            } else {
                Err("HELP kan alleen direct vanaf de prompt worden uitgevoerd (regelnummer niet toegestaan).".to_string())
            }
        },
        Sleutelwoord::KLAAR => {
            if regelnummer != 0 {
                is_alleen_keyword(rest_na_regelnummer, regelnummer, keyword)
            } else {
                Err("KLAAR kan alleen in een programma worden uitgevoerd (regelnummer verplicht).".to_string())
            }
        },
        Sleutelwoord::LIJST => {
            if regelnummer == 0 {
                is_alleen_keyword(rest_na_regelnummer, regelnummer, keyword)
            } else {
                Err("LIJST kan alleen direct vanaf de prompt worden uitgevoerd (regelnummer niet toegestaan).".to_string())
            }
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
        Sleutelwoord::START => {
            if regelnummer == 0 {
                is_alleen_keyword(rest_na_regelnummer, regelnummer, keyword)
            } else {
                Err("START kan alleen direct vanaf de prompt worden uitgevoerd (regelnummer niet toegestaan).".to_string())
            }
        }
    }

}
pub(super) fn parse_string(token: &str) -> Result<EcolString, String> {
    EcolString::from_literal(token)

}
pub(super) fn parseer_variabele(expressie: &str) -> Option<VariabeleAanroep> {
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