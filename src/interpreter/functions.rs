use std::collections::{BTreeMap, HashMap};
use std::fmt::Display;
use crate::interpreter::{EcolMachine, LeesGeheugen};
use crate::interpreter::errors::{EcolFout, EcolFoutVariant, EcolSignaal};
use crate::interpreter::helpers::{argumenten_to_vec, geen_spaties};
use crate::interpreter::opdrachten::{execute_all, Context, WhatsNext};
use crate::interpreter::program::{FunDef, Line, LineInhoud};
use crate::interpreter::waarden::{VariabeleType, Waarde};

const MAG_ALLEEN_HELE_GETALLEN: bool = true;
const MAG_ALLEEN_POSITIEVE_GETALLEN: bool = true;

#[allow(clippy::upper_case_acronyms)]
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
    pub(super) fn verwacht_argumenten(&self) -> usize {
        #[allow(clippy::match_same_arms)]
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
impl Display for FunctieNaam {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
#[allow(clippy::upper_case_acronyms)]
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
            return Err(EcolFout::melding(EcolFoutVariant::VerkeerdAantalArgumenten(functienaam.to_string(), argumenten.len(), functienaam.verwacht_argumenten())));
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
}
pub(super) struct EigenFunctie {
    functie_omgeving: Box<EcolMachine>,
}

impl EigenFunctie {
    fn new() -> Self {
        EigenFunctie { functie_omgeving: Box::new(EcolMachine::new()) }
    }

    fn eigen_functie(functies: &mut HashMap<String, FunDef>, functie: &FunDef, argumenten: Vec<Waarde>, diepte: u16, lees_geheugen: &mut LeesGeheugen) -> Result<f32, EcolFout> {
        let max_diepte = 50u16;
        if diepte > max_diepte {
            return Err(EcolFout::melding(EcolFoutVariant::MaximumRecursieDiepte(max_diepte)));
        }
        let mut machine = Self::new();
        machine.functie_omgeving.stel_functie_diepte_in(diepte);
        if argumenten.len() != functie.parameters().len() {
            return Err(EcolFout::melding(EcolFoutVariant::VerkeerdAantalArgumenten("Functie".to_string(), functie.parameters().len(), argumenten.len())));
        }
        machine.functie_omgeving.laad_programma(functie.body());
        machine.functie_omgeving.laad_functies(functies);

        for (index, (param, argument)) in functie.parameters().iter()
            .zip(argumenten.iter())
            .enumerate()
        {
            let mut naam = param.clone();
            if geen_spaties(&naam).starts_with("RIJSYM") {
                naam = geen_spaties(&naam[6..]);

                if type_test(argument.type_van(), VariabeleType::Rijsym, &naam, index)? {
                    let (begin, einde) = argument.rij_haal_grenswaarden();
                    machine.functie_omgeving.var_reserveer_rijsym(&naam, begin, einde)?;
                }
            } else if geen_spaties(&naam).starts_with("RIJ") {
                naam = geen_spaties(&naam[3..]);

                if type_test(argument.type_van(), VariabeleType::Rij, &naam, index)? {
                    let (begin, einde) = argument.rij_haal_grenswaarden();
                    machine.functie_omgeving.var_reserveer_rij(&naam, begin, einde)?;
                }

            } else if argument.type_van() != Some(VariabeleType::Getal) {
                if type_test(argument.type_van(), VariabeleType::Getal, &naam, index)? {};
            }
            machine.functie_omgeving.var_schrijf_waarde(&naam, argument.clone())?;
        }

        let programma = machine.functie_omgeving.programma().clone();
        let mut current = 1;
        loop {

            let Some((&regelnummer, current_regel)) = programma.range(current..).next() else {
                return Err(EcolFout::melding(EcolFoutVariant::GeenFunEinde));
            };
            current = regelnummer + 1;
            let regel = Line::new(current, current_regel.clone());

            if let LineInhoud::FunEind { expressie, ..} = current_regel {
                return machine.functie_omgeving.solve_expression(expressie, lees_geheugen);
            } else {
                let (reply_option, nextline_option, whatsnext_option) = execute_all(&regel, &mut machine.functie_omgeving, &programma, Context::Functie, lees_geheugen, &mut |_| {})?;

                if let Some(r) = reply_option {
                    let reply: String = r;
                    return reply.parse::<f32>().map_err(|_| EcolFout::melding(EcolFoutVariant::OngeldigGetal("als uitkomst van de functie".to_string())));
                }

                if let Some(nextline) = nextline_option {
                    current = nextline;
                    continue;
                }

                if let Some(whatsnext) = whatsnext_option {
                    match whatsnext {
                        WhatsNext::Break => break,
                        WhatsNext::Continue => { },
                    }
                }
            }



        }

        Err(EcolFout::melding(EcolFoutVariant::FunctieZonderResultaat))
    }
}

impl EcolMachine {
    pub(super) fn execute_function(&mut self, lees_geheugen: &mut LeesGeheugen, functie: &Functie) -> Result<f32, EcolFout> {

        match functie {
            Functie::ABS { getal } => { let uitkomst = Self::execute_function_abs(*getal)?; Ok(uitkomst) },
            Functie::ARCTAN { getal } => { let uitkomst = Self::execute_function_arctan(*getal)?; Ok(uitkomst) },
            Functie::BOVIN { variabele_naam } => { let uitkomst = self.execute_function_bovin(variabele_naam)?; Ok(uitkomst) },
            Functie::COS { getal } => { let uitkomst = Self::execute_function_cos(*getal)?; Ok(uitkomst) },
            Functie::EXP { getal } => { let uitkomst = Self::execute_function_exp(*getal)?; Ok(uitkomst) },
            Functie::G { getal } => { let uitkomst = Self::execute_function_g(*getal)?; Ok(uitkomst) },
            Functie::GOK { laag, hoog } => { let uitkomst = self.execute_function_gok(*laag, *hoog)?; Ok(uitkomst) },
            Functie::GOKC { } => { let uitkomst = self.execute_function_gokc(); Ok(uitkomst) },
            Functie::LEES { } => { let uitkomst = self.execute_function_lees(lees_geheugen)?; Ok(uitkomst) },
            Functie::LEESSYM { } => { let uitkomst = self.execute_function_leessym(lees_geheugen)?; Ok(uitkomst) },
            Functie::LN { getal } => { let uitkomst = Self::execute_function_ln(*getal)?; Ok(uitkomst) },
            Functie::LOG { getal } => { let uitkomst = Self::execute_function_log(*getal)?; Ok(uitkomst) },
            Functie::ONDIN { variabele_naam } => { let uitkomst = self.execute_function_ondin(variabele_naam)?; Ok(uitkomst) },
            Functie::PS { } => { let uitkomst = self.execute_function_ps(); Ok(uitkomst) },
            Functie::SIN { getal } => { let uitkomst = Self::execute_function_sin(*getal)?; Ok(uitkomst) },
            Functie::WRTL { getal} => { let uitkomst = Self::execute_function_wrtl(*getal)?; Ok(uitkomst) },
        }
    }
    fn execute_function_abs(getal: f32) -> Result<f32, EcolFout> {
        let getal = grens_bewaking(getal, !MAG_ALLEEN_POSITIEVE_GETALLEN, !MAG_ALLEEN_HELE_GETALLEN)?;

        Ok(getal.abs())
    }
    fn execute_function_arctan(getal: f32) -> Result<f32, EcolFout> {
        let getal = grens_bewaking(getal, !MAG_ALLEEN_POSITIEVE_GETALLEN, !MAG_ALLEEN_HELE_GETALLEN)?;

        Ok(getal.atan())
    }
    fn execute_function_bovin(&self, variabele_naam: &str) -> Result<f32, EcolFout> {
        let Some(waarde) = self.var_lees_waarde(variabele_naam) else { return Err(EcolFout::melding(EcolFoutVariant::VariabeleNietGevonden(variabele_naam.to_string())));};
        match waarde.type_van() {
            Some(VariabeleType::Rij | VariabeleType::Rijsym) => {}
            _ => return Err(EcolFout::melding(EcolFoutVariant::GeenRij(variabele_naam.to_string()))),
        }
        let (_, result) = waarde.rij_haal_grenswaarden();
        Ok(result as f32)
    }
    fn execute_function_cos(getal: f32) -> Result<f32, EcolFout> {
        let getal = grens_bewaking(getal, !MAG_ALLEEN_POSITIEVE_GETALLEN, !MAG_ALLEEN_HELE_GETALLEN)?;

        Ok(getal.cos())
    }
    fn execute_function_exp(getal: f32) -> Result<f32, EcolFout> {
        let getal = grens_bewaking(getal, !MAG_ALLEEN_POSITIEVE_GETALLEN, !MAG_ALLEEN_HELE_GETALLEN)?;

        Ok(getal.exp())
    }
    fn execute_function_g(getal: f32) -> Result<f32, EcolFout> {
        let getal = grens_bewaking(getal, !MAG_ALLEEN_POSITIEVE_GETALLEN, !MAG_ALLEEN_HELE_GETALLEN)?;

        Ok(getal.floor())
    }
    fn execute_function_gok(&mut self, laag: f32, hoog: f32) -> Result<f32, EcolFout> {
        let laag = grens_bewaking(laag, !MAG_ALLEEN_POSITIEVE_GETALLEN, !MAG_ALLEEN_HELE_GETALLEN)?;
        let hoog = grens_bewaking(hoog, !MAG_ALLEEN_POSITIEVE_GETALLEN, !MAG_ALLEEN_HELE_GETALLEN)?;
        if laag >= hoog {
            return Err(EcolFout::melding(EcolFoutVariant::GrenzenVolgorde("GOK".to_string(), laag, hoog)));
        }

        Ok(self.volgende_willekeurig(laag, hoog).round())
    }
    fn execute_function_gokc(&mut self) -> f32 {
        self.volgende_willekeurig(0.0, 1.0)
    }
    fn execute_function_lees(&mut self, lees_geheugen: &mut LeesGeheugen) -> Result<f32, EcolFout> {
        if lees_geheugen.wacht_op_lees() {
            let Some(getal) = lees_geheugen.lees_waarde() else {
                return Err(EcolFout::melding(EcolFoutVariant::GeenWaarde("LEES".to_string())));
            };
            lees_geheugen.lees_hervat_none();
            Ok(getal)
        } else {
            Err(EcolFout::Signaal(EcolSignaal::WachtOpLees(0)))
        }
    }

    fn execute_function_leessym(&mut self, lees_geheugen: &mut LeesGeheugen) -> Result<f32, EcolFout> {
        if lees_geheugen.wacht_op_leessym() {
            let Some(getal) = lees_geheugen.leessym_waarde() else {
                return Err(EcolFout::melding(EcolFoutVariant::GeenWaarde("LEESSYM".to_string())));
            };
            lees_geheugen.leessym_hervat_none();

            Ok(getal)
        } else {
            Err(EcolFout::Signaal(EcolSignaal::WachtOpLeessym(0)))
        }
    }
    fn execute_function_ln(getal: f32) -> Result<f32, EcolFout> {
        let getal = grens_bewaking(getal, MAG_ALLEEN_POSITIEVE_GETALLEN, !MAG_ALLEEN_HELE_GETALLEN)?;

        Ok(getal.ln())
    }
    fn execute_function_log(getal: f32) -> Result<f32, EcolFout> {
        let getal = grens_bewaking(getal, MAG_ALLEEN_POSITIEVE_GETALLEN, !MAG_ALLEEN_HELE_GETALLEN)?;

        Ok(getal.log10())
    }
    fn execute_function_ondin(&self, variabele_naam: &str) -> Result<f32, EcolFout> {
        let Some(waarde) = self.var_lees_waarde(variabele_naam) else { return Err(EcolFout::melding(EcolFoutVariant::VariabeleNietGevonden(variabele_naam.to_string())));};
        match waarde.type_van() {
            Some(VariabeleType::Rij | VariabeleType::Rijsym) => {}
            _ => return Err(EcolFout::melding(EcolFoutVariant::GeenRij(variabele_naam.to_string()))),
        }
        let (result, _) = waarde.rij_haal_grenswaarden();
        Ok(result as f32)
    }

    fn execute_function_ps(&self) -> f32 {
        let werk = self.lees_regel();
        werk.len() as f32
    }
    fn execute_function_sin(getal: f32) -> Result<f32, EcolFout> {
        let getal = grens_bewaking(getal, !MAG_ALLEEN_POSITIEVE_GETALLEN, !MAG_ALLEEN_HELE_GETALLEN)?;

        Ok(getal.sin())
    }
    fn execute_function_wrtl(getal: f32) -> Result<f32, EcolFout> {
        let getal = grens_bewaking(getal, MAG_ALLEEN_POSITIEVE_GETALLEN, !MAG_ALLEEN_HELE_GETALLEN)?;

        Ok(getal.sqrt())
    }
    pub(super) fn execute_eigen_functie(&mut self, naam: &str, argumenten: Vec<Waarde>, lees_geheugen: &mut LeesGeheugen) -> Result<f32, EcolFout> {
        let definitie = self.haal_functiedefinitie(naam)
            .ok_or_else(|| EcolFout::melding(EcolFoutVariant::GeenFunctie(naam.to_string())))?
            .clone();
        let diepte = self.functie_diepte() + 1;
        EigenFunctie::eigen_functie(self.haal_functie_register(), &definitie, argumenten, diepte, lees_geheugen)
    }
    pub(super) fn get_fundef_parameters(&self, naam: &str) -> Option<Vec<String>> {
        let definitie = self.haal_functiedefinitie(naam)?;
        Some(definitie.parameters().clone())
    }
    pub(super) fn extract_functie_definities(&mut self, volledige_programma: &BTreeMap<u16,LineInhoud>) -> Result<BTreeMap<u16,LineInhoud>, EcolFout> {
        let mut nieuwe_programma: BTreeMap<u16, LineInhoud> = BTreeMap::new();
        let mut in_functie_definitie = false;
        let mut fundef = FunDef::new();
        let mut naam_van_functie: &str = "";

        for (regelnummer, regel) in volledige_programma {
            match regel {
                LineInhoud::FunStart { variabele_naam, argumenten, .. } => {
                    in_functie_definitie = true;
                    naam_van_functie = variabele_naam;
                    let parameters = argumenten_to_vec(argumenten);
                    fundef = FunDef::new();
                    fundef.set_parameters(&parameters);
                }
                LineInhoud::FunEind {  .. } => {
                    if in_functie_definitie {
                        fundef.body_insert(*regelnummer, regel);
                        self.schrijf_nieuwe_functie(naam_van_functie, &fundef)?;
                        naam_van_functie = "";
                        in_functie_definitie = false;
                    } else {
                        nieuwe_programma.insert(*regelnummer, regel.clone());
                    }
                }
                _ => {
                    if in_functie_definitie {
                        fundef.body_insert(*regelnummer, regel);
                    } else {
                        nieuwe_programma.insert(*regelnummer, regel.clone());
                    }
                }
            }
        }

        Ok(nieuwe_programma)
    }

}
fn grens_bewaking (getal: f32, alleen_positieve_getallen: bool, alleen_hele_getallen: bool) -> Result<f32, EcolFout> {
    if getal.fract() != 0.0 && alleen_hele_getallen {
        return Err(EcolFout::melding(EcolFoutVariant::GeenGeheelGetal(getal)));
    }
    if getal <= 0f32 && alleen_positieve_getallen {
        return Err(EcolFout::melding(EcolFoutVariant::GeenPositiefGetal(getal)));
    }

    Ok(getal)
}
fn type_test(type_: Option<VariabeleType>, verwacht: VariabeleType, naam: &str, index: usize) -> Result<bool, EcolFout> {
    if type_ != Some(verwacht) {
        if type_.is_none() {
            return Err(EcolFout::melding(EcolFoutVariant::OnbekendType(naam.to_string())))
        }
        return Err(EcolFout::melding(EcolFoutVariant::ArgumentVerkeerdType(
            "Functie".to_string(),
            verwacht,
            index + 1,
            type_.unwrap(), //Veilig te doen omdat we hier geen onbekend type kunnen krijgen.
        )));
    }
    Ok(true)
}