use std::collections::BTreeMap;
use crate::interpreter::functions::EcolFout;
use crate::interpreter::helpers::{extract_opmerking, geen_spaties, vind_opmerking};

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
            LineInhoud::Bewaar { } => Some(Sleutelwoord::BEWAAR),
            LineInhoud::End { .. } => Some(Sleutelwoord::END),
            LineInhoud::FunStart { .. } => Some(Sleutelwoord::FUNstart),
            LineInhoud::FunEind { .. } => Some(Sleutelwoord::FUNeind),
            LineInhoud::GaSub { .. } => Some(Sleutelwoord::GASUB),
            LineInhoud::Help {} => Some(Sleutelwoord::HELP),
            LineInhoud::Herhaal { .. } => Some(Sleutelwoord::HERHAAL),
            LineInhoud::Klaar { .. } => Some(Sleutelwoord::KLAAR),
            LineInhoud::Laad { } => Some(Sleutelwoord::LAAD),
            LineInhoud::LegeRegel { .. } => None,
            LineInhoud::Lijst {} => Some(Sleutelwoord::LIJST),
            LineInhoud::Met { .. } => Some(Sleutelwoord::MET),
            LineInhoud::Naar { .. } => Some(Sleutelwoord::NAAR),
            LineInhoud::NP { .. } => Some(Sleutelwoord::NP),
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
            LineInhoud::Als { vergelijking, dan, anders, opm } => {
                match anders {
                    Some( x ) => format!("{}ALS {} DAN {} ANDERS {}{}"
                                         ,regelnummer
                                         ,vergelijking
                                         ,dan.to_string()
                                         ,x.to_string()
                                         ,if opm.is_some() { format!(" ; {}", opm.as_ref().unwrap()) } else { "".to_string() })
                        .trim_start()
                        .to_string(),
                    None => format!("{}ALS {} DAN {}{}"
                                    ,regelnummer
                                    ,vergelijking
                                    ,dan.to_string()
                                    ,if opm.is_some() { format!(" ; {}", opm.as_ref().unwrap()) } else { "".to_string() })
                        .trim_start()
                        .to_string(),
                }
            }
            LineInhoud::End { opm } => {
                format!("{}END{}"
                        ,regelnummer
                        ,if opm.is_some() { format!(" ; {}", opm.as_ref().unwrap()) } else { "".to_string() })
                    .trim_start()
                    .to_string()
            }
            LineInhoud::FunStart { variabele_naam, argumenten, opm } => {
                format!("{}FUN {}({}){}"
                        ,regelnummer
                        ,variabele_naam
                        ,argumenten
                        ,if opm.is_some() { format!(" ; {}", opm.as_ref().unwrap()) } else { "".to_string() })
                    .trim_start()
                    .to_string()
            },
            LineInhoud::FunEind { expressie, opm } => {
                format!("{}FUN := {}{}"
                        ,regelnummer
                        ,expressie
                        ,if opm.is_some() { format!(" ; {}", opm.as_ref().unwrap()) } else { "".to_string() })
                    .trim_start()
                    .to_string()
            },
            LineInhoud::GaSub { sub_naam, opm } => {
                format!("{}{}{}"
                        ,regelnummer
                        ,sub_naam
                        ,if opm.is_some() { format!(" ; {}", opm.as_ref().unwrap()) } else { "".to_string() })
                    .trim_start()
                    .to_string()
            },
            LineInhoud::Bewaar {} => format!("{}BEWAAR", regelnummer)
                .trim_start()
                .to_string(),
            LineInhoud::Help {} => format!("{}HELP", regelnummer)
                .trim_start()
                .to_string(),
            LineInhoud::Herhaal { opm } => format!("{}HERHAAL{}"
                                                   ,regelnummer
                                                   ,if opm.is_some() { format!(" ; {}", opm.as_ref().unwrap()) } else { "".to_string() })
                .trim_start()
                .to_string(),
            LineInhoud::Laad {} => format!("{}LAAD", regelnummer)
                .trim_start()
                .to_string(),
            LineInhoud::Klaar { opm } => format!("{}KLAAR{}"
                                                 ,regelnummer
                                                 ,if opm.is_some() { format!(" ; {}", opm.as_ref().unwrap()) } else { "".to_string() })
                .trim_start()
                .to_string(),
            LineInhoud::LegeRegel { opm } => format!("{}:{}"
                                                     ,regelnummer
                                                     ,if opm.is_some() { format!(" ; {}", opm.as_ref().unwrap()) } else { "".to_string() })
                .trim_start()
                .to_string(),
            LineInhoud::Lijst {} => format!("{}LIJST", regelnummer)
                .trim_start()
                .to_string(),
            LineInhoud::Met { stap_expressie, start_expressie, stop_expressie, variabele_naam, opm } => {
                format!("{}MET {}, {} := {}, {}{}"
                        ,regelnummer
                        ,stap_expressie
                        ,variabele_naam
                        ,start_expressie
                        ,stop_expressie
                        ,if opm.is_some() { format!(" ; {}", opm.as_ref().unwrap()) } else { "".to_string() })
                    .trim_start()
                    .to_string() },
            LineInhoud::Naar { sprong_doel, opm } => format!("{}NAAR {}{}"
                                                             ,regelnummer
                                                             ,sprong_doel.to_string()
                                                             ,if opm.is_some() { format!(" ; {}", opm.as_ref().unwrap()) } else { "".to_string() })
                .trim_start()
                .to_string(),
            LineInhoud::NP { opm } => format!("{}NP{}"
                                              ,regelnummer
                                              ,if opm.is_some() { format!(" ; {}", opm.as_ref().unwrap()) } else { "".to_string() })
                .trim_start()
                .to_string(),
            LineInhoud::NR { aantal, opm } =>
                format!("{}NR({}){}"
                        ,regelnummer
                        ,aantal
                        ,if opm.is_some() { format!(" ; {}", opm.as_ref().unwrap()) } else { "".to_string() })
                .trim_start()
                .to_string(),
            LineInhoud::Rij { start, eind, variabele_naam, opm } =>
                format!("{}RIJ({}, {}) {}{}"
                        ,regelnummer
                        ,start
                        ,eind
                        ,variabele_naam
                        ,if opm.is_some() { format!(" ; {}", opm.as_ref().unwrap()) } else { "".to_string() })
                .trim_start()
                .to_string(),
            LineInhoud::Rijsym { start, eind, variabele_naam, opm } =>
                format!("{}RIJSYM({}, {}) {}{}"
                        ,regelnummer
                        ,start
                        ,eind
                        ,variabele_naam
                        ,if opm.is_some() { format!(" ; {}", opm.as_ref().unwrap()) } else { "".to_string() })
                .trim_start()
                .to_string(),
            LineInhoud::Schrijf { breedte, decimalen, expressie, opm } =>
                format!("{}SCHRIJF({}, {}) := {}{}"
                        ,regelnummer
                        ,breedte
                        ,decimalen
                        ,expressie
                        ,if opm.is_some() { format!(" ; {}", opm.as_ref().unwrap()) } else { "".to_string() })
                    .trim_start()
                    .to_string(),
            LineInhoud::Schrijfsym { expressie, opm } =>
                format!("{}SCHRIJFSYM := {}{}"
                        ,regelnummer
                        ,expressie
                        ,if opm.is_some() { format!(" ; {}", opm.as_ref().unwrap()) } else { "".to_string() })
                    .trim_start()
                    .to_string(),
            LineInhoud::Schrijm { expressie, opm } =>
                format!("{}SCHRIJM := {}{}"
                        ,regelnummer
                        ,expressie
                        ,if opm.is_some() { format!(" ; {}", opm.as_ref().unwrap()) } else { "".to_string() })
                    .trim_start()
                    .to_string(),
            LineInhoud::Spatie { aantal, opm } =>
                format!("{}SPATIE({}){}"
                        ,regelnummer
                        ,aantal
                        ,if opm.is_some() { format!(" ; {}", opm.as_ref().unwrap()) } else { "".to_string() })
                    .trim_start()
                    .to_string(),
            LineInhoud::Start { } => "".to_string(),
            LineInhoud::Sub { sub_naam, opm} =>
                format!("{}SUB {}{}"
                        ,regelnummer
                        ,sub_naam
                        ,if opm.is_some() { format!(" ; {}", opm.as_ref().unwrap()) } else { "".to_string() })
                    .trim_start()
                    .to_string(),
            LineInhoud::Tekst { expressie, opm } =>
                format!("{}TEKST := {}{}"
                        ,regelnummer
                        ,expressie
                        ,if opm.is_some() { format!(" ; {}", opm.as_ref().unwrap()) } else { "".to_string() })
                    .trim_start()
                    .to_string(),
            LineInhoud::Toekennen { variabele_naam, argument, expressie,opm } =>
                if *argument == "" {
                    format!("{}{} := {}{}"
                            ,regelnummer
                            ,variabele_naam
                            ,expressie
                            ,if opm.is_some() { format!(" ; {}", opm.as_ref().unwrap()) } else { "".to_string() })
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
    Als {vergelijking: String, dan: SprongDoel, anders: Option<SprongDoel>, opm: Option<String>},
    Bewaar {},
    End { opm: Option<String>},
    FunStart {variabele_naam: String, argumenten: String, opm: Option<String>},
    FunEind {expressie: String, opm: Option<String> },
    GaSub { sub_naam: String, opm: Option<String> },
    Help {},
    Herhaal { opm: Option<String>},
    Klaar { opm: Option<String>},
    Laad {},
    Met {
        variabele_naam: String,
        stap_expressie: String,
        start_expressie: String,
        stop_expressie: String,
        opm: Option<String>,
    },
    LegeRegel {opm: Option<String>},
    Lijst {},
    Naar {sprong_doel: SprongDoel, opm: Option<String>},
    NP {opm: Option<String>},
    NR {
        aantal: String,
        opm: Option<String>,
    },
    Rij {
        start: String,
        eind: String,
        variabele_naam: String,
        opm: Option<String>,
    },
    Rijsym {
        start: String,
        eind: String,
        variabele_naam: String,
        opm: Option<String>,
    },
    Schrijf {
        breedte: usize,
        decimalen: usize,
        expressie: String,
        opm: Option<String>,
    },
    Schrijfsym {
        expressie: String,
        opm: Option<String>,
    },
    Schrijm {
        expressie: String,
        opm: Option<String>,
    },
    Spatie {
        aantal: String,
        opm: Option<String>,
    },
    Start {},
    Sub { sub_naam: String, opm: Option<String>},
    Tekst {
        expressie: String,
        opm: Option<String>,
    },
    Toekennen {
        variabele_naam: String,
        argument: String,
        expressie: String,
        opm: Option<String>,
    },
    Verwijderen {},
}
impl LineInhoud {
    pub(super) fn from_sleutelwoord(sleutelwoord: Sleutelwoord, input: &str) -> Result<Self, EcolFout> {
        let (_, opm) = extract_opmerking(input);

        match sleutelwoord {
            Sleutelwoord::BEWAAR => Ok(Self::Bewaar {}),
            Sleutelwoord::END => Ok(Self::End { opm: vind_opmerking(&opm)? }),
            Sleutelwoord::HELP => Ok(Self::Help { }),
            Sleutelwoord::HERHAAL => Ok(Self::Herhaal { opm: vind_opmerking(&opm)? }),
            Sleutelwoord::KLAAR => Ok(Self::Klaar { opm: vind_opmerking(&opm)? }),
            Sleutelwoord::LAAD => Ok(Self::Laad {}),
            Sleutelwoord::LIJST => Ok(Self::Lijst {}),
            Sleutelwoord::NP => Ok(Self::NP { opm: vind_opmerking(&opm)?}),
            Sleutelwoord::NR => Ok(Self::NR { aantal: "1".to_string(), opm: vind_opmerking(&opm)? }),
            Sleutelwoord::START => Ok(Self::Start {}),
            _ => Err(EcolFout::FoutMelding("Incomplete syntax".to_string())),
        }
    }
    pub(super) fn as_str(&self) -> &str {
        match self {
            LineInhoud::Als { .. } => "Als",
            LineInhoud::Bewaar { } => "Bewaar",
            LineInhoud::End { .. } => "End",
            LineInhoud::FunStart { .. } => "FunStart",
            LineInhoud::FunEind { .. } => "FunEind",
            LineInhoud::GaSub { .. } => "GaSub",
            LineInhoud::Help { } => "Help",
            LineInhoud::Herhaal { .. } => "Herhaal",
            LineInhoud::Klaar { .. } => "Klaar",
            LineInhoud::Laad { } => "Laad",
            LineInhoud::LegeRegel { .. } => "",
            LineInhoud::Lijst { } => "Lijst",
            LineInhoud::Met { .. } => "Met",
            LineInhoud::Naar { .. } => "Naar",
            LineInhoud::NP { .. } => "NP",
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
    Machtsverheffen,
}
impl Operator {
    pub(super) fn from_char(c: char) -> Option<Self> {
        match c {
            '+' => Some(Self::Plus),
            '-' => Some(Self::Min),
            '*' => Some(Self::Vermenigvuldig),
            '/' => Some(Self::Deel),
            'M' => Some(Self::Machtsverheffen),
            _ => None,
        }
    }

    pub(super) fn to_char(&self) -> char {
        match self {
            Self::Plus => '+',
            Self::Min => '-',
            Self::Vermenigvuldig => '*',
            Self::Deel => '/',
            Self::Machtsverheffen => 'M',
        }
    }

    fn from_string(s: &str) -> Option<Self> {
        match s {
            "+" => Some(Self::Plus),
            "-" => Some(Self::Min),
            "*" => Some(Self::Vermenigvuldig),
            "/" => Some(Self::Deel),
            "M" => Some(Self::Machtsverheffen),
            _ => None,
        }
    }

    pub(super) fn is_operator_char(c: char) -> bool {
        Self::from_char(c).is_some()
    }
    pub(super) fn is_operator_string(s: &str) -> bool {
        Self::from_string(s).is_some()
    }

    pub(super) fn operator_volgorde() -> impl Iterator<Item = Self> {
        IntoIterator::into_iter([
            Self::Machtsverheffen,
            Self::Vermenigvuldig,
            Self::Deel,
            Self::Plus,
            Self::Min,
        ])
    }
    pub(super) fn operator_prioriteiten() -> Vec<Vec<Self>> {
        vec![
            vec![Self::Machtsverheffen],
            vec![Self::Vermenigvuldig, Self::Deel],
            vec![Self::Plus, Self::Min],
        ]
    }

    pub(super) fn bereken(&self, links: f32, rechts: f32) -> Result<f32, EcolFout> {
        match self {
            Self::Plus => plus(links, rechts),
            Self::Min => min(links, rechts),
            Self::Vermenigvuldig => maal(links, rechts),
            Self::Deel => deel(links, rechts),
            Self::Machtsverheffen => macht(links, rechts),

        }
    }
}
fn plus(links: f32, rechts: f32) -> Result<f32, EcolFout> {
    let resultaat = links + rechts;
    if resultaat.is_infinite() || resultaat.is_nan() {
        return Err(EcolFout::FoutMelding(format!("Het resultaat van de berekening {} + {} ligt buiten de grenzen van een variabele in ECOL.", links, rechts)));
    }
    Ok(resultaat)
}
fn min(links: f32, rechts: f32) -> Result<f32, EcolFout> {
    let resultaat = links - rechts;
    if resultaat.is_infinite() || resultaat.is_nan() {
        return Err(EcolFout::FoutMelding(format!("Het resultaat van de berekening {} - {} ligt buiten de grenzen van een variabele in ECOL.", links, rechts)));
    }
    Ok(resultaat)
}
fn maal(links: f32, rechts: f32) -> Result<f32, EcolFout> {
    let resultaat = links * rechts;
    if resultaat.is_infinite() || resultaat.is_nan() {
        return Err(EcolFout::FoutMelding(format!("Het resultaat van de berekening {} * {} ligt buiten de grenzen van een variabele in ECOL.", links, rechts)));
    }
    Ok(resultaat)
}
fn deel(links: f32, rechts: f32) -> Result<f32, EcolFout> {
    if rechts == 0.0 {
        return Err(EcolFout::FoutMelding("Delen door nul is niet toegestaan in ECOL.".to_string()));
    }
    let resultaat = links / rechts;
    if resultaat.is_infinite() || resultaat.is_nan() {
        return Err(EcolFout::FoutMelding(format!("Het resultaat van de berekening {} / {} ligt buiten de grenzen van een variabele in ECOL.", links, rechts)));
    }
    Ok(resultaat)
}
fn macht(links: f32, rechts: f32) -> Result<f32, EcolFout> {
    if links < 0.0 && rechts.fract() != 0.0 {
        return Err(EcolFout::FoutMelding(format!("Machtsverheffen met een negatieve basis en een niet-gehele exponent is niet toegestaan in ECOL ({} M {}).", links, rechts)));
    }
    if rechts.abs() > 1000000.0 {
        return Err(EcolFout::FoutMelding(format!("Exponent {} is te groot voor machtsverheffen in ECOL ({} M {}).", rechts, links, rechts)));
    }
    let resultaat: f32;
    if rechts.fract() == 0.0 {
        resultaat = links.powi(rechts as i32)
    } else {
        resultaat = links.powf(rechts);
    }
    if resultaat.is_infinite() || resultaat.is_nan() {
        return Err(EcolFout::FoutMelding(format!("Het resultaat van de berekening {} M {} ligt buiten de grenzen van een variabele in ECOL.", links, rechts)));
    }
    Ok(resultaat)
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
    BEWAAR,
    END,
    FUNstart,
    FUNeind,
    GASUB,
    HELP,
    HERHAAL,
    KLAAR,
    LAAD,
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
            "BEWAAR" => Some(Sleutelwoord::BEWAAR),
            "END" => Some(Sleutelwoord::END),
            "HELP" => Some(Sleutelwoord::HELP),
            "HERHAAL" => Some(Sleutelwoord::HERHAAL),
            "KLAAR" => Some(Sleutelwoord::KLAAR),
            "LAAD" => Some(Sleutelwoord::LAAD),
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
            Sleutelwoord::BEWAAR => "BEWAAR",
            Sleutelwoord::END => "END",
            Sleutelwoord::FUNstart => "FUN",
            Sleutelwoord::FUNeind => "FUN",
            Sleutelwoord::GASUB => "GASUB",
            Sleutelwoord::HELP => "HELP",
            Sleutelwoord::HERHAAL => "HERHAAL",
            Sleutelwoord::KLAAR => "KLAAR",
            Sleutelwoord::LAAD => "LAAD",
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




