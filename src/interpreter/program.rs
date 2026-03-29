use std::collections::BTreeMap;

pub(super) const WORDT_TEKEN: &str = ":=";

pub(super) struct Line {
    regelnummer: u16,
    inhoud: LineInhoud,
}
impl Line {
    pub(super) fn new(
        regelnummer: u16,
        inhoud: LineInhoud,
    ) -> Self {
        Line {
            regelnummer,
            inhoud,
        }
    }
    pub(super) fn extract_sleutelwoord(&self) -> Option<Sleutelwoord> {
        match &self.inhoud {
            LineInhoud::Help {} => Some(Sleutelwoord::HELP),
            LineInhoud::Klaar {} => Some(Sleutelwoord::KLAAR),
            LineInhoud::Lijst {} => Some(Sleutelwoord::LIJST),
            LineInhoud::NR {} => Some(Sleutelwoord::NR),
            LineInhoud::Tekst { .. } => Some(Sleutelwoord::TEKST),
            LineInhoud::Schrijf { .. } => Some(Sleutelwoord::SCHRIJF),
            LineInhoud::Start { } => Some(Sleutelwoord::START),
            LineInhoud::Toekennen { .. } => Some(Sleutelwoord::TOEKENNEN),
            LineInhoud::Verwijderen { } => None,
        }
    }
    pub(super) fn genereer_regel(&self) -> String {
        let regelnummer:String;
        if self.regelnummer == 0 {
            regelnummer = "".to_string();
        } else {
            regelnummer = format!("{:>4} ", self.regelnummer);
        }
        match &self.inhoud {
            LineInhoud::Help {} => format!("{}HELP", regelnummer)
                .trim_start()
                .to_string(),
            LineInhoud::Klaar {} => format!("{}KLAAR", regelnummer)
                .trim_start()
                .to_string(),
            LineInhoud::Lijst {} => format!("{}LIJST", regelnummer)
                .trim_start()
                .to_string(),
            LineInhoud::NR {} => format!("{}NR", regelnummer)
                .trim_start()
                .to_string(),
            LineInhoud::Tekst { expressie } => format!("{}TEKST := {}"
                                                       ,regelnummer
                                                       ,expressie)
                .trim_start()
                .to_string(),
            LineInhoud::Schrijf { breedte, decimalen, expressie } =>
                format!("{}SCHRIJF({}, {}) := {}"
                        ,regelnummer
                        ,breedte
                        ,decimalen
                        ,expressie)
                    .trim_start()
                    .to_string(),
            LineInhoud::Start { } => "".to_string(),
            LineInhoud::Toekennen { variabele_naam, expressie } =>
                format!("{}{} := {}"
                        ,regelnummer
                        ,variabele_naam
                        ,expressie)
                    .trim_start()
                    .to_string(),
            LineInhoud::Verwijderen { } => "".to_string(),
        }
    }
    pub(super) fn regelnummer(&self) -> u16 {
        self.regelnummer
    }
    pub(super) fn inhoud(&self) -> &LineInhoud {
        &self.inhoud
    }
}
#[derive(Debug, Clone)]
pub(super) enum LineInhoud {
    Help {},
    Klaar {},
    Lijst {},
    NR {},
    Tekst {
        expressie: String,
    },
    Schrijf {
        breedte: usize,
        decimalen: usize,
        expressie: String,
    },
    Start {},
    Toekennen {
        variabele_naam: String,
        expressie: String,
    },
    Verwijderen {},
}
impl LineInhoud {
    pub(super) fn from_sleutelwoord(sleutelwoord: Sleutelwoord) -> Self {
        match sleutelwoord {
            Sleutelwoord::HELP => Self::Help {},
            Sleutelwoord::KLAAR => Self::Klaar {},
            Sleutelwoord::LIJST => Self::Lijst {},
            Sleutelwoord::NR => Self::NR {},
            Sleutelwoord::TEKST => Self::Tekst { expressie: String::new() },
            Sleutelwoord::SCHRIJF => Self::Schrijf { breedte: 0, decimalen: 0, expressie: String::new() },
            Sleutelwoord::START => Self::Start {},
            Sleutelwoord::TOEKENNEN => Self::Toekennen { variabele_naam: String::new(), expressie: String::new() },
        }
    }
    pub(super) fn as_str(&self) -> &str {
        match self {
            LineInhoud::Help { } => "Help",
            LineInhoud::Klaar { } => "Klaar",
            LineInhoud::Lijst { } => "Lijst",
            LineInhoud::NR { } => "NR",
            LineInhoud::Tekst { .. } => "Tekst",
            LineInhoud::Schrijf { .. } => "Schrijf",
            LineInhoud::Start { } => "Start",
            LineInhoud::Toekennen { .. } => "Toekennen",
            LineInhoud::Verwijderen { } => "Verwijderen",
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Operator {
    Plus,
    Min,
    Vermenigvuldig,
    Deel,
}
impl Operator {
    pub(super) fn from_char(c: char) -> Option<Self> {
        match c {
            '+' => Some(Self::Plus),
            '-' => Some(Self::Min),
            '*' => Some(Self::Vermenigvuldig),
            '/' => Some(Self::Deel),
            _ => None,
        }
    }

    pub(super) fn to_char(&self) -> char {
        match self {
            Self::Plus => '+',
            Self::Min => '-',
            Self::Vermenigvuldig => '*',
            Self::Deel => '/',
        }
    }

    pub(super) fn is_operator_char(c: char) -> bool {
        Self::from_char(c).is_some()
    }

    pub(super) fn operator_volgorde() -> impl Iterator<Item = Self> {
        IntoIterator::into_iter([
            Self::Vermenigvuldig,
            Self::Deel,
            Self::Plus,
            Self::Min,
        ])
    }

    pub(super) fn bereken(&self, links: f32, rechts: f32) -> Result<f32, String> {
        match self {
            Self::Plus => Ok(links + rechts),
            Self::Min => Ok(links - rechts),
            Self::Vermenigvuldig => Ok(links * rechts),
            Self::Deel => {
                if rechts == 0.0 {
                    return Err("Deel door 0 is niet toegestaan".to_string());
                }
                Ok(links / rechts)
            },
        }
    }
}
pub(super) struct Programma {
    programma: BTreeMap<u16, LineInhoud>
}
impl Programma {
    pub(super) fn new() -> Self {
        Self {
            programma: BTreeMap::new(),
        }
    }
    pub(super) fn laad(&mut self, bron: &BTreeMap<u16, LineInhoud>) {
        self.programma = bron.clone();
    }
    pub(super) fn programma(&self) -> &BTreeMap<u16, LineInhoud> {
        &self.programma
    }
    pub(super) fn regel_toevoegen(&mut self, regel: Line) -> String {
        let regelnummer = regel.regelnummer;
        let regel_inhoud = regel.inhoud;

        let Some(oude_regel) = self.programma.insert(regelnummer, regel_inhoud) else {
            return "".to_string();
        };

        format!("{} // vervangen", Line::new(regelnummer, oude_regel).genereer_regel())
    }

    pub(super) fn regel_verwijderen(&mut self, regelnummer: u16) -> Option<String> {
        let regel_inhoud = self.programma.remove(&regelnummer)?;
        let regel=Line::new(regelnummer, regel_inhoud);
        let reply = format!("{} // verwijderd", regel.genereer_regel());

        Some(reply)

    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Sleutelwoord {
    HELP,
    KLAAR,
    LIJST,
    NR,
    SCHRIJF,
    START,
    TEKST,
    TOEKENNEN,
}
impl Sleutelwoord {

    pub(super) fn from_string(input: &str) -> Option<Self> {
        match input {
            "HELP" => Some(Sleutelwoord::HELP),
            "KLAAR" => Some(Sleutelwoord::KLAAR),
            "LIJST" => Some(Sleutelwoord::LIJST),
            "NR" => Some(Sleutelwoord::NR),
            "SCHRIJF" => Some(Sleutelwoord::SCHRIJF),
            "START" => Some(Sleutelwoord::START),
            "TEKST" => Some(Sleutelwoord::TEKST),
            "TOEKENNEN" => Some(Sleutelwoord::TOEKENNEN),
            _ => None
        }
    }

    pub(super) fn is_sleutelwoord(woord: &str) -> bool {
        Self::from_string(woord).is_some()
    }

    pub(super) fn to_string(&self) -> &str {
        match self {
            Sleutelwoord::HELP => "HELP",
            Sleutelwoord::KLAAR => "KLAAR",
            Sleutelwoord::LIJST => "LIJST",
            Sleutelwoord::NR => "NR",
            Sleutelwoord::SCHRIJF => "SCHRIJF",
            Sleutelwoord::START => "START",
            Sleutelwoord::TEKST => "TEKST",
            Sleutelwoord::TOEKENNEN => "TOEKENNEN",
        }
    }

}




