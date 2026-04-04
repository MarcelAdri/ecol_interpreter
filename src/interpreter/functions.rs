use crate::interpreter::EcolMachine;
use crate::interpreter::helpers::grens_bewaking;
use crate::interpreter::waarden::VariabeleType;

pub(super) const MAG_ALLEEN_HELE_GETALLEN: bool = true;
pub(super) const MAG_ALLEEN_POSITIEVE_GETALLEN: bool = true;

#[derive(Debug, Clone, PartialEq)]
pub(super) enum FunctieNaam {
    ABS,
    ARCTAN,
    BOVIN,
    COS,
    EXP,
    G,
    GOK,
    GOKC,
    LN,
    LOG,
    ONDIN,
    PS,
    SIN,
    WRTL,
}

impl FunctieNaam {
    pub(super) fn from_str(s: &str) -> Option<Self> {
        match s {
            "ABS" => Some(Self::ABS),
            "ARCTAN" => Some(Self::ARCTAN),
            "BOVIN" => Some(Self::BOVIN),
            "COS" => Some(Self::COS),
            "EXP" => Some(Self::EXP),
            "G" => Some(Self::G),
            "GOK" => Some(Self::GOK),
            "GOKC" => Some(Self::GOKC),
            "LN" => Some(Self::LN),
            "LOG" => Some(Self::LOG),
            "ONDIN" => Some(Self::ONDIN),
            "PS" => Some(Self::PS),
            "SIN" => Some(Self::SIN),
            "WRTL" => Some(Self::WRTL),
            _ => None,
        }
    }

    pub(super) fn to_string(&self) -> String {
        match self {
            Self::ABS => "ABS".to_string(),
            Self::ARCTAN => "ARCTAN".to_string(),
            Self::BOVIN => "BOVIN".to_string(),
            Self::COS => "COS".to_string(),
            Self::EXP => "EXP".to_string(),
            Self::G => "G".to_string(),
            Self::GOK => "GOK".to_string(),
            Self::GOKC => "GOKC".to_string(),
            Self::LN => "LN".to_string(),
            Self::LOG => "LOG".to_string(),
            Self::ONDIN => "ONDIN".to_string(),
            Self::PS => "PS".to_string(),
            Self::SIN => "SIN".to_string(),
            Self::WRTL => "WRTL".to_string(),
        }
    }

    pub(super) fn verwacht_argumenten(&self) -> usize {
        match self {
            Self::ABS => 1,
            Self::ARCTAN => 1,
            Self::BOVIN => 0,
            Self::COS => 1,
            Self::EXP => 1,
            Self::G => 1,
            Self::GOK => 2,
            Self::GOKC => 0,
            Self::LN => 1,
            Self::LOG => 1,
            Self::ONDIN => 0,
            Self::PS => 0,
            Self::SIN => 1,
            Self::WRTL => 1,
        }
    }

    pub(super) fn verwacht_string_argument(&self) -> bool {
        matches!(self, Self::ONDIN | Self::BOVIN)
    }
}
pub(super) enum Functie {
    ABS { getal: f32 },
    ARCTAN { getal: f32 },
    BOVIN { variabele_naam: String },
    COS { getal: f32 },
    EXP { getal: f32 },
    G { getal: f32 },
    GOK { laag: f32, hoog: f32 },
    GOKC { },
    LN { getal: f32 },
    LOG { getal: f32 },
    ONDIN { variabele_naam: String },
    PS { },
    SIN { getal: f32 },
    WRTL { getal: f32 },
}
impl Functie {
    pub(super) fn new(functienaam: FunctieNaam, argumenten: Vec<f32>, rij_naam: &str) -> Result<Self, String> {
        if argumenten.len() != functienaam.verwacht_argumenten() {
            return Err(format!("Verkeerd aantal argumenten voor '{}'", functienaam.to_string()));
        }
        match functienaam {
            FunctieNaam::ABS => Ok(Functie::ABS { getal: argumenten[0] }),
            FunctieNaam::ARCTAN => Ok(Functie::ARCTAN { getal: argumenten[0] }),
            FunctieNaam::BOVIN => Ok(Functie::BOVIN { variabele_naam: rij_naam.to_string() }),
            FunctieNaam::COS => Ok(Functie::COS { getal: argumenten[0] }),
            FunctieNaam::EXP => Ok(Functie::EXP { getal: argumenten[0] }),
            FunctieNaam::G => Ok(Functie::G { getal: argumenten[0] }),
            FunctieNaam::GOK => Ok(Functie::GOK { laag: argumenten[0], hoog: argumenten[1] }),
            FunctieNaam::GOKC => Ok(Functie::GOKC { }),
            FunctieNaam::LN => Ok(Functie::LN { getal: argumenten[0] }),
            FunctieNaam::LOG => Ok(Functie::LOG { getal: argumenten[0] }),
            FunctieNaam::ONDIN => Ok(Functie::ONDIN { variabele_naam: rij_naam.to_string() }),
            FunctieNaam::PS => Ok(Functie::PS { }),
            FunctieNaam::SIN => Ok(Functie::SIN { getal: argumenten[0] }),
            FunctieNaam::WRTL => Ok(Functie::WRTL { getal: argumenten[0] }),
        }
    }

