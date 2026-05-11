use std::collections::{BTreeMap, HashMap};
use std::f32;
use web_sys::js_sys;
use crate::interpreter::errors::{EcolFout, EcolFoutVariant, EcolWachtSignaal};
use crate::interpreter::helpers::{argumenten_to_vec, geen_spaties, is_geldige_variabele_naam, vind_opmerking};
pub(crate) use crate::interpreter::leesgeheugen::LeesGeheugen;
use crate::interpreter::opdrachten::{execute_all, Context};
use super::waarden::{VariabeleType, Waarde};
use super::program::{FunDef, Line, LineInhoud, Sleutelwoord, SprongDoel, SubDef, WORDT_TEKEN};

pub(super) const SYMBOLEN: [Option<char>; 100] = {
    let mut t = [None; 100];
    let mut teller = 0u8;
    loop {
        t[teller as usize] = Some((b'0' + teller) as char);
        teller += 1;
        if teller > 9 {
            break;
        }
    }
    loop {
        t[teller as usize] = Some((b'a' + (teller - 10)) as char);
        teller += 1;
        if teller > 35 {
            break;
        }
    }
    loop {
        t[teller as usize] = None;
        teller += 1;
        if teller > 39 {
            break;
        }
    }
    loop {
        t[teller as usize] = Some((b'A' + (teller - 40)) as char);
        teller += 1;
        if teller > 65 {
            break;
        }
    }
    loop {
        t[teller as usize] = None;
        teller += 1;
        if teller > 69 {
            break;
        }
    }
    t[70] = Some('.');
    t[71] = Some(',');
    t[72] = Some(':');
    t[73] = Some(';');
    t[74] = Some('?');
    t[75] = Some('!');
    t[76] = Some('\'');
    t[77] = Some('"');
    t[78] = Some('(');
    t[79] = Some(')');
    t[80] = Some('+');
    t[81] = Some('-');
    t[82] = Some('*');
    t[83] = Some('/');
    t[84] = Some('=');
    t[85] = Some('<');
    t[86] = Some('>');
    t[87] = Some('≤');
    t[88] = Some('≥');
    t[89] = Some('≠');
    t[90] = Some(' ');
    t[91] = Some('\n');

    teller = 92;
    loop {
        t[teller as usize] = None;
        teller += 1;
        if teller > 98 {
            break;
        }
    }
    t[99] = Some(']');


    t
};
fn symbolen_reverse(symbool: char) -> Option<u8> {
    let mut i = 0usize;

    loop {
        #[allow(clippy::cast_possible_truncation)]
        if let Some(s) = SYMBOLEN[i] {if s == symbool {return Some(i as u8)}} //Veilig, want SYMBOLEN[i] is al gecontroleerd.
        i += 1;
        if i > 99 {
            break;
        }
    }
    None
}
enum RegelBuffer {
    Regel(String),
}
impl RegelBuffer {
    fn new() -> Self {
        RegelBuffer::Regel(String::new())
    }
    fn naar_regel_buffer(&mut self, regel: &str) -> Result<(), EcolFout> {
        match self {
            RegelBuffer::Regel(r) => {
                if r.len() + regel.len() > 80 {
                    return Err(EcolFout::melding(EcolFoutVariant::BufferOverflow));
                }
                r.push_str(regel);
            },
        }
        Ok(())
    }

    fn lees_regel(&self) -> String {
        match self {
            RegelBuffer::Regel(r) => r.clone(),
        }
    }
    fn leeg_regel_buffer(&mut self) {
        match self {
            RegelBuffer::Regel(r) => r.clear(),
        }
    }
}
pub(super) struct VariabelenOpslag {
    symbolen: HashMap<String, usize>,
    data_pool: Vec<Waarde>,
}
impl VariabelenOpslag {
    fn new() -> Self {
        VariabelenOpslag {
            symbolen: HashMap::new(),
            data_pool: Vec::with_capacity(100),
        }
    }
    fn type_van(&self, naam: &str) -> Option<VariabeleType> {
        if let Some(index) = self.symbolen.get(naam) {
            self.data_pool[*index].type_van()
        } else {
            None
        }
    }
    fn reserveer_rij(&mut self, variabele_naam: &str, start: usize, eind: usize) -> Result<(), EcolFout> {
        let index = self.pak_of_maak_index(variabele_naam);
        self.data_pool[index]=Waarde::new_rij(start, eind)?;
        Ok(())
    }
    fn reserveer_rijsym(&mut self, variabele_naam: &str, start: usize, eind: usize) -> Result<(), EcolFout> {
        let index = self.pak_of_maak_index(variabele_naam);
        self.data_pool[index]=Waarde::new_rijsym(start, eind)?;
        Ok(())
    }
    fn wis_variabele(&mut self, variabele_naam: &str)  {
        if let Some(index) = self.symbolen.get(variabele_naam) {
            self.data_pool[*index] = Waarde::NogNietBepaald;
            self.symbolen.remove(variabele_naam);
        }
    }
    fn lees_waarde(&self, naam: &str) -> Option<Waarde> {
        if let Some(index) = self.symbolen.get(naam) {
            let waarde = self.data_pool[*index].clone();
            waarde.type_van()?;
            Some(waarde)
        } else {
            None
        }
    }

