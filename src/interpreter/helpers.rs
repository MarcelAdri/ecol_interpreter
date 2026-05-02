use crate::interpreter::errors::EcolFout;
use crate::interpreter::program::{Line, LineInhoud, Sleutelwoord, SprongDoel, WORDT_TEKEN};

pub(super) fn extract_argumenten(input: &str) -> Option<(Vec<String>, &str)> {
    let werkstring = input.trim_start();

    let (mut inhoud, rest) = werkstring.split_once(')')?;
    inhoud = inhoud.strip_prefix('(')?;

    let reply = argumenten_to_vec(inhoud);
    Some((reply, rest.trim_start()))
}
pub(super) fn argumenten_to_vec(input: &str) -> Vec<String> {
    input.split(',').filter_map(|s| { let t = s.trim().to_string(); if t.is_empty() { None } else { Some(t) } }).collect()

}
pub(super) fn extract_keyword(input: &str) -> Option<(Sleutelwoord, &str)> {
    let werkstring = input.trim_start();

    if werkstring.chars().next().is_some_and(|c| c.is_ascii_lowercase()) {
        let (variabele, rest_na_variabele) = extract_variabele_naam(werkstring).unwrap_or(("", werkstring.trim_start()));
        if variabele.is_empty() { return None }
        if geen_spaties(rest_na_variabele).is_empty() { return Some((Sleutelwoord::GASUB, variabele)); }

        return Some((Sleutelwoord::TOEKENNEN, rest_na_variabele.trim_start()));
    }
    let position = werkstring.find(|c: char| !c.is_ascii_uppercase()).unwrap_or(werkstring.len());
    let (keyword_string, rest) = werkstring.split_at(position);

    let resultaat: Sleutelwoord = if keyword_string.trim_start().starts_with("FUN") {
        if rest.trim_start().chars().next().is_some_and(|c| c.is_ascii_lowercase()) {
            Sleutelwoord::FUNstart
        } else {
            Sleutelwoord::FUNeind
        }
    } else {
        Sleutelwoord::from_string(keyword_string.trim_start())?
    };


    Some((resultaat, rest.trim_start()))
}
pub(super) fn extract_regelnummer(input: &str) -> Result<(u16, &str, bool), EcolFout> {
    let is_alleen_regelnummer: bool = input.trim().chars().all(|c| c.is_ascii_digit());

    let (nummer, restregel) = if is_alleen_regelnummer {
        (input.trim(), "")
    } else if let Some(positie) = input.find(|c: char| c.is_ascii_alphabetic() || c == ':') {
        let (gevonden_nummer, rest) = input.split_at(positie);
        (gevonden_nummer, rest.trim_start())
    } else {
        ("0", input.trim_start())
    };

    let Some(resultaat_parsing) = nummer.trim().parse::<u16>().ok() else { return Ok((0u16, input.trim_start(), is_alleen_regelnummer)) };
    let regelnummer = resultaat_parsing;
    if regelnummer > 999u16 { return Err(EcolFout::FoutMelding("Regelnummer mag niet groter zijn dan 999".to_string())) }


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
pub(super) fn extract_stap_expressie(input: &str) -> Result<(String, String), EcolFout> {
    let werkstring = geen_spaties(input);

    match vind_top_level_komma(&werkstring) {
        Some(p) => {
            let (stap_tekst, r) = werkstring.split_at(p);
            Ok((stap_tekst.to_string(), r[1..].to_string()))
        },
        None => Err(EcolFout::FoutMelding("Ongeldige syntax voor MET bij de STAP-expressie.".to_string())),
    }
}
pub(super) fn extract_start_expressie(input: &str) -> Result<(String, String, String), EcolFout> {
    let werkstring = geen_spaties(input);

    match vind_top_level_komma(&werkstring) {
        Some(p) => {
            let (toekenning_string, r) = werkstring.split_at(p);
            let Some((naam, rest_na_variabele)) = extract_variabele_naam(toekenning_string) else { return Err(EcolFout::FoutMelding("Ongeldige variabele-naam in MET.".to_string())) };
            let Some(rest_na_wordt_teken) = is_geldig_wordt_teken(rest_na_variabele) else { return Err(EcolFout::FoutMelding("Ongeldig 'wordt'-teken in MET.".to_string())) };
            Ok((naam.to_string(), rest_na_wordt_teken.to_string(), r[1..].to_string()))
        },
        None => Err(EcolFout::FoutMelding("Ongeldige syntax voor MET bij de START-expressie.".to_string())),
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
pub(super) fn extract_dan(input: &str) -> Result<(SprongDoel, &str), EcolFout> {
    let werkstring = input.trim_start();
    let pos = werkstring.find("ANDERS");
    let dan_reply: SprongDoel;

    if let Some(positie) = pos {
        let dan = &werkstring[..positie];
        let rest = &werkstring[positie + 6..];

        dan_reply = SprongDoel::vul(&geen_spaties(dan))?;

        Ok((dan_reply, rest.trim_start()))
    } else {
        let dan = &geen_spaties(werkstring);
        dan_reply = SprongDoel::vul(dan)?;

        Ok((dan_reply, ""))
    }
}
pub(super) fn extract_anders(input: &str) -> Result<(SprongDoel, &str), EcolFout> {
    let werkstring = geen_spaties(input);

    let anders = SprongDoel::vul(&werkstring)?;
    
    Ok((anders, ""))
}
pub(super) fn extract_opmerking(input: &str) -> (String, String) {
    let deel_voor_opmerking: String;
    let opmerking: String;

    let rp = input.find(';');
    if let Some(p) = rp {
        deel_voor_opmerking = geen_spaties(&input[..p]);
        opmerking = input[p..].to_string();
    } else {
        deel_voor_opmerking = geen_spaties(input);
        opmerking = String::new();
    }

    (deel_voor_opmerking, opmerking)
}
pub(super) fn format_getal(getal: f32, breedte: usize, decimalen: usize) -> Result<String, EcolFout> {
    let mut b = breedte;
    let mut d = decimalen;
    let x = getal;

    if breedte == 0 {
        b = 6;
        if x == x.trunc() {
            d = 0;
        } else {
            d = 4;
        }
    }

    let ruimte_voor_teken = usize::from(x < 0f32);
    let b_totaal = if d == 0 { b + ruimte_voor_teken } else { b + d + ruimte_voor_teken + 1 };

    let reply = format!("{x:b_totaal$.d$}");

    if reply.len() > b_totaal {
        return Err(EcolFout::FoutMelding(format!("De opgegeven waarde {x} past niet in de opgegeven breedte {breedte}.")))
    }

    Ok(reply)

}
pub(super) fn geen_rest_gewenst() -> EcolFout {
    EcolFout::FoutMelding("FOUTMELDING: Tekst aangetroffen na complete programmaregel.".to_string())
}
pub(super) fn geen_spaties(input: &str) -> String {
    input.chars().filter(|c| !c.is_whitespace()).collect::<String>()
}
pub(super) fn get_sym_value(getal: &f32) -> Result<u8, EcolFout> {
    if getal.is_nan() || !(0.0..=99.0).contains(getal) || getal.fract() != 0.0 {
        return Err(EcolFout::FoutMelding(format!("Waarde {getal} is ongeldig (xxxSYM verwacht een geheel getal 0–99).")));
    }
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    Ok(*getal as u8) //veilig, want hierboven gevalideerd als geheel getal tussen 0 en 99
}
pub(super) fn grens_bewaking (getal: f32, alleen_positieve_getallen: bool, alleen_hele_getallen: bool) -> Result<f32, EcolFout> {
    if getal.fract() != 0.0 && alleen_hele_getallen {
        return Err(EcolFout::FoutMelding(format!("Getal moet een heel getal zijn, maar is {getal}")));
    }
    if getal <= 0f32 && alleen_positieve_getallen {
        return Err(EcolFout::FoutMelding(format!("Getal moet positief zijn, maar is {getal}")));
    }

    Ok(getal)
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
pub(super) fn is_alleen_keyword(input: &str, regelnummer: u16, keyword: Sleutelwoord) -> Result<Line, EcolFout> {
    Ok(Line::new(regelnummer, LineInhoud::from_sleutelwoord(keyword, input)?))
}
pub(super) fn is_geldig_wordt_teken(input: &str) -> Option<&str> {
    let rest = input.strip_prefix(WORDT_TEKEN)?;
    Some(rest.trim_start())
}
pub(super) fn is_geldige_variabele_naam(naam: &str) -> bool {
    heeft_geldige_variabele_syntax(naam)


}
pub(super) fn literal_to_string (literal: &str) -> Result<String, EcolFout> {
    let werk_string = literal.trim();

    if !werk_string.starts_with('"') {
        return Err(EcolFout::FoutMelding("Tekstblok moet beginnen met een aanhalingsteken.".to_string()));
    }

    if !werk_string.ends_with('"') {
        return Err(EcolFout::FoutMelding("Tekstblok moet eindigen met een aanhalingsteken.".to_string()));
    }

    let inhoud = &werk_string[1..werk_string.len() - 1];

    if inhoud.contains('"') {
        return Err(EcolFout::FoutMelding("Slechts één aaneengesloten tekstblok toegestaan.".to_string()));
    }

    Ok(inhoud.to_string())

}
pub(super) fn vind_opmerking(input: &str) -> Result<Option<String>, EcolFout> {
    if input.trim().is_empty() {
        Ok(None)
    } else if input.trim_start().starts_with(';') {
        Ok(Some(input.trim_start().trim_start_matches(';').trim_start().to_string()))
    } else {
        Err(geen_rest_gewenst())
    }
}