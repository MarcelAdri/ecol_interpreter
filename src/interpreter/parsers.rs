use crate::interpreter::helpers::{extract_argumenten, extract_keyword, extract_regelnummer, extract_variabele_naam, is_alleen_keyword, is_geldig_wordt_teken, is_geldige_variabele_naam, verbijzonder_argumenten};
use crate::interpreter::program::{Line, LineInhoud, Sleutelwoord};
use crate::interpreter::functions::{Functie, FunctieNaam};
use crate::interpreter::waarden::{VariabeleAanroep};

pub(super) struct FunctieAanroep {
    functie: FunctieNaam,
    start: usize,
    einde: usize,
    argumenten: Vec<String>,
}
impl FunctieAanroep {
    pub(super) fn new(functie: FunctieNaam, start: usize, einde: usize, argumenten: Vec<String>) -> Self {
        FunctieAanroep {
            functie,
            start,
            einde,
            argumenten
        }
    }
    pub(super) fn functie(&self) -> &FunctieNaam {
        &self.functie
    }
    pub(super) fn start(&self) -> usize {
        self.start
    }
    pub(super) fn einde(&self) -> usize {
        self.einde
    }
    pub(super) fn argumenten(&self) -> &Vec<String> {
        &self.argumenten
    }
}

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

pub(super) fn parseer_functie(expressie: &str) -> Result<Option<FunctieAanroep>, String> {
    // 1. Zoek begin van functienaam (eerste hoofdletter)
    let start_naam = match expressie.find(|c: char| c.is_ascii_uppercase()) {
        Some(pos) => pos,
        None => return Ok(None),
    };

    // 2. Zoek einde van functienaam
    let einde_naam = expressie[start_naam..]
        .find(|c: char| !c.is_ascii_uppercase())
        .map(|p| start_naam + p)
        .unwrap_or(expressie.len());

    let functienaam = &expressie[start_naam..einde_naam];
    let Some(functie) = FunctieNaam::from_str(functienaam) else {
        return Err(format!("Ongeldige functienaam: '{}'", functienaam));
    };

    // 3. Geen argumenten: direct klaar
    if functie.verwacht_argumenten() == 0 {
        return Ok(Some(FunctieAanroep::new(functie, start_naam, einde_naam, vec![])));
    }

    // 4. Zoek openingshaakje
    if !expressie[einde_naam..].trim_start().starts_with('(') {
        return Err(format!("'(' verwacht na '{}'", functienaam));
    }
    let abs_open = einde_naam + expressie[einde_naam..].find('(').unwrap();

    // 5. Zoek bijbehorend sluithaakje (haakjesdiepte meettellen)
    let abs_sluit = vind_sluitende_haak(expressie, abs_open)?;

    // 6. Parseer argumenten
    let argumenten_str = &expressie[abs_open + 1..abs_sluit];
    let argumenten = splits_argumenten(argumenten_str);

    if argumenten.len() != functie.verwacht_argumenten() {
        return Err(format!(
            "'{}' verwacht {} argumenten, maar {} zijn gegeven",
            functienaam,
            functie.verwacht_argumenten(),
            argumenten.len()
        ));
    }

    Ok(Some(FunctieAanroep::new(functie, start_naam, abs_sluit + 1, argumenten)))
}

/// Zoekt de `)` die hoort bij de `(` op `open`-positie in `s`.
fn vind_sluitende_haak(s: &str, open: usize) -> Result<usize, String> {
    let mut diepte = 0usize;

    for (i, c) in s[open..].char_indices() {
        match c {
            '(' => diepte += 1,
            ')' => {
                diepte -= 1;
                if diepte == 0 {
                    return Ok(open + i);
                }
            }
            _ => {}
        }
    }
    Err("Haakjespaar is niet gesloten".to_string())
}

