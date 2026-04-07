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

use std::collections::{BTreeMap, HashMap};
use std::f32;
use web_sys::js_sys;
use crate::interpreter::helpers::{result_to_string};
use crate::interpreter::parsers::{parseer_regel};
use super::waarden::{VariabeleType, Waarde};
use super::program::{LineInhoud, Programma};

enum RegelBuffer {
    Regel(String),
}
impl RegelBuffer {
    fn new() -> Self {
        RegelBuffer::Regel(String::new())
    }
    fn naar_regel_buffer(&mut self, regel: &str) -> Result<(), String> {
        match self {
            RegelBuffer::Regel(r) => {
                if r.len() + regel.len() > 80 {
                    return Err("Regelbuffer overschrijdt het maximum van 80 tekens. Vergeet NR niet.".to_string());
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
    fn reserveer_rij(&mut self, variabele_naam: &str, start: usize, eind: usize) -> Result<(), String> {
        let index = self.pak_of_maak_index(variabele_naam);
        self.data_pool[index]=Waarde::new_rij(start, eind)?;
        Ok(())
    }
    fn reserveer_rijsym(&mut self, variabele_naam: &str, start: usize, eind: usize) -> Result<(), String> {
        let index = self.pak_of_maak_index(variabele_naam);
        self.data_pool[index]=Waarde::new_rijsym(start, eind)?;
        Ok(())
    }
    fn wis_rij(&mut self, variabele_naam: &str)  {
        let var_type = self.type_van(variabele_naam);
        match var_type {
            Some(VariabeleType::Rij) | Some(VariabeleType::Rijsym) => {
                let index = self.symbolen[variabele_naam];
                self.data_pool[index] = Waarde::NogNietBepaald;
                self.symbolen.remove(variabele_naam);
            },
            _ => { },
        }
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

    fn schrijf_waarde(&mut self, naam: &str, waarde: Waarde) -> Result<(), String> {
        let mut doelwaarde: Waarde;
        if !self.bestaat(naam) {
            if waarde.type_van() == Some(VariabeleType::Rij) {
                return Err(format!("RIJ-variabele '{}' is nog niet gedefinieerd.", naam));
            } else if waarde.type_van() == Some(VariabeleType::Rijsym) {
                return Err(format!("RIJSYM-variabele '{}' is nog niet gedefinieerd.", naam));
            }
        } else {
            let Some(doel_waarde) = self.lees_waarde(naam) else {
                return Err(format!("INTERNE FOUT: variabele '{}' niet opgeslagen.", naam));
            };
            doelwaarde = doel_waarde;
            if doelwaarde.type_van() != None {
                if doelwaarde.type_van() != waarde.type_van() {
                    return Err(format!("Variabele '{}' is gedefinieerd als een {}, en de waarde om op te slaan is een {}."
                                       ,naam
                                       ,doelwaarde.type_van().unwrap().to_string()
                                       ,waarde.type_van().unwrap().to_string()));
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
pub struct EcolMachine {
    variabelen_opslag: VariabelenOpslag,
    regel_buffer: RegelBuffer,
    programma: Programma,
    seed: u64,
}
impl EcolMachine {
    pub fn new() -> Self {
        EcolMachine {
            variabelen_opslag: VariabelenOpslag::new(),
            regel_buffer: RegelBuffer::new(),
            programma: Programma::new(),
            seed: js_sys::Date::now() as u64,
        }
    }

    pub(super) fn var_type_van(&self, naam: &str) -> Option<VariabeleType> {
        self.variabelen_opslag.type_van(naam)
    }
    pub(super) fn var_lees_waarde(&self, naam: &str) -> Option<Waarde> {
        self.variabelen_opslag.lees_waarde(naam)
    }
    pub(super) fn var_schrijf_waarde(&mut self, naam: &str, waarde: Waarde) -> Result<(), String> {
        self.variabelen_opslag.schrijf_waarde(naam, waarde)
    }
    pub(super) fn var_reserveer_rij(&mut self, variabele_naam: &str, start: usize, eind: usize) -> Result<(), String> {
        self.variabelen_opslag.reserveer_rij(variabele_naam, start, eind)
    }
    pub(super) fn var_reserveer_rijsym(&mut self, variabele_naam: &str, start: usize, eind: usize) -> Result<(), String> {
        self.variabelen_opslag.reserveer_rijsym(variabele_naam, start, eind)
    }
    pub(super) fn var_wis_rij(&mut self, variabele_naam: &str) {
        self.variabelen_opslag.wis_rij(variabele_naam)
    }
    pub(super) fn var_bestaat(&self, naam: &str) -> bool {
        self.variabelen_opslag.bestaat(naam)
    }
    pub(super) fn naar_regel_buffer(&mut self, regel: &str) -> Result<(), String>{
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
    pub(super) fn volgende_willekeurig(&mut self, laag: f32, hoog: f32) -> f32 {
        self.seed ^= self.seed << 13;
        self.seed ^= self.seed >> 7;
        self.seed ^= self.seed << 17;
        let basis = (self.seed as f32) / (u64::MAX as f32);  // getal tussen 0.0 en 1.0
        basis * (hoog - laag) + laag
    }

    pub fn execute(&mut self, input: &str, output: &mut dyn FnMut(&str)) -> String {
        let reply: String;

        match parseer_regel(&input){
            Ok(regel) => {
                if regel.regelnummer() == 0 {
                    match &regel.inhoud() {
                        LineInhoud::Als { .. } => {
                            reply = "".to_string();
                        }
                        LineInhoud::Help { } => {
                            reply = result_to_string(self.execute_help());
                        },
                        LineInhoud::Klaar { } => {
                            //No action needed, just return an empty string.
                            reply = "".to_string();
                        },
                        LineInhoud::LegeRegel { } => {
                            reply = "".to_string();
                        },
                        LineInhoud::Lijst { } => {
                            reply = result_to_string(self.execute_lijst());
                        },
                        LineInhoud::Naar { .. } => {
                            reply = "".to_string();
                        },
                        LineInhoud::NP { } => {
                            reply = result_to_string(self.execute_np( output));
                        },
                        LineInhoud::NR { aantal } => {
                            reply =  result_to_string(self.execute_nr( *aantal ));
                        },
                        LineInhoud::Rij { start, eind, variabele_naam } => {
                            reply = result_to_string(self.execute_rij(*start, *eind, variabele_naam));
                        },
                        LineInhoud::Rijsym { start, eind, variabele_naam } => {
                            reply = result_to_string(self.execute_rijsym(*start, *eind, variabele_naam));
                        },
                        LineInhoud::Schrijf { breedte, decimalen, expressie } => {
                            reply = result_to_string(self.execute_schrijf(*breedte, *decimalen, expressie));
                        },
                        LineInhoud::Schrijfsym { expressie } => {
                            reply = result_to_string(self.execute_schrijfsym(expressie));
                        },
                        LineInhoud::Schrijm { expressie } => {
                            reply = result_to_string(self.execute_schrijm(expressie));
                        },
                        LineInhoud::Spatie { aantal } => {
                            reply = result_to_string(self.execute_spatie(*aantal));
                        },
                        LineInhoud::Start { } => {
                            reply = result_to_string(self.execute_start(output));
                        }
                        LineInhoud::Tekst { expressie } => {
                            reply = result_to_string(self.execute_tekst(expressie));
                        },
                        LineInhoud::Toekennen {variabele_naam, argument,  expressie} => {
                            reply = result_to_string(self.execute_toekennen(variabele_naam, *argument, expressie));
                        },
                        LineInhoud::Verwijderen { } => {
                            reply = "Verwijderen van een ongenummerde regel is niet mogelijk.".to_string();
                        },
                    }
                } else {
                    if regel.inhoud().as_str() == "Verwijderen" {
                        let verwijder_resultaat = self.programma.regel_verwijderen(regel.regelnummer());
                        match verwijder_resultaat {
                            Some( reactie) => reply = reactie,
                            None => reply = "Geen regel om te verwijderen.".to_string(),
                        }
                    } else {
                        reply = self.programma.regel_toevoegen(regel);
                    }
                }
            }
            Err(e) => {
                return format!("Ongeldige invoer: {}" ,e);
            }
        }
       
        reply
    }

    // fn lees_getal_variabele_argument(&mut self, naam: &str) -> Result<&Waarde, String> {
    //     if !(self.var_type_van(naam) == Some(VariabeleType::Getal)) && !(self.var_type_van(naam) == Some(VariabeleType::Rij))
    //     {
    //         return Err("Argument moet een getal of een Rij zijn".to_string());
    //     }
    //     let Some(waarde) = self.var_lees_waarde(naam) else {
    //         return Err("Fout bij ophalen variabele".to_string());
    //     };
    //     Ok(waarde)
    // }
    // pub(super) fn lees_integer_argument(&mut self, argument: &str) -> Result<usize, String> {
    //     if is_geldige_variabele_naam(argument) {
    //
    //         let complete_waarde = self.lees_getal_variabele_argument(argument)?;
    //         Ok(parse_i32(&haal_data(&complete_waarde)?) as usize)
    //     } else {
    //         argument
    //             .parse::<usize>()
    //             .map_err(|_| "Ongeldige integer-waarde".to_string())
    //     }
    // }

    // fn lees_getal_argument(&mut self, argument: &str) -> Result<f32, String> {
    //     if is_geldige_variabele_naam(argument) {
    //
    //         let complete_waarde = self.lees_getal_variabele_argument(argument)?;
    //         Ok(parse_f32(&haal_data(&complete_waarde)))
    //     } else {
    //         argument
    //             .parse::<f32>()
    //             .map_err(|_| "Ongeldig getal".to_string())
    //     }
    // }


}
