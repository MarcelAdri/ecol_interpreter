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
pub(super) fn format_getal(x: f32) -> String {
    let mut s = format!("{:.6}", x);

    while s.contains('.') && s.ends_with('0') {
        s.pop();
    }

    if s.ends_with('.') {
        s.push('0');
    }

    s
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

