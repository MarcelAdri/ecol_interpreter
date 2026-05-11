use crate::EcolMachine;
use crate::interpreter::errors::{EcolFout, EcolFoutVariant};
use crate::interpreter::helpers::geen_spaties;
use crate::interpreter::LeesGeheugen;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Vergelijking {
    Gelijk,
    NietGelijk,
    GroterDan,
    MinderDan,
    GroterOfGelijk,
    MinderOfGelijk,
}

impl Vergelijking {
    fn from_string(tekst: &str) -> Result<Self, EcolFout> {
        match tekst {
            "=" => Ok(Self::Gelijk),
            "<>" | "≠" => Ok(Self::NietGelijk),
            ">" => Ok(Self::GroterDan),
            "<" => Ok(Self::MinderDan),
            ">=" | "≥" => Ok(Self::GroterOfGelijk),
            "<=" | "≤" => Ok(Self::MinderOfGelijk),
            _ => Err(EcolFout::melding(EcolFoutVariant::OngeldigVergelijkingsteken(tekst.to_string()))),
        }
    }
    fn alle_vergelijking_operatoren() -> Vec<&'static str> {
        vec!["<=", ">=", "<>", "≤", "≥", "≠", "=", ">", "<"]
    }
    #[allow(clippy::float_cmp)] //De precieze vergelijking is gewenst gedrag
    fn vergelijk(self, links: f32, rechts: f32) -> bool {
        match self {
            Self::Gelijk => links == rechts,
            Self::NietGelijk => links != rechts,
            Self::GroterDan => links > rechts,
            Self::MinderDan => links < rechts,
            Self::GroterOfGelijk => links >= rechts,
            Self::MinderOfGelijk => links <= rechts,
        }
    }
}

// Zoekt naar `zoek` alleen op haakjes-diepte 0.
fn vind_top_level_logisch(s: &str, zoek: &str) -> Option<usize> {
    let mut depth = 0i32;
    for (i, c) in s.char_indices() {
        if depth == 0 && s[i..].starts_with(zoek) {
            return Some(i);
        }
        match c {
            '(' => depth += 1,
            ')' => depth -= 1,
            _ => {}
        }
    }
    None
}

// Verwijdert recursief buitenste haakjes als de openingshaak de hele expressie omsluit.
fn strip_outer_haakjes(s: &str) -> &str {
    if !s.starts_with('(') || !s.ends_with(')') {
        return s;
    }
    let last = s.len() - 1;
    let mut depth = 0i32;
    for (i, c) in s.char_indices() {
        match c {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    return if i == last {
                        strip_outer_haakjes(&s[1..last])
                    } else {
                        s
                    };
                }
            }
            _ => {}
        }
    }
    s
}

impl EcolMachine {
    pub(super) fn parseer_vergelijking(&mut self, expressie: &str, lees_geheugen: &mut LeesGeheugen) -> Result<bool, EcolFout> {
        let werk = geen_spaties(expressie);
        let werk = strip_outer_haakjes(&werk);

        // OF heeft lagere prioriteit dan EN: splits eerst op OF.
        if let Some(pos) = vind_top_level_logisch(werk, "OF") {
            let links = self.parseer_vergelijking(&werk[..pos], lees_geheugen)?;
            let rechts = self.parseer_vergelijking(&werk[pos + 2..], lees_geheugen)?;
            return Ok(links || rechts);
        }

        if let Some(pos) = vind_top_level_logisch(werk, "EN") {
            let links = self.parseer_vergelijking(&werk[..pos], lees_geheugen)?;
            let rechts = self.parseer_vergelijking(&werk[pos + 2..], lees_geheugen)?;
            return Ok(links && rechts);
        }

        self.parse_simpele_vergelijking(werk, lees_geheugen)
    }

    fn parse_simpele_vergelijking(&mut self, expressie: &str, lees_geheugen: &mut LeesGeheugen) -> Result<bool, EcolFout> {
        let mut links: &str = "";
        let mut rechts: &str = "";
        let mut operator_teken = Vergelijking::Gelijk;

        for operator in Vergelijking::alle_vergelijking_operatoren() {
            if let Some(pos) = expressie.find(operator) {
                links = &expressie[..pos];
                rechts = &expressie[pos + operator.len()..];
                operator_teken = Vergelijking::from_string(operator)?;
                break;
            }
        }

        if links.is_empty() || rechts.is_empty() {
            return Err(EcolFout::melding(EcolFoutVariant::OngeldigeVergelijking));
        }

        let links_getal = self.solve_expression(links, lees_geheugen)?;
        let rechts_getal = self.solve_expression(rechts, lees_geheugen)?;

        Ok(operator_teken.vergelijk(links_getal, rechts_getal))
    }
}