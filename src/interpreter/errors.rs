//! Fouttypen en uitvoeringsignalen van de ECOL-interpreter.
//!
//! [`EcolFout`] heeft twee varianten: [`EcolFout::FoutMelding`] voor echte fouten
//! en [`EcolFout::Signaal`] voor control-flow-signalen (`LEES`, `LEESSYM`, `LAAD`).
//! Signalen zijn geen fouten maar pauze-verzoeken: de uitvoering moet gestopt worden
//! zodat de browser invoer kan ophalen, waarna [`crate::interpreter::LeesGeheugen`]
//! de toestand bewaart voor hervatting.
use std::fmt::{Display, Formatter};
use crate::interpreter::waarden::VariabeleType;

//001 help_document Kon het help-document niet openen
//002 geen_browser_venster Geen browser-venster
//003 geen_document Geen document
//004 geen_bestand Kon geen bestand aanmaken
//005 geen_url Kon geen URL aanmaken
//006 geen_download_link Kon geen download-link aanmaken
//007 sprong_fout Sprong naar niet gedefinieerde regel {sprong_doel}
//008 ongeldig_aantal Het aantal bij {functie} moet een geheel getal tussen {laag} en {hoog} zijn, maar is {aantal}
//009 grenzen_rij_waarden De reeks van een RIJ of RIJSYM moet tussen 1 en 9999 liggen, maar loopt van {begin} tot {einde}
//010 grenzen_volgorde De reeks van een {functie_naam} moet oplopen, maar loopt van {begin} tot {einde}.
//011 grenzen_rij_integers De reeks van een RIJ of RIJSYM moet uit gehele getallen bestaan, maar loopt van {begin} tot {einde}
//012 symbool_waarde Symboolwaarde {symbool_nummer:.2} is niet geldig, moet tussen 0 en 99 liggen en een geheel getal zijn
//013 symbool_bestaat_niet Symbool {symbool_nummer} is niet gedefinieerd
//014 ongeldig_getal Ongeldig getal aangetroffen {situatie}.
//015 onverwacht_notatie_waarde Onverwachte notatie voor waarde: {waarde}
//016 onverwacht_notatie_exponent Onverwachte notatie voor exponent: {exponent}
//017 programma_afgebroken Programma afgebroken na 5 seconden (mogelijke eindeloze lus)
//018 geen_klaar Er zijn geen regels meer om uit te voeren. KLAAR niet aangetroffen
//019 laad_in_programma Interne fout: LAAD in programma
//020 waarde_zonder_index Variabele {naam} is van type {variabele_type} en kan daarom geen index hebben
//021 waarde_met_index Variabele {naam} is van type {variabele_type} en moet daarom een index hebben. De index is niet aangetroffen
//022 rij_niet_gedefinieerd Variabele {variabele_naam} is van type RIJ of RIJSYM en moet daarom eerst gedefinieerd worden met een RIJ- of RIJSYM-opdracht
//023 niet_in_programma_of_functie Interne fout: {functie} kan niet voorkomen in een programma of FUNctie
//024 niet_in_programma {functie} kan niet voorkomen in een programma
//025 niet_in_functie Interne fout: {functie} kan niet voorkomen binnen een functie-definitie
//026 onbekende_subroutine Onbekende subroutine: {subroutine}
//027 niet_vanuit_functie Een subroutine kan niet vanuit een functie worden aangeroepen
//028 herhaal_intern Interne FOUT bij HERHAAL-opdracht
//029 met_zonder_herhaal MET-lus niet afgesloten met HERHAAL
//030 niet_in_subroutine_of_functie {functie} kan niet voorkomen in een subroutine of een FUNctie
//031 met_zonder_regels geen regels na MET-opdracht
//032 niet_in_functie_uitvoer {functie} kan niet in een FUN definitie (geen uitvoer-apparaat)
//033 in_subroutine Interne fout: fout bij verwerking subroutine
//034 verwijderen_ongenummerd Verwijderen van een ongenummerde regel is niet mogelijk
//035 niet_in_programma_of_subroutine {functie} kan niet voorkomen in een programma of subroutine
//036 buffer_overflow Regelbuffer overschrijdt het maximum van 80 tekens. Vergeet NR niet
//037 variabele_niet_gevonden INTERNE FOUT: variabele '{naam}' niet opgeslagen
//038 verkeerd_type Variabele '{}' is gedefinieerd als een {}, en de waarde om op te slaan is een {}.
//039 onbekend_type Het type van variabele {naam} kon niet bepaald worden
//040 subroutine_bestaat_al Interne fout: Subroutine met naam '{naam}' bestaat al
//041 naam_is_fun Ongeldige variabele naam: '{naam}' is gedefinieerd als FUN
//042 naam_is_subroutine Ongeldige variabele naam: '{naam}' is gedefinieerd als SUB
//043 stapgrootte_nul Stapgrootte mag niet 0 zijn.
//044 teller_bestaat_al Teller-variabele met naam {naam} bestaat al met type {variabele_type}.
//045 herhaal_zonder_met HERHAAL zonder MET aangetroffen.
//046 onbekende_teller Interne fout: Teller {naam} bestaat niet.
//047 functie_bestaat_al Functie met naam {naam} bestaat al.
//048 invoer_fout Er is een fout opgetreden. Geef opnieuw in.
//049 invoer_niet_numeriek Alleen numerieke waarden kunnen ingegeven worden. Geef opnieuw in.
//050 invoer_lengte_niet_nul Voor LEESSYM mag slechts één karakter worden ingegeven. Geef opnieuw in.
//051 invoer_geen_symbool Alleen symbolen kunnen ingegeven worden. Geef opnieuw in.
//052 alleen_in_programma {functie_naam} kan alleen in programma's gebruikt worden (geen regelnummer gevonden).
//053 ongeldige_invoer Ongeldige invoer: {invoer}.
//054 ongeldige_functie_naam Ongeldige functienaam: {functie_naam}.
//055 geen_argumenten Geen argumenten tussen haakjes gevonden na {functie-naam}.
//056 verkeerd_aantal_argumenten {functie_naam} verwacht {aantal_argumenten} argumenten, maar {aantal_opgegeven} zijn gegeven.
//057 geen_string_argument {functie_naam} verwacht 1 string argument, maar er zijn {aantal} argumenten opgegeven.
//058 geen_haak_sluiten Haak openen gevonden zonder haak sluiten.
//059 onbekend_sleutelwoord Onbekend of geen sleutelwoord.
//060 geen_dan Geen DAN gevonden na ALS.
//061 geen_variabele_naam Variabele naam ontbreekt.
//062 ongeldig_wordt_teken Ongeldig 'wordt'-teken (:=).
//063 verkeerd_aantal_argumenten_dubbel {functie_naam} verwacht {verwacht_aantal} argument(en), maar {aantal_opgegeven} argumenten zijn aangetroffen.
//064 getal_niet_groter_dan {functie} verwacht een getal groter dan {verwacht_getal}.
//065 maximum_recursie_diepte Maximum recursie-diepte van {diepte} overschreden.
//066 argument_verkeerd_type {functie_naam} verwacht argument van type {verwacht_type} bij argument nummer {argument_nummer}, maar ontving een {opgegeven_type}.
//067 geen_fun_einde Er zijn geen regels meer om uit te voeren. FUN := niet aangetroffen.
//068 functie_zonder_resultaat FUNctie eindigde zonder resultaat.
//069 geen_rij {variabele_naam} is geen RIJ of RIJSYM variabele
//070 geen_waarde Interne fout: geen waarde na {functienaam}.
//071 geen_functie Functie {functie_naam} bestaat niet.
//072 regel_nummer_te_groot Regelnummer mag niet groter zijn dan 9999.
//073 ongeldige_syntax_met Ongeldige syntax voor MET bij de {expressie}-expressie.
//074 ongeldige_variabele_naam Ongeldige variabele naam.
//075 breedte_te_klein De opgegeven waarde {waarde} past niet in de opgegeven breedte {breedte}.
//076 tekst_na_regel Tekst aangetroffen na complete programmaregel.
//077 geen_geheel_getal Getal moet een geheel getal zijn, maar is {getal}.
//078 geen_positief_getal Getal moet positief zijn, maar is {getal}.
//079 geen_aanhaling_open Tekstblok moet beginnen met een aanhalingsteken.
//080 geen_aanhaling_sluiten Tekstblok moet eindigen met een aanhalingsteken.
//081 meer_tekstblokken Slechts één aaneengesloten tekstblok is toegestaan.
//082 incomplete_syntax Incomplete syntax.
//083 Getal_buiten_grens Het resultaat van de berekening {expressie} ligt buiten de grenzen van een variabele in ECOL.
//084 delen_door_nul Delen door nul is niet toegestaan in ECOL.
//085 machtsverheffen_ongeldig Machtsverheffen met een negatieve basis en een niet-gehele exponent is niet toegestaan in ECOL ({expressie}).
//086 machtsverheffen_exponent_te_groot Exponent {rechts} is te groot voor machtsverheffen in ECOL ({expressie}).
//087 ongeldig_vergelijkingsteken Ongeldig vergelijkingsteken: {vergelijkingsteken}.
//088 ongeldige_vergelijking Ongeldige vergelijking.
//089 ongeldige_index Index {positie} is niet geldig; moet liggen tussen {start} en {einde}.
//090 grenzen_rij_start De index van een {functie} kan niet lager zijn dan 1.
//091 grenzen_rij_aantal Een {functie} moet tenminste 2 waarden bevatten (start={start}, eind={einde}).
//092 geen_teller Waarde is geen teller.
//093 geen_haak_openen Haak sluiten gevonden zonder haak openen.
//094 verkeerd_argument Argument {index} van functie {functie} verwacht een {verwacht_var_type}, maar {var_type} is opgegeven.
//095 geen_variabele Variabele {variabele_naam} niet gevonden.
//096 geen_waarde_mogelijk Variabele {naam} is type {var_type} en kan niet herleid worden tot een waarde.
//097 ongeldige_tekens Ongeldige tekens in numerieke expressie {expressie}.
//098 precisie_verlies Precisie-verlies: '{source}' kan niet exact worden weergegeven.



