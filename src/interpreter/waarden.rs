use crate::interpreter::helpers::format_getal;

pub(super) struct VariabeleAanroep {
    variabele_naam: String,
    start: usize,
    einde: usize,
}
impl VariabeleAanroep {
    pub(super) fn new(variabele_naam: String, start: usize, einde: usize) -> Self {
        VariabeleAanroep {
            variabele_naam,
            start,
            einde,
        }
    }
    pub(super) fn variabele_naam(&self) -> &str {
        &self.variabele_naam
    }
    pub(super) fn start(&self) -> usize {
        self.start
    }
    pub(super) fn einde(&self) -> usize {
        self.einde
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum VariabeleType {
    Getal,
}
#[derive(Debug, Clone)]
pub(super) enum Waarde {
    Getal(f32),
}
impl Waarde {
    pub(super) fn standaard_voor_type(var_type: VariabeleType) -> Self {
        match var_type {
            VariabeleType::Getal => Waarde::Getal(0.0),
        }
    }
    pub(super) fn type_van(&self) -> Option<VariabeleType> {
        match self {
            Waarde::Getal(_) => Some(VariabeleType::Getal),
        }
    }
    pub(super) fn haal_getal(&self) -> f32 {
        match self {
            Waarde::Getal(x) => *x,
        }
    }
    pub(super) fn format_getal(&self, breedte: usize, decimalen: usize) -> Result<String, String> {
        let mut b = breedte;
        let mut d = decimalen;
        let x = self.haal_getal();

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
}

//Helpers
fn escape_char(c: char) -> Option<&'static str> {
    match c {
        '"' => Some("\\\""),
        '\\' => Some("\\\\"),
        '\n' => Some("\\n"),
        '\r' => Some("\\r"),
        '\t' => Some("\\t"),
        '\0' => Some("\\0"),
        _ => None,
    }
}

pub(super) fn haal_data(token: &Waarde) -> String {

    match token {
        Waarde::Getal(x) => format_getal(*x),
    }
}
pub(super) fn waarde_naar_expressie(waarde: &Waarde) -> String {
    match waarde {
        Waarde::Getal(x) => format_getal(*x),
    }
}