    fn schrijf_waarde(&mut self, naam: &str, waarde: Waarde) -> Result<(), EcolFout> {
        let doelwaarde: Waarde;
        if self.bestaat(naam) {
            let Some(dw) = self.lees_waarde(naam) else {
                return Err(EcolFout::melding(EcolFoutVariant::VariabeleNietGevonden(naam.to_string())));
            };
            if waarde.type_van().is_none() {
                return Err(EcolFout::melding(EcolFoutVariant::OnbekendType(naam.to_string())));
            }
            doelwaarde = dw;
            if doelwaarde.type_van().is_some()
                && doelwaarde.type_van() != waarde.type_van() {
                    return Err(EcolFout::melding(EcolFoutVariant::VerkeerdType(
                        naam.to_string(),
                        doelwaarde.type_van().unwrap(), //veilig, want is_some() is al gecontroleerd.
                        waarde.type_van().unwrap() //veilig, want is_some() is al gecontroleerd.
                    )));
            }
        } else if waarde.type_van() == Some(VariabeleType::Rij) || waarde.type_van() == Some(VariabeleType::Rijsym) {
            return Err(EcolFout::melding(EcolFoutVariant::RijNietGedefinieerd(naam.to_string())));
        }

        let index = self.pak_of_maak_index(naam);
        self.data_pool[index] = waarde;
        Ok(())
    }
    fn pak_of_maak_index(&mut self, naam: &str) -> usize {
        use std::collections::hash_map::Entry;
        let volgende_vrije_index = self.data_pool.len();
        match self.symbolen.entry(naam.to_string()) {
            Entry::Vacant(entry) => {
                self.data_pool.push(Waarde::NogNietBepaald); // placeholder
                entry.insert(volgende_vrije_index);
                volgende_vrije_index
            }
            Entry::Occupied(entry) => *entry.get(),
        }
    }
    fn bestaat(&self, naam: &str) -> bool {
        self.symbolen.contains_key(naam)
    }
}
pub(super) struct Programma {
    programma: BTreeMap<u16, LineInhoud>
}
impl Programma {
    fn new() -> Self {
        Self {
            programma: BTreeMap::new(),
        }
    }
    fn laad(&mut self, bron: &BTreeMap<u16, LineInhoud>) {
        self.programma = bron.clone();
    }
    fn programma(&self) -> &BTreeMap<u16, LineInhoud> {
        &self.programma
    }
    fn regel_toevoegen(&mut self, regel: &Line) -> String {
        let regelnummer = regel.regelnummer();
        let regel_inhoud = regel.inhoud().clone();

        let Some(oude_regel) = self.programma.insert(regelnummer, regel_inhoud) else {
            return String::new();
        };

        format!("{} // vervangen", Line::new(regelnummer, oude_regel))
    }

    fn regel_verwijderen(&mut self, regelnummer: u16) -> Option<String> {
        let regel_inhoud = self.programma.remove(&regelnummer)?;
        let regel=Line::new(regelnummer, regel_inhoud);

        Some(format!("{regel} // verwijderd"))
    }
}




pub struct EcolMachine {
    variabelen_opslag: VariabelenOpslag,
    regel_buffer: RegelBuffer,
    programma: Programma,
    actieve_tellers: Vec<String>,
    functie_register: HashMap<String, FunDef>,
    functie_recursie_diepte: u16,
    sub_register: HashMap<String, SubDef>,
    sub_return_stack: Vec<u16>,
    seed: u64,
}
impl EcolMachine {
    #[allow(clippy::new_without_default)] // seed is time-dependent, Default would be misleading
    #[must_use] 
    pub fn new() -> Self {
        EcolMachine {
            variabelen_opslag: VariabelenOpslag::new(),
            regel_buffer: RegelBuffer::new(),
            programma: Programma::new(),
            actieve_tellers: Vec::new(),
            functie_register: HashMap::new(),
            functie_recursie_diepte: 0,
            sub_register: HashMap::new(),
            sub_return_stack: Vec::new(),
            #[allow(clippy::cast_possible_truncation)]
            #[allow(clippy::cast_sign_loss)]
            seed: js_sys::Date::now() as u64, //Veilig, omdat deze waarde altijd kleiner is dan 2^53 en positief.
        }
    }
    pub(super) fn schrijf_subregister(&mut self, naam: &str, definitie: SubDef) -> Result<(), EcolFout> {

        if self.sub_register.insert(naam.to_string(), definitie).is_some() {
            return Err(EcolFout::melding(EcolFoutVariant::SubRoutineBestaatAl(naam.to_string())));
        }
        Ok(())
    }
    pub(super) fn lees_subregister(&self, naam: &str) -> Option<&SubDef> {
        self.sub_register.get(naam)
    }
    pub(super) fn is_sub(&self, naam: &str) -> bool {
        self.sub_register.contains_key(naam)
    }
    pub(super) fn start_sub(&mut self, naam: &str, regelnummer: u16) -> Result<(), EcolFout> {
        if !self.is_sub(naam) {
            return Err(EcolFout::melding(EcolFoutVariant::OnbekendeSubroutine(naam.to_string())));
        }
        self.sub_return_stack.push(regelnummer);
        Ok(())
    }

