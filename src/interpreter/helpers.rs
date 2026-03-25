use crate::interpreter::program::{Line, LineInhoud, Operator, Sleutelwoord, WORDT_TEKEN};

pub(super) fn extract_argumenten(input: &str) -> Option<(Vec<usize>, &str)> {
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
pub(super) fn extract_keyword(input: &str) -> Option<(Sleutelwoord, &str)> {
    let werkstring = input.trim_start();

    if werkstring.chars().next().is_some_and(|c| c.is_ascii_lowercase()) {
        return Some((Sleutelwoord::TOEKENNEN, werkstring));
    }
    let position = werkstring.find(|c: char| !c.is_ascii_uppercase()).unwrap_or(werkstring.len());
    let (keyword_string, rest) = werkstring.split_at(position);

    let Some(resultaat) = Sleutelwoord::from_string(keyword_string.trim_start()) else { return None };

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
    } else if let Some(positie) = input.find(|c: char| c.is_ascii_alphabetic()) {
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
pub(super) fn first_word(input: &str) -> &str {
    for (i, c) in input.char_indices() {
        if Operator::is_operator_char(c) || c == '(' {
            return &input[..i];
        }
    }
    input
}
pub(super) fn geen_spaties_buiten_literals(input: &str) -> String {
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
        Ok(Line::new(regelnummer, LineInhoud::from_sleutelwoord(keyword)))
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