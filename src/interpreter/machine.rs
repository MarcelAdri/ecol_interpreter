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
        if let Some(s) = SYMBOLEN[i] {if s == symbool {return Some(i as u8)}}
        i += 1;
        if i > 99 {
            break;
        }
    }
    None
}

use std::collections::{BTreeMap, HashMap};
use std::f32;
use web_sys::js_sys;
use crate::interpreter::errors::EcolFout;
pub(crate) use crate::interpreter::leesgeheugen::LeesGeheugen;
use crate::interpreter::opdrachten::{execute_all, Context};
use crate::interpreter::parsers::{parseer_regel};
use super::waarden::{VariabeleType, Waarde};
use super::program::{FunDef, LineInhoud, Programma, SubDef};

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
            if doelwaarde.type_van().is_some()
                && doelwaarde.type_van() != waarde.type_van() {
                    return Err(EcolFout::FoutMelding(format!("Variabele '{}' is gedefinieerd als een {}, en de waarde om op te slaan is een {}."
                                       ,naam
                                       ,doelwaarde.type_van().unwrap().to_string()
                                       ,waarde
                                                                 .type_van()
                                                                 .map(|t| t.to_string())
                                                                 .unwrap_or_else(|| "onbekend type" )
                                                                 .to_string())));

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
                LineInhoud::Herhaal { .. } => {
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
        let naam = self.actieve_tellers.last()
            .ok_or_else(|| EcolFout::FoutMelding("HERHAAL zonder MET aangetroffen.".to_string()))?
            .clone();

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
        self.programma.programma()
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
                    return Err(EcolFout::FoutMelding("Er is een fout opgetreden. Geef opnieuw in.".to_string()));
                };
                self.hervat_uitvoering(regel, lees_geheugen, output)
            } else {
                return Err(EcolFout::FoutMelding("Alleen numerieke waarden kunnen ingegeven worden. Geef opnieuw in.".to_string()));
            }
        } else if lees_geheugen.wacht_op_leessym() {
            //let waarde_input = input;
            if input.len() != 1 {
                return Err(EcolFout::FoutMelding("Voor LEESSYM mag slechts één karakter worden ingegeven. Geef opnieuw in.".to_string()));
            }
            let waarde_char = input.chars().next().unwrap_or_default();
            let waarde = symbolen_reverse(waarde_char);
            match waarde {
                Some(c) => {
                    lees_geheugen.schrijf_leessym_waarde(c as f32);
                    let Some(regel) = lees_geheugen.leessym_hervat_bij() else {
                        return Err(EcolFout::FoutMelding("Er is een fout opgetreden. Geef opnieuw in.".to_string()));
                    };
                    self.hervat_uitvoering(regel, lees_geheugen, output)
                },
                None => return Err(EcolFout::FoutMelding("Alleen symbolen kunnen ingegeven worden. Geef opnieuw in.".to_string())),
            }
        } else {
            match parseer_regel(input){
                Ok(regel) => {
                    if regel.regelnummer() == 0 {
                        let programma = BTreeMap::new();
                        match execute_all(&regel, self, &programma, Context::Direct, lees_geheugen, output) {
                            Ok((r, _, _)) => r,
                            Err(EcolFout::FoutMelding(e)) => return Err(EcolFout::FoutMelding(e)),
                            Err(EcolFout::WachtOpLees(r)) => {
                                if r == 0 {
                                    return Err(EcolFout::FoutMelding("FOUTMELDING: LEES kan alleen in programma's gebruikt worden (geen regelnummer gevonden).".to_string()));
                                } else {
                                   lees_geheugen.lees_hervat_bij_op_regel(r);
                                    None
                                }
                            },
                            Err(EcolFout::WachtOpLeessym(r)) => {
                                if r == 0 {
                                    return Err(EcolFout::FoutMelding("FOUTMELDING: LEES SYM kan alleen in programma's gebruikt worden (geen regelnummer gevonden).".to_string()));
                                } else {
                                    lees_geheugen.leessym_hervat_bij_op_regel(r);
                                    None
                                }
                            },
                            Err(EcolFout::WachtOpLaad) => {
                                lees_geheugen.stel_laad_in();
                                None
                            },
                        }
                    } else {
                        if regel.inhoud().as_str() == "Verwijderen" {
                            let verwijder_resultaat = self.programma.regel_verwijderen(regel.regelnummer());
                            Some(verwijder_resultaat.unwrap_or("Geen regel om te verwijderen.".to_string()))
                        } else {
                            Some(self.programma.regel_toevoegen(regel))
                        }
                    }
                }
                Err(e) => {
                    return Err(EcolFout::FoutMelding(format!("Ongeldige invoer: {}" ,e)));
                }
            }
        };
        Ok(Some(reply.unwrap_or_default()))
    }
    pub fn execute_direct(&mut self, input: &str, output: &mut dyn FnMut(&str)) -> Result<String, String> {
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
            Err(EcolFout::FoutMelding(e)) => Some(e),
            Err(EcolFout::WachtOpLees(_)) | Err(EcolFout::WachtOpLeessym(_)) | Err(EcolFout::WachtOpLaad) => None,
        }
    }

}