    pub(super) fn var_type_van(&self, naam: &str) -> Option<VariabeleType> {
        self.variabelen_opslag.type_van(naam)
    }
    pub(super) fn var_lees_waarde(&self, naam: &str) -> Option<Waarde> {
        self.variabelen_opslag.lees_waarde(naam)
    }
    pub(super) fn var_schrijf_waarde(&mut self, naam: &str, waarde: Waarde) -> Result<(), EcolFout> {
        if self.is_fun(naam) {
            return Err(EcolFout::melding(EcolFoutVariant::NaamIsFun(naam.to_string())));
        }
        if self.is_sub(naam) {
            return Err(EcolFout::melding(EcolFoutVariant::NaamIsSubroutine(naam.to_string())))
        }
        self.variabelen_opslag.schrijf_waarde(naam, waarde)
    }
    pub(super) fn var_reserveer_rij(&mut self, variabele_naam: &str, start: usize, eind: usize) -> Result<(),EcolFout> {
        self.variabelen_opslag.reserveer_rij(variabele_naam, start, eind)
    }
    pub(super) fn var_reserveer_rijsym(&mut self, variabele_naam: &str, start: usize, eind: usize) -> Result<(), EcolFout> {
        self.variabelen_opslag.reserveer_rijsym(variabele_naam, start, eind)
    }
    pub(super) fn var_wis(&mut self, variabele_naam: &str) {
        self.variabelen_opslag.wis_variabele(variabele_naam);
    }
    pub(super) fn teller_nieuw(&mut self, naam: &str, stap: f32, start: f32, einde: f32, regel: u16) -> Result<Option<()>, EcolFout> {
        if (stap > 0.0 && start > einde) || (stap < 0.0 && start < einde) {
            return Ok(None)
        }
        if stap == 0.0 {
            return Err(EcolFout::melding(EcolFoutVariant::StapgrootteNul))
        }

        let var_typ = self.var_type_van(naam);
        match var_typ {
            Some(VariabeleType::Getal) | None => {
                if var_typ.is_some() {
                    self.var_wis(naam);
                }
                let mut teller = Waarde::new_teller(stap, start, einde);
                teller.teller_schrijf_regel(regel)?;
                self.var_schrijf_waarde(naam, teller)?;
                self.actieve_tellers.push(naam.to_string());

                Ok(Some(()))
            },
            Some(v_type) => Err(EcolFout::melding(EcolFoutVariant::TellerBestaatAl(naam.to_string(), v_type))),
        }
    }
    pub(super) fn teller_herhaal(&mut self) -> Result<Option<u16>, EcolFout> {
        let naam = self.actieve_tellers.last()
            .ok_or_else(|| EcolFout::melding(EcolFoutVariant::HerhaalZonderMet))?
            .clone();

        let Some(mut teller) = self.var_lees_waarde(&naam) else {
            return Err(EcolFout::melding(EcolFoutVariant::OnbekendeTeller(naam)))
        };

        let regel = teller.teller_lees_regel()?;
        let stap = teller.teller_lees_stap()?;
        let current = teller.haal_getal(0usize)?;
        let new_current = current + stap;

        if teller.teller_is_klaar(new_current)? {
            self.actieve_tellers.pop();
            self.var_wis(&naam);
            self.var_schrijf_waarde(&naam, Waarde::new_getal(new_current))?;
            Ok(None)
        } else {
            teller.teller_schrijf_current(new_current)?;
            self.var_schrijf_waarde(&naam, teller)?;
            Ok(Some(regel))
        }

    }
    pub(super) fn heeft_open_tellers(&self) -> bool {
        !self.actieve_tellers.is_empty()
    }
    pub(super) fn naar_regel_buffer(&mut self, regel: &str) -> Result<(), EcolFout>{
        self.regel_buffer.naar_regel_buffer(regel)?;

        Ok(())
    }
    pub(super) fn lees_regel(&self) -> String {
        self.regel_buffer.lees_regel()
    }
    pub(super) fn leeg_regel_buffer(&mut self) {
        self.regel_buffer.leeg_regel_buffer();
    }
    pub(super) fn programma(&self) -> &BTreeMap<u16, LineInhoud> {
        self.programma.programma()
    }
    pub(super) fn laad_programma(&mut self, bron: &BTreeMap<u16, LineInhoud>) {
        self.programma.laad(bron);
    }
    pub(super) fn laad_functies(&mut self, bron: &HashMap<String, FunDef>) {
        self.functie_register.clone_from(bron);
    }
    pub(super) fn haal_functiedefinitie(&self, naam: &str) -> Option<&FunDef> {
        self.functie_register.get(naam)
    }
    pub(super) fn schrijf_nieuwe_functie(&mut self, naam: &str, fundef: &FunDef) -> Result<(), EcolFout>{
        if self.functie_register.contains_key(naam) {
            return Err(EcolFout::melding(EcolFoutVariant::FunctieBestaatAl(naam.to_string())));
        }
        self.functie_register.insert(naam.to_string(), fundef.clone());
        Ok(())
    }
    pub(super) fn functie_diepte(&self) -> u16 {
        self.functie_recursie_diepte
    }
    pub(super) fn stel_functie_diepte_in(&mut self, diepte: u16) {
        self.functie_recursie_diepte = diepte;
    }
    pub(super) fn haal_functie_register(&mut self) -> &mut HashMap<String,FunDef>
    {
        &mut self.functie_register
    }

    pub(super) fn is_fun(&self, naam: &str) -> bool {
        self.functie_register.contains_key(naam)
    }
    pub(super) fn volgende_willekeurig(&mut self, laag: f32, hoog: f32) -> f32 {
        self.seed ^= self.seed << 13;
        self.seed ^= self.seed >> 7;
        self.seed ^= self.seed << 17;
        #[allow(clippy::cast_precision_loss)] // f32 heeft 23 bits mantisse; voor willekeurige getallen is dat voldoende
        let basis = (self.seed as f32) / (u64::MAX as f32);  // getal tussen 0.0 en 1.0
        basis * (hoog - laag) + laag
    }
    pub fn execute(&mut self, input: &str, lees_geheugen: &mut LeesGeheugen, output: &mut dyn FnMut(&str)) -> String {
        match self.execute_intern(input, lees_geheugen, output) {
            Ok(Some(s)) => s,
            Ok(None) => String::new(),
            Err(e) => e.to_string(),

        }
    }

