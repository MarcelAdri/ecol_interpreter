use std::collections::HashMap;
use crate::interpreter::helpers::{
    extract_als
    ,extract_anders
    ,extract_argumenten
    ,extract_dan
    ,extract_keyword
    ,extract_regelnummer
    ,extract_stap_expressie
    ,extract_start_expressie
    ,extract_variabele_naam
    ,geen_spaties
    ,is_alleen_keyword
    ,is_geldig_wordt_teken
    ,is_geldige_variabele_naam};
use crate::interpreter::program::{Line, LineInhoud, Sleutelwoord, SprongDoel};
use crate::interpreter::functions::{FunDef, FunctieNaam};
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
    if functie.verwacht_argumenten() == 0 && !functie.verwacht_string_argument() {
        return Ok(Some(FunctieAanroep::new(functie, start_naam, einde_naam, vec![])));
    }

    // 4. Zoek openingshaakje
    if !expressie[einde_naam..].trim_start().starts_with('(') {
        return Err(format!("'(' verwacht na '{}'", functienaam));
    }
    let abs_open = einde_naam + expressie[einde_naam..].find('(').unwrap();

    // 5. Zoek bijbehorend sluithaakje (haakjes-diepte meetellen)
    let abs_sluit = vind_sluitende_haak(expressie, abs_open)?;

    // 6. Parseer argumenten
    let argumenten_str = &expressie[abs_open + 1..abs_sluit];
    let argumenten = splits_argumenten(argumenten_str);

    if argumenten.len() != functie.verwacht_argumenten() && !functie.verwacht_string_argument() {
        return Err(format!(
            "'{}' verwacht {} argumenten, maar {} zijn gegeven",
            functienaam,
            functie.verwacht_argumenten(),
            argumenten.len()
        ));
    } else if functie.verwacht_string_argument()  && argumenten.len() != 1 {
        return Err(format!(
            "'{}' verwacht 1 string argument, maar {} zijn gegeven",
            functienaam,
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
    if rest_na_regelnummer.trim_start().starts_with(':') {
        return Ok(Line::new(regelnummer, LineInhoud::LegeRegel {}));
    }

    let Some((keyword, rest_na_keyword)) = extract_keyword(rest_na_regelnummer) else { return Err("Onbekend of geen keyword.".to_string()) };

    match keyword {
        Sleutelwoord::ALS => {
            if regelnummer != 0 {
                let Some((vergelijking_str, rest_na_dan)) = extract_als(rest_na_keyword) else { return Err("Geen DAN gevonden na ALS".to_string()) };
                let vergelijking = vergelijking_str.to_string();
                let (dan, rest_na_anders) = extract_dan(rest_na_dan)?;
                let anders: Option<SprongDoel>;
                if rest_na_anders == "" {
                    anders = None;
                } else {
                    let (anders_getal, _rest_na_dan) = extract_anders(rest_na_anders)?;
                    anders = Some(anders_getal);
                }

                Ok(Line::new(regelnummer, LineInhoud::Als { vergelijking, dan, anders }))
            } else {
                Err("KLAAR kan alleen in een programma worden uitgevoerd (regelnummer verplicht).".to_string())
            }

        },
        Sleutelwoord::FUNstart => {
            if regelnummer == 0 {
                return Err("FUN definitie kan alleen in een programma worden uitgevoerd (regelnummer verplicht).".to_string())
            }
            let Some((variabele_naam, rest_na_variabele)) = extract_variabele_naam(rest_na_keyword) else { return Err("Variabele naam ontbreekt.".to_string()) };
            let parameters = rest_na_variabele.trim().trim_start_matches("(").trim_end_matches(")");

            Ok(Line::new(regelnummer, LineInhoud::FunStart { variabele_naam: variabele_naam.to_string() , argumenten: parameters.to_string() }))
        },
        Sleutelwoord::FUNeind => {
            if regelnummer == 0 {
                return Err("FUN toekenning kan alleen in een programma worden uitgevoerd (regelnummer verplicht).".to_string())
            }
            let Some(rest_na_wordt_teken) = is_geldig_wordt_teken(rest_na_keyword) else { return Err("Ongeldig 'wordt'-teken.".to_string()) };
            let expressie = geen_spaties(rest_na_wordt_teken);

            Ok(Line::new(regelnummer, LineInhoud::FunEind { expressie }))
        }
        Sleutelwoord::HELP => {
            if regelnummer == 0 {
                is_alleen_keyword(rest_na_regelnummer, regelnummer, keyword)
            } else {
                Err("HELP kan alleen direct vanaf de prompt worden uitgevoerd (regelnummer niet toegestaan).".to_string())
            }
        },
        Sleutelwoord::HERHAAL => {
            if regelnummer != 0 {
                is_alleen_keyword(rest_na_regelnummer, regelnummer, keyword)
            } else {
                Err("HERHAAL kan alleen in een programma worden uitgevoerd (regelnummer verplicht).".to_string())
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
        Sleutelwoord::MET => {
            if regelnummer == 0 {
                return Err("MET kan alleen in een programma worden uitgevoerd (regelnummer verplicht).".to_string())
            }
            let (stap_expressie, rest_na_stap) = extract_stap_expressie(rest_na_keyword)?;
            let (variabele_naam, start_expressie, stop_expressie) = extract_start_expressie(&rest_na_stap)?;

            Ok(Line::new(regelnummer, LineInhoud::Met{ variabele_naam, stap_expressie, start_expressie, stop_expressie }))
        },
        Sleutelwoord::NAAR => {
            Ok(Line::new(regelnummer, LineInhoud::Naar{ sprong_doel: SprongDoel::vul(rest_na_keyword)? }))
        },
        Sleutelwoord::NP => {
            is_alleen_keyword(rest_na_regelnummer, regelnummer, keyword)
        },
        Sleutelwoord::NR => {
            let (mut argumenten, rest_na_argumenten) = extract_argumenten(rest_na_keyword).unwrap_or( (Vec::new(), rest_na_keyword));
            if argumenten.len() != 1 && argumenten.len() != 0 { return Err(format!("NR verwacht geen of één argument. {} argumenten aangetroffen", argumenten.len())) }
            if argumenten.len() == 0 {
                argumenten.push("1".to_string());
            }
            let expressie = rest_na_argumenten.to_string();
            if !expressie.is_empty() {
                return Err("NR verwacht geen argumenten na de aantal-aanduiding.".to_string());
            }
            Ok(Line::new(regelnummer, LineInhoud::NR{ aantal: argumenten[0].clone()}))
        },
        Sleutelwoord::RIJ => {
            let (argumenten, rest_na_argumenten) = extract_argumenten(rest_na_keyword).unwrap_or( (Vec::new(), rest_na_keyword));
            if argumenten.len() != 2 { return Err(format!("RIJ verwacht twee argumenten. {} argumenten aangetroffen", argumenten.len())) }

            let Some((naam, rest_na_variabele)) = extract_variabele_naam(rest_na_argumenten) else { return Err("Variabele naam ontbreekt.".to_string()) };
            let expressie = rest_na_variabele.to_string();
            if !expressie.is_empty() {
                return Err("RIJ verwacht geen argumenten na de naam van de variabele.".to_string());
            }

            Ok(Line::new(regelnummer, LineInhoud::Rij{ start: argumenten[0].clone(), eind: argumenten[1].clone(), variabele_naam: naam.to_string() }))
        },
        Sleutelwoord::RIJSYM => {
            let (argumenten, rest_na_argumenten) = extract_argumenten(rest_na_keyword).unwrap_or( (Vec::new(), rest_na_keyword));
            if argumenten.len() != 2 { return Err(format!("RIJSYM verwacht twee argumenten. {} argumenten aangetroffen", argumenten.len())) }

            let Some((naam, rest_na_variabele)) = extract_variabele_naam(rest_na_argumenten) else { return Err("Variabele naam ontbreekt.".to_string()) };
            let expressie = rest_na_variabele.to_string();
            if !expressie.is_empty() {
                return Err("RIJSYM verwacht geen argumenten na de naam van de variabele.".to_string());
            }

            Ok(Line::new(regelnummer, LineInhoud::Rijsym{ start: argumenten[0].clone(), eind: argumenten[1].clone(), variabele_naam: naam.to_string() }))
        }
        Sleutelwoord::TOEKENNEN => {
            let Some((variabele_naam, rest_na_variabele)) = extract_variabele_naam(rest_na_regelnummer) else { return Err("Variabele naam ontbreekt.".to_string()) };
            let (argumenten, rest_na_argumenten) = extract_argumenten(rest_na_variabele).unwrap_or( (Vec::new(), rest_na_variabele));
            let argument: String;
            if argumenten.len() == 0 {
                argument = "".to_string();
            } else if argumenten.len() == 1 {
                argument = argumenten[0].clone();
            } else {
                return Err("Een RIJ variabele verwacht slechts één argument.".to_string());
            }
            let Some(rest_na_wordt_teken) = is_geldig_wordt_teken(rest_na_argumenten) else { return Err("Ongeldig 'wordt'-teken.".to_string()) };
            let expressie = rest_na_wordt_teken.to_string();
            let variabele_naam = variabele_naam.to_string();
            Ok(Line::new(regelnummer, LineInhoud::Toekennen{ variabele_naam, argument, expressie }))
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
                breedte = argumenten[0].trim().parse::<usize>().map_err(|_| "SCHRIJF verwacht een getal als breedte.".to_string())? ;
                decimalen = argumenten[1].trim().parse::<usize>().map_err(|_| "SCHRIJF verwacht een getal als aantal decimalen.".to_string())? ;
                if breedte == 0 {
                    return Err("SCHRIJF verwacht een breedte groter dan 0 als eerste argument.".to_string());
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
            let (mut argumenten, rest_na_argumenten) = extract_argumenten(rest_na_keyword).unwrap_or( (Vec::new(), rest_na_keyword));
            if argumenten.len() != 1 && argumenten.len() != 0 { return Err(format!("SPATIE verwacht geen of één argument. {} argumenten aangetroffen", argumenten.len())) }
            if argumenten.len() == 0 {
                argumenten.push("1".to_string());
            }
            let expressie = rest_na_argumenten.to_string();
            if !expressie.is_empty() {
                return Err("SPATIE verwacht geen argumenten na de aantal-aanduiding.".to_string());
            }
            Ok(Line::new(regelnummer, LineInhoud::NR{ aantal: argumenten[0].clone() }))
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

/// Na de variabelenaam: sla spaties over, en als er een `(...)` volgt neem die mee als index.
/// Geeft `(Some(index_expressie), einde_na_haakje)` of `(None, naam_einde)` terug.
fn haal_index_expressie(expressie: &str, naam_einde: usize) -> (Option<String>, usize) {
    let rest = &expressie[naam_einde..];
    if !rest.trim_start().starts_with('(') {
        return (None, naam_einde);
    }
    let abs_open = naam_einde + rest.find('(').unwrap();
    match vind_sluitende_haak(expressie, abs_open) {
        Ok(abs_sluit) => {
            let index_str = expressie[abs_open + 1..abs_sluit].trim().to_string();
            (Some(index_str), abs_sluit + 1)
        }
        Err(_) => (None, naam_einde),
    }
}
pub(super) fn parseer_eigen_functie(functie_register: &HashMap<String, FunDef>, expressie: &str) -> Option<(String, usize, usize, Vec<String>)> {
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