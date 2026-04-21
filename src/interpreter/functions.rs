use std::collections::{BTreeMap, HashMap};
use crate::interpreter::{EcolMachine, LeesGeheugen};
use crate::interpreter::helpers::{geen_spaties, grens_bewaking};
use crate::interpreter::opdrachten::{execute_all, Context, WhatsNext};
use crate::interpreter::program::{Line, LineInhoud};
use crate::interpreter::waarden::{VariabeleType, Waarde};

pub(super) const MAG_ALLEEN_HELE_GETALLEN: bool = true;
pub(super) const MAG_ALLEEN_POSITIEVE_GETALLEN: bool = true;

#[derive(Debug, Clone, PartialEq)]
pub(super) enum EcolFout {
    FoutMelding(String),
    WachtOpLees(u16),
    WachtOpLeessym(u16),
}

impl EcolFout {
    pub(super) fn to_string(&self) -> String {
        match self {
            EcolFout::FoutMelding(melding) => format!("{}", melding),
            EcolFout::WachtOpLees(regel) => format!("Wachten op LEES regel {}.", regel),
            EcolFout::WachtOpLeessym(regel) => format!("Wachten op LEESSYM regel {}.", regel),
        }
    }
    pub(super) fn met_regel(self, regel: u16) -> Self {
        match self {
            EcolFout::FoutMelding(m) => EcolFout::FoutMelding(format!("FOUTMELDING in regel {}: {}", regel, m)),
            other => other,
        }
    }
}
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
    LEES,
    LEESSYM,
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
            "LEES" => Some(Self::LEES),
            "LEESSYM" => Some(Self::LEESSYM),
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
            Self::LEES => "LEES".to_string(),
            Self::LEESSYM => "LEESSYM".to_string(),
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
            Self::LEES => 0,
            Self::LEESSYM => 0,
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
    LEES { },
    LEESSYM { },
    LN { getal: f32 },
    LOG { getal: f32 },
    ONDIN { variabele_naam: String },
    PS { },
    SIN { getal: f32 },
    WRTL { getal: f32 },
}
impl Functie {
    pub(super) fn new(functienaam: FunctieNaam, argumenten: Vec<f32>, rij_naam: &str) -> Result<Self, EcolFout> {
        if argumenten.len() != functienaam.verwacht_argumenten() {
            return Err(EcolFout::FoutMelding(format!("Verkeerd aantal argumenten voor '{}'", functienaam.to_string())));
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
            FunctieNaam::LEES => Ok(Functie::LEES { }),
            FunctieNaam::LEESSYM => Ok(Functie::LEESSYM { }),
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
            Functie::LEES { .. } => Some(FunctieNaam::LEES),
            Functie::LEESSYM { .. } => Some(FunctieNaam::LEESSYM),
            Functie::LN { .. } => Some(FunctieNaam::LN),
            Functie::LOG { .. } => Some(FunctieNaam::LOG),
            Functie::ONDIN { .. } => Some(FunctieNaam::ONDIN),
            Functie::PS { .. } => Some(FunctieNaam::PS),
            Functie::SIN { .. } => Some(FunctieNaam::SIN),
            Functie::WRTL { .. } => Some(FunctieNaam::WRTL),
        }
    }
}
#[derive(Debug, Clone)]
pub(super) struct FunDef {
    parameters: Vec<String>,
    body: BTreeMap<u16, LineInhoud>,
}
impl FunDef {
    fn new() -> Self {
        FunDef { parameters: Vec::new(), body: BTreeMap::new() }
    }

    fn get_parameters(&self) -> &Vec<String> {
        &self.parameters
    }
    fn get_body(&self) -> &BTreeMap<u16, LineInhoud> {
        &self.body
    }
}

pub(super) struct EigenFunctie {
    functie_omgeving: EcolMachine,
}

impl EigenFunctie {
    fn new() -> Self {
        EigenFunctie { functie_omgeving: EcolMachine::new() }
    }

