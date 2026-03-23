#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Sleutelwoord {
    HELP,
    NR,
    SCHRIJF,
    TEKST,
    TOEKENNEN,
}

impl Sleutelwoord {

    pub(super) fn from_string(input: &str) -> Option<Self> {
        match input {
            "HELP" => Some(Sleutelwoord::HELP),
            "NR" => Some(Sleutelwoord::NR),
            "SCHRIJF" => Some(Sleutelwoord::SCHRIJF),
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
            Sleutelwoord::NR => "NR",
            Sleutelwoord::SCHRIJF => "SCHRIJF",
            Sleutelwoord::TEKST => "TEKST",
            Sleutelwoord::TOEKENNEN => "TOEKENNEN",
        }
    }

}




pub(super) const WORDT_TEKEN: &str = ":=";

pub(super) struct Line {
    pub(super) regelnummer: usize,
    pub(super) inhoud: LineInhoud,
}

pub(super) enum LineInhoud {
    Help {

    },
    NR {

    },
    Tekst {
        expressie: String,
    },
    Schrijf {
        breedte: usize,
        decimalen: usize,
        expressie: String,
    },
    Toekennen {
        variabele_naam: String,
        expressie: String,
    },

}

impl LineInhoud {
    pub(super) fn from_sleutelwoord(sleutelwoord: Sleutelwoord) -> Self {
        match sleutelwoord {
            Sleutelwoord::HELP => Self::Help {},
            Sleutelwoord::NR => Self::NR {},
            Sleutelwoord::TEKST => Self::Tekst { expressie: String::new() },
            Sleutelwoord::SCHRIJF => Self::Schrijf { breedte: 0, decimalen: 0, expressie: String::new() },
            Sleutelwoord::TOEKENNEN => Self::Toekennen { variabele_naam: String::new(), expressie: String::new() },
        }
    }
}

impl Line {
    pub(super) fn new(
        regelnummer: usize,
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
            LineInhoud::NR {} => Some(Sleutelwoord::NR),
            LineInhoud::Tekst { .. } => Some(Sleutelwoord::TEKST),
            LineInhoud::Schrijf { .. } => Some(Sleutelwoord::SCHRIJF),
            LineInhoud::Toekennen { .. } => Some(Sleutelwoord::TOEKENNEN),
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
        [Self::Vermenigvuldig, Self::Deel, Self::Plus, Self::Min].into_iter().copied()

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Functie {
    LinksString,
    RechtsString,
    MiddenString,
    INT,
}

impl Functie {
    pub(super) fn from_str(s: &str) -> Option<Self> {
        match s {
            "LINKS$" => Some(Self::LinksString),
            "RECHTS$" => Some(Self::RechtsString),
            "MIDDEN$" => Some(Self::MiddenString),
            "INT" => Some(Self::INT),
            _ => None,
        }
    }

    pub(super) fn as_str(self) -> &'static str {
        match self {
            Self::LinksString => "LINKS$",
            Self::RechtsString => "RECHTS$",
            Self::MiddenString => "MIDDEN$",
            Self::INT => "INT",
        }
    }
    

    pub(super) fn is_functie(s: &str) -> bool {
        Self::from_str(s).is_some()
    }
    

    pub(super) fn is_string_functie (s: &str) -> bool {
        Self::is_functie(s) && s.ends_with('$')
    }
    
    
    pub(super) fn is_nummer_functie (s: &str) -> bool {
        Self::is_functie(s) && !s.ends_with('$')
    }
}