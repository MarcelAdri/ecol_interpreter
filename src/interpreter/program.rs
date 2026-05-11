//! AST-typen voor het opgeslagen ECOL-programma.
//!
//! Een ECOL-programma is een geordende reeks genummerde regels (1–999), opgeslagen
//! als `BTreeMap<u16, LineInhoud>`. Elke [`Line`] bevat een regelnummer en een
//! [`LineInhoud`]-variant die overeenkomt met één ECOL-opdracht. [`FunDef`] en
//! [`SubDef`] slaan geëxtraheerde functie- en subroutine-definities op.
use std::fmt;
use std::collections::BTreeMap;
use std::fmt::Display;
use crate::interpreter::errors::{EcolFout, EcolFoutVariant};
use crate::interpreter::helpers::{geen_spaties, vind_opmerking};

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

    pub(super) fn regelnummer(&self) -> u16 {
        self.regelnummer
    }
    pub(super) fn inhoud(&self) -> &LineInhoud {
        &self.inhoud
    }
}
impl Display for Line {
    #[allow(clippy::too_many_lines)]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let regelnummer:String =  if self.regelnummer() == 0 {
            String::new()
        } else {
            format!("{:>4} ", self.regelnummer())
        };
        #[allow(clippy::match_same_arms)]
        let regel_inhoud = match &self.inhoud() {
            LineInhoud::Als { vergelijking, dan, anders, opm } => {
                match anders {
                    Some( x ) => format!("{}ALS {} DAN {} ANDERS {}{}"
                                         ,regelnummer
                                         ,vergelijking
                                         ,dan
                                         ,x
                                         ,if opm.is_some() { format!(" ; {}", opm.as_ref().unwrap()) } else { String::new() })
                        .trim_start()
                        .to_string(),
                    None => format!("{}ALS {} DAN {}{}"
                                    ,regelnummer
                                    ,vergelijking
                                    ,dan
                                    ,if opm.is_some() { format!(" ; {}", opm.as_ref().unwrap()) } else { String::new() })
                        .trim_start()
                        .to_string(),
                }
            }
            LineInhoud::End { opm } => {
                format!("{}END{}"
                        ,regelnummer
                        ,if opm.is_some() { format!(" ; {}", opm.as_ref().unwrap()) } else { String::new() })
                    .trim_start()
                    .to_string()
            }
            LineInhoud::FunStart { variabele_naam, argumenten, opm } => {
                format!("{}FUN {}({}){}"
                        ,regelnummer
                        ,variabele_naam
                        ,argumenten
                        ,if opm.is_some() { format!(" ; {}", opm.as_ref().unwrap()) } else { String::new() })
                    .trim_start()
                    .to_string()
            },
            LineInhoud::FunEind { expressie, opm } => {
                format!("{}FUN := {}{}"
                        ,regelnummer
                        ,expressie
                        ,if opm.is_some() { format!(" ; {}", opm.as_ref().unwrap()) } else { String::new() })
                    .trim_start()
                    .to_string()
            },
            LineInhoud::GaSub { sub_naam, opm } => {
                format!("{}{}{}"
                        ,regelnummer
                        ,sub_naam
                        ,if opm.is_some() { format!(" ; {}", opm.as_ref().unwrap()) } else { String::new() })
                    .trim_start()
                    .to_string()
            },
            LineInhoud::Bewaar {} => format!("{regelnummer}BEWAAR")
                .trim_start()
                .to_string(),
            LineInhoud::Help {} => format!("{regelnummer}HELP")
                .trim_start()
                .to_string(),
            LineInhoud::Herhaal { opm } => format!("{}HERHAAL{}"
                                                   ,regelnummer
                                                   ,if opm.is_some() { format!(" ; {}", opm.as_ref().unwrap()) } else { String::new() })
                .trim_start()
                .to_string(),
            LineInhoud::Laad {} => format!("{regelnummer}LAAD")
                .trim_start()
                .to_string(),
            LineInhoud::Klaar { opm } => format!("{}KLAAR{}"
                                                 ,regelnummer
                                                 ,if opm.is_some() { format!(" ; {}", opm.as_ref().unwrap()) } else { String::new() })
                .trim_start()
                .to_string(),
            LineInhoud::LegeRegel { opm } => format!("{}:{}"
                                                     ,regelnummer
                                                     ,if opm.is_some() { format!(" ; {}", opm.as_ref().unwrap()) } else { String::new() })
                .trim_start()
                .to_string(),
            LineInhoud::Lijst {} => format!("{regelnummer}LIJST")
                .trim_start()
                .to_string(),
            LineInhoud::Met { stap_expressie, start_expressie, stop_expressie, variabele_naam, opm } => {
                format!("{}MET {}, {} := {}, {}{}"
                        ,regelnummer
                        ,stap_expressie
                        ,variabele_naam
                        ,start_expressie
                        ,stop_expressie
                        ,if opm.is_some() { format!(" ; {}", opm.as_ref().unwrap()) } else { String::new() })
                    .trim_start()
                    .to_string() },
            LineInhoud::Naar { sprong_doel, opm } => format!("{}NAAR {}{}"
                                                             ,regelnummer
                                                             ,sprong_doel
                                                             ,if opm.is_some() { format!(" ; {}", opm.as_ref().unwrap()) } else { String::new() })
                .trim_start()
                .to_string(),
            LineInhoud::NP { opm } => format!("{}NP{}"
                                              ,regelnummer
                                              ,if opm.is_some() { format!(" ; {}", opm.as_ref().unwrap()) } else { String::new() })
                .trim_start()
                .to_string(),
            LineInhoud::NR { aantal, opm } =>
                format!("{}NR({}){}"
                        ,regelnummer
                        ,aantal
                        ,if opm.is_some() { format!(" ; {}", opm.as_ref().unwrap()) } else { String::new() })
                    .trim_start()
                    .to_string(),
            LineInhoud::Rij { start, eind, variabele_naam, opm } =>
                format!("{}RIJ({}, {}) {}{}"
                        ,regelnummer
                        ,start
                        ,eind
                        ,variabele_naam
                        ,if opm.is_some() { format!(" ; {}", opm.as_ref().unwrap()) } else { String::new() })
                    .trim_start()
                    .to_string(),
            LineInhoud::Rijsym { start, eind, variabele_naam, opm } =>
                format!("{}RIJSYM({}, {}) {}{}"
                        ,regelnummer
                        ,start
                        ,eind
                        ,variabele_naam
                        ,if opm.is_some() { format!(" ; {}", opm.as_ref().unwrap()) } else { String::new() })
                    .trim_start()
                    .to_string(),
            LineInhoud::Schrijf { breedte, decimalen, expressie, opm } =>
                format!("{}SCHRIJF({}, {}) := {}{}"
                        ,regelnummer
                        ,breedte
                        ,decimalen
                        ,expressie
                        ,if opm.is_some() { format!(" ; {}", opm.as_ref().unwrap()) } else { String::new() })
                    .trim_start()
                    .to_string(),
            LineInhoud::Schrijfsym { expressie, opm } =>
                format!("{}SCHRIJFSYM := {}{}"
                        ,regelnummer
                        ,expressie
                        ,if opm.is_some() { format!(" ; {}", opm.as_ref().unwrap()) } else { String::new() })
                    .trim_start()
                    .to_string(),
            LineInhoud::Schrijm { expressie, opm } =>
                format!("{}SCHRIJM := {}{}"
                        ,regelnummer
                        ,expressie
                        ,if opm.is_some() { format!(" ; {}", opm.as_ref().unwrap()) } else { String::new() })
                    .trim_start()
                    .to_string(),
            LineInhoud::Spatie { aantal, opm } =>
                format!("{}SPATIE({}){}"
                        ,regelnummer
                        ,aantal
                        ,if opm.is_some() { format!(" ; {}", opm.as_ref().unwrap()) } else { String::new() })
                    .trim_start()
                    .to_string(),
            LineInhoud::Start { } => String::new(),
            LineInhoud::Sub { sub_naam, opm} =>
                format!("{}SUB {}{}"
                        ,regelnummer
                        ,sub_naam
                        ,if opm.is_some() { format!(" ; {}", opm.as_ref().unwrap()) } else { String::new() })
                    .trim_start()
                    .to_string(),
            LineInhoud::Tekst { expressie, opm } =>
                format!("{}TEKST := {}{}"
                        ,regelnummer
                        ,expressie
                        ,if opm.is_some() { format!(" ; {}", opm.as_ref().unwrap()) } else { String::new() })
                    .trim_start()
                    .to_string(),
            LineInhoud::Toekennen { variabele_naam, argument, expressie,opm } =>
                if argument.is_empty() {
                    format!("{}{} := {}{}"
                            ,regelnummer
                            ,variabele_naam
                            ,expressie
                            ,if opm.is_some() { format!(" ; {}", opm.as_ref().unwrap()) } else { String::new() })
                        .trim_start()
                        .to_string()
                } else {
                    format!("{regelnummer}{variabele_naam}({argument}) := {expressie}")
                        .trim_start()
                        .to_string()
                },

            LineInhoud::Verwijderen { } => String::new(),
        };
        write!(f, "{regel_inhoud}")
    }
}
/// Het doel van een `NAAR`-, `DAN`- of `ANDERS`-sprong: een regelnummer of `STOP`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum SprongDoel {
    Regel(u16),
    Stop,
}

