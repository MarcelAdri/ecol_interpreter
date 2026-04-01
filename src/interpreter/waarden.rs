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
#[derive(Debug, Clone, PartialEq)]
pub(super) struct EcolRij {
    start: usize,
    einde: usize,
    value: Vec<f32>,
}
impl EcolRij {
    fn new(start: usize, einde: usize) -> Result<Self, String> {
        if start < 1 {
            return Err("De index van een RIJ kan niet lager zijn dan 1.".to_string());
        }
        if start > einde{
            return Err(format!("Een RIJ moet tenminste 2 waarden  bevatten (start={}, eind={}).", start, einde));
        }

        let lengte = einde - start + 1;
        let value = vec![0.0; lengte];
        Ok(EcolRij { start, einde, value })
    }

    fn set_value(&mut self, positie: usize, value: f32) -> Result<(), String>{
        let index = self.get_index(positie)?;

        self.value[index] = value;
        Ok(())
    }

    fn get_value(&self, positie: usize) -> Result<f32, String> {
        let index = self.get_index(positie)?;
        Ok(self.value[index])
    }

    fn get_index(&self, positie: usize) -> Result<usize, String> {
        if positie < self.start || positie > self.einde {
            return Err(format!("Index {} is niet geldig; moet liggen tussen {} en {}.", positie, self.start, self.einde))
        }
        let index = positie - self.start;

        Ok(index)
    }
}
#[derive(Debug, Clone, PartialEq)]
pub(super) struct EcolRijsym {
    start: usize,
    einde: usize,
    value: Vec<u8>,
}
impl EcolRijsym {
    fn new(start: usize, einde: usize) -> Result<Self, String> {
        if start < 1 {
            return Err("De index van een RIJSYM kan niet lager zijn dan 1.".to_string());
        }
        if start > einde{
            return Err(format!("Een RIJSYM moet tenminste 2 waarden  bevatten (start={}, eind={}).", start, einde));
        }

        let lengte = einde - start + 1;
        let value = vec![0u8; lengte];
        Ok(EcolRijsym { start, einde, value })
    }

    fn set_value(&mut self, positie: usize, value: f32) -> Result<(), String>{

        let small_value: u8 = get_sym_value(&value)?;
        let index = self.get_index(positie)?;

        self.value[index] = small_value;
        Ok(())
    }

    fn get_value(&self, positie: usize) -> Result<f32, String> {
        let index = self.get_index(positie)?;
        Ok(self.value[index] as f32)
    }

    fn get_index(&self, positie: usize) -> Result<usize, String> {

        if positie < self.start || positie > self.einde {
            return Err(format!("Index {} is niet geldig; moet liggen tussen {} en {}.", positie, self.start, self.einde))
        }
        let index = positie - self.start;

        Ok(index)
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum VariabeleType {
    Getal,
    Rij,
    Rijsym,
}

impl VariabeleType {
    pub(super) fn to_string(&self) -> &'static str {
        match self {
            VariabeleType::Getal => "Getal",
            VariabeleType::Rij => "Rij",
            VariabeleType::Rijsym => "Rijsym",
        }
    }
}
#[derive(Debug, Clone, PartialEq)]
pub(super) enum Waarde {
    Getal(f32),
    Rij(EcolRij),
    Rijsym(EcolRijsym),
    NogNietBepaald,
}
impl Waarde {
    pub(super) fn new_getal(value: f32) -> Self {
        Waarde::Getal(value)
    }
    pub(super) fn new_rij(start: usize, einde: usize) -> Result<Self, String> {
        Ok(Waarde::Rij(EcolRij::new(start, einde)?))
    }
    pub(super) fn new_rijsym(start: usize, einde: usize) -> Result<Self, String> {
        Ok(Waarde::Rijsym(EcolRijsym::new(start, einde)?))
    }
    pub(super) fn type_van(&self) -> Option<VariabeleType> {
        match self {
            Waarde::Getal(_) => Some(VariabeleType::Getal),
            Waarde::Rij(_) => Some(VariabeleType::Rij),
            Waarde::Rijsym(_) => Some(VariabeleType::Rijsym),
            Waarde::NogNietBepaald => None,
        }
    }
    pub(super) fn rij_set_value(&mut self, value: f32, positie: usize) -> Result<(), String> {
        match self {
            Waarde::Rij(x) => { x.set_value(positie, value) }
            _ => Err("Waarde is geen rij".to_string()),
        }
    }
    pub(super) fn rijsym_set_value(&mut self, value: f32, positie: usize) -> Result<(), String> {
        match self {
            Waarde::Rijsym(x) => { x.set_value(positie, value) }
            _ => Err("Waarde is geen rijsym".to_string()),
        }
    }
    pub(super) fn haal_getal(&self, positie: usize) -> Result<f32, String> {
        match self {
            Waarde::Getal(x) => Ok(*x),
            Waarde::Rij(x) => { Ok(x.get_value(positie)?) }
            Waarde::Rijsym(x) => { Ok(x.get_value(positie)?) }
            Waarde::NogNietBepaald => Err("Waarde is nog niet bepaald".to_string()),
        }
    }

}

//Helpers
// fn escape_char(c: char) -> Option<&'static str> {
//     match c {
//         '"' => Some("\\\""),
//         '\\' => Some("\\\\"),
//         '\n' => Some("\\n"),
//         '\r' => Some("\\r"),
//         '\t' => Some("\\t"),
//         '\0' => Some("\\0"),
//         _ => None,
//     }
// }

// pub(super) fn haal_data(token: &Waarde, positie: usize) -> Result<String, String> {
//
//     match token {
//         Waarde::Getal(x) => Ok(format_getal(*x)),
//         Waarde::Rij(x) => Ok(format_getal(x.get_value(positie)?))
//     }
// }
