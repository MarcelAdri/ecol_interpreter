use std::collections::BTreeMap;
use crate::interpreter::functions::EcolFout;
use crate::interpreter::helpers::geen_spaties;

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
            LineInhoud::End { } => Some(Sleutelwoord::END),
            LineInhoud::FunStart { .. } => Some(Sleutelwoord::FUNstart),
            LineInhoud::FunEind { .. } => Some(Sleutelwoord::FUNeind),
            LineInhoud::GaSub { .. } => Some(Sleutelwoord::GASUB),
            LineInhoud::Help {} => Some(Sleutelwoord::HELP),
            LineInhoud::Herhaal {} => Some(Sleutelwoord::HERHAAL),
            LineInhoud::Klaar {} => Some(Sleutelwoord::KLAAR),
            LineInhoud::LegeRegel { } => None,
            LineInhoud::Lijst {} => Some(Sleutelwoord::LIJST),
            LineInhoud::Met { .. } => Some(Sleutelwoord::MET),
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
            LineInhoud::Sub { .. } => Some(Sleutelwoord::SUB),
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
                                         ,dan.to_string()
                                         ,x.to_string())
                        .trim_start()
                        .to_string(),
                    None => format!("{}ALS {} DAN {}"
                                    ,regelnummer
                                    ,vergelijking
                                    ,dan.to_string())
                        .trim_start()
                        .to_string(),
                }
            }
            LineInhoud::End { } => {
                format!("{}END", regelnummer)
                    .trim_start()
                    .to_string()
            }
            LineInhoud::FunStart { variabele_naam, argumenten } => {
                format!("{}FUN {}({})"
                        ,regelnummer
                        ,variabele_naam
                        ,argumenten)
                    .trim_start()
                    .to_string()
            },
            LineInhoud::FunEind { expressie } => {
                format!("{}FUN := {}", regelnummer, expressie)
                    .trim_start()
                    .to_string()
            },
            LineInhoud::GaSub { sub_naam } => {
                format!("{}{}", regelnummer, sub_naam)
                    .trim_start()
                    .to_string()
            },
            LineInhoud::Help {} => format!("{}HELP", regelnummer)
                .trim_start()
                .to_string(),
            LineInhoud::Herhaal {} => format!("{}HERHAAL", regelnummer)
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
            LineInhoud::Met { stap_expressie, start_expressie, stop_expressie, variabele_naam } => {
                format!("{}MET {}, {} := {}, {}", regelnummer, stap_expressie, variabele_naam, start_expressie, stop_expressie)
                    .trim_start()
                    .to_string() },
            LineInhoud::Naar { sprong_doel } => format!("{}NAAR {}", regelnummer, sprong_doel.to_string())
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
            LineInhoud::Sub { sub_naam} =>
                format!("{}SUB {}", regelnummer, sub_naam)
                    .trim_start()
                    .to_string(),
            LineInhoud::Tekst { expressie } =>
                format!("{}TEKST := {}"
                        ,regelnummer
                        ,expressie)
                    .trim_start()
                    .to_string(),
            LineInhoud::Toekennen { variabele_naam, argument, expressie } =>
                if *argument == "" {
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum SprongDoel {
    Regel(u16),
    Stop,
}

impl SprongDoel {
    fn to_string(&self) -> String {
        match self {
            SprongDoel::Regel(regelnummer) => format!("{}", regelnummer),
            SprongDoel::Stop => "STOP".to_string(),
        }
    }

    pub(super) fn regelnummer(&self) -> Option<u16> {
        match self {
            SprongDoel::Regel(regelnummer) => Some(*regelnummer),
            SprongDoel::Stop => None,
        }
    }

    pub(super) fn vul(bron: &str) -> Result<Self, EcolFout> {
        let reply: SprongDoel;
        if geen_spaties(bron) == "STOP" {
            reply = SprongDoel::Stop;
        } else {
            let regelnummer = bron.trim().parse::<u16>().map_err(|_| "Ongeldig regelnummer-getal.".to_string())
                .map_err(|e| EcolFout::FoutMelding(e))?;
            if regelnummer <1 || regelnummer > 999 {
                return Err(EcolFout::FoutMelding("Regelnummer moet tussen 1 en 999 (inclusief) liggen.".to_string()));
            }
            reply = SprongDoel::Regel(regelnummer);

        }
        Ok(reply)
    }
}

#[derive(Debug, Clone)]
pub(super) enum LineInhoud {
    Als {vergelijking: String, dan: SprongDoel, anders: Option<SprongDoel>},
    End {},
    FunStart {variabele_naam: String, argumenten: String},
    FunEind {expressie: String },
    GaSub { sub_naam: String },
    Help {},
    Herhaal {},
    Klaar {},
    Met {
        variabele_naam: String,
        stap_expressie: String,
        start_expressie: String,
        stop_expressie: String,
    },
    LegeRegel {},
    Lijst {},
    Naar {sprong_doel: SprongDoel},
    NP {},
    NR {
        aantal: String
    },
    Rij {
        start: String,
        eind: String,
        variabele_naam: String,
    },
    Rijsym {
        start: String,
        eind: String,
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
        aantal: String,
    },
    Start {},
    Sub { sub_naam: String},
    Tekst {
        expressie: String,
    },
    Toekennen {
        variabele_naam: String,
        argument: String,
        expressie: String,
    },
    Verwijderen {},
}
impl LineInhoud {
    pub(super) fn from_sleutelwoord(sleutelwoord: Sleutelwoord) -> Result<Self, EcolFout> {
        match sleutelwoord {
            Sleutelwoord::END => Ok(Self::End {}),
            Sleutelwoord::HELP => Ok(Self::Help {}),
            Sleutelwoord::HERHAAL => Ok(Self::Herhaal {}),
            Sleutelwoord::KLAAR => Ok(Self::Klaar {}),
            Sleutelwoord::LIJST => Ok(Self::Lijst {}),
            Sleutelwoord::NP => Ok(Self::NP {}),
            Sleutelwoord::NR => Ok(Self::NR { aantal: "1".to_string() }),
            Sleutelwoord::START => Ok(Self::Start {}),
            _ => Err(EcolFout::FoutMelding("Incomplete syntax".to_string())),
        }
    }
    pub(super) fn as_str(&self) -> &str {
        match self {
            LineInhoud::Als { .. } => "Als",
            LineInhoud::End { } => "End",
            LineInhoud::FunStart { .. } => "FunStart",
            LineInhoud::FunEind { .. } => "FunEind",
            LineInhoud::GaSub { .. } => "GaSub",
            LineInhoud::Help { } => "Help",
            LineInhoud::Herhaal { } => "Herhaal",
            LineInhoud::Klaar { } => "Klaar",
            LineInhoud::LegeRegel { } => "",
            LineInhoud::Lijst { } => "Lijst",
            LineInhoud::Met { .. } => "Met",
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
            LineInhoud::Sub { .. } => "Sub",
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

    pub(super) fn bereken(&self, links: f32, rechts: f32) -> Result<f32, EcolFout> {
        match self {
            Self::Plus => Ok(links + rechts),
            Self::Min => Ok(links - rechts),
            Self::Vermenigvuldig => Ok(links * rechts),
            Self::Deel => {
                if rechts == 0.0 {
                    return Err(EcolFout::FoutMelding("Deel door 0 is niet toegestaan".to_string()));
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
    END,
    FUNstart,
    FUNeind,
    GASUB,
    HELP,
    HERHAAL,
    KLAAR,
    LIJST,
    MET,
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
    SUB,
    TEKST,
    TOEKENNEN,
}
impl Sleutelwoord {

    pub(super) fn from_string(input: &str) -> Option<Self> {
        match input {
            "ALS" => Some(Sleutelwoord::ALS),
            "END" => Some(Sleutelwoord::END),
            "HELP" => Some(Sleutelwoord::HELP),
            "HERHAAL" => Some(Sleutelwoord::HERHAAL),
            "KLAAR" => Some(Sleutelwoord::KLAAR),
            "LIJST" => Some(Sleutelwoord::LIJST),
            "MET" => Some(Sleutelwoord::MET),
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
            "SUB" => Some(Sleutelwoord::SUB),
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
            Sleutelwoord::END => "END",
            Sleutelwoord::FUNstart => "FUN",
            Sleutelwoord::FUNeind => "FUN",
            Sleutelwoord::GASUB => "GASUB",
            Sleutelwoord::HELP => "HELP",
            Sleutelwoord::HERHAAL => "HERHAAL",
            Sleutelwoord::KLAAR => "KLAAR",
            Sleutelwoord::LIJST => "LIJST",
            Sleutelwoord::MET => "MET",
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
            Sleutelwoord::SUB => "SUB",
            Sleutelwoord::TEKST => "TEKST",
            Sleutelwoord::TOEKENNEN => "TOEKENNEN",
        }
    }

}




