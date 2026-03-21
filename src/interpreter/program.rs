#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Sleutelwoord {
    HELP,
    NR,
    TEKST,
    TOEKENNEN,
}

impl Sleutelwoord {

    pub(super) fn from_string(input: &str) -> Option<Self> {
        match input {
            "HELP" => Some(Sleutelwoord::HELP),
            "NR" => Some(Sleutelwoord::NR),
            "TEKST" => Some(Sleutelwoord::TEKST),
            "TOEKENNEN" => Some(Sleutelwoord::TOEKENNEN),
            _ => None
        }
    }

    pub(super) fn is_sleutelwoord(woord: &str) -> bool {
        Self::from_string(woord).is_some()
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
    Toekennen {
        variabele_naam: String,
        expressie: String,
    },

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

    pub(super) fn is_operator_char(c: char) -> bool {
        Self::from_char(c).is_some()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Functie {
    LinksString,
    RechtsString,
    MiddenString,
    Val,
}

impl Functie {
    pub(super) fn from_str(s: &str) -> Option<Self> {
        match s {
            "LINKS$" => Some(Self::LinksString),
            "RECHTS$" => Some(Self::RechtsString),
            "MIDDEN$" => Some(Self::MiddenString),
            "VAL" => Some(Self::Val),
            _ => None,
        }
    }

    pub(super) fn as_str(self) -> &'static str {
        match self {
            Self::LinksString => "LINKS$",
            Self::RechtsString => "RECHTS$",
            Self::MiddenString => "MIDDEN$",
            Self::Val => "VAL",
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