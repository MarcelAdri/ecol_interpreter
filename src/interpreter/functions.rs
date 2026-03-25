use crate::interpreter::EcolMachine;
use crate::interpreter::parsers::parseer_argumenten;
use crate::interpreter::program::{Functie, FunctieAanroep};
use crate::interpreter::waarden::{haal_data, EcolString, Waarde};

impl EcolMachine {
    pub(super) fn execute_function(&mut self, werk_functie: &FunctieAanroep) -> Result<Waarde, String> {
        let uitkomst = match werk_functie.functie() {
            Functie::LinksString => self.execute_function_links(werk_functie.argumenten().as_str()),
            Functie::RechtsString => self.execute_function_rechts(werk_functie.argumenten().as_str()),
            Functie::MiddenString => self.execute_function_midden(werk_functie.argumenten().as_str()),
            Functie::INT => self.execute_function_int(werk_functie.argumenten().as_str()),
        };

        uitkomst.map_err(|e| format!("Fout bij uitvoeren van functie: {:?}: {}", werk_functie.functie(), e))
    }
    pub(super) fn execute_function_int(&mut self, argumenten: &str) -> Result<Waarde, String> {
        let arguments = parseer_argumenten(argumenten, 1)?;
        let getal = self.solve_number_expression(&arguments[0])?;
        let werk_getal = haal_data(&getal).parse::<f32>().map_err(|_| "Ongeldig getal".to_string())?;
        let reply = werk_getal.trunc();

        Ok(Waarde::Getal(reply))
    }
    pub(super) fn execute_function_links(&mut self, argumenten: &str) -> Result<Waarde, String> {
        let arguments = parseer_argumenten(argumenten, 2)?;
        let tekst = self.solve_string_expression(&arguments[0])?;
        let lengte = self.lees_integer_argument(&arguments[1])?;
        let reply:String;

        let werk_tekst = haal_data(&tekst);
        reply = werk_tekst.chars().take(lengte).collect::<String>();

        Ok(Waarde::Tekst(EcolString::new(reply)))
    }
    pub(super) fn execute_function_midden(&mut self, argumenten: &str) -> Result<Waarde, String> {
        let arguments = parseer_argumenten(argumenten, 3)?;
        let tekst = self.solve_string_expression(&arguments[0])?;
        let start = self.lees_integer_argument(&arguments[1])?;
        let lengte = self.lees_integer_argument(&arguments[2])?;
        let reply: String;
        let werk_tekst = haal_data(&tekst);

        reply = werk_tekst
            .chars()
            .skip(start.saturating_sub(1))
            .take(lengte)
            .collect::<String>();

        Ok(Waarde::Tekst(EcolString::new(reply)))
    }
    pub(super) fn execute_function_rechts(&mut self, argumenten: &str) -> Result<Waarde, String> {
        let arguments = parseer_argumenten(argumenten, 2)?;
        let tekst = self.solve_string_expression(&arguments[0])?;
        let lengte = self.lees_integer_argument(&arguments[1])?;
        let reply: String;

        let werk_tekst = haal_data(&tekst);
        let totaal = werk_tekst.chars().count();
        reply = werk_tekst
            .chars()
            .skip(totaal.saturating_sub(lengte))
            .collect::<String>();

        Ok(Waarde::Tekst(EcolString::new(reply)))

    }
}