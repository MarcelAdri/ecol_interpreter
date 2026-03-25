#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(super) struct EcolString {
    inhoud: String,
}
impl std::fmt::Display for EcolString {

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.inhoud)
    }
}
impl EcolString {
    pub(super) fn new(inhoud: impl Into<String>) -> Self {
        EcolString { inhoud: inhoud.into() }
    }
    pub(super) fn as_str(&self) -> &str {
        &self.inhoud
    }
    pub(super) fn chars(&self) -> impl Iterator<Item = char> + '_ {
        self.inhoud.chars()
    }
    pub(super) fn from_literal(literal: &str) -> Result<Self, String> {

        let Some(zonder_begin) = literal.strip_prefix('"') else {
            return Err("String moet binnen dubbele aanhalingstekens worden geplaatst".to_string());
        };

        let Some(inhoud) = zonder_begin.strip_suffix('"') else {
            return Err("String moet binnen dubbele aanhalingstekens worden geplaatst".to_string());
        };

        let mut werk = String::new();
        let mut escaped = false;

        for c in inhoud.chars() {
            if !escaped && c == '\\' {
                escaped = true;
                continue;
            }

            if escaped {
                match c {
                    '"' => werk.push('"'),
                    '\\' => werk.push('\\'),
                    'n' => werk.push('\n'),
                    'r' => werk.push('\r'),
                    't' => werk.push('\t'),
                    '0' => werk.push('\0'),
                    _ => {
                        werk.push('\\');
                        werk.push(c);
                    }
                }
                escaped = false;
                continue;
            }

            werk.push(c);
        }

        if escaped {
            werk.push('\\');
        }

        Ok(Self::new(werk))
    }
    pub(super) fn push(&mut self, c: char) {
        self.inhoud.push(c);
    }
    pub(super) fn to_expressions(&self) -> String {
        let mut werk = String::new();

        for c in self.chars(){
            if let Some(escaped) = escape_char(c) {
                werk.push_str(escaped);
            } else {
                werk.push(c);
            }
        }

        format!("\"{}\"", werk)
    }
}
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
    Tekst,
}
#[derive(Debug, Clone)]
pub(super) enum Waarde {
    Getal(f32),
    Tekst(EcolString),
}
impl Waarde {
    pub(super) fn standaard_voor_type(var_type: VariabeleType) -> Self {
        match var_type {
            VariabeleType::Getal => Waarde::Getal(0.0),
            VariabeleType::Tekst => Waarde::Tekst(EcolString::default()),
        }
    }
    pub(super) fn type_van(&self) -> Option<VariabeleType> {
        match self {
            Waarde::Getal(_) => Some(VariabeleType::Getal),
            Waarde::Tekst(_) => Some(VariabeleType::Tekst),
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
        Waarde::Tekst(x) => format!("{}", x),
    }
}
pub(super) fn waarde_naar_expressie(waarde: &Waarde) -> String {
    match waarde {
        Waarde::Getal(x) => format_getal(*x),
        Waarde::Tekst(x) => {
            x.to_expressions()
        }
    }
}

