use crate::interpreter::errors::{EcolFout, EcolFoutVariant};
use crate::interpreter::helpers::get_sym_value;

pub(super) struct VariabeleAanroep {
    variabele_naam: String,
    start: usize,
    einde: usize,
    index: Option<String>,
}
impl VariabeleAanroep {
    pub(super) fn new(variabele_naam: String, start: usize, einde: usize, index: Option<String>) -> Self {
        VariabeleAanroep {
            variabele_naam,
            start,
            einde,
            index,
        }
    }
    pub(super) fn variabele_naam(&self) -> &str {
        &self.variabele_naam
    }
    pub(super) fn start(&self) -> usize {
        self.start
    }
    pub(super) fn einde(&self) -> usize {
        self.einde
    }
    pub(super) fn index(&self) -> Option<&str> {
        self.index.as_deref()
    }
}
fn get_index(positie: usize, start: usize, einde: usize) -> Result<usize, EcolFout> {
    if positie < start || positie > einde {
        return Err(EcolFout::melding(EcolFoutVariant::OngeldigeIndex(positie, start, einde)))
    }
    let index = positie - start;

    Ok(index)
}
#[derive(Debug, Clone, PartialEq)]
pub(super) struct EcolRij {
    start: usize,
    einde: usize,
    value: Vec<f32>,
}
impl EcolRij {
    fn new(start: usize, einde: usize) -> Result<Self, EcolFout> {
        if start < 1 {
            return Err(EcolFout::melding(EcolFoutVariant::GrenzenRijStart("RIJ".to_string())));
        }
        if start > einde{
            return Err(EcolFout::melding(EcolFoutVariant::GrenzenRijAantal("RIJ".to_string(), start, einde)));
        }

        let lengte = einde - start + 1;
        let value = vec![0.0; lengte];
        Ok(EcolRij { start, einde, value })
    }

    fn set_value(&mut self, positie: usize, value: f32) -> Result<(), EcolFout>{
        let index = self.get_index(positie)?;

        self.value[index] = value;
        Ok(())
    }

    fn get_value(&self, positie: usize) -> Result<f32, EcolFout> {
        let index = self.get_index(positie)?;
        Ok(self.value[index])
    }
    fn haal_grenswaarden(&self) -> (usize, usize) {
        (self.start, self.einde)
    }

    fn get_index(&self, positie: usize) -> Result<usize, EcolFout> {
        get_index(positie, self.start, self.einde)
    }
}
#[derive(Debug, Clone, PartialEq)]
pub(super) struct EcolRijsym {
    start: usize,
    einde: usize,
    value: Vec<u8>,
}
impl EcolRijsym {
    fn new(start: usize, einde: usize) -> Result<Self, EcolFout> {
        if start < 1 {
            return Err(EcolFout::melding(EcolFoutVariant::GrenzenRijStart("RIJSYM".to_string())));
        }
        if start > einde{
            return Err(EcolFout::melding(EcolFoutVariant::GrenzenRijAantal("RIJSYM".to_string(), start, einde)));
        }

        let lengte = einde - start + 1;
        let value = vec![0u8; lengte];
        Ok(EcolRijsym { start, einde, value })
    }

    fn set_value(&mut self, positie: usize, value: f32) -> Result<(), EcolFout>{

        let small_value: u8 = get_sym_value(value)?;
        let index = self.get_index(positie)?;

        self.value[index] = small_value;
        Ok(())
    }

    fn get_value(&self, positie: usize) -> Result<f32, EcolFout> {
        let index = self.get_index(positie)?;
        Ok(f32::from(self.value[index]))
    }
    fn haal_grenswaarden(&self) -> (usize, usize) {
        (self.start, self.einde)
    }

