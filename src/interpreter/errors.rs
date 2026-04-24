#[derive(Debug, Clone, PartialEq)]
pub(super) enum EcolFout {
    FoutMelding(String),
    WachtOpLees(u16),
    WachtOpLeessym(u16),
    WachtOpLaad,
}

impl EcolFout {
    pub(super) fn to_string(&self) -> String {
        match self {
            EcolFout::FoutMelding(melding) => format!("{}", melding),
            EcolFout::WachtOpLees(regel) => format!("Wachten op LEES regel {}.", regel),
            EcolFout::WachtOpLeessym(regel) => format!("Wachten op LEESSYM regel {}.", regel),
            EcolFout::WachtOpLaad => "Wachten op LAAD.".to_string(),
        }
    }
    pub(super) fn met_regel(self, regel: u16) -> Self {
        match self {
            EcolFout::FoutMelding(m) => EcolFout::FoutMelding(format!("FOUTMELDING in regel {}: {}", regel, m)),
            other => other,
        }
    }
}
