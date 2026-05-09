use crate::interpreter::errors::{EcolFout, EcolFoutVariant};
use crate::interpreter::program::{Line, LineInhoud, Sleutelwoord, SprongDoel, WORDT_TEKEN};

pub(super) fn argumenten_to_vec(input: &str) -> Vec<String> {
    input.split(',').filter_map(|s| { let t = s.trim().to_string(); if t.is_empty() { None } else { Some(t) } }).collect()

}
fn geen_rest_gewenst() -> EcolFout {
    EcolFout::melding(EcolFoutVariant::TekstNaRegel)
}
pub(super) fn geen_spaties(input: &str) -> String {
    input.chars().filter(|c| !c.is_whitespace()).collect::<String>()
}
pub(super) fn get_sym_value(getal: &f32) -> Result<u8, EcolFout> {
    if getal.is_nan() || !(0.0..=99.0).contains(getal) || getal.fract() != 0.0 {
        return Err(EcolFout::melding(EcolFoutVariant::SymboolWaarde(*getal)));
    }
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    Ok(*getal as u8) //veilig, want hierboven gevalideerd als geheel getal tussen 0 en 99
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
pub(super) fn is_geldige_variabele_naam(naam: &str) -> bool {
    heeft_geldige_variabele_syntax(naam)
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