impl SprongDoel {
    pub(super) fn regelnummer(self) -> Option<u16> {
        match self {
            SprongDoel::Regel(regelnummer) => Some(regelnummer),
            SprongDoel::Stop => None,
        }
    }

    pub(super) fn vul(bron: &str) -> Result<Self, EcolFout> {
        let reply: SprongDoel = if geen_spaties(bron) == "STOP" {
            SprongDoel::Stop
        } else {
            let regelnummer = bron.trim().parse::<u16>().map_err(|_| EcolFout::melding(EcolFoutVariant::OngeldigGetal("als regelnummer".to_string())))?;
            if !(1..=999).contains(&regelnummer) {
                return Err(EcolFout::melding(EcolFoutVariant::OngeldigAantal("regelnummer".to_string(), 1f32, 999f32, f32::from(regelnummer))));
            }
            SprongDoel::Regel(regelnummer)

        };
        Ok(reply)
    }
}
impl Display for SprongDoel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SprongDoel::Regel(n) => write!(f, "{n}"),
            SprongDoel::Stop => write!(f, "STOP"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
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
        match sleutelwoord {
            Sleutelwoord::BEWAAR => { vind_opmerking(input)?; Ok(Self::Bewaar {}) },
            Sleutelwoord::END => Ok(Self::End { opm: vind_opmerking(input)? }),
            Sleutelwoord::HELP => { vind_opmerking(input)?; Ok(Self::Help {}) },
            Sleutelwoord::HERHAAL => Ok(Self::Herhaal { opm: vind_opmerking(input)? }),
            Sleutelwoord::KLAAR => Ok(Self::Klaar { opm: vind_opmerking(input)? }),
            Sleutelwoord::LAAD => { vind_opmerking(input)?; Ok(Self::Laad {}) },
            Sleutelwoord::LIJST => { vind_opmerking(input)?; Ok(Self::Lijst {}) },
            Sleutelwoord::NP => Ok(Self::NP { opm: vind_opmerking(input)? }),
            Sleutelwoord::NR => Ok(Self::NR { aantal: "1".to_string(), opm: vind_opmerking(input)? }),
            Sleutelwoord::START => { vind_opmerking(input)?; Ok(Self::Start {}) },
            _ => Err(EcolFout::melding(EcolFoutVariant::IncompleteSyntax)),
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
    pub(super) fn is_operator_char(c: char) -> bool {
        Self::from_char(c).is_some()
    }
    pub(super) fn is_operator_string(s: &str) -> bool {
        s.chars().next().is_some_and(|c| Self::from_char(c).is_some())
    }
    pub(super) fn operator_prioriteiten() -> Vec<Vec<Self>> {
        vec![
            vec![Self::Machtsverheffen],
            vec![Self::Vermenigvuldig, Self::Deel],
            vec![Self::Plus, Self::Min],
        ]
    }

    pub(super) fn bereken(self, links: f32, rechts: f32) -> Result<f32, EcolFout> {
        match self {
            Self::Plus => plus(links, rechts),
            Self::Min => min(links, rechts),
            Self::Vermenigvuldig => maal(links, rechts),
            Self::Deel => deel(links, rechts),
            Self::Machtsverheffen => macht(links, rechts),

        }
    }
}

impl Display for Operator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Plus => write!(f, "+"),
            Self::Min => write!(f, "-"),
            Self::Vermenigvuldig => write!(f, "*"),
            Self::Deel => write!(f, "/"),
            Self::Machtsverheffen => write!(f, "M"),
        }

    }
}
fn plus(links: f32, rechts: f32) -> Result<f32, EcolFout> {
    let resultaat = links + rechts;
    if resultaat.is_infinite() || resultaat.is_nan() {
        return Err(EcolFout::melding(EcolFoutVariant::GetalBuitenGrens(format!("{links} + {rechts}"))));
    }
    Ok(resultaat)
}
fn min(links: f32, rechts: f32) -> Result<f32, EcolFout> {
    let resultaat = links - rechts;
    if resultaat.is_infinite() || resultaat.is_nan() {
        return Err(EcolFout::melding(EcolFoutVariant::GetalBuitenGrens(format!("{links} - {rechts}"))));
    }
    Ok(resultaat)
}
fn maal(links: f32, rechts: f32) -> Result<f32, EcolFout> {
    let resultaat = links * rechts;
    if resultaat.is_infinite() || resultaat.is_nan() {
        return Err(EcolFout::melding(EcolFoutVariant::GetalBuitenGrens(format!("{links} * {rechts}"))));
    }
    Ok(resultaat)
}
fn deel(links: f32, rechts: f32) -> Result<f32, EcolFout> {
    if rechts == 0.0 {
        return Err(EcolFout:: melding(EcolFoutVariant::DelenDoorNul));
    }
    let resultaat = links / rechts;
    if resultaat.is_infinite() || resultaat.is_nan() {
        return Err(EcolFout::melding(EcolFoutVariant::GetalBuitenGrens(format!("{links} / {rechts}"))));
    }
    Ok(resultaat)
}
fn macht(links: f32, rechts: f32) -> Result<f32, EcolFout> {
    if links < 0.0 && rechts.fract() != 0.0 {
        return Err(EcolFout::melding(EcolFoutVariant::MachtsverheffenOngeldig(format!("{links} M {rechts}"))));
    }
    if rechts.abs() > 1_000_000.0 {
        return Err(EcolFout::melding(EcolFoutVariant::MachtsverheffenExponentTeGroot(rechts, format!("{links} M {rechts}"))));
    }
    let resultaat: f32 = if rechts.fract() == 0.0 {
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        links.powi(rechts as i32) //Veilig, want hierboven gevalideerd als geheel getal
    } else {
        links.powf(rechts)
    };
    if resultaat.is_infinite() || resultaat.is_nan() {
        return Err(EcolFout::melding(EcolFoutVariant::GetalBuitenGrens(format!("{links} M {rechts}"))));
    }
    Ok(resultaat)
}
#[allow(clippy::upper_case_acronyms)]
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
}
/// Definitie van een gebruikersfunctie (`FUN naam(param, …) … FUN := expressie`).
///
/// Parameters zijn de formele parameternamen als strings; `body` is het
/// deelprogramma tussen `FUN naam(…)` en `FUN :=`, geïndexeerd op regelnummer.
/// De definitie wordt vóór uitvoering geëxtraheerd uit het hoofdprogramma.
#[derive(Debug, Clone)]
pub(super) struct FunDef {
    parameters: Vec<String>,
    body: BTreeMap<u16, LineInhoud>,
}
impl FunDef {
    pub(super) fn new() -> Self {
        FunDef { parameters: Vec::new(), body: BTreeMap::new() }
    }