    fn eigen_functie(functies: &HashMap<String, FunDef>, functie: &FunDef, argumenten: Vec<Waarde>, diepte: u16, lees_geheugen: &mut LeesGeheugen) -> Result<f32, EcolFout> {
        if diepte > 100 {
            return Err(EcolFout::FoutMelding("Maximum recursie-diepte overschreden (100).".to_string()));
        }
        let mut machine = Self::new();
        machine.functie_omgeving.stel_functie_diepte_in(diepte);
        if argumenten.len() != functie.parameters.len() {
            return Err(EcolFout::FoutMelding(format!("Functie verwacht {} argumenten, er staan {} argumenten in de aanroep.", functie.parameters.len(), argumenten.len())));
        }
        machine.functie_omgeving.laad_programma(&functie.body);
        machine.functie_omgeving.laad_functies(functies);

        for index in 0..functie.parameters.len() {
            let mut naam = functie.parameters[index].to_string();
            if geen_spaties(&naam).starts_with("RIJSYM") {
                naam = geen_spaties(&naam[6..]);
                if argumenten[index].type_van() != Some(VariabeleType::Rijsym) {
                    return Err(EcolFout::FoutMelding(format!("Functie verwacht RIJSYM als argument nr. {}, maar ontving een {}.", index + 1, argumenten[index].type_van().unwrap().to_string())));
                }
                let (begin, einde) = argumenten[index].rij_haal_grenswaarden();
                machine.functie_omgeving.var_reserveer_rijsym(&naam, begin, einde)?;
            } else if geen_spaties(&naam).starts_with("RIJ") {
                naam = geen_spaties(&naam[3..]);
                if argumenten[index].type_van() != Some(VariabeleType::Rij) {
                    return Err(EcolFout::FoutMelding(format!("Functie verwacht RIJ als argument nr. {}, maar ontving een {}.", index + 1, argumenten[index].type_van().unwrap().to_string())));
                }
                let (begin, einde) = argumenten[index].rij_haal_grenswaarden();
                machine.functie_omgeving.var_reserveer_rij(&naam, begin, einde)?;
            } else {
                if argumenten[index].type_van() != Some(VariabeleType::Getal) {
                    return Err(EcolFout::FoutMelding(format!("Functie verwacht GETAL als argument nr. {}, maar ontving een {}.", index + 1, argumenten[index].type_van().unwrap().to_string())));
                }
            }
            machine.functie_omgeving.var_schrijf_waarde(&naam, argumenten[index].clone())?;
        }

        let programma = machine.functie_omgeving.programma().clone();
        let mut current = 1;
        loop {

            let Some((&regelnummer, current_regel)) = programma.range(current..).next() else {
                return Err(EcolFout::FoutMelding("FOUTMELDING: Er zijn geen regels meer om uit te voeren. FUN := niet aangetroffen.".to_string()));
            };
            current = regelnummer + 1;
            let regel = Line::new(current, current_regel.clone());

            match current_regel {
                LineInhoud::FunEind { expressie} => {
                    return Ok(machine.functie_omgeving.solve_expression(&expressie, lees_geheugen)?);
                },
                _ => {
                    let (reply_option, nextline_option, whatsnext_option) = execute_all(&regel, &mut machine.functie_omgeving, &programma, Context::Functie, lees_geheugen, &mut |_| {})?;
                    match reply_option {
                        Some(reply) => {
                            return Ok(reply.parse::<f32>().map_err(|_| EcolFout::FoutMelding("FOUTMELDING: De functie heeft een ongeldig resultaat.".to_string()))?);
                        },
                        None => {},
                    }

                    match nextline_option {
                        Some(nextline) => {
                            current = nextline;
                            continue;
                        },
                        None => {},
                    }

                    match whatsnext_option {
                        Some(whatsnext) => {
                            match whatsnext {
                                WhatsNext::Break => break,
                                WhatsNext::Continue => continue,
                            }
                        },
                        None => {},
                    }
                }
            }



        }

        Err(EcolFout::FoutMelding("FOUTMELDING: FUNctie  eindigde zonder resultaat".to_string()))
    }
}

