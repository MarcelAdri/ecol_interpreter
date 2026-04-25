use crate::EcolMachine;
use crate::interpreter::errors::EcolFout;
use crate::interpreter::helpers::geen_spaties;
use crate::interpreter::LeesGeheugen;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Vergelijking {
    Gelijk,
    NietGelijk,
    GroterDan,
    MinderDan,
    GroterOfGelijk,
    MinderOfGelijk,
    EN,
    OF,
}

impl Vergelijking {
    fn from_string(tekst: &str) -> Result<Self, String> {
        match tekst {
            "=" => Ok(Self::Gelijk),
            "<>" | "≠" => Ok(Self::NietGelijk),
            ">" => Ok(Self::GroterDan),
            "<" => Ok(Self::MinderDan),
            ">=" | "≥" => Ok(Self::GroterOfGelijk),
            "<=" | "≤" => Ok(Self::MinderOfGelijk),
            "EN" => Ok(Self::EN),
            "OF" => Ok(Self::OF),
            _ => Err(format!("Ongeldig vergelijkingsteken: {}", tekst)),
        }
    }
    fn alle_vergelijking_operatoren() -> Vec<&'static str> {
        vec!["<=", ">=", "<>", "≤", "≥", "≠", "=", ">", "<"]
    }
    pub(super) fn is_simpele_vergelijking(expressie: &str) -> bool {
        let mut links: &str = "";
        let mut rechts: &str = "";
        
        for operator in Self::alle_vergelijking_operatoren() {
            let positie = expressie.find(operator);

            match positie {
                Some(pos) => {
                    links = &expressie[..pos];
                    rechts = &expressie[pos + operator.len()..];
                    break;
                }
                None => continue,
            }
        }

        if links.is_empty() || rechts.is_empty() { return false; }

        for operator in Self::alle_vergelijking_operatoren() {
            if rechts.contains(operator) { return false; }
        }

        true
    }
    pub(super) fn vergelijk(&self, links: f32, rechts: f32) -> bool {
        match self {
            Self::Gelijk => links == rechts,
            Self::NietGelijk => links != rechts,
            Self::GroterDan => links > rechts,
            Self::MinderDan => links < rechts,
            Self::GroterOfGelijk => links >= rechts,
            Self::MinderOfGelijk => links <= rechts,
            _ => false,
        }
    }
    fn voeg_samen(&self, links: bool, rechts: bool) -> bool {
        match self {
            Self::EN => links && rechts,
            Self::OF => links || rechts,
            _ => false,
        }
    }
}

impl EcolMachine {
    pub(super) fn parseer_vergelijking(&mut self, expressie: &str, lees_geheugen: &mut LeesGeheugen) -> Result<bool, EcolFout> {
        let werk_expressie = geen_spaties(expressie);

        // OF heeft lagere prioriteit dan EN: splits eerst op OF (buitenste knoop),
        // dan pas op EN. Zonder haakjes in vergelijkings-expressies is deze volgorde vast.
        let positie_of = werk_expressie.find("OF");
        let positie_en = werk_expressie.find("EN");

        let (positie, samenvoeg_teken) = match (positie_of, positie_en) {
            (Some(of), _)        => (Some(of), Vergelijking::OF),
            (None, Some(en))     => (Some(en), Vergelijking::EN),
            (None, None)         => (None,     Vergelijking::EN),
        };

        let links_expressie: &str;
        let rechts_expressie: &str;

        if let Some(pos) = positie {
            links_expressie  = &werk_expressie[..pos];
            rechts_expressie = &werk_expressie[pos + 2..];
        } else {
            links_expressie  = &werk_expressie;
            rechts_expressie = "";
        }

        let links = if Vergelijking::is_simpele_vergelijking(links_expressie) {
            self.parse_simpele_vergelijking(links_expressie, lees_geheugen)?
        } else if rechts_expressie.is_empty() {
            return Err(EcolFout::FoutMelding(format!("Ongeldige vergelijkingsexpressie: '{}'", links_expressie)));
        } else {
            self.parseer_vergelijking(links_expressie, lees_geheugen)?
        };

        if rechts_expressie.is_empty() {
            return Ok(links);
        }

        let rechts = if Vergelijking::is_simpele_vergelijking(rechts_expressie) {
            self.parse_simpele_vergelijking(rechts_expressie, lees_geheugen)?
        } else {
            self.parseer_vergelijking(rechts_expressie, lees_geheugen)?
        };

        Ok(samenvoeg_teken.voeg_samen(links, rechts))
    }
    pub(super) fn parse_simpele_vergelijking(&mut self, expressie: &str, lees_geheugen: &mut LeesGeheugen) -> Result<bool, EcolFout> {
        let mut links: &str = "";
        let mut rechts: &str = "";
        let mut operator_teken: Vergelijking = Vergelijking::Gelijk;

        for operator in Vergelijking::alle_vergelijking_operatoren() {
            let positie = expressie.find(operator);

            match positie {
                Some(pos) => {
                    links = &expressie[..pos];
                    rechts = &expressie[pos + operator.len()..];
                    operator_teken = Vergelijking::from_string(operator).map_err(EcolFout::FoutMelding)?;
                    break;
                }
                None => continue,
            }
        }

        if links.is_empty() || rechts.is_empty() {
            return Err(EcolFout::FoutMelding("Ongeldige vergelijking".to_string())); }

        let links_getal = self.solve_expression(links, lees_geheugen)?;
        let rechts_getal = self.solve_expression(rechts, lees_geheugen)?;

        Ok(operator_teken.vergelijk(links_getal, rechts_getal))
    }
}
