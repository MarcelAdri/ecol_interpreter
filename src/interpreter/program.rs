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
            LineInhoud::Als { .. } => Some(Sleutelwoord::ALS),
            LineInhoud::Help {} => Some(Sleutelwoord::HELP),
            LineInhoud::Klaar {} => Some(Sleutelwoord::KLAAR),
            LineInhoud::LegeRegel { } => None,
            LineInhoud::Lijst {} => Some(Sleutelwoord::LIJST),
            LineInhoud::Naar { .. } => Some(Sleutelwoord::NAAR),
            LineInhoud::NP {} => Some(Sleutelwoord::NP),
            LineInhoud::NR { .. } => Some(Sleutelwoord::NR),
            LineInhoud::Rij { .. } => Some(Sleutelwoord::RIJ),
            LineInhoud::Rijsym { .. } => Some(Sleutelwoord::RIJSYM),
            LineInhoud::Tekst { .. } => Some(Sleutelwoord::TEKST),
            LineInhoud::Schrijf { .. } => Some(Sleutelwoord::SCHRIJF),
            LineInhoud::Schrijfsym { .. } => Some(Sleutelwoord::SCHRIJFSYM),
            LineInhoud::Schrijm { .. } => Some(Sleutelwoord::SCHRIJM),
            LineInhoud::Spatie { .. } => Some(Sleutelwoord::SPATIE),
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
            LineInhoud::Als { vergelijking, dan, anders } => {
                match anders {
                    Some( x ) => format!("{}ALS {} DAN {} ANDERS {}"
                                         ,regelnummer
                                         ,vergelijking
                                         ,dan
                                         ,x)
                        .trim_start()
                        .to_string(),
                    None => format!("{}ALS {} DAN {}"
                                    ,regelnummer
                                    ,vergelijking
                                    ,dan)
                        .trim_start()
                        .to_string(),
                }
            }
            LineInhoud::Help {} => format!("{}HELP", regelnummer)
                .trim_start()
                .to_string(),
            LineInhoud::Klaar {} => format!("{}KLAAR", regelnummer)
                .trim_start()
                .to_string(),
            LineInhoud::LegeRegel { } => format!("{}:", regelnummer)
                .trim_start()
                .to_string(),
            LineInhoud::Lijst {} => format!("{}LIJST", regelnummer)
                .trim_start()
                .to_string(),
            LineInhoud::Naar { sprong_doel } => format!("{}NAAR {}", regelnummer, sprong_doel)
                .trim_start()
                .to_string(),
            LineInhoud::NP {} => format!("{}NP", regelnummer)
                .trim_start()
                .to_string(),
            LineInhoud::NR { aantal } =>
                format!("{}NR({})"
                        ,regelnummer
                        ,aantal)
                .trim_start()
                .to_string(),
            LineInhoud::Rij { start, eind, variabele_naam } =>
                format!("{}RIJ({}, {}) {}"
                        ,regelnummer
                        ,start
                        ,eind
                        ,variabele_naam)
                .trim_start()
                .to_string(),
            LineInhoud::Rijsym { start, eind, variabele_naam } =>
                format!("{}RIJSYM({}, {}) {}"
                        ,regelnummer
                        ,start
                        ,eind
                        ,variabele_naam)
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
            LineInhoud::Schrijfsym { expressie } =>
                format!("{}SCHRIJFSYM := {}"
                        ,regelnummer
                        ,expressie)
                    .trim_start()
                    .to_string(),
            LineInhoud::Schrijm { expressie } =>
                format!("{}SCHRIJM := {}"
                        ,regelnummer
                        ,expressie)
                    .trim_start()
                    .to_string(),
            LineInhoud::Spatie { aantal } =>
                format!("{}SPATIE({})"
                        ,regelnummer
                        ,aantal)
                    .trim_start()
                    .to_string(),
            LineInhoud::Start { } => "".to_string(),
            LineInhoud::Tekst { expressie } =>
                format!("{}TEKST := {}"
                        ,regelnummer
                        ,expressie)
                    .trim_start()
                    .to_string(),
            LineInhoud::Toekennen { variabele_naam, argument, expressie } =>
                if *argument == 0 {
                    format!("{}{} := {}"
                            ,regelnummer
                            ,variabele_naam
                            ,expressie)
                        .trim_start()
                        .to_string()
                } else {
                    format!("{}{}({}) := {}"
                            ,regelnummer
                            ,variabele_naam
                            ,argument
                            ,expressie)
                        .trim_start()
                        .to_string()
                },

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
    Als {vergelijking: String, dan: u16, anders: Option<u16>},
    Help {},
    Klaar {},
    LegeRegel {},
    Lijst {},
    Naar {sprong_doel: usize},
    NP {},
    NR {
        aantal: usize
    },
    Rij {
        start: usize,
        eind: usize,
        variabele_naam: String,
    },
    Rijsym {
        start: usize,
        eind: usize,
        variabele_naam: String,
    },
    Schrijf {
        breedte: usize,
        decimalen: usize,
        expressie: String,
    },
    Schrijfsym {
        expressie: String,
    },
    Schrijm {
        expressie: String,
    },
    Spatie {
        aantal: usize,
    },
    Start {},
    Tekst {
        expressie: String,
    },
    Toekennen {
        variabele_naam: String,
        argument: usize,
        expressie: String,
    },
    Verwijderen {},
}
impl LineInhoud {
    pub(super) fn from_sleutelwoord(sleutelwoord: Sleutelwoord) -> Self {
        match sleutelwoord {
            Sleutelwoord::ALS => Self::Als {vergelijking: String::new(), dan: 0, anders: None},
            Sleutelwoord::HELP => Self::Help {},
            Sleutelwoord::KLAAR => Self::Klaar {},
            Sleutelwoord::LIJST => Self::Lijst {},
            Sleutelwoord::NAAR => Self::Naar {sprong_doel: 0},
            Sleutelwoord::NP => Self::NP {},
            Sleutelwoord::NR => Self::NR { aantal: 0 },
            Sleutelwoord::RIJ => Self::Rij { start: 0, eind: 0, variabele_naam: String::new() },
            Sleutelwoord::RIJSYM => Self::Rijsym { start: 0, eind: 0, variabele_naam: String::new() },
            Sleutelwoord::SCHRIJF => Self::Schrijf { breedte: 0, decimalen: 0, expressie: String::new() },
            Sleutelwoord::SCHRIJFSYM => Self::Schrijfsym { expressie: String::new() },
            Sleutelwoord::SCHRIJM => Self::Schrijm { expressie: String::new() },
            Sleutelwoord::SPATIE => Self::Spatie { aantal: 0 },
            Sleutelwoord::START => Self::Start {},
            Sleutelwoord::TEKST => Self::Tekst { expressie: String::new() },
            Sleutelwoord::TOEKENNEN => Self::Toekennen { variabele_naam: String::new(), argument: 0, expressie: String::new() },
        }
    }
    pub(super) fn as_str(&self) -> &str {
        match self {
            LineInhoud::Als { .. } => "Als",
            LineInhoud::Help { } => "Help",
            LineInhoud::Klaar { } => "Klaar",
            LineInhoud::LegeRegel { } => "",
            LineInhoud::Lijst { } => "Lijst",
            LineInhoud::Naar { .. } => "Naar",
            LineInhoud::NP { } => "NP",
            LineInhoud::NR { .. } => "NR",
            LineInhoud::Rij { .. } => "Rij",
            LineInhoud::Rijsym { .. } => "Rijsym",
            LineInhoud::Schrijf { .. } => "Schrijf",
            LineInhoud::Schrijfsym { .. } => "Schrijfsym",
            LineInhoud::Schrijm { .. } => "Schrijm",
            LineInhoud::Spatie { .. } => "Spatie",
            LineInhoud::Start { } => "Start",
            LineInhoud::Tekst { .. } => "Tekst",
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
    ALS,
    HELP,
    KLAAR,
    LIJST,
    NAAR,
    NP,
    NR,
    RIJ,
    RIJSYM,
    SCHRIJF,
    SCHRIJFSYM,
    SCHRIJM,
    SPATIE,
    START,
    TEKST,
    TOEKENNEN,
}
impl Sleutelwoord {

    pub(super) fn from_string(input: &str) -> Option<Self> {
        match input {
            "ALS" => Some(Sleutelwoord::ALS),
            "HELP" => Some(Sleutelwoord::HELP),
            "KLAAR" => Some(Sleutelwoord::KLAAR),
            "LIJST" => Some(Sleutelwoord::LIJST),
            "NAAR" => Some(Sleutelwoord::NAAR),
            "NP" => Some(Sleutelwoord::NP),
            "NR" => Some(Sleutelwoord::NR),
            "RIJ" => Some(Sleutelwoord::RIJ),
            "RIJSYM" => Some(Sleutelwoord::RIJSYM),
            "SCHRIJF" => Some(Sleutelwoord::SCHRIJF),
            "SCHRIJFSYM" => Some(Sleutelwoord::SCHRIJFSYM),
            "SCHRIJM" => Some(Sleutelwoord::SCHRIJM),
            "SPATIE" => Some(Sleutelwoord::SPATIE),
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
            Sleutelwoord::ALS => "ALS",
            Sleutelwoord::HELP => "HELP",
            Sleutelwoord::KLAAR => "KLAAR",
            Sleutelwoord::LIJST => "LIJST",
            Sleutelwoord::NAAR => "NAAR",
            Sleutelwoord::NP => "NP",
            Sleutelwoord::NR => "NR",
            Sleutelwoord::RIJ => "RIJ",
            Sleutelwoord::RIJSYM => "RIJSYM",
            Sleutelwoord::SCHRIJF => "SCHRIJF",
            Sleutelwoord::SCHRIJFSYM => "SCHRIJFSYM",
            Sleutelwoord::SCHRIJM => "SCHRIJM",
            Sleutelwoord::SPATIE => "SPATIE",
            Sleutelwoord::START => "START",
            Sleutelwoord::TEKST => "TEKST",
            Sleutelwoord::TOEKENNEN => "TOEKENNEN",
        }
    }

}