    fn execute_intern(&mut self, input: &str, lees_geheugen: &mut LeesGeheugen, output: &mut dyn FnMut(&str)) -> Result<Option<String>, EcolFout> {

        let reply: Option<String> = if lees_geheugen.wacht_op_lees() {
            let waarde = input.trim().parse::<f32>();
            if let Ok(waarde) = waarde {
                lees_geheugen.schrijf_lees_waarde(waarde);
                let Some(regel) = lees_geheugen.lees_hervat_bij() else {
                    return Err(EcolFout::melding(EcolFoutVariant::InvoerFout));
                };
                self.hervat_uitvoering(regel, lees_geheugen, output)
            } else {
                return Err(EcolFout::melding(EcolFoutVariant::InvoerNietNumeriek));
            }
        } else if lees_geheugen.wacht_op_leessym() {
            //let waarde_input = input;
            if input.len() != 1 {
                return Err(EcolFout::melding(EcolFoutVariant::InvoerLengteNietNul));
            }
            let waarde_char = input.chars().next().unwrap_or_default();
            let waarde = symbolen_reverse(waarde_char);
            match waarde {
                Some(c) => {
                    lees_geheugen.schrijf_leessym_waarde(f32::from(c))?;
                    let Some(regel) = lees_geheugen.leessym_hervat_bij() else {
                        return Err(EcolFout::melding(EcolFoutVariant::InvoerFout));
                    };
                    self.hervat_uitvoering(regel, lees_geheugen, output)
                },
                None => return Err(EcolFout::melding(EcolFoutVariant::InvoerGeenSymbool)),
            }
        } else {
            match parseer_regel(input){
                Ok(regel) => {
                    if regel.regelnummer() == 0 {
                        let programma = BTreeMap::new();
                        match execute_all(&regel, self, &programma, &Context::Direct, lees_geheugen, output) {
                            Ok((r, _, _)) => r,
                            Err(EcolFout::FoutMelding(e)) => return Err(EcolFout::FoutMelding(e)),
                            Err(EcolFout::Signaal(s)) => {
                                match s {
                                    EcolWachtSignaal::Lees(r) => {
                                        if r == 0 {
                                            return Err(EcolFout::melding(EcolFoutVariant::AlleenInProgramma("LEES".to_string())));
                                        }
                                        lees_geheugen.lees_hervat_bij_op_regel(r);
                                        None
                                    },
                                    EcolWachtSignaal::Leessym(r) => {
                                        if r == 0 {
                                            return Err(EcolFout::melding(EcolFoutVariant::AlleenInProgramma("LEESSYM".to_string())));
                                        }
                                        lees_geheugen.leessym_hervat_bij_op_regel(r);
                                        None
                                    },
                                    EcolWachtSignaal::Laad => {
                                        lees_geheugen.stel_laad_in();
                                        None
                                    }
                                }
                            },
                        }
                    } else if matches!(regel.inhoud(), LineInhoud::Verwijderen {}) {
                        let verwijder_resultaat = self.programma.regel_verwijderen(regel.regelnummer());
                        Some(verwijder_resultaat.unwrap_or("Geen regel om te verwijderen.".to_string()))
                    } else {
                        Some(self.programma.regel_toevoegen(&regel))
                    }
                }
                Err(e) => {
                    return Err(EcolFout::melding(EcolFoutVariant::OngeldigeInvoer(e.to_string())));
                }
            }
        };
        Ok(Some(reply.unwrap_or_default()))
    }
    #[allow(clippy::missing_errors_doc)]
    pub fn execute_direct(&mut self, input: &str, _output: &mut dyn FnMut(&str)) -> Result<String, String> {
        let mut lees_geheugen = LeesGeheugen::new();
        let mut output_buffer = String::new();
        match self.execute_intern(input, &mut lees_geheugen, &mut |s| output_buffer.push_str(s)) {
            Ok(Some(reply)) => {
                if output_buffer.is_empty() { Ok(reply) } else { Ok(output_buffer) }
            },
            Ok(None) => Ok(output_buffer),
            Err(e) => Err(e.to_string()),
        }

    }
    fn hervat_uitvoering(&mut self, regel: u16, lees_geheugen: &mut LeesGeheugen, output: &mut dyn FnMut(&str)) -> Option<String> {
        match self.execute_start(Some(regel), lees_geheugen, output) {
            Ok(r) => Some(r),
            Err(EcolFout::FoutMelding(e)) => Some(e.to_string()),
            Err(EcolFout::Signaal(_)) => None,
        }
    }

}
fn extract_als(input: &str) -> Option<(&str, &str)> {
    let werkstring = input.trim_start();
    let pos = werkstring.find("DAN");
    match pos {
        Some(positie) => {
            let vergelijking = &werkstring[..positie];
            let rest = &werkstring[positie + 3..];
            Some((vergelijking.trim(), rest.trim_start()))
        }
        None => None,
    }
}
pub(super) fn extract_anders(input: &str) -> Result<(SprongDoel, &str), EcolFout> {
    let werkstring = geen_spaties(input);

    let anders = SprongDoel::vul(&werkstring)?;

    Ok((anders, ""))
}
fn extract_argumenten(input: &str) -> Option<(Vec<String>, &str)> {
    let werkstring = input.trim_start();

    let (mut inhoud, rest) = werkstring.split_once(')')?;
    inhoud = inhoud.strip_prefix('(')?;

    let reply = argumenten_to_vec(inhoud);
    Some((reply, rest.trim_start()))
}
pub(super) fn extract_dan(input: &str) -> Result<(SprongDoel, &str), EcolFout> {
    let werkstring = input.trim_start();
    let pos = werkstring.find("ANDERS");
    let dan_reply: SprongDoel;

    if let Some(positie) = pos {
        let dan = &werkstring[..positie];
        let rest = &werkstring[positie + 6..];

        dan_reply = SprongDoel::vul(&geen_spaties(dan))?;

        Ok((dan_reply, rest.trim_start()))
    } else {
        let dan = &geen_spaties(werkstring);
        dan_reply = SprongDoel::vul(dan)?;

        Ok((dan_reply, ""))
    }
}
fn extract_keyword(input: &str) -> Option<(Sleutelwoord, &str)> {
    let werkstring = input.trim_start();

    if werkstring.chars().next().is_some_and(|c| c.is_ascii_lowercase()) {
        let (variabele, rest_na_variabele) = extract_variabele_naam(werkstring).unwrap_or(("", werkstring.trim_start()));
        if variabele.is_empty() { return None }
        if geen_spaties(rest_na_variabele).is_empty() { return Some((Sleutelwoord::GASUB, variabele)); }

        return Some((Sleutelwoord::TOEKENNEN, rest_na_variabele.trim_start()));
    }
    let position = werkstring.find(|c: char| !c.is_ascii_uppercase()).unwrap_or(werkstring.len());
    let (keyword_string, rest) = werkstring.split_at(position);

    let resultaat: Sleutelwoord = if keyword_string.trim_start().starts_with("FUN") {
        if rest.trim_start().chars().next().is_some_and(|c| c.is_ascii_lowercase()) {
            Sleutelwoord::FUNstart
        } else {
            Sleutelwoord::FUNeind
        }
    } else {
        Sleutelwoord::from_string(keyword_string.trim_start())?
    };


    Some((resultaat, rest.trim_start()))
}
fn extract_opmerking(input: &str) -> (String, String) {
    let deel_voor_opmerking: String;
    let opmerking: String;

    let rp = input.find(';');
    if let Some(p) = rp {
        deel_voor_opmerking = input[..p].trim_start().to_string();
        opmerking = input[p..].to_string();
    } else {
        deel_voor_opmerking = input.trim_start().to_string();
        opmerking = String::new();
    }

    (deel_voor_opmerking, opmerking)
}
fn extract_regelnummer(input: &str) -> Result<(u16, &str, bool), EcolFout> {
    let is_alleen_regelnummer: bool = input.trim().chars().all(|c| c.is_ascii_digit());

    let (nummer, restregel) = if is_alleen_regelnummer {
        (input.trim(), "")
    } else if let Some(positie) = input.find(|c: char| c.is_ascii_alphabetic() || c == ':') {
        let (gevonden_nummer, rest) = input.split_at(positie);
        (gevonden_nummer, rest.trim_start())
    } else {
        ("0", input.trim_start())
    };

    let Some(resultaat_parsing) = nummer.trim().parse::<u16>().ok() else { return Ok((0u16, input.trim_start(), is_alleen_regelnummer)) };
    let regelnummer = resultaat_parsing;
    if regelnummer > 999u16 { return Err(EcolFout::melding(EcolFoutVariant::RegelnummerTeGroot)) }


    Ok((regelnummer, restregel, is_alleen_regelnummer))
}
fn extract_stap_expressie(input: &str) -> Result<(String, String), EcolFout> {
    let werkstring = geen_spaties(input);

    match vind_top_level_komma(&werkstring) {
        Some(p) => {
            let (stap_tekst, r) = werkstring.split_at(p);
            Ok((stap_tekst.to_string(), r[1..].to_string()))
        },
        None => Err(EcolFout::melding(EcolFoutVariant::OngeldigeSyntaxMet("STAP".to_string()))),
    }
}
fn extract_start_expressie(input: &str) -> Result<(String, String, String), EcolFout> {
    let werkstring = geen_spaties(input);

    match vind_top_level_komma(&werkstring) {
        Some(p) => {
            let (toekenning_string, r) = werkstring.split_at(p);
            let Some((naam, rest_na_variabele)) = extract_variabele_naam(toekenning_string) else { return Err(EcolFout::melding(EcolFoutVariant::OngeldigeVariabeleNaam)) };
            let Some(rest_na_wordt_teken) = is_geldig_wordt_teken(rest_na_variabele) else { return Err(EcolFout::melding(EcolFoutVariant::OngeldigWordtTeken)) };
            Ok((naam.to_string(), rest_na_wordt_teken.to_string(), r[1..].to_string()))
        },
        None => Err(EcolFout::melding(EcolFoutVariant::OngeldigeSyntaxMet("START".to_string()))),
    }
}
fn extract_variabele_naam(input: &str) -> Option<(&str, &str)> {
    let werkstring = input.trim_start();
    let position = werkstring.find(|c: char| !c.is_ascii_lowercase() && !c.is_ascii_digit() && c != '_').unwrap_or(werkstring.len());
    let (variabele_naam, rest) = werkstring.split_at(position);

    if is_geldige_variabele_naam(variabele_naam.trim()) {

        Some((variabele_naam.trim(), rest.trim_start()))
    } else {
        None
    }

}
fn is_alleen_keyword(input: &str, regelnummer: u16, keyword: Sleutelwoord) -> Result<Line, EcolFout> {
    Ok(Line::new(regelnummer, LineInhoud::from_sleutelwoord(keyword, input)?))
}
fn is_geldig_wordt_teken(input: &str) -> Option<&str> {
    let rest = input.strip_prefix(WORDT_TEKEN)?;
    Some(rest.trim_start())
}
#[allow(clippy::too_many_lines)]
fn parseer_regel(input: &str) -> Result<Line, EcolFout> {
    let (regelnummer, rnn, is_alleen_regelnummer) = extract_regelnummer(input)?;
    if is_alleen_regelnummer {
        return Ok(Line::new(regelnummer, LineInhoud::Verwijderen {}));
    }
    let rest_na_regelnummer: &str = rnn;
    if rest_na_regelnummer.trim_start().starts_with(':') {
        let rest_na_dubbele_punt = rest_na_regelnummer.trim_start().trim_start_matches(':');

        return Ok(Line::new(regelnummer, LineInhoud::LegeRegel { opm: vind_opmerking(rest_na_dubbele_punt)? }));
    }

    let Some((keyword, rest_na_keyword)) = extract_keyword(rest_na_regelnummer) else {
        return Err(EcolFout::melding(EcolFoutVariant::OnbekendSleutelwoord))
    };

    match keyword {
        Sleutelwoord::ALS => {
            if regelnummer != 0 {
                let Some((vergelijking_str, rest_na_als)) = extract_als(rest_na_keyword) else {
                    return Err(EcolFout::melding(EcolFoutVariant::GeenDan))
                };
                let vergelijking = vergelijking_str.to_string();
                let (dan, rest_na_dan) = extract_dan(rest_na_als)?;
                let anders: Option<SprongDoel>;
                let rest_tekst: &str;
                if geen_spaties(rest_na_dan).is_empty() || rest_na_dan.trim_start().starts_with(';') {
                    rest_tekst = rest_na_dan;
                    anders = None;
                } else {
                    let (anders_getal, rest) = extract_anders(rest_na_dan)?;
                    rest_tekst = rest;
                    anders = Some(anders_getal);
                }

                Ok(Line::new(regelnummer, LineInhoud::Als { vergelijking, dan, anders, opm: vind_opmerking(rest_tekst)? }))
            } else {
                Err(EcolFout::melding(EcolFoutVariant::AlleenInProgramma("ALS".to_string())))
            }

        },
        Sleutelwoord::END => {
            if regelnummer == 0 {
                return Err(EcolFout::melding(EcolFoutVariant::AlleenInProgramma("END".to_string())))
            }

            Ok(Line::new(regelnummer, LineInhoud::End { opm: vind_opmerking(rest_na_keyword)?}))
        },
        Sleutelwoord::FUNstart => {
            if regelnummer == 0 {
                return Err(EcolFout::melding(EcolFoutVariant::AlleenInProgramma("FUN definitie".to_string())))
            }
            let Some((variabele_naam, rest_na_variabele)) = extract_variabele_naam(rest_na_keyword) else {
                return Err(EcolFout::melding(EcolFoutVariant::GeenVariabeleNaam))
            };
            let parameters = rest_na_variabele.trim().trim_start_matches('(').trim_end_matches(')');
            let rest_na_parameters = rest_na_variabele
                .find(')')
                .map_or("", |i| &rest_na_variabele[i + 1..]);


            Ok(Line::new(regelnummer, LineInhoud::FunStart { variabele_naam: variabele_naam.to_string() , argumenten: parameters.to_string(), opm: vind_opmerking(rest_na_parameters)? }))
        },
        Sleutelwoord::FUNeind => {
            if regelnummer == 0 {
                return Err(EcolFout::melding(EcolFoutVariant::AlleenInProgramma("FUN :=".to_string())))
            }
            let Some(rest_na_wordt_teken) = is_geldig_wordt_teken(rest_na_keyword) else {
                return Err(EcolFout::melding(EcolFoutVariant::OngeldigWordtTeken))
            };
            let (expressie, rest_na_expressie) = extract_opmerking(rest_na_wordt_teken);

            Ok(Line::new(regelnummer, LineInhoud::FunEind { expressie: geen_spaties(&expressie), opm: vind_opmerking(&rest_na_expressie)? }))
        }
        Sleutelwoord::GASUB => {
            if regelnummer == 0 {
                return Err(EcolFout::melding(EcolFoutVariant::AlleenInProgramma("Een subroutine aanroepen".to_string())))
            }
            let (subroutine, rest_na_subroutine) = extract_opmerking(rest_na_keyword);

            Ok(Line::new(regelnummer, LineInhoud::GaSub { sub_naam: geen_spaties(&subroutine), opm: vind_opmerking(&rest_na_subroutine)? }))
        }
        Sleutelwoord::HELP => {
            if regelnummer == 0 {
                is_alleen_keyword(rest_na_keyword, regelnummer, keyword)
            } else {
                Err(EcolFout::melding(EcolFoutVariant::NietInProgramma("HELP".to_string())))
            }
        },
        Sleutelwoord::HERHAAL => {
            if regelnummer != 0 {
                is_alleen_keyword(rest_na_keyword, regelnummer, keyword)
            } else {
                Err(EcolFout::melding(EcolFoutVariant::AlleenInProgramma("HERHAAL".to_string())))
            }
        },
        Sleutelwoord::KLAAR => {
            if regelnummer != 0 {
                is_alleen_keyword(rest_na_keyword, regelnummer, keyword)
            } else {
                Err(EcolFout::melding(EcolFoutVariant::AlleenInProgramma("KLAAR".to_string())))
            }
        },
        Sleutelwoord::LIJST => {
            if regelnummer == 0 {
                is_alleen_keyword(rest_na_keyword, regelnummer, keyword)
            } else {
                Err(EcolFout::melding(EcolFoutVariant::NietInProgramma("LIJST".to_string())))
            }
        },
        Sleutelwoord::MET => {
            if regelnummer == 0 {
                return Err(EcolFout::melding(EcolFoutVariant::AlleenInProgramma("MET".to_string())))
            }
            let (stap_expressie, rest_na_stap) = extract_stap_expressie(rest_na_keyword)?;
            let (variabele_naam, start_expressie, rest_na_start) = extract_start_expressie(&rest_na_stap)?;
            let (einde_expressie, opm) = extract_opmerking(&rest_na_start);

            Ok(Line::new(regelnummer, LineInhoud::Met{ variabele_naam, stap_expressie, start_expressie, stop_expressie: einde_expressie, opm: vind_opmerking(&opm)? }))
        },
        Sleutelwoord::NAAR => {
            let (sprong_doel, opmerking) = extract_opmerking(rest_na_keyword);
            Ok(Line::new(regelnummer, LineInhoud::Naar{ sprong_doel: SprongDoel::vul(&sprong_doel)?, opm: vind_opmerking(&opmerking)? }))
        },
        Sleutelwoord::NP => {
            is_alleen_keyword(rest_na_keyword, regelnummer, keyword)
        },
        Sleutelwoord::NR => {
            let (mut argumenten, rest_na_argumenten) = extract_argumenten(rest_na_keyword).unwrap_or( (Vec::new(), rest_na_keyword));
            if argumenten.len() != 1 && !argumenten.is_empty() {
                return Err(EcolFout::melding(EcolFoutVariant::VerkeerdAantalArgumentenDubbel("NR".to_string(), "geen of één".to_string(), argumenten.len())))
            }
            if argumenten.is_empty() {
                argumenten.push("1".to_string());
            }

            Ok(Line::new(regelnummer, LineInhoud::NR{ aantal: argumenten[0].clone(), opm: vind_opmerking(rest_na_argumenten)? }))
        },
        Sleutelwoord::RIJ => {
            let (argumenten, rest_na_argumenten) = extract_argumenten(rest_na_keyword).unwrap_or( (Vec::new(), rest_na_keyword));
            if argumenten.len() != 2 {
                return Err(EcolFout::melding(EcolFoutVariant::VerkeerdAantalArgumenten("RIJ".to_string(), 2, argumenten.len())))
            }

            let Some((naam, rest_na_variabele)) = extract_variabele_naam(rest_na_argumenten) else {
                return Err(EcolFout::melding(EcolFoutVariant::GeenVariabeleNaam))
            };

            Ok(Line::new(regelnummer, LineInhoud::Rij{ start: argumenten[0].clone(), eind: argumenten[1].clone(), variabele_naam: naam.to_string(), opm: vind_opmerking(rest_na_variabele)? }))
        },
        Sleutelwoord::RIJSYM => {
            let (argumenten, rest_na_argumenten) = extract_argumenten(rest_na_keyword).unwrap_or( (Vec::new(), rest_na_keyword));
            if argumenten.len() != 2 {
                return Err(EcolFout::melding(EcolFoutVariant::VerkeerdAantalArgumenten("RIJSYM".to_string(), 2, argumenten.len())))
            }

            let Some((naam, rest_na_variabele)) = extract_variabele_naam(rest_na_argumenten) else {
                return Err(EcolFout::melding(EcolFoutVariant::GeenVariabeleNaam))
            };

            Ok(Line::new(regelnummer, LineInhoud::Rijsym{ start: argumenten[0].clone(), eind: argumenten[1].clone(), variabele_naam: naam.to_string(), opm: vind_opmerking(rest_na_variabele)? }))
        }
        Sleutelwoord::TOEKENNEN => {
            let Some((variabele_naam, rest_na_variabele)) = extract_variabele_naam(rest_na_regelnummer) else {
                return Err(EcolFout::melding(EcolFoutVariant::GeenVariabeleNaam))
            };
            let (argumenten, rest_na_argumenten) = extract_argumenten(rest_na_variabele).unwrap_or( (Vec::new(), rest_na_variabele));
            let argument: String;
            if argumenten.is_empty() {
                argument = String::new();
            } else if argumenten.len() == 1 {
                argument = argumenten[0].clone();
            } else {
                return Err(EcolFout::melding(EcolFoutVariant::VerkeerdAantalArgumenten("Een RIJ variabele".to_string(), 1, argumenten.len())));
            }
            let Some(rest_na_wordt_teken) = is_geldig_wordt_teken(rest_na_argumenten) else {
                return Err(EcolFout::melding(EcolFoutVariant::OngeldigWordtTeken))
            };
            let (expressie, opmerking) = extract_opmerking(rest_na_wordt_teken);
            Ok(Line::new(regelnummer, LineInhoud::Toekennen{ variabele_naam: variabele_naam.to_string(), argument, expressie, opm: vind_opmerking(&opmerking)? }))
        },
        Sleutelwoord::TEKST => {
            let Some(rest_na_wordt_teken) = is_geldig_wordt_teken(rest_na_keyword) else {
                return Err(EcolFout::melding(EcolFoutVariant::OngeldigWordtTeken))
            };
            let (expressie, opmerking) = extract_opmerking(rest_na_wordt_teken);

            Ok(Line::new(regelnummer, LineInhoud::Tekst{ expressie, opm: vind_opmerking(&opmerking)? }))
        },
        Sleutelwoord::SCHRIJF => {
            let (argumenten, rest_na_argumenten) = extract_argumenten(rest_na_keyword).unwrap_or( (Vec::new(), rest_na_keyword));
            if argumenten.len() != 2 && !argumenten.is_empty() {
                return Err(EcolFout::melding(EcolFoutVariant::VerkeerdAantalArgumentenDubbel("SCHRIJF".to_string(), "geen of 2".to_string(), argumenten.len())))
            }
            let breedte: usize;
            let decimalen: usize;
            if argumenten.is_empty() {
                breedte = 0;
                decimalen = 0;
            } else {
                breedte = argumenten[0].trim().parse::<usize>().map_err(|_| EcolFout::melding(EcolFoutVariant::OngeldigGetal("als breedte bij SCHRIJF".to_string())))? ;
                decimalen = argumenten[1].trim().parse::<usize>().map_err(|_| EcolFout::melding(EcolFoutVariant::OngeldigGetal("als aantal decimalen bij SCHRIJF".to_string())))? ;
                if breedte == 0 {
                    return Err(EcolFout::melding(EcolFoutVariant::GetalNietGroterDan("De breedte bij SCHRIJF".to_string(), 0)));
                }
            }

            let Some(rest_na_wordt_teken) = is_geldig_wordt_teken(rest_na_argumenten) else {
                return Err(EcolFout::melding(EcolFoutVariant::OngeldigWordtTeken))
            };

            let (expressie, opmerking) = extract_opmerking(rest_na_wordt_teken);

            Ok(Line::new(regelnummer, LineInhoud::Schrijf{ breedte, decimalen, expressie, opm: vind_opmerking(&opmerking)? }))
        },
        Sleutelwoord::SCHRIJFSYM => {
            let Some(rest_na_wordt_teken) = is_geldig_wordt_teken(rest_na_keyword) else {
                return Err(EcolFout::melding(EcolFoutVariant::OngeldigWordtTeken))
            };
            let (expressie, opmerking) = extract_opmerking(rest_na_wordt_teken);

            Ok(Line::new(regelnummer, LineInhoud::Schrijfsym{ expressie, opm: vind_opmerking(&opmerking)? }))
        },
        Sleutelwoord::SCHRIJM => {
            let Some(rest_na_wordt_teken) = is_geldig_wordt_teken(rest_na_keyword) else {
                return Err(EcolFout::melding(EcolFoutVariant::OngeldigWordtTeken))
            };
            let (expressie, opmerking) = extract_opmerking(rest_na_wordt_teken);

            Ok(Line::new(regelnummer, LineInhoud::Schrijm{ expressie, opm: vind_opmerking(&opmerking)? }))
        },
        Sleutelwoord::SPATIE => {
            let (mut argumenten, rest_na_argumenten) = extract_argumenten(rest_na_keyword).unwrap_or( (Vec::new(), rest_na_keyword));
            if argumenten.len() != 1 && !argumenten.is_empty() {
                return Err(EcolFout::melding(EcolFoutVariant::VerkeerdAantalArgumentenDubbel("SPATIE".to_string(), "geen of één".to_string(), argumenten.len())))
            }
            if argumenten.is_empty() {
                argumenten.push("1".to_string());
            }

            Ok(Line::new(regelnummer, LineInhoud::Spatie{ aantal: argumenten[0].clone(), opm: vind_opmerking(rest_na_argumenten)? }))
        },
        Sleutelwoord::BEWAAR => {
            if regelnummer == 0 {
                is_alleen_keyword(rest_na_keyword, regelnummer, keyword)
            } else {
                Err(EcolFout::melding(EcolFoutVariant::NietInProgramma("BEWAAR".to_string())))
            }
        },
        Sleutelwoord::LAAD => {
            if regelnummer == 0 {
                is_alleen_keyword(rest_na_keyword, regelnummer, keyword)
            } else {
                Err(EcolFout::melding(EcolFoutVariant::NietInProgramma("LAAD".to_string())))
            }
        },
        Sleutelwoord::START => {
            if regelnummer == 0 {
                is_alleen_keyword(rest_na_keyword, regelnummer, keyword)
            } else {
                Err(EcolFout::melding(EcolFoutVariant::NietInProgramma("START".to_string())))
            }
        },
        Sleutelwoord::SUB => {
            if regelnummer == 0 {
                return Err(EcolFout::melding(EcolFoutVariant::AlleenInProgramma("SUB".to_string())))
            }
            let Some((variabele_naam, rest_na_variabele)) = extract_variabele_naam(rest_na_keyword) else {
                return Err(EcolFout::melding(EcolFoutVariant::GeenVariabeleNaam))
            };

            Ok(Line::new(regelnummer, LineInhoud::Sub { sub_naam: geen_spaties(variabele_naam), opm: vind_opmerking(rest_na_variabele)? }))
        }
    }
}
fn vind_top_level_komma(s: &str) -> Option<usize> {
    let mut diepte = 0usize;
    for (i, c) in s.char_indices() {
        match c {
            '(' => diepte += 1,
            ')' => diepte -= 1,
            ',' if diepte == 0 => return Some(i),
            _ => {}
        }
    }
    None
}