    pub(super) fn haal_naam(&self) -> Option<FunctieNaam> {
        match self {
            Functie::ABS { .. } => Some(FunctieNaam::ABS),
            Functie::ARCTAN { .. } => Some(FunctieNaam::ARCTAN),
            Functie::BOVIN { .. } => Some(FunctieNaam::BOVIN),
            Functie::COS { .. } => Some(FunctieNaam::COS),
            Functie::EXP { .. } => Some(FunctieNaam::EXP),
            Functie::G { .. } => Some(FunctieNaam::G),
            Functie::GOK { .. } => Some(FunctieNaam::GOK),
            Functie::GOKC { .. } => Some(FunctieNaam::GOKC),
            Functie::LN { .. } => Some(FunctieNaam::LN),
            Functie::LOG { .. } => Some(FunctieNaam::LOG),
            Functie::ONDIN { .. } => Some(FunctieNaam::ONDIN),
            Functie::PS { .. } => Some(FunctieNaam::PS),
            Functie::SIN { .. } => Some(FunctieNaam::SIN),
            Functie::WRTL { .. } => Some(FunctieNaam::WRTL),
        }
    }
}

impl EcolMachine {
    pub(super) fn execute_function(&mut self, functie: &Functie) -> Result<f32, String> {

        let result = match functie {
            Functie::ABS { getal } => { let uitkomst = self.execute_function_abs(getal)?; Ok(uitkomst) },
            Functie::ARCTAN { getal } => { let uitkomst = self.execute_function_arctan(getal)?; Ok(uitkomst) },
            Functie::BOVIN { variabele_naam } => { let uitkomst = self.execute_function_bovin(&variabele_naam)?; Ok(uitkomst) },
            Functie::COS { getal } => { let uitkomst = self.execute_function_cos(getal)?; Ok(uitkomst) },
            Functie::EXP { getal } => { let uitkomst = self.execute_function_exp(getal)?; Ok(uitkomst) },
            Functie::G { getal } => { let uitkomst = self.execute_function_g(getal)?; Ok(uitkomst) },
            Functie::GOK { laag, hoog } => { let uitkomst = self.execute_function_gok(laag, hoog)?; Ok(uitkomst) },
            Functie::GOKC { } => { let uitkomst = self.execute_function_gokc()?; Ok(uitkomst) },
            Functie::LN { getal } => { let uitkomst = self.execute_function_ln(getal)?; Ok(uitkomst) },
            Functie::LOG { getal } => { let uitkomst = self.execute_function_log(getal)?; Ok(uitkomst) },
            Functie::ONDIN { variabele_naam } => { let uitkomst = self.execute_function_ondin(&variabele_naam)?; Ok(uitkomst) },
            Functie::PS { } => { let uitkomst = self.execute_function_ps()?; Ok(uitkomst) },
            Functie::SIN { getal } => { let uitkomst = self.execute_function_sin(getal)?; Ok(uitkomst) },
            Functie::WRTL { getal} => { let uitkomst = self.execute_function_wrtl(getal)?; Ok(uitkomst) },
        };

        result
    }
    fn execute_function_abs(&self, getal: &f32) -> Result<f32, String> {
        let getal = grens_bewaking(getal, !MAG_ALLEEN_POSITIEVE_GETALLEN, !MAG_ALLEEN_HELE_GETALLEN)?;

        Ok(getal.abs())
    }
    fn execute_function_arctan(&self, getal: &f32) -> Result<f32, String> {
        let getal = grens_bewaking(getal, !MAG_ALLEEN_POSITIEVE_GETALLEN, !MAG_ALLEEN_HELE_GETALLEN)?;

        Ok(getal.atan())
    }
    fn execute_function_bovin(&self, variabele_naam: &str) -> Result<f32, String> {
        let Some(waarde) = self.var_lees_waarde(variabele_naam) else { return Err(format!("De variabele '{}' bestaat niet", variabele_naam));};
        match waarde.type_van() {
            Some(VariabeleType::Rij) | Some(VariabeleType::Rijsym) => {}
            _ => return Err(format!("'{}' is geen RIJ of RIJSYM variabele", variabele_naam)),
        }
        let (_, result) = waarde.rij_haal_grenswaarden();
        Ok(result as f32)
    }
    fn execute_function_cos(&self, getal: &f32) -> Result<f32, String> {
        let getal = grens_bewaking(getal, !MAG_ALLEEN_POSITIEVE_GETALLEN, !MAG_ALLEEN_HELE_GETALLEN)?;

        Ok(getal.cos())
    }
    fn execute_function_exp(&self, getal: &f32) -> Result<f32, String> {
        let getal = grens_bewaking(getal, !MAG_ALLEEN_POSITIEVE_GETALLEN, !MAG_ALLEEN_HELE_GETALLEN)?;

        Ok(getal.exp())
    }
    fn execute_function_g(&self, getal: &f32) -> Result<f32, String> {
        let getal = grens_bewaking(getal, !MAG_ALLEEN_POSITIEVE_GETALLEN, !MAG_ALLEEN_HELE_GETALLEN)?;

        Ok(getal.floor())
    }
    fn execute_function_gok(&mut self, laag: &f32, hoog: &f32) -> Result<f32, String> {
        let laag = grens_bewaking(laag, !MAG_ALLEEN_POSITIEVE_GETALLEN, !MAG_ALLEEN_HELE_GETALLEN)?;
        let hoog = grens_bewaking(hoog, !MAG_ALLEEN_POSITIEVE_GETALLEN, !MAG_ALLEEN_HELE_GETALLEN)?;
        if laag >= hoog {
            return Err(format!("De lage waarde van de GOK ({}) mag niet groter zijn dan de hoge waarde ({})", laag, hoog));
        }

        Ok(self.volgende_willekeurig(laag, hoog).round())
    }
    fn execute_function_gokc(&mut self) -> Result<f32, String> {
        Ok(self.volgende_willekeurig(0.0, 1.0))
    }