#[derive(Debug, Clone, PartialEq)]
pub(super) enum EcolFoutVariant {
    AlleenInProgramma(String),                  //052
    ArgumentVerkeerdType(String, VariabeleType, usize, VariabeleType), //066
    BreedteTeKlein(f32, usize),                 //075
    BufferOverflow,                             //036
    DelenDoorNul,                               //084
    FunctieBestaatAl(String),                   //047
    FunctieZonderResultaat,                     //068
    GeenAanhalingOpen,                          //079
    GeenAanhalingSluiten,                       //080
    GeenArgumenten(String),                     //055
    GeenBestand,                                //004
    GeenBrowserVenster,                         //002
    GeenDan,                                    //060
    GeenDocument,                               //003
    GeenDownloadLink,                           //006
    GeenFunctie(String),                        //071
    GeenFunEinde,                               //067
    GeenGeheelGetal(f32),                       //077
    GeenHaakOpenen,                             //093
    GeenHaakSluiten,                            //058
    GeenKlaar,                                  //018
    GeenPositiefGetal(f32),                     //078
    GeenRij(String),                            //069
    GeenStringArgument(String, usize),          //057
    GeenTeller,                                 //092   
    GeenUrl,                                    //005
    GeenVariabele(String),                      //095
    GeenVariabeleNaam,                          //061
    GeenWaarde(String),                         //070
    GeenWaardeMogelijk(String, VariabeleType),  //096
    GetalBuitenGrens(String),                   //083
    GetalNietGroterDan(String, usize),          //064
    GrenzenRijAantal(String, usize, usize),     //091
    GrenzenRijIntegers(f32, f32),               //011
    GrenzenRijStart(String),                    //090
    GrenzenRijWaarde(f32, f32),                 //009
    GrenzenVolgorde(String, f32, f32),          //010
    HelpDocument,                               //001
    HerhaalIntern,                              //028
    HerhaalZonderMet,                           //045
    IncompleteSyntax,                           //082
    InSubroutine,                               //033
    InvoerFout,                                 //048
    InvoerGeenSymbool,                          //051
    InvoerLengteNietNul,                        //050
    InvoerNietNumeriek,                         //049
    LaadInProgramma,                            //019
    MachtsverheffenOngeldig(String),            //085
    MachtsverheffenExponentTeGroot(f32, String),//086
    MaximumRecursieDiepte(u16),                 //065
    MeerTekstblokken,                           //081
    MetZonderHerhaal,                           //029
    MetZonderRegels,                            //031
    NaamIsFun(String),                          //041
    NaamIsSubroutine(String),                   //042
    NietInFunctie(String),                      //025
    NietInFunctieUitvoer(String),               //032
    NietVanuitFunctie,                          //027
    NietInProgramma(String),                    //024
    NietInProgrammaOfFunctie(String),           //023
    NietInProgrammaOfSubroutine(String),        //035
    NietInSubroutineOfFunctie(String),          //030
    OnbekendeSubroutine(String),                //026
    OnbekendeTeller(String),                    //046
    OnbekendSleutelwoord,                       //059
    OnbekendType(String),                       //039
    OngeldigAantal(String, f32, f32, f32),      //008
    OngeldigeFunctieNaam(String),               //054
    OngeldigeIndex(usize, usize, usize),        //089
    OngeldigeInvoer(String),                    //053
    OngeldigeSyntaxMet(String),                 //073
    OngeldigeTekens(String),                    //097
    OngeldigeVariabeleNaam,                     //074
    OngeldigeVergelijking,                      //088
    OngeldigGetal(String),                      //014
    OngeldigVergelijkingsteken(String),         //087
    OngeldigWordtTeken,                         //062
    OnverwachtNotatieWaarde(String),            //015
    OnverwachtNotatieExponent(String),          //016
    PrecisieVerlies(String),                    //098
    ProgrammaAfgebroken,                        //017
    RegelnummerTeGroot,                         //072
    RijNietGedefinieerd(String),                //022
    SprongFout(u16),                            //007
    StapgrootteNul,                             //043
    SubRoutineBestaatAl(String),                //040
    SymboolBestaatNiet(u8),                     //013
    SymboolWaarde(f32),                         //012
    TekstNaRegel,                               //076
    TellerBestaatAl(String, VariabeleType),     //044
    VariabeleNietGevonden(String),              //037
    VerkeerdAantalArgumenten(String, usize, usize),//056
    VerkeerdAantalArgumentenDubbel(String, String, usize),//063
    VerkeerdArgument(usize, String, String, String), //094
    VerkeerdType(String, VariabeleType, VariabeleType),//038
    VerwijderenOngenummerd,                     //034
    WaardeZonderIndex(String, VariabeleType),   //020
    WaardeMetIndex(String, VariabeleType),      //021
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct EcolFoutMelding {
    regel: Option<u16>,
    fout: EcolFoutVariant,
}

impl EcolFoutMelding {
    fn nieuw(fout: EcolFoutVariant) -> Self {
        Self { regel: None, fout }
    }