    pub(super) fn parameters(&self) -> &Vec<String> {
        &self.parameters
    }
    pub(super) fn set_parameters(&mut self, parameters: &[String]) {
         parameters.clone_into(&mut self.parameters);
    }
    pub(super) fn body(&self) -> &BTreeMap<u16, LineInhoud> {
        &self.body
    }
    pub(super) fn body_insert(&mut self, regelnummer: u16, regel: &LineInhoud) {
        self.body.insert(regelnummer, regel.clone());
    }
}
/// Definitie van een subroutine (`SUB naam … END`).
///
/// `regels` bevat alle regels van de subroutine inclusief de afsluitende `END`.
/// Net als [`FunDef`] wordt de definitie vóór uitvoering geëxtraheerd.
#[derive(Debug, Clone)]
pub(super) struct SubDef {
    regels: BTreeMap<u16, LineInhoud>
}
impl SubDef {
    pub(super) fn new() -> Self {
        Self {
            regels: BTreeMap::new(),
        }
    }
    pub(super) fn regels(&self) -> &BTreeMap<u16, LineInhoud> {
        &self.regels
    }
    pub(super) fn regels_insert(&mut self, regelnummer: u16, regel: &LineInhoud) {
        self.regels.insert(regelnummer, regel.clone());
    }
    pub(crate) fn clone(&self) -> SubDef {
        let mut nieuw = SubDef::new();
        for (regelnummer, regel_inhoud) in &self.regels {
            nieuw.regels.insert(*regelnummer, regel_inhoud.clone());
        }
        nieuw
    }
}