    fn get_index(&self, positie: usize) -> Result<usize, EcolFout> {
        get_index(positie, self.start, self.einde)
    }
}
#[derive(Debug, Clone, PartialEq)]
pub(super) struct EcolTeller {
    stap: f32,
    regel: u16,
    start: f32,
    einde: f32,
    current: f32,
}
impl EcolTeller {
    fn new(stap: f32, start: f32, einde: f32) -> Self {
        EcolTeller { stap, regel: 0u16, start, einde, current: start }
    }
    fn haal_waarde(&self) -> f32 {
        self.current
    }
    fn klaar(&self, new_current: f32) -> bool {
        if self.stap < 0.0 {
            new_current < self.einde
        } else {
            new_current > self.einde
        }
    }
    fn lees_regel(&self) -> u16 {
        self.regel
    }
    fn lees_stap(&self) -> f32 {
        self.stap
    }
    fn schrijf_current(&mut self, current: f32) {
        self.current = current;
    }
    fn schrijf_regel(&mut self, regel: u16) {
        self.regel = regel;
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum VariabeleType {
    Getal,
    Rij,
    Rijsym,
    Teller,
}
impl std::fmt::Display for VariabeleType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VariabeleType::Getal => write!(f, "Getal"),
            VariabeleType::Rij => write!(f, "RIJ"),
            VariabeleType::Rijsym => write!(f, "RIJSYM"),
            VariabeleType::Teller => write!(f, "Teller"),
        }
    }
}
#[derive(Debug, Clone, PartialEq)]
pub(super) enum Waarde {
    Getal(f32),
    Rij(EcolRij),
    Rijsym(EcolRijsym),
    Teller(EcolTeller),
    NogNietBepaald,
}
impl Waarde {
    pub(super) fn is_rij(&self) -> bool {
        matches!(self, Waarde::Rij(_) | Waarde::Rijsym(_))
    }
    pub(super) fn new_getal(value: f32) -> Self {
        Waarde::Getal(value)
    }
    pub(super) fn new_rij(start: usize, einde: usize) -> Result<Self, EcolFout> {
        Ok(Waarde::Rij(EcolRij::new(start, einde)?))
    }
    pub(super) fn new_rijsym(start: usize, einde: usize) -> Result<Self, EcolFout> {
        Ok(Waarde::Rijsym(EcolRijsym::new(start, einde)?))
    }
    pub(super) fn new_teller(stap: f32, start: f32, einde: f32) -> Self {
        Waarde::Teller(EcolTeller::new(stap, start, einde))
    }
    pub(super) fn type_van(&self) -> Option<VariabeleType> {
        match self {
            Waarde::Getal(_) => Some(VariabeleType::Getal),
            Waarde::Rij(_) => Some(VariabeleType::Rij),
            Waarde::Rijsym(_) => Some(VariabeleType::Rijsym),
            Waarde::Teller(_) => Some(VariabeleType::Teller),
            Waarde::NogNietBepaald => None,
        }
    }    
    pub(super) fn rij_set_value(&mut self, value: f32, positie: usize) -> Result<(), EcolFout> {
        match self {
            Waarde::Rij(x) => { x.set_value(positie, value) },
            Waarde::Rijsym(x) => { x.set_value(positie, value)},
            _ => Err(EcolFout::melding(EcolFoutVariant::GeenRij("Waarde".to_string()))),
        }
    }
    pub(super) fn rij_haal_grenswaarden(&self) -> (usize, usize) {
        match self {
            Waarde::Rij(x) => { x.haal_grenswaarden() },
            Waarde::Rijsym(x) => { x.haal_grenswaarden() },
            _ => (0, 0),
        }
    }
    pub(super) fn haal_getal(&self, positie: usize) -> Result<f32, EcolFout> {
        match self {
            Waarde::Getal(x) => Ok(*x),
            Waarde::Rij(x) => { Ok(x.get_value(positie)?) }
            Waarde::Rijsym(x) => { Ok(x.get_value(positie)?) }
            Waarde::Teller(x) => Ok(x.haal_waarde()),
            Waarde::NogNietBepaald => Err(EcolFout::melding(EcolFoutVariant::GeenWaarde("ophalen van variabele".to_string()))),
        }
    }
    pub(super) fn teller_is_klaar(&self, new_current: f32) -> Result<bool, EcolFout> {
        match self {
            Waarde::Teller(x) => {
                Ok(x.klaar(new_current))
            }
            _ => Err(EcolFout::melding(EcolFoutVariant::GeenTeller)),
        }
    }
    pub(super) fn teller_schrijf_regel(&mut self, regel: u16) -> Result<(), EcolFout> {
        match self {
            Waarde::Teller(x) => {
                x.schrijf_regel(regel);
                Ok(())
            }
            _ => Err(EcolFout::melding(EcolFoutVariant::GeenTeller)),
        }
    }
    pub(super) fn teller_lees_regel(&self) -> Result<u16, EcolFout> {
        match self {
            Waarde::Teller(x) => {
                Ok(x.lees_regel())
            }
            _ => Err(EcolFout::melding(EcolFoutVariant::GeenTeller)),
        }
    }
    pub(super) fn teller_lees_stap(&self) -> Result<f32, EcolFout> {
        match self {
            Waarde::Teller(x) => {
                Ok(x.lees_stap())
            }
            _ => Err(EcolFout::melding(EcolFoutVariant::GeenTeller)),
        }
    }
    pub(super) fn teller_schrijf_current(&mut self, current: f32) -> Result<(), EcolFout> {
        match self {
            Waarde::Teller(x) => {
                x.schrijf_current(current);
                Ok(())
            }
            _ => Err(EcolFout::melding(EcolFoutVariant::GeenTeller)),
        }
    }

}