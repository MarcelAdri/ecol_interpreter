use std::collections::HashMap;
use crate::interpreter::errors::{EcolFout, EcolFoutVariant};
use crate::interpreter::helpers::{extract_als, extract_anders, extract_argumenten, extract_dan, extract_keyword, extract_opmerking, extract_regelnummer, extract_stap_expressie, extract_start_expressie, extract_variabele_naam, geen_spaties, is_alleen_keyword, is_geldig_wordt_teken, is_geldige_variabele_naam, vind_opmerking};
use crate::interpreter::program::{FunDef, Line, LineInhoud, Operator, Sleutelwoord, SprongDoel};
use crate::interpreter::functions::{FunctieNaam};
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
pub(super) fn parseer_functie(expressie: &str) -> Result<Option<FunctieAanroep>, EcolFout> {
    // 1+2. Zoek de eerste hoofdletterreeks die geen operator is
    let mut zoek_vanaf = 0;
    let (start_naam, einde_naam) = loop {
        let rel = match expressie[zoek_vanaf..].find(|c: char| c.is_ascii_uppercase()) {
            Some(pos) => pos,
            None => return Ok(None),
        };
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

/// Splitst argumenten-string op komma's, waarbij haakjes-diepte wordt gerespecteerd.
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
pub(super) fn parseer_regel(input: &str) -> Result<Line, EcolFout> {
    let (regelnummer, rnn, is_alleen_regelnummer) = extract_regelnummer(input)?;
    if is_alleen_regelnummer {
        return Ok(Line::new(regelnummer, LineInhoud::Verwijderen {}));
    }
    let rest_na_regelnummer: &str = rnn;
    if rest_na_regelnummer.trim_start().starts_with(':') {
        let rest_na_dubbele_punt = rest_na_regelnummer.trim_start().trim_start_matches(':');

        return Ok(Line::new(regelnummer, LineInhoud::LegeRegel { opm: vind_opmerking(rest_na_dubbele_punt)? }));
    }

    let Some((keyword, rest_na_keyword)) = extract_keyword(rest_na_regelnummer) else {
        return Err(EcolFout::melding(EcolFoutVariant::OnbekendSleutelwoord))
    };

    match keyword {
        Sleutelwoord::ALS => {
            if regelnummer != 0 {
                let Some((vergelijking_str, rest_na_als)) = extract_als(rest_na_keyword) else {
                    return Err(EcolFout::melding(EcolFoutVariant::GeenDan))
                };
                let vergelijking = vergelijking_str.to_string();
                let (dan, rest_na_dan) = extract_dan(rest_na_als)?;
                let anders: Option<SprongDoel>;
                let rest_tekst: &str;
                if geen_spaties(rest_na_dan).is_empty() || rest_na_dan.trim_start().starts_with(';') {
                    rest_tekst = rest_na_dan;
                    anders = None;
                } else {
                    let (anders_getal, rest) = extract_anders(rest_na_dan)?;
                    rest_tekst = rest;
                    anders = Some(anders_getal);
                }

                Ok(Line::new(regelnummer, LineInhoud::Als { vergelijking, dan, anders, opm: vind_opmerking(rest_tekst)? }))
            } else {
                Err(EcolFout::melding(EcolFoutVariant::AlleenInProgramma("ALS".to_string())))
            }

        },
        Sleutelwoord::END => {
            if regelnummer == 0 {
                return Err(EcolFout::melding(EcolFoutVariant::AlleenInProgramma("END".to_string())))
            }

            Ok(Line::new(regelnummer, LineInhoud::End { opm: vind_opmerking(rest_na_keyword)?}))
        },
        Sleutelwoord::FUNstart => {
            if regelnummer == 0 {
                return Err(EcolFout::melding(EcolFoutVariant::AlleenInProgramma("FUN definitie".to_string())))
            }
            let Some((variabele_naam, rest_na_variabele)) = extract_variabele_naam(rest_na_keyword) else {
                return Err(EcolFout::melding(EcolFoutVariant::GeenVariabeleNaam))
            };
            let parameters = rest_na_variabele.trim().trim_start_matches('(').trim_end_matches(')');
            let rest_na_parameters = rest_na_variabele
                .find(')')
                .map_or("", |i| &rest_na_variabele[i + 1..]);


            Ok(Line::new(regelnummer, LineInhoud::FunStart { variabele_naam: variabele_naam.to_string() , argumenten: parameters.to_string(), opm: vind_opmerking(rest_na_parameters)? }))
        },
        Sleutelwoord::FUNeind => {
            if regelnummer == 0 {
                return Err(EcolFout::melding(EcolFoutVariant::AlleenInProgramma("FUN :=".to_string())))
            }
            let Some(rest_na_wordt_teken) = is_geldig_wordt_teken(rest_na_keyword) else {
                return Err(EcolFout::melding(EcolFoutVariant::OngeldigWordtTeken))
            };
            let (expressie, rest_na_expressie) = extract_opmerking(rest_na_wordt_teken);
            
            Ok(Line::new(regelnummer, LineInhoud::FunEind { expressie: geen_spaties(&expressie), opm: vind_opmerking(&rest_na_expressie)? }))
        }
        Sleutelwoord::GASUB => {
            if regelnummer == 0 {
                return Err(EcolFout::melding(EcolFoutVariant::AlleenInProgramma("Een subroutine aanroepen".to_string())))
            }
            let (subroutine, rest_na_subroutine) = extract_opmerking(rest_na_keyword);
            
            Ok(Line::new(regelnummer, LineInhoud::GaSub { sub_naam: geen_spaties(&subroutine), opm: vind_opmerking(&rest_na_subroutine)? }))
        }
        Sleutelwoord::HELP => {
            if regelnummer == 0 {
                is_alleen_keyword(rest_na_keyword, regelnummer, keyword)
            } else {
                Err(EcolFout::melding(EcolFoutVariant::NietInProgramma("HELP".to_string())))
            }
        },
        Sleutelwoord::HERHAAL => {
            if regelnummer != 0 {
                is_alleen_keyword(rest_na_keyword, regelnummer, keyword)
            } else {
                Err(EcolFout::melding(EcolFoutVariant::AlleenInProgramma("HERHAAL".to_string())))
            }
        },
        Sleutelwoord::KLAAR => {
            if regelnummer != 0 {
                is_alleen_keyword(rest_na_keyword, regelnummer, keyword)
            } else {
                Err(EcolFout::melding(EcolFoutVariant::AlleenInProgramma("KLAAR".to_string())))
            }
        },
        Sleutelwoord::LIJST => {
            if regelnummer == 0 {
                is_alleen_keyword(rest_na_keyword, regelnummer, keyword)
            } else {
                Err(EcolFout::melding(EcolFoutVariant::NietInProgramma("LIJST".to_string())))
            }
        },
        Sleutelwoord::MET => {
            if regelnummer == 0 {
                return Err(EcolFout::melding(EcolFoutVariant::AlleenInProgramma("MET".to_string())))
            }
            let (stap_expressie, rest_na_stap) = extract_stap_expressie(rest_na_keyword)?;
            let (variabele_naam, start_expressie, rest_na_start) = extract_start_expressie(&rest_na_stap)?;
            let (stop_expressie, opm) = extract_opmerking(&rest_na_start);

            Ok(Line::new(regelnummer, LineInhoud::Met{ variabele_naam, stap_expressie, start_expressie, stop_expressie, opm: vind_opmerking(&opm)? }))
        },
        Sleutelwoord::NAAR => {
            let (sprong_doel, opmerking) = extract_opmerking(rest_na_keyword);
            Ok(Line::new(regelnummer, LineInhoud::Naar{ sprong_doel: SprongDoel::vul(&sprong_doel)?, opm: vind_opmerking(&opmerking)? }))
        },
        Sleutelwoord::NP => {
            is_alleen_keyword(rest_na_keyword, regelnummer, keyword)
        },
        Sleutelwoord::NR => {
            let (mut argumenten, rest_na_argumenten) = extract_argumenten(rest_na_keyword).unwrap_or( (Vec::new(), rest_na_keyword));
            if argumenten.len() != 1 && !argumenten.is_empty() {
                return Err(EcolFout::melding(EcolFoutVariant::VerkeerdAantalArgumentenDubbel("NR".to_string(), "geen of één".to_string(), argumenten.len())))
            }
            if argumenten.is_empty() {
                argumenten.push("1".to_string());
            }

            Ok(Line::new(regelnummer, LineInhoud::NR{ aantal: argumenten[0].clone(), opm: vind_opmerking(rest_na_argumenten)? }))
        },
        Sleutelwoord::RIJ => {
            let (argumenten, rest_na_argumenten) = extract_argumenten(rest_na_keyword).unwrap_or( (Vec::new(), rest_na_keyword));
            if argumenten.len() != 2 {
                return Err(EcolFout::melding(EcolFoutVariant::VerkeerdAantalArgumenten("RIJ".to_string(), 2, argumenten.len())))
            }

            let Some((naam, rest_na_variabele)) = extract_variabele_naam(rest_na_argumenten) else {
                return Err(EcolFout::melding(EcolFoutVariant::GeenVariabeleNaam))
            };

            Ok(Line::new(regelnummer, LineInhoud::Rij{ start: argumenten[0].clone(), eind: argumenten[1].clone(), variabele_naam: naam.to_string(), opm: vind_opmerking(rest_na_variabele)? }))
        },
        Sleutelwoord::RIJSYM => {
            let (argumenten, rest_na_argumenten) = extract_argumenten(rest_na_keyword).unwrap_or( (Vec::new(), rest_na_keyword));
            if argumenten.len() != 2 {
                return Err(EcolFout::melding(EcolFoutVariant::VerkeerdAantalArgumenten("RIJSYM".to_string(), 2, argumenten.len())))
            }

            let Some((naam, rest_na_variabele)) = extract_variabele_naam(rest_na_argumenten) else {
                return Err(EcolFout::melding(EcolFoutVariant::GeenVariabeleNaam))
            };

            Ok(Line::new(regelnummer, LineInhoud::Rijsym{ start: argumenten[0].clone(), eind: argumenten[1].clone(), variabele_naam: naam.to_string(), opm: vind_opmerking(rest_na_variabele)? }))
        }
        Sleutelwoord::TOEKENNEN => {
            let Some((variabele_naam, rest_na_variabele)) = extract_variabele_naam(rest_na_regelnummer) else {
                return Err(EcolFout::melding(EcolFoutVariant::GeenVariabeleNaam))
            };
            let (argumenten, rest_na_argumenten) = extract_argumenten(rest_na_variabele).unwrap_or( (Vec::new(), rest_na_variabele));
            let argument: String;
            if argumenten.is_empty() {
                argument = String::new();
            } else if argumenten.len() == 1 {
                argument = argumenten[0].clone();
            } else {
                return Err(EcolFout::melding(EcolFoutVariant::VerkeerdAantalArgumenten("Een RIJ variabele".to_string(), 1, argumenten.len())));
            }
            let Some(rest_na_wordt_teken) = is_geldig_wordt_teken(rest_na_argumenten) else {
                return Err(EcolFout::melding(EcolFoutVariant::OngeldigWordtTeken))
            };
            let (expressie, opmerking) = extract_opmerking(rest_na_wordt_teken);
            Ok(Line::new(regelnummer, LineInhoud::Toekennen{ variabele_naam: variabele_naam.to_string(), argument, expressie, opm: vind_opmerking(&opmerking)? }))
        },
        Sleutelwoord::TEKST => {
            let Some(rest_na_wordt_teken) = is_geldig_wordt_teken(rest_na_keyword) else {
                return Err(EcolFout::melding(EcolFoutVariant::OngeldigWordtTeken))
            };
            let (expressie, opmerking) = extract_opmerking(rest_na_wordt_teken);

            Ok(Line::new(regelnummer, LineInhoud::Tekst{ expressie, opm: vind_opmerking(&opmerking)? }))
        },
        Sleutelwoord::SCHRIJF => {
            let (argumenten, rest_na_argumenten) = extract_argumenten(rest_na_keyword).unwrap_or( (Vec::new(), rest_na_keyword));
            if argumenten.len() != 2 && !argumenten.is_empty() {
                return Err(EcolFout::melding(EcolFoutVariant::VerkeerdAantalArgumentenDubbel("SCHRIJF".to_string(), "geen of 2".to_string(), argumenten.len())))
            }
            let breedte: usize;
            let decimalen: usize;
            if argumenten.is_empty() {
                breedte = 0;
                decimalen = 0;
            } else {
                breedte = argumenten[0].trim().parse::<usize>().map_err(|_| EcolFout::melding(EcolFoutVariant::OngeldigGetal("als breedte bij SCHRIJF".to_string())))? ;
                decimalen = argumenten[1].trim().parse::<usize>().map_err(|_| EcolFout::melding(EcolFoutVariant::OngeldigGetal("als aantal decimalen bij SCHRIJF".to_string())))? ;
                if breedte == 0 {
                    return Err(EcolFout::melding(EcolFoutVariant::GetalNietGroterDan("De breedte bij SCHRIJF".to_string(), 0)));
                }
            }

            let Some(rest_na_wordt_teken) = is_geldig_wordt_teken(rest_na_argumenten) else {
                return Err(EcolFout::melding(EcolFoutVariant::OngeldigWordtTeken))
            };

            let (expressie, opmerking) = extract_opmerking(rest_na_wordt_teken);

            Ok(Line::new(regelnummer, LineInhoud::Schrijf{ breedte, decimalen, expressie, opm: vind_opmerking(&opmerking)? }))
        },
        Sleutelwoord::SCHRIJFSYM => {
            let Some(rest_na_wordt_teken) = is_geldig_wordt_teken(rest_na_keyword) else {
                return Err(EcolFout::melding(EcolFoutVariant::OngeldigWordtTeken))
            };
            let (expressie, opmerking) = extract_opmerking(rest_na_wordt_teken);

            Ok(Line::new(regelnummer, LineInhoud::Schrijfsym{ expressie, opm: vind_opmerking(&opmerking)? }))
        },
        Sleutelwoord::SCHRIJM => {
            let Some(rest_na_wordt_teken) = is_geldig_wordt_teken(rest_na_keyword) else {
                return Err(EcolFout::melding(EcolFoutVariant::OngeldigWordtTeken))
            };
            let (expressie, opmerking) = extract_opmerking(rest_na_wordt_teken);

            Ok(Line::new(regelnummer, LineInhoud::Schrijm{ expressie, opm: vind_opmerking(&opmerking)? }))
        },
        Sleutelwoord::SPATIE => {
            let (mut argumenten, rest_na_argumenten) = extract_argumenten(rest_na_keyword).unwrap_or( (Vec::new(), rest_na_keyword));
            if argumenten.len() != 1 && !argumenten.is_empty() {
                return Err(EcolFout::melding(EcolFoutVariant::VerkeerdAantalArgumentenDubbel("SPATIE".to_string(), "geen of één".to_string(), argumenten.len())))
            }
            if argumenten.is_empty() {
                argumenten.push("1".to_string());
            }
            
            Ok(Line::new(regelnummer, LineInhoud::Spatie{ aantal: argumenten[0].clone(), opm: vind_opmerking(rest_na_argumenten)? }))
        },
        Sleutelwoord::BEWAAR => {
            if regelnummer == 0 {
                is_alleen_keyword(rest_na_keyword, regelnummer, keyword)
            } else {
                Err(EcolFout::melding(EcolFoutVariant::NietInProgramma("BEWAAR".to_string())))
            }
        },
        Sleutelwoord::LAAD => {
            if regelnummer == 0 {
                is_alleen_keyword(rest_na_keyword, regelnummer, keyword)
            } else {
                Err(EcolFout::melding(EcolFoutVariant::NietInProgramma("LAAD".to_string())))
            }
        },
        Sleutelwoord::START => {
            if regelnummer == 0 {
                is_alleen_keyword(rest_na_keyword, regelnummer, keyword)
            } else {
                Err(EcolFout::melding(EcolFoutVariant::NietInProgramma("START".to_string())))
            }
        },
        Sleutelwoord::SUB => {
            if regelnummer == 0 {
                return Err(EcolFout::melding(EcolFoutVariant::AlleenInProgramma("SUB".to_string())))
            }
            let Some((variabele_naam, rest_na_variabele)) = extract_variabele_naam(rest_na_keyword) else {
                return Err(EcolFout::melding(EcolFoutVariant::GeenVariabeleNaam))
            };

            Ok(Line::new(regelnummer, LineInhoud::Sub { sub_naam: geen_spaties(variabele_naam), opm: vind_opmerking(rest_na_variabele)? }))
        }
    }

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

/// Na de variabelenaam: sla spaties over, en als er een `(...)` volgt, neem die mee als index.
/// Geeft `(Some(index_expressie), einde_na_haakje)` of `(None, naam_einde)` terug.
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