    fn met_regel(mut self, regel: u16) -> Self {
        self.regel = Some(regel);
        self
    }
}

impl Display for EcolFoutMelding {
    #[allow(clippy::too_many_lines)]
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let voorloop = match self.regel {
            Some(r) => format!("Regel {r}: "),
            None => String::new(),
        };
        let tekst = match &self.fout {
            EcolFoutVariant::AlleenInProgramma(functie_naam) =>
                format!("E052: {functie_naam} kan alleen in programma's gebruikt worden (geen regelnummer gevonden)."),
            EcolFoutVariant::ArgumentVerkeerdType(functie_naam, verwacht_type, argument_nummer, opgegeven_type) =>
                format!("E066: {functie_naam} verwacht argument van type {verwacht_type} bij argument nummer {argument_nummer}, maar ontving een {opgegeven_type}."),
            EcolFoutVariant::BreedteTeKlein(waarde, breedte) =>
                format!("E075: De opgegeven waarde {waarde} past niet in de opgegeven breedte {breedte}."),
            EcolFoutVariant::BufferOverflow =>
                "E036: Regelbuffer overschrijdt het maximum van 80 tekens. Vergeet NR niet".to_string(),
            EcolFoutVariant::DelenDoorNul =>
                "E084: Delen door nul is niet toegestaan in ECOL.".to_string(),
            EcolFoutVariant::FunctieBestaatAl(naam) =>
                format!("E047: Functie met naam {naam} bestaat al."),
            EcolFoutVariant::FunctieZonderResultaat =>
                "E068: FUNctie eindigde zonder resultaat.".to_string(),
            EcolFoutVariant::GeenAanhalingOpen =>
                "E079: Tekstblok moet beginnen met een aanhalingsteken.".to_string(),
            EcolFoutVariant::GeenAanhalingSluiten =>
                "E080: Tekstblok moet eindigen met een aanhalingsteken.".to_string(),
            EcolFoutVariant::GeenArgumenten(functie_naam) =>
                format!("E055: Geen argumenten tussen haakjes gevonden na {functie_naam}."),
            EcolFoutVariant::GeenBestand =>
                "E004: Kon geen bestand aanmaken.".to_string(),
            EcolFoutVariant::GeenBrowserVenster =>
                "E002: Geen browser-venster.".to_string(),
            EcolFoutVariant::GeenDan =>
                "E060: Geen DAN gevonden na ALS.".to_string(),
            EcolFoutVariant::GeenDocument =>
                "E003: Geen document.".to_string(),
            EcolFoutVariant::GeenDownloadLink =>
                "E006: Kon geen download-link aanmaken.".to_string(),
            EcolFoutVariant::GeenFunctie(functie_naam) =>
                format!("E071: Functie '{functie_naam}' bestaat niet."),
            EcolFoutVariant::GeenFunEinde =>
                "E067: Er zijn geen regels meer om uit te voeren. FUN := niet aangetroffen.".to_string(),
            EcolFoutVariant::GeenGeheelGetal(getal) =>
                format!("E077: Getal moet een geheel getal zijn, maar is {getal}"),
            EcolFoutVariant::GeenHaakOpenen =>
                "E093: Haak sluiten gevonden zonder haak openen.".to_string(),
            EcolFoutVariant::GeenHaakSluiten =>
                "E058: Haak openen gevonden zonder haak sluiten.".to_string(),
            EcolFoutVariant::GeenKlaar =>
                "E018: Er zijn geen regels meer om uit te voeren. KLAAR niet aangetroffen.".to_string(),
            EcolFoutVariant::GeenPositiefGetal(getal) => 
                format!("E078: Getal moet positief zijn, maar is {getal}"),
            EcolFoutVariant::GeenRij(variabele_naam) =>
                format!("E069: {variabele_naam} is geen RIJ of RIJSYM variabele"),
            EcolFoutVariant::GeenStringArgument(functienaam, aantal) =>
                format!("E057: {functienaam} verwacht 1 string argument, maar er zijn {aantal} argumenten opgegeven."),
            EcolFoutVariant::GeenTeller =>
                "E092: Waarde is geen teller.".to_string(),
            EcolFoutVariant::GeenUrl =>
                "E005: Kon geen URL aanmaken.".to_string(),
            EcolFoutVariant::GeenVariabele(variabele_naam) =>
                format!("E095: Variabele {variabele_naam} niet gevonden."),
            EcolFoutVariant::GeenVariabeleNaam =>
                "E061: Variabele naam ontbreekt.".to_string(),
            EcolFoutVariant::GeenWaarde(functienaam) =>
                format!("E070: Interne fout: geen waarde na {functienaam}."),
            EcolFoutVariant::GeenWaardeMogelijk(naam, var_type) =>
                format!("E096: Variabele {naam} is type {var_type} en kan niet herleid worden tot een waarde."),
            EcolFoutVariant::GetalBuitenGrens(expressie) =>
                format!("E083: Het resultaat van de berekening {expressie} ligt buiten de grenzen van een variabele in ECOL."),
            EcolFoutVariant::GetalNietGroterDan(functienaam, verwacht_getal) =>
                format!("E064: {functienaam} verwacht een getal groter dan {verwacht_getal}."),
            EcolFoutVariant::GrenzenRijAantal(functie, start, einde) =>
                format!("E091: Een {functie} moet tenminste 2 waarden bevatten (start={start}, eind={einde})."),
            EcolFoutVariant::GrenzenRijIntegers(begin, einde) =>
                format!("E011: De reeks van een RIJ of RIJSYM moet uit gehele getallen bestaan, maar loopt van {begin} tot {einde}."),
            EcolFoutVariant::GrenzenRijStart(functie) =>
                format!("E090: De index van een {functie} kan niet lager zijn dan 1."),
            EcolFoutVariant::GrenzenRijWaarde(begin, einde) =>
                format!("E009: De reeks van een RIJ of RIJSYM moet tussen 1 en 9999 liggen, maar loopt van {begin} tot {einde}."),
            EcolFoutVariant::GrenzenVolgorde(functie_naam,begin, einde) =>
                format!("E010: De reeks van een {functie_naam} moet oplopen, maar loopt van {begin} tot {einde}."),
            EcolFoutVariant::HelpDocument =>
                "E001: Kon het help-document niet openen.".to_string(),
            EcolFoutVariant::HerhaalIntern =>
                "E028: Interne fout bij HERHAAL-opdracht.".to_string(),
            EcolFoutVariant::HerhaalZonderMet =>
                "E045: HERHAAL zonder MET aangetroffen.".to_string(),
            EcolFoutVariant::IncompleteSyntax =>
                "E082: Incomplete syntax.".to_string(),
            EcolFoutVariant::InSubroutine =>
                "E033: Interne fout: fout bij verwerking subroutine.".to_string(),
            EcolFoutVariant::InvoerFout =>
                "E048: Er is een fout opgetreden. Geef opnieuw in.".to_string(),
            EcolFoutVariant::InvoerGeenSymbool =>
                "E051: Alleen symbolen kunnen ingegeven worden. Geef opnieuw in.".to_string(),
            EcolFoutVariant::InvoerLengteNietNul =>
                "E050: Voor LEESSYM mag slechts één karakter worden ingegeven. Geef opnieuw in.".to_string(),
            EcolFoutVariant::InvoerNietNumeriek =>
                "E049: Alleen numerieke waarden kunnen ingegeven worden. Geef opnieuw in.".to_string(),
            EcolFoutVariant::LaadInProgramma =>
                "E019: Interne fout: LAAD in programma.".to_string(),
            EcolFoutVariant::MachtsverheffenOngeldig(expressie) =>
                format!("E085: Machtsverheffen met een negatieve basis en een niet-gehele exponent is niet toegestaan in ECOL ({expressie})."),
            EcolFoutVariant::MachtsverheffenExponentTeGroot(exponent, expressie) =>
                format!("E086: Exponent {exponent} is te groot voor machtsverheffen in ECOL ({expressie})."),
            EcolFoutVariant::MaximumRecursieDiepte(diepte) =>
                format!("E065: Maximum recursie-diepte van {diepte} overschreden."),
            EcolFoutVariant::MeerTekstblokken =>
                "E081: Slechts één aaneengesloten tekstblok is toegestaan.".to_string(),
            EcolFoutVariant::MetZonderHerhaal =>
                "E029: MET-lus niet afgesloten met HERHAAL.".to_string(),
            EcolFoutVariant::MetZonderRegels =>
                "E031: geen regels na MET-opdracht.".to_string(),
            EcolFoutVariant::NaamIsFun(naam) =>
                format!("E041: Ongeldige variabele naam: '{naam}' is gedefinieerd als FUN."),
            EcolFoutVariant::NaamIsSubroutine(naam) =>
                format!("E042: Ongeldige variabele naam: '{naam}' is gedefinieerd als subroutine."),
            EcolFoutVariant::NietInFunctie(functie) =>
                format!("E025: Interne fout: {functie} kan niet voorkomen binnen een functie-definitie."),
            EcolFoutVariant::NietInFunctieUitvoer(functie) =>
                format!("E032: {functie} kan niet in een FUN definitie (geen uitvoer-apparaat)."),
            EcolFoutVariant::NietVanuitFunctie =>
                "E027: Een subroutine kan niet vanuit een functie worden aangeroepen.".to_string(),
            EcolFoutVariant::NietInProgramma(functie) =>
                format!("E024: {functie} kan niet voorkomen in een programma."),
            EcolFoutVariant::NietInProgrammaOfFunctie(functie) =>
                format!("E023: Interne fout: {functie} kan niet voorkomen in een programma of FUNctie."),
            EcolFoutVariant::NietInProgrammaOfSubroutine(functie) =>
                format!("E035: {functie} kan niet voorkomen in een programma of subroutine."),
            EcolFoutVariant::NietInSubroutineOfFunctie(functie) =>
                format!("E030: {functie} kan niet voorkomen in een subroutine of een FUNctie."),
            EcolFoutVariant::OnbekendeSubroutine(subroutine) =>
                format!("E026: Onbekende subroutine: {subroutine}."),
            EcolFoutVariant::OnbekendeTeller(naam) =>
                format!("E046: Interne fout: Teller {naam} bestaat niet."),
            EcolFoutVariant::OnbekendSleutelwoord =>
                "E059: Onbekend of geen sleutelwoord.".to_string(),
            EcolFoutVariant::OnbekendType(naam) =>
                format!("E039: Het type van variabele {naam} kon niet bepaald worden."),
            EcolFoutVariant::OngeldigAantal(functie, laag, hoog, aantal) =>
                format!("E008: Het aantal bij {functie} moet een geheel getal tussen {laag} en {hoog} zijn, maar is {aantal}."),
            EcolFoutVariant::OngeldigeFunctieNaam(functie_naam) =>
                format!("E054: Ongeldige functienaam: {functie_naam}."),
            EcolFoutVariant::OngeldigeIndex(positie, start, einde) =>
                format!("E089: Index {positie} is niet geldig; moet liggen tussen {start} en {einde}."),
            EcolFoutVariant::OngeldigeInvoer(invoer) =>
                format!("E053: Ongeldige invoer: {invoer}."),
            EcolFoutVariant::OngeldigeSyntaxMet(expressie) =>
                format!("E073: Ongeldige syntax voor MET bij de {expressie}-expressie."),
            EcolFoutVariant::OngeldigeTekens(expressie) =>
                format!("E097: Ongeldige tekens in numerieke expressie: {expressie}."),
            EcolFoutVariant::OngeldigeVariabeleNaam =>
                "E074: Ongeldige variabele naam.".to_string(),
            EcolFoutVariant::OngeldigeVergelijking =>
                "E088: Ongeldige vergelijking.".to_string(),
            EcolFoutVariant::OngeldigGetal(situatie) =>
                format!("E014: Ongeldig getal aangetroffen {situatie}."),
            EcolFoutVariant::OngeldigVergelijkingsteken(vergelijkingsteken) =>
                format!("E087: Ongeldig vergelijkingsteken: {vergelijkingsteken}."),
            EcolFoutVariant::OngeldigWordtTeken =>
                "E062: Ongeldig 'wordt'-teken (:=).".to_string(),
            EcolFoutVariant::OnverwachtNotatieWaarde(waarde) =>
                format!("E015: Onverwachte notatie voor waarde: {waarde}."),
            EcolFoutVariant::OnverwachtNotatieExponent(exponent) =>
                format!("E016: Onverwachte notatie voor exponent: {exponent}."),
            EcolFoutVariant::PrecisieVerlies(getal) =>
                format!("E098: Precisie-verlies: '{getal}' kan niet exact worden weergegeven."),
            EcolFoutVariant::ProgrammaAfgebroken =>
                "E017: Programma afgebroken na 5 seconden (mogelijke eindeloze lus).".to_string(),
            EcolFoutVariant::RegelnummerTeGroot =>
                "E072: Regelnummer mag niet groter zijn dan 9999.".to_string(),
            EcolFoutVariant::RijNietGedefinieerd(variabele_naam) =>
                format!("E022: Variabele {variabele_naam} is van type RIJ of RIJSYM en moet daarom eerst gedefinieerd worden met een RIJ- of RIJSYM-opdracht."),
            EcolFoutVariant::SprongFout(sprong_doel) =>
                format!("E007: Sprong naar niet gedefinieerde regel {sprong_doel}."),
            EcolFoutVariant::StapgrootteNul =>
                "E043: Stapgrootte mag niet 0 zijn.".to_string(),
            EcolFoutVariant::SubRoutineBestaatAl(subroutine_naam) =>
                format!("E040: Interne fout: Subroutine '{subroutine_naam}' bestaat al."),
            EcolFoutVariant::SymboolBestaatNiet(symbool_nummer) =>
                format!("E013: Symbool {symbool_nummer} is niet gedefinieerd."),
            EcolFoutVariant::SymboolWaarde(symbool_nummer) =>
                format!("E012: Symboolwaarde {symbool_nummer:.2} is niet geldig, moet tussen 0 en 99 liggen en een geheel getal zijn."),
            EcolFoutVariant::TekstNaRegel =>
                "E076: Tekst aangetroffen na complete programmaregel.".to_string(),
            EcolFoutVariant::TellerBestaatAl(naam, variabele_type) =>
                format!("E044: Teller-variabele met naam {naam} bestaat al met type {variabele_type}."),
            EcolFoutVariant::VariabeleNietGevonden(naam) =>
                format!("E037: Interne fout: variabele '{naam}' niet opgeslagen."),
            EcolFoutVariant::VerkeerdAantalArgumenten(functie_naam, verwachte, gegeven) =>
                format!("E056: {functie_naam} verwacht {verwachte} argumenten, maar {gegeven} argumenten zijn aangetroffen."),
            EcolFoutVariant::VerkeerdAantalArgumentenDubbel(functie_naam, verwacht_aantal, aantal_opgegeven) =>
                format!("E063: {functie_naam} verwacht {verwacht_aantal} argument(en), maar {aantal_opgegeven} argumenten zijn aangetroffen."),
            EcolFoutVariant::VerkeerdArgument(index, functie_naam, verwacht_var_type, var_type) =>
                format!("E094: Argument {index} van functie {functie_naam} verwacht een {verwacht_var_type}, maar {var_type} is opgegeven."),
            EcolFoutVariant::VerkeerdType(naam, gegeven_type, verwacht_type) =>
                format!("E038: Variabele '{naam}' is gedefinieerd als een {gegeven_type}, en de waarde om op te slaan is een {verwacht_type}."),
            EcolFoutVariant::VerwijderenOngenummerd =>
                "E034: Verwijderen van een ongenummerde regel is niet mogelijk.".to_string(),
            EcolFoutVariant::WaardeMetIndex(naam, variabele_type) =>
                format!("E021: Variabele {naam} is van type {variabele_type} en moet daarom een index hebben. De index is niet aangetroffen."),
            EcolFoutVariant::WaardeZonderIndex(naam, variabele_type) =>
                format!("E020: Variabele {naam} is van type {variabele_type} en kan daarom geen index hebben."),
        };