impl EcolMachine {
    pub(super) fn execute_function(&mut self, lees_geheugen: &mut LeesGeheugen, functie: &Functie) -> Result<f32, EcolFout> {

        let result = match functie {
            Functie::ABS { getal } => { let uitkomst = self.execute_function_abs(getal)?; Ok(uitkomst) },
            Functie::ARCTAN { getal } => { let uitkomst = self.execute_function_arctan(getal)?; Ok(uitkomst) },
            Functie::BOVIN { variabele_naam } => { let uitkomst = self.execute_function_bovin(&variabele_naam)?; Ok(uitkomst) },
            Functie::COS { getal } => { let uitkomst = self.execute_function_cos(getal)?; Ok(uitkomst) },
            Functie::EXP { getal } => { let uitkomst = self.execute_function_exp(getal)?; Ok(uitkomst) },
            Functie::G { getal } => { let uitkomst = self.execute_function_g(getal)?; Ok(uitkomst) },
            Functie::GOK { laag, hoog } => { let uitkomst = self.execute_function_gok(laag, hoog)?; Ok(uitkomst) },
            Functie::GOKC { } => { let uitkomst = self.execute_function_gokc()?; Ok(uitkomst) },
            Functie::LEES { } => { let uitkomst = self.execute_function_lees(lees_geheugen)?; Ok(uitkomst) },
            Functie::LEESSYM { } => { let uitkomst = self.execute_function_leessym(lees_geheugen)?; Ok(uitkomst) },
            Functie::LN { getal } => { let uitkomst = self.execute_function_ln(getal)?; Ok(uitkomst) },
            Functie::LOG { getal } => { let uitkomst = self.execute_function_log(getal)?; Ok(uitkomst) },
            Functie::ONDIN { variabele_naam } => { let uitkomst = self.execute_function_ondin(&variabele_naam)?; Ok(uitkomst) },
            Functie::PS { } => { let uitkomst = self.execute_function_ps()?; Ok(uitkomst) },
            Functie::SIN { getal } => { let uitkomst = self.execute_function_sin(getal)?; Ok(uitkomst) },
            Functie::WRTL { getal} => { let uitkomst = self.execute_function_wrtl(getal)?; Ok(uitkomst) },
        };

        result
    }
    fn execute_function_abs(&self, getal: &f32) -> Result<f32, EcolFout> {
        let getal = grens_bewaking(getal, !MAG_ALLEEN_POSITIEVE_GETALLEN, !MAG_ALLEEN_HELE_GETALLEN)?;

        Ok(getal.abs())
    }
    fn execute_function_arctan(&self, getal: &f32) -> Result<f32, EcolFout> {
        let getal = grens_bewaking(getal, !MAG_ALLEEN_POSITIEVE_GETALLEN, !MAG_ALLEEN_HELE_GETALLEN)?;

        Ok(getal.atan())
    }
    fn execute_function_bovin(&self, variabele_naam: &str) -> Result<f32, EcolFout> {
        let Some(waarde) = self.var_lees_waarde(variabele_naam) else { return Err(EcolFout::FoutMelding(format!("De variabele '{}' bestaat niet", variabele_naam)));};
        match waarde.type_van() {
            Some(VariabeleType::Rij) | Some(VariabeleType::Rijsym) => {}
            _ => return Err(EcolFout::FoutMelding(format!("'{}' is geen RIJ of RIJSYM variabele", variabele_naam))),
        }
        let (_, result) = waarde.rij_haal_grenswaarden();
        Ok(result as f32)
    }
    fn execute_function_cos(&self, getal: &f32) -> Result<f32, EcolFout> {
        let getal = grens_bewaking(getal, !MAG_ALLEEN_POSITIEVE_GETALLEN, !MAG_ALLEEN_HELE_GETALLEN)?;

        Ok(getal.cos())
    }
    fn execute_function_exp(&self, getal: &f32) -> Result<f32, EcolFout> {
        let getal = grens_bewaking(getal, !MAG_ALLEEN_POSITIEVE_GETALLEN, !MAG_ALLEEN_HELE_GETALLEN)?;

        Ok(getal.exp())
    }
    fn execute_function_g(&self, getal: &f32) -> Result<f32, EcolFout> {
        let getal = grens_bewaking(getal, !MAG_ALLEEN_POSITIEVE_GETALLEN, !MAG_ALLEEN_HELE_GETALLEN)?;

        Ok(getal.floor())
    }
    fn execute_function_gok(&mut self, laag: &f32, hoog: &f32) -> Result<f32, EcolFout> {
        let laag = grens_bewaking(laag, !MAG_ALLEEN_POSITIEVE_GETALLEN, !MAG_ALLEEN_HELE_GETALLEN)?;
        let hoog = grens_bewaking(hoog, !MAG_ALLEEN_POSITIEVE_GETALLEN, !MAG_ALLEEN_HELE_GETALLEN)?;
        if laag >= hoog {
            return Err(EcolFout::FoutMelding(format!("De lage waarde van de GOK ({}) mag niet groter zijn dan de hoge waarde ({})", laag, hoog)));
        }

        Ok(self.volgende_willekeurig(laag, hoog).round())
    }
    fn execute_function_gokc(&mut self) -> Result<f32, EcolFout> {
        Ok(self.volgende_willekeurig(0.0, 1.0))
    }
    fn execute_function_lees(&mut self, lees_geheugen: &mut LeesGeheugen) -> Result<f32, EcolFout> {
        if lees_geheugen.wacht_op_lees() {
            let Some(getal) = lees_geheugen.lees_waarde() else {
                return Err(EcolFout::FoutMelding("Geen waarde na LEES(SYM) (interne fout).".to_string()));
            };
            lees_geheugen.lees_hervat_none();
            //panic!("Getal {} ", getal);
            Ok(getal)
        } else {
            Err(EcolFout::WachtOpLees(0))
        }
    }