    fn execute_function_ln(&self, getal: &f32) -> Result<f32, String> {
        let getal = grens_bewaking(getal, MAG_ALLEEN_POSITIEVE_GETALLEN, MAG_ALLEEN_HELE_GETALLEN)?;

        Ok(getal.ln())
    }
    fn execute_function_log(&self, getal: &f32) -> Result<f32, String> {
        let getal = grens_bewaking(getal, MAG_ALLEEN_POSITIEVE_GETALLEN, MAG_ALLEEN_HELE_GETALLEN)?;

        Ok(getal.log10())
    }
    fn execute_function_ondin(&self, variabele_naam: &str) -> Result<f32, String> {
        let Some(waarde) = self.var_lees_waarde(variabele_naam) else { return Err(format!("De variabele '{}' bestaat niet", variabele_naam));};
        match waarde.type_van() {
            Some(VariabeleType::Rij) | Some(VariabeleType::Rijsym) => {}
            _ => return Err(format!("'{}' is geen RIJ of RIJSYM variabele", variabele_naam)),
        }
        let (result, _) = waarde.rij_haal_grenswaarden();
        Ok(result as f32)
    }

    fn execute_function_ps(&self) -> Result<f32, String> {
        let werk = self.lees_regel();
        Ok(werk.len() as f32)
    }
    fn execute_function_sin(&self, getal: &f32) -> Result<f32, String> {
        let getal = grens_bewaking(getal, !MAG_ALLEEN_POSITIEVE_GETALLEN, !MAG_ALLEEN_HELE_GETALLEN)?;

        Ok(getal.sin())
    }
    fn execute_function_wrtl(&self, getal: &f32) -> Result<f32, String> {
        let getal = grens_bewaking(getal, MAG_ALLEEN_POSITIEVE_GETALLEN, MAG_ALLEEN_HELE_GETALLEN)?;

        Ok(getal.sqrt())
    }
}