        write!(f, "{voorloop}{tekst}")
    }
}
/// Pauze-signalen die geen fout zijn maar om externe invoer vragen.
///
/// `Lees` en `Leessym` bevatten het regelnummer waarop de uitvoering moet hervatten
/// nadat de gebruiker invoer heeft geleverd. `Laad` triggert een bestandsdialoog.
#[derive(Debug, Clone, PartialEq)]
pub(super) enum EcolWachtSignaal {
    Lees(u16),
    Leessym(u16),
    Laad,
}
impl Display for EcolWachtSignaal {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            EcolWachtSignaal::Lees(regel) => write!(f, "Wacht op LEES op regel {regel}."),
            EcolWachtSignaal::Leessym(regel) => write!(f, "Wacht op LEESSYM op regel {regel}."),
            EcolWachtSignaal::Laad => write!(f, "Wacht op LAAD."),
        }
    }
}

/// Het centrale fouttype dat door de hele interpreter via `Result` wordt gepropageerd.
///
/// `FoutMelding` beëindigt de uitvoering met een leesbare fout.
/// `Signaal` onderbreekt de uitvoering tijdelijk; de aanroeper vangt dit op
/// en hervat de uitvoering via [`crate::interpreter::LeesGeheugen`] zodra invoer beschikbaar is.
#[derive(Debug, Clone, PartialEq)]
pub(super) enum EcolFout {
    FoutMelding(EcolFoutMelding),
    Signaal(EcolWachtSignaal),
}

impl EcolFout {
    pub(super) fn melding(fout: EcolFoutVariant) -> Self {
        EcolFout::FoutMelding(EcolFoutMelding::nieuw(fout))
    }
    pub(super) fn met_regel(self, regel: u16) -> EcolFout {
        match self {
            EcolFout::FoutMelding(melding) => EcolFout::FoutMelding(melding.met_regel(regel)),
            EcolFout::Signaal(_) => self,
        }
    }
}

impl Display for EcolFout {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let melding = match self {
            EcolFout::FoutMelding(m) => m.to_string(),
            EcolFout::Signaal(s) => s.to_string(),
        };

        write!(f, "{melding}")
    }
}