/// Splitst argumentenstring op komma's, waarbij haakjesdiepte wordt gerespecteerd.
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
        Sleutelwoord::NP => {
            is_alleen_keyword(rest_na_regelnummer, regelnummer, keyword)
        },
        Sleutelwoord::NR => {
            let (argumenten, rest_na_argumenten) = extract_argumenten(rest_na_keyword).unwrap_or( (Vec::new(), rest_na_keyword));
            if argumenten.len() != 1 && argumenten.len() != 0 { return Err(format!("NR verwacht geen of één argument. {} argumenten aangetroffen", argumenten.len())) }
            let aantal: usize;
            if argumenten.len() == 0 {
                aantal = 1;
            } else {
                aantal = argumenten[0];
            }
            let expressie = rest_na_argumenten.to_string();
            if !expressie.is_empty() {
                return Err("NR verwacht geen argumenten na de aantal-aanduiding.".to_string());
            }
            Ok(Line::new(regelnummer, LineInhoud::NR{ aantal }))
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
            let (argumenten, rest_na_argumenten) = extract_argumenten(rest_na_keyword).unwrap_or( (Vec::new(), rest_na_keyword));
            if argumenten.len() != 2 && argumenten.len() != 0 { return Err(format!("SCHRIJF verwacht geen of twee argumenten. {} argumenten aangetroffen", argumenten.len())) }
            let breedte: usize;
            let decimalen: usize;
            if argumenten.len() == 0 {
                breedte = 0;
                decimalen = 0;
            } else {
                if argumenten[0] == 0 {
                    return Err("SCHRIJF verwacht een breedte groter dan 0 als eerste argument.".to_string());
                } else {
                    breedte = argumenten[0];
                    decimalen = argumenten[1];
                }
            }

            let Some(rest_na_wordt_teken) = is_geldig_wordt_teken(rest_na_argumenten) else { return Err("Ongeldig 'wordt'-teken.".to_string()) };

            let expressie = rest_na_wordt_teken.to_string();
            Ok(Line::new(regelnummer, LineInhoud::Schrijf{ breedte, decimalen, expressie }))
        },
        Sleutelwoord::SCHRIJFSYM => {
            let Some(rest_na_wordt_teken) = is_geldig_wordt_teken(rest_na_keyword) else { return Err("Ongeldig 'wordt'-teken.".to_string()) };
            let expressie = rest_na_wordt_teken.to_string();
            Ok(Line::new(regelnummer, LineInhoud::Schrijfsym{ expressie }))
        },
        Sleutelwoord::SCHRIJM => {
            let Some(rest_na_wordt_teken) = is_geldig_wordt_teken(rest_na_keyword) else { return Err("Ongeldig 'wordt'-teken.".to_string()) };
            let expressie = rest_na_wordt_teken.to_string();
            Ok(Line::new(regelnummer, LineInhoud::Schrijm{ expressie }))
        },
        Sleutelwoord::SPATIE => {
            let (argumenten, rest_na_argumenten) = extract_argumenten(rest_na_keyword).unwrap_or( (Vec::new(), rest_na_keyword));
            if argumenten.len() != 1 && argumenten.len() != 0 { return Err(format!("SPATIE verwacht geen of één argument. {} argumenten aangetroffen", argumenten.len())) }
            let aantal: usize;
            if argumenten.len() == 0 {
                aantal = 1;
            } else {
                if argumenten[0] == 0 {
                    return Err("SPATIE verwacht een aantal groter dan 0 als eerste argument.".to_string());
                } else {
                    aantal = argumenten[0];
                }
            }
            if aantal > 80 {
                return Err(format!("SPATIE verwacht een aantal kleiner dan 80 (maximale regelgrootte). Aantal: {}", aantal));
            }
            let expressie = rest_na_argumenten.to_string();
            if !expressie.is_empty() {
                return Err("SPATIE verwacht geen argumenten na de aantal-aanduiding.".to_string());
            }
            Ok(Line::new(regelnummer, LineInhoud::Spatie{ aantal }))
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
pub(super) fn parse_string(token: &str) -> Result<String, String> {
    Ok(token.to_string())

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