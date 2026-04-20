pub(super) const HELP_PAGINA: &str = concat!("ecol_syntaxis.html?v=", env!("CARGO_PKG_VERSION"));
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
pub(super) fn symbolen_reverse(symbool: char) -> Option<u8> {
    let mut i = 0usize;

    loop {
        match SYMBOLEN[i] {
            Some(s) => {if s == symbool {return Some(i as u8)}},
            None => { },
        }
        i += 1;
        if i > 99 {
            break;
        }
    }
    None
}

use std::collections::{BTreeMap, HashMap, VecDeque};
use std::f32;
use web_sys::js_sys;
use crate::interpreter::opdrachten::{execute_all, Context, SubDef, WhatsNext};
use crate::interpreter::parsers::{parseer_regel};
use super::functions::{EcolFout, FunDef};
use super::waarden::{VariabeleType, Waarde};
use super::program::{LineInhoud, Programma};

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
                    return Err(EcolFout::FoutMelding("Regelbuffer overschrijdt het maximum van 80 tekens. Vergeet NR niet.".to_string()));
                }
                r.push_str(regel)
            },
        }
        Ok(())
    }

    fn lees_regel(&self) -> String {
        match self {
            RegelBuffer::Regel(r) => r.to_string(),
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
        let index = self.symbolen[variabele_naam];
        self.data_pool[index] = Waarde::NogNietBepaald;
        self.symbolen.remove(variabele_naam);
    }
    fn lees_waarde(&self, naam: &str) -> Option<Waarde> {
        if let Some(index) = self.symbolen.get(naam) {
            let waarde = self.data_pool[*index].clone();
            if waarde.type_van().is_none() {
                return None;
            }
            Some(waarde)
        } else {
            None
        }
    }

    fn schrijf_waarde(&mut self, naam: &str, waarde: Waarde) -> Result<(), EcolFout> {
        let mut doelwaarde: Waarde;
        if !self.bestaat(naam) {
            if waarde.type_van() == Some(VariabeleType::Rij) {
                return Err(EcolFout::FoutMelding(format!("RIJ-variabele '{}' is nog niet gedefinieerd.", naam)));
            } else if waarde.type_van() == Some(VariabeleType::Rijsym) {
                return Err(EcolFout::FoutMelding(format!("RIJSYM-variabele '{}' is nog niet gedefinieerd.", naam)));
            }
        } else {
            let Some(doel_waarde) = self.lees_waarde(naam) else {
                return Err(EcolFout::FoutMelding(format!("INTERNE FOUT: variabele '{}' niet opgeslagen.", naam)));
            };
            doelwaarde = doel_waarde;
            if doelwaarde.type_van() != None {
                if doelwaarde.type_van() != waarde.type_van() {
                    return Err(EcolFout::FoutMelding(format!("Variabele '{}' is gedefinieerd als een {}, en de waarde om op te slaan is een {}."
                                       ,naam
                                       ,doelwaarde.type_van().unwrap().to_string()
                                       ,waarde.type_van().unwrap().to_string())));
                }
            }
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

pub struct LeesGeheugen {
    lees_hervat_bij: Option<u16>,
    lees_buffer: VecDeque<f32>,
    leessym_hervat_bij: Option<u16>,
    leessym_buffer: VecDeque<u8>,
    lopende_machine: Option<EcolMachine>,
    lopend_programma: Option<BTreeMap<u16, LineInhoud>>,
}

impl LeesGeheugen {
    pub fn new() -> Self {
        LeesGeheugen {
            lees_hervat_bij: None,
            lees_buffer: VecDeque::new(),
            leessym_hervat_bij: None,
            leessym_buffer: VecDeque::new(),
            lopende_machine: None,
            lopend_programma: None,
        }
    }
    pub fn wacht_op_lees(&self) -> bool {
        self.lees_hervat_bij.is_some()
    }
    pub(super) fn lees_hervat_bij(&mut self) -> Option<u16> {
        self.lees_hervat_bij
    }
    pub(super) fn lees_hervat_none(&mut self)  {
        self.lees_hervat_bij = None
    }
    pub(super) fn lees_hervat_bij_op_regel(&mut self, regelnummer: u16)  {
        self.lees_hervat_bij = Some(regelnummer)
    }
    pub(super) fn lees_waarde(&mut self) -> Option<f32> {
        self.lees_buffer.pop_front()
    }
    pub(super) fn schrijf_lees_waarde(&mut self, waarde: f32) {
        self.lees_buffer.push_back(waarde);
    }
    pub fn wacht_op_leessym(&self) -> bool {
        self.leessym_hervat_bij.is_some()
    }
    pub(super) fn leessym_hervat_bij(&mut self) -> Option<u16> {
        self.leessym_hervat_bij
    }
    pub(super) fn leessym_hervat_none(&mut self)  {
        self.leessym_hervat_bij = None
    }
    pub(super) fn leessym_hervat_bij_op_regel(&mut self, regelnummer: u16)  {
        self.leessym_hervat_bij = Some(regelnummer)
    }
    pub(super) fn leessym_waarde(&mut self) -> Option<f32> {
        match self.leessym_buffer.pop_front() {
            Some(w) => Some(w as f32),
            None => None,
        }
    }
    pub(super) fn schrijf_leessym_waarde(&mut self, waarde: f32) {
        self.leessym_buffer.push_back(waarde as u8);
    }
    pub(super) fn neem_lopende_toestand(&mut self) -> Option<(EcolMachine, BTreeMap<u16, LineInhoud>)> {
        match (self.lopende_machine.take(), self.lopend_programma.take()) {
            (Some(m), Some(p)) => Some((m, p)),
            _ => None,
        }
    }
    pub(super) fn sla_lopende_toestand_op(&mut self, machine: EcolMachine, programma: BTreeMap<u16, LineInhoud>) {
        self.lopende_machine = Some(machine);
        self.lopend_programma = Some(programma);
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
            seed: js_sys::Date::now() as u64,
        }
    }
    pub(super) fn schrijf_subregister(&mut self, naam: &str, definitie: SubDef) -> Result<(), EcolFout> {

        if self.sub_register.insert(naam.to_string(), definitie).is_some() {
            return Err(EcolFout::FoutMelding(format!("Interne fout: Subroutine met naam '{}' bestaat al", naam)));
        };
        Ok(())
    }
    pub(super) fn lees_subregister(&self, naam: &str) -> Option<&SubDef> {
        self.sub_register.get(naam)
    }
    pub(super) fn is_sub(&self, naam: &str) -> bool {
        self.sub_register.contains_key(naam)
    }
    pub(super) fn return_from_sub(&mut self) -> Result<u16, EcolFout> {
        let Some(reply) = self.sub_return_stack.pop() else {
            return Err(EcolFout::FoutMelding("Geen subroutine om uit terug te keren.".to_string()))
        };

        Ok(reply)
    }
    pub(super) fn start_sub(&mut self, naam: &str, regelnummer: u16) -> Result<(), EcolFout> {
        if !self.is_sub(naam) {
            return Err(EcolFout::FoutMelding(format!("Subroutine met naam '{}' bestaat niet", naam)));
        };
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
            return Err(EcolFout::FoutMelding(format!("Ongeldige variabele naam: '{}' is gedefinieerd als FUN.", naam)))
        }
        if self.is_sub(naam) {
            return Err(EcolFout::FoutMelding(format!("Ongeldige variabele naam: '{}' is gedefinieerd als SUB.", naam)))
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
        self.variabelen_opslag.wis_variabele(variabele_naam)
    }
    pub(super) fn teller_nieuw(&mut self, naam: &str, stap: f32, start: f32, stop: f32, regel: u16) -> Result<Option<()>, EcolFout> {
        if (stap > 0.0 && start > stop) || (stap < 0.0 && start < stop) {
            return Ok(None)
        }
        if stap == 0.0 {
            return Err(EcolFout::FoutMelding("Stapgrootte mag niet 0 zijn.".to_string()))
        }

        let var_typ = self.var_type_van(naam);
        match var_typ {
            Some(VariabeleType::Getal) | None => {
                if var_typ.is_some() {
                    self.var_wis(naam);
                }
                let mut teller = Waarde::new_teller(stap, start, stop);
                teller.teller_schrijf_regel(regel)?;
                self.var_schrijf_waarde(naam, teller)?;
                self.actieve_tellers.push(naam.to_string());

                Ok(Some(()))
            },
            Some(v_type) => Err(EcolFout::FoutMelding(format!("Variabele '{}' bestaat al met type '{}'.", naam, v_type.to_string()))),
        }
    }
    pub(super) fn teller_naar_herhaal(programma: &BTreeMap<u16, LineInhoud>, regel: &u16) -> Result<u16, EcolFout> {
        let mut current = *regel;
        let mut met_diepte = 0u16;
        loop{
            let Some((&regelnummer, current_regel)) = programma.range(current..).next() else {
                return Err(EcolFout::FoutMelding("FOUTMELDING: Geen HERHAAL aangetroffen na MET.".to_string()));
            };
            current = regelnummer + 1;
            match current_regel {
                LineInhoud::Met { .. } => {
                    met_diepte += 1;
                    continue;
                }
                LineInhoud::Herhaal {} => {
                    if met_diepte == 0 {
                        return Ok(current)
                    }
                    met_diepte -= 1;
                },
                _ => { continue; },
            }
        }
    }
    pub(super) fn teller_herhaal(&mut self) -> Result<Option<u16>, EcolFout> {
        if self.actieve_tellers.len() == 0 {
            return Err(EcolFout::FoutMelding("HERHAAL zonder MET aangetroffen.".to_string()))
        }
        let last = self.actieve_tellers.len() - 1;
        let naam = self.actieve_tellers[last].clone();
        let Some(mut teller) = self.var_lees_waarde(&naam) else {
            return Err(EcolFout::FoutMelding(format!("INTERNE FOUT: Teller '{}' bestaat niet.", naam)))
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
    pub(super) fn var_bestaat(&self, naam: &str) -> bool {
        self.variabelen_opslag.bestaat(naam)
    }
    pub(super) fn naar_regel_buffer(&mut self, regel: &str) -> Result<(), EcolFout>{
        self.regel_buffer.naar_regel_buffer(regel)?;

        Ok(())
    }
    pub(super) fn lees_regel(&self) -> String {
        self.regel_buffer.lees_regel()
    }
    pub(super) fn leeg_regel_buffer(&mut self) {
        self.regel_buffer.leeg_regel_buffer()
    }
    pub(super) fn programma(&self) -> &BTreeMap<u16, LineInhoud> {
        &self.programma.programma()
    }
    pub(super) fn laad_programma(&mut self, bron: &BTreeMap<u16, LineInhoud>) {
        self.programma.laad(bron);
    }
    pub(super) fn laad_functies(&mut self, bron: &HashMap<String, FunDef>) {
        self.functie_register = bron.clone();
    }
    pub(super) fn haal_functiedefinitie(&self, naam: &str) -> Option<&FunDef> {
        self.functie_register.get(naam)
    }
    pub(super) fn schrijf_nieuwe_functie(&mut self, naam: &str, fundef: &FunDef) -> Result<(), EcolFout>{
        if self.functie_register.contains_key(naam) {
            return Err(EcolFout::FoutMelding(format!("Functie met naam '{}' bestaat al", naam)));
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
    pub(super) fn haal_functie_register(&self) -> &HashMap<String,FunDef>
    {
        &self.functie_register
    }

    pub(super) fn is_fun(&self, naam: &str) -> bool {
        self.functie_register.contains_key(naam)
    }



    pub(super) fn volgende_willekeurig(&mut self, laag: f32, hoog: f32) -> f32 {
        self.seed ^= self.seed << 13;
        self.seed ^= self.seed >> 7;
        self.seed ^= self.seed << 17;
        let basis = (self.seed as f32) / (u64::MAX as f32);  // getal tussen 0.0 en 1.0
        basis * (hoog - laag) + laag
    }

    pub fn execute(&mut self, input: &str, lees_geheugen: &mut LeesGeheugen, output: &mut dyn FnMut(&str)) -> String {
        let mut reply: Option<String> = None;

        if lees_geheugen.wacht_op_lees() {
            let waarde = input.trim().parse::<f32>();
            if let Ok(waarde) = waarde {
                lees_geheugen.schrijf_lees_waarde(waarde);
                let Some(regel) = lees_geheugen.lees_hervat_bij() else {
                    return "Er is een fout opgetreden. Geef opnieuw in.".to_string();
                };
                match self.execute_start(Some(regel), lees_geheugen, output) {
                    Ok(r) => reply = Some(r),
                    Err(EcolFout::FoutMelding(e)) => reply = Some(e),
                    Err(EcolFout::WachtOpLees(_)) | Err(EcolFout::WachtOpLeessym(_)) => reply = None
                }

            } else {
                return "Alleen numerieke waarden kunnen ingegeven worden. Geef opnieuw in.".to_string();
            }
        } else if lees_geheugen.wacht_op_leessym() {
            let waarde_input = input;
            if waarde_input.len() != 1 {
                return "Voor LEESSYM mag slechts één karakter worden ingegeven. Geef opnieuw in.".to_string();
            }
            let waarde_char = waarde_input.chars().next().unwrap_or_default();
            let waarde = symbolen_reverse(waarde_char);
            match waarde {
                Some(c) => {
                    lees_geheugen.schrijf_leessym_waarde(c as f32);
                    let Some(regel) = lees_geheugen.leessym_hervat_bij() else {
                        return "Er is een fout opgetreden. Geef opnieuw in.".to_string();
                    };
                    match self.execute_start(Some(regel), lees_geheugen, output) {
                        Ok(r) => reply = Some(r),
                        Err(EcolFout::FoutMelding(e)) => reply = Some(e),
                        Err(EcolFout::WachtOpLees(_)) | Err(EcolFout::WachtOpLeessym(_)) => reply = None
                    }

                },
                None => return "Alleen symbolen kunnen ingegeven worden. Geef opnieuw in.".to_string(),
            }
        } else {
            match parseer_regel(&input){
                Ok(regel) => {
                    if regel.regelnummer() == 0 {
                        let programma = BTreeMap::new();
                        reply = match execute_all(&regel, self, &programma, Context::Direct, lees_geheugen, output) {
                            Ok((r, _, _)) => r,
                            Err(EcolFout::FoutMelding(e)) => Some(e),
                            Err(EcolFout::WachtOpLees(r)) => {
                                if r == 0 {
                                    Some("FOUTMELDING: LEES kan alleen in programma's gebruikt worden (geen regelnummer gevonden).".to_string())
                                } else {
                                   lees_geheugen.lees_hervat_bij_op_regel(r);
                                    None
                                }
                            },
                            Err(EcolFout::WachtOpLeessym(r)) => {
                                if r == 0 {
                                    Some("FOUTMELDING: LEES SYM kan alleen in programma's gebruikt worden (geen regelnummer gevonden).".to_string())
                                } else {
                                    lees_geheugen.leessym_hervat_bij_op_regel(r);
                                    None
                                }
                            },
                        };
                    } else {
                        if regel.inhoud().as_str() == "Verwijderen" {
                            let verwijder_resultaat = self.programma.regel_verwijderen(regel.regelnummer());
                            reply = Some(verwijder_resultaat.unwrap_or("Geen regel om te verwijderen.".to_string()));
                        } else {
                            reply = Some(self.programma.regel_toevoegen(regel));
                        }
                    }
                }
                Err(e) => {
                    return format!("Ongeldige invoer: {}" ,e.to_string());
                }
            }
        }

        reply.unwrap_or("".to_string())
    }
}
