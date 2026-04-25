use std::fmt::Display;

#[derive(Debug, Clone, PartialEq)]
pub(super) enum EcolFout {
    FoutMelding(String),
    WachtOpLees(u16),
    WachtOpLeessym(u16),
    WachtOpLaad,
}

impl EcolFout {
    pub(super) fn met_regel(self, regel: u16) -> Self {
        match self {
            EcolFout::FoutMelding(m) => EcolFout::FoutMelding(format!("FOUTMELDING in regel {}: {}", regel, m)),
            other => other,
        }
    }
}

impl Display for EcolFout {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let melding = match self {
            EcolFout::FoutMelding(melding) => melding.to_string(),
            EcolFout::WachtOpLees(regel) => format!("Wachten op LEES regel {}.", regel),
            EcolFout::WachtOpLeessym(regel) => format!("Wachten op LEESSYM regel {}.", regel),
            EcolFout::WachtOpLaad => "Wachten op LAAD.".to_string(),
        };

        write!(f, "{}", melding)
    }
}