    fn execute_function_leessym(&mut self, lees_geheugen: &mut LeesGeheugen) -> Result<f32, EcolFout> {
        if lees_geheugen.wacht_op_leessym() {
            let Some(getal) = lees_geheugen.leessym_waarde() else {
                return Err(EcolFout::FoutMelding("Geen waarde na LEES(SYM) (interne fout).".to_string()));
            };
            //self.wacht_op_lees_none();
            lees_geheugen.leessym_hervat_none();

            Ok(getal)
        } else {
            Err(EcolFout::WachtOpLeessym(0))
        }
    }
    fn execute_function_ln(&self, getal: &f32) -> Result<f32, EcolFout> {
        let getal = grens_bewaking(getal, MAG_ALLEEN_POSITIEVE_GETALLEN, MAG_ALLEEN_HELE_GETALLEN)?;

        Ok(getal.ln())
    }
    fn execute_function_log(&self, getal: &f32) -> Result<f32, EcolFout> {
        let getal = grens_bewaking(getal, MAG_ALLEEN_POSITIEVE_GETALLEN, MAG_ALLEEN_HELE_GETALLEN)?;

        Ok(getal.log10())
    }
    fn execute_function_ondin(&self, variabele_naam: &str) -> Result<f32, EcolFout> {
        let Some(waarde) = self.var_lees_waarde(variabele_naam) else { return Err(EcolFout::FoutMelding(format!("De variabele '{}' bestaat niet", variabele_naam)));};
        match waarde.type_van() {
            Some(VariabeleType::Rij) | Some(VariabeleType::Rijsym) => {}
            _ => return Err(EcolFout::FoutMelding(format!("'{}' is geen RIJ of RIJSYM variabele", variabele_naam))),
        }
        let (result, _) = waarde.rij_haal_grenswaarden();
        Ok(result as f32)
    }

    fn execute_function_ps(&self) -> Result<f32, EcolFout> {
        let werk = self.lees_regel();
        Ok(werk.len() as f32)
    }
    fn execute_function_sin(&self, getal: &f32) -> Result<f32, EcolFout> {
        let getal = grens_bewaking(getal, !MAG_ALLEEN_POSITIEVE_GETALLEN, !MAG_ALLEEN_HELE_GETALLEN)?;

        Ok(getal.sin())
    }
    fn execute_function_wrtl(&self, getal: &f32) -> Result<f32, EcolFout> {
        let getal = grens_bewaking(getal, MAG_ALLEEN_POSITIEVE_GETALLEN, MAG_ALLEEN_HELE_GETALLEN)?;

        Ok(getal.sqrt())
    }
    pub(super) fn execute_eigen_functie(&self, naam: &str, argumenten: Vec<Waarde>, lees_geheugen: &mut LeesGeheugen) -> Result<f32, EcolFout> {
        let definitie = self.haal_functiedefinitie(naam)
            .ok_or_else(|| EcolFout::FoutMelding(format!("Functie '{}' bestaat niet", naam)))?
            .clone();
        let diepte = self.functie_diepte() + 1;
        EigenFunctie::eigen_functie(self.haal_functie_register(), &definitie, argumenten, diepte, lees_geheugen)
    }
    pub(super) fn get_fundef_parameters(&self, naam: &str) -> Option<Vec<String>> {
        let definitie = self.haal_functiedefinitie(naam)?;
        Some(definitie.get_parameters().clone())
    }
    pub(super) fn get_fundef_body(&self, naam: &str) -> Option<BTreeMap<u16, LineInhoud>> {
        let definitie = self.haal_functiedefinitie(naam)?;
        Some(definitie.get_body().clone())
    }

    pub(super) fn extract_functie_definities(&mut self, volledige_programma: &BTreeMap<u16,LineInhoud>) -> Result<BTreeMap<u16,LineInhoud>, EcolFout> {
        let mut nieuwe_programma: BTreeMap<u16, LineInhoud> = BTreeMap::new();
        let mut in_functie_definitie = false;
        let mut fundef = FunDef::new();
        let mut naam_van_functie: &str = "";

        for (regelnummer, regel) in volledige_programma.iter() {
            match regel {
                LineInhoud::FunStart { variabele_naam, argumenten } => {
                    in_functie_definitie = true;
                    naam_van_functie = variabele_naam;
                    let parameters = argumenten.split(',')
                        .filter_map(|s| {
                            let t = s.trim().to_string();
                            if t.is_empty() { None } else { Some(t) }
                        })
                        .collect::<Vec<String>>();
                    fundef = FunDef::new();
                    fundef.parameters = parameters;
                }
                LineInhoud::FunEind { expressie } => {
                    if in_functie_definitie {
                        fundef.body.insert(*regelnummer, regel.clone());
                        self.schrijf_nieuwe_functie(naam_van_functie, &fundef)?;
                        naam_van_functie = "";
                        in_functie_definitie = false;
                    } else {
                        nieuwe_programma.insert(*regelnummer, regel.clone());
                    }
                }
                _ => {
                    if in_functie_definitie {
                        fundef.body.insert(*regelnummer, regel.clone());
                    } else {
                        nieuwe_programma.insert(*regelnummer, regel.clone());
                    }
                }
            }
        }

        Ok(nieuwe_programma)
    }

}