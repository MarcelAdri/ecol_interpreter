const HELP_PAGINA: &str = "ecol_syntaxis.html";

use std::collections::HashMap;
use super::waarden::{haal_data, waarde_naar_expressie, VariabeleType, Waarde, EcolString};
use super::program::{Line, LineInhoud, Sleutelwoord, Operator, TOEKENNING_TEKEN, Functie};

struct FunctieAanroep {
    functie: Functie,
    start: usize,
    einde: usize,
    argumenten: String,
}


impl FunctieAanroep {
    fn new(functie: Functie, start: usize, einde: usize, argumenten: String) -> Self {
        FunctieAanroep {
            functie,
            start,
            einde,
            argumenten,
        }
    }
}

struct VariabeleAanroep {
    variabele_naam: String,
    start: usize,
    einde: usize,
}
 impl VariabeleAanroep {
     fn new(variabele_naam: String, start: usize, einde: usize) -> Self {
         VariabeleAanroep {
             variabele_naam,
             start,
             einde,
         }
     }
 }
pub struct EcolMachine {
    symbolen: HashMap<String, usize>,
    data_pool: Vec<Waarde>,
}

impl EcolMachine {
    pub fn new() -> Self {
        EcolMachine {
            symbolen: HashMap::new(),
            data_pool: Vec::with_capacity(100),
        }
    }

    fn pak_of_maak_index(&mut self, naam: &str, variabele_type: Option<VariabeleType>) -> Result<usize, String> {
        use std::collections::hash_map::Entry;

        let volgende_vrije_index = self.data_pool.len();

        match self.symbolen.entry(naam.to_string()) {
            Entry::Vacant(entry) => {
                let var_type = variabele_type
                    .ok_or_else(|| format!("Variabele '{}' heeft geen type", naam))?;

                self.data_pool.push(Waarde::standaard_voor_type(var_type));
                entry.insert(volgende_vrije_index);
                Ok(volgende_vrije_index)
            }
            Entry::Occupied(entry) => Ok(*entry.get()),
        }
    }

    
    fn haal_variabele_type(&mut self, naam: &str) -> Option<VariabeleType> {
        if let Some(index) = self.symbolen.get(naam) {
            self.data_pool[*index].type_van()
        } else {
            None
        }
    }


    pub fn execute(&mut self, input: &str) -> String {
        let reply: String;

        match lexer(&input){
            Ok(regel) => {
                match &regel.inhoud {
                    LineInhoud::Schrijf { expressie } => {
                        reply = self.execute_schrijf(expressie);
                    },
                    LineInhoud::Tekst {variabele_naam, expressie} => {
                        reply = self.execute_toekennen(variabele_naam, expressie, VariabeleType::Tekst);
                    },
                    LineInhoud::Nr { variabele_naam, expressie } => {
                        reply = self.execute_toekennen(variabele_naam, expressie, VariabeleType::Float);
                    }
                    LineInhoud::Help { } => {
                        reply = self.execute_help();
                    }
                }
            }
            Err(e) => {
                return format!("Fout in execute: {}" ,e);
            }
        }
       
        reply
    }

    fn execute_help(&mut self) -> String {
        if let Some(window) = web_sys::window() {
            let _ = window.open_with_url_and_target(HELP_PAGINA, "_blank");
        }

        "Zie het help-document in ander tabblad.".to_string()
    }

    fn execute_schrijf(&mut self, expressie: &str) -> String {

        match self.solve_expression(expressie) {
            Ok(value) => {
                haal_data(&value)
            }
            Err(error_message) => error_message,
        }
    }


    fn execute_toekennen(&mut self, variabele_naam: &str, expressie: &str, variabele_type: VariabeleType) -> String {

        let variabele_index = match self.pak_of_maak_index(variabele_naam, Some(variabele_type)) {
            Ok(index) => index,
            Err(e) => return e,
        };

        if self.expressie_type(expressie) != variabele_type {
            return format!("Fout in toekennen: {} is niet van het juiste type voor de opgegeven waarde", variabele_naam);
        }
        match self.solve_expression(expressie) {
            Ok(value) => {
                self.data_pool[variabele_index] = value;
                "".to_string()
            }
            Err(error_message) => {
                error_message
            }
        }
    }


    fn expressie_type(&mut self, expressie: &str) -> VariabeleType {
        let mut werk_expressie = geen_spaties_buiten_literals(expressie);

        //1: string of getal?
        let mut is_string = false;
        let first_word = first_word(&werk_expressie);
        //1.1 eerste waarde string literal?
        if werk_expressie.starts_with('"') {
            is_string = true;
        }
        else if werk_expressie.starts_with('(') {
            is_string = false;
        }
        //1.2 eerste waarde een tekst variabele
        else if is_geldige_variabele_naam(first_word) {
            if let Some(variabele_type) = self.haal_variabele_type(first_word) {
                if variabele_type == VariabeleType::Tekst {
                    is_string = true;
                }
            }
        }
        //1.3 eerste waarde een string functie?
        else if Functie::is_string_functie(first_word) {
            is_string = true;
        }

        if is_string {
            VariabeleType::Tekst
        } else {
            VariabeleType::Float
        }
    }

    fn solve_expression(&mut self, expression: &str) -> Result<Waarde, String> {
        let mut werk_expressie = geen_spaties_buiten_literals(expression);

        match self.expressie_type(&werk_expressie) {
            VariabeleType::Tekst => self.solve_string_expression(&werk_expressie),
            VariabeleType::Float => {
                //TODO
                //self.solve_nummer_expression(&werk_expressie, return_type)
                Err("Numerieke expressies zijn nog niet geïmplementeerd".to_string())
            }
        }

    }



    fn solve_string_expression(&mut self, expressie: &str) -> Result<Waarde, String> {
        let mut werk_expressie = expressie.to_string();

        self.vervang_variabelen_in_tekst_expressie(&mut werk_expressie)?;
        self.vervang_functies_in_tekst_expressie(&mut werk_expressie)?;

        let tekst_resultaat = self.samenstellen_tekst_resultaat(&werk_expressie)?;
        Ok(Waarde::Tekst(EcolString::new(tekst_resultaat)))
    }


    fn vervang_variabelen_in_tekst_expressie(&mut self, werk_expressie: &mut String) -> Result<(), String> {
        while let Some(werk_variabele) = parseer_variabele(werk_expressie) {
            let idx = self.pak_of_maak_index(&werk_variabele.variabele_naam, Some(VariabeleType::Tekst))?;
            let complete_waarde = Waarde::Tekst(EcolString::new(&haal_data(&self.data_pool[idx])));
            let result = waarde_naar_expressie(&complete_waarde);

            werk_expressie.replace_range(werk_variabele.start..werk_variabele.einde, &result);
        }

        Ok(())
    }


    fn vervang_functies_in_tekst_expressie(&mut self, werk_expressie: &mut String) -> Result<(), String> {
        while let Some(werk_functie) = parseer_functie(werk_expressie) {
            let uitkomst = self.execute_string_function(&werk_functie)?;
            let result = waarde_naar_expressie(&uitkomst);

            werk_expressie.replace_range(werk_functie.start..werk_functie.einde, &result);
        }

        Ok(())
    }


    fn execute_string_function(&mut self, werk_functie: &FunctieAanroep) -> Result<Waarde, String> {
        let uitkomst = match werk_functie.functie {
            Functie::LinksString => self.execute_function_links(werk_functie.argumenten.as_str()),
            Functie::RechtsString => self.execute_function_rechts(werk_functie.argumenten.as_str()),
            Functie::MiddenString => self.execute_function_midden(werk_functie.argumenten.as_str()),
            _ => return Err(format!("Functie {:?} niet ondersteund", werk_functie.functie)),
        };

        uitkomst.map_err(|e| format!("Fout bij uitvoeren van functie: {:?}: {}", werk_functie.functie, e))
    }


    fn samenstellen_tekst_resultaat(&self, werk_expressie: &str) -> Result<String, String> {

        const ONGELDIGE_TEKST_EXPRESSIE: &str = "Ongeldige tekst-expressie";

        let mut tekst_resultaat = String::new();
        let mut in_quotes = false;
        let mut escape = false;
        let mut plus_seen = false;

        for c in werk_expressie.chars() {
            if !in_quotes {
                match c {
                    '"' => {
                        in_quotes = true;
                        if plus_seen {
                            plus_seen = false;
                        }
                    }
                    '+' if !plus_seen => {
                        plus_seen = true;
                    }
                    ' ' if plus_seen => {}
                    _ if plus_seen => return Err(ONGELDIGE_TEKST_EXPRESSIE.to_string()),
                    _ => {}
                }
            } else {
                if c == '\\' && !escape {
                    escape = true;
                    tekst_resultaat.push(c);
                    continue;
                }

                if escape {
                    escape = false;
                    tekst_resultaat.push(c);
                    continue;
                }

                if c == '"' {
                    in_quotes = false;
                } else {
                    tekst_resultaat.push(c);
                }
            }
        }

        if in_quotes || plus_seen {
            return Err(ONGELDIGE_TEKST_EXPRESSIE.to_string());
        }

        Ok(tekst_resultaat)
    }

    fn execute_function_links(&mut self, argumenten: &str) -> Result<Waarde, String> {
        let arguments = parseer_argumenten(argumenten, 2)?;
        let tekst = self.solve_string_expression(&arguments[0])?;
        let lengte = self.lees_integer_argument(&arguments[1])?;
        let mut reply = String::new();

        let werk_tekst = haal_data(&tekst);
        reply = werk_tekst.chars().take(lengte).collect::<String>();

        Ok(Waarde::Tekst(EcolString::new(reply)))
    }


    fn execute_function_rechts(&mut self, argumenten: &str) -> Result<Waarde, String> {
        let arguments = parseer_argumenten(argumenten, 2)?;
        let tekst = self.solve_string_expression(&arguments[0])?;
        let lengte = self.lees_integer_argument(&arguments[1])?;
        let mut reply = String::new();

        let werk_tekst = haal_data(&tekst);
        let totaal = werk_tekst.chars().count();
        reply = werk_tekst
            .chars()
            .skip(totaal.saturating_sub(lengte))
            .collect::<String>();

        Ok(Waarde::Tekst(EcolString::new(reply)))

    }

    fn execute_function_midden(&mut self, argumenten: &str) -> Result<Waarde, String> {
        let arguments = parseer_argumenten(argumenten, 3)?;
        let tekst = self.solve_string_expression(&arguments[0])?;
        let start = self.lees_integer_argument(&arguments[1])?;
        let lengte = self.lees_integer_argument(&arguments[2])?;
        let mut reply = String::new();
        let werk_tekst = haal_data(&tekst);

        reply = werk_tekst
            .chars()
            .skip(start.saturating_sub(1))
            .take(lengte)
            .collect::<String>();

        Ok(Waarde::Tekst(EcolString::new(reply)))
    }


    fn lees_integer_argument(&mut self, argument: &str) -> Result<usize, String> {
        if is_geldige_variabele_naam(argument) {
            if !(self.haal_variabele_type(argument) == Some(VariabeleType::Float))
            {
                return Err("Argument moet een getal zijn".to_string());
            }

            let idx = self.pak_of_maak_index(argument, Some(VariabeleType::Float))?;
            let complete_waarde = self.data_pool[idx].clone();
            Ok(parse_i32(&haal_data(&complete_waarde)) as usize)
        } else {
            argument
                .parse::<usize>()
                .map_err(|_| "Ongeldige integerwaarde".to_string())
        }
    }
}





fn verbijzonder_argumenten(werk_expressie: &str) -> String {
    let mut argumenten = "".to_string();

    if let Some(i) = werk_expressie.find('(') {
        let inhoud = &werk_expressie[i + 1..];
        if let Some(inhoud) = inhoud.strip_suffix(')') {
            argumenten.push_str(inhoud);
        }
    }
    argumenten
}


fn parseer_functie(expressie: &str) -> Option<FunctieAanroep> {
    let mut stack: Vec<(Functie, usize, usize)> = Vec::new();
    let mut naam_start: Option<usize> = None;
    let mut tussen_quotes = false;

    for (i, c) in expressie.char_indices() {
        if c == '"' {
            tussen_quotes = !tussen_quotes;
            naam_start = None;
            continue;
        }

        if tussen_quotes {
            continue;
        }

        if c.is_ascii_alphabetic() {
            if naam_start.is_none() {
                naam_start = Some(i);
            }
            continue;
        }

        if c.is_ascii_alphanumeric() || c == '_' || c == '$' || c == '%' {
            continue;
        }

        if c == '(' {

            if let Some(start) = naam_start {
                let naam = &expressie[start..i];
                if let Some(functie) = Functie::from_str(naam) {
                    if Functie::is_string_functie(naam) {
                        stack.push((functie, start, i));
                    }
                }
            }
            naam_start = None;
            continue;
        }

        if c == ')' {
            if let Some((functie, start, _open_index)) = stack.pop() {
                let volledige_aanroep = &expressie[start..=i];
                let argumenten = verbijzonder_argumenten(volledige_aanroep);
                return Some(FunctieAanroep::new(functie, start, i + 1, argumenten));
            }
            naam_start = None;
            continue;
        }

        naam_start = None;
    }

    None
}


fn parseer_variabele(expressie: &str) -> Option<VariabeleAanroep> {
    let mut naam_start: Option<usize> = None;
    let mut tussen_quotes = false;

    for (i, c) in expressie.char_indices() {
        if c == '"' {
            tussen_quotes = !tussen_quotes;
            naam_start = None;
            continue;
        }

        if tussen_quotes {
            continue;
        }


        match naam_start {
            None => {
                if c.is_ascii_lowercase() {
                    naam_start = Some(i);
                }
            }
            Some(start) => {
                if c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' {
                    continue;
                }

                let naam = &expressie[start..i];
                if is_geldige_variabele_naam(naam) {
                    return Some(VariabeleAanroep::new(naam.to_string(), start, i));
                }
                naam_start = None;
            }
        }

    }
    if let Some(start) = naam_start {
        let naam = &expressie[start..];
        if is_geldige_variabele_naam(naam) {
            return Some(VariabeleAanroep::new(naam.to_string(), start, expressie.len()));
        }
    }

    None
}


fn lexer(input: &str) -> Result<Line, String> {

    let Some(mut tokens) = parseer_regel(input) else {
        return Err("Syntax fout: regel wordt niet herkend".to_string());
    };


    //Programma?
    if tokens.len() > 0 {
        if tokens[0].parse::<usize>().is_ok() {
            //TODO: hele programma's
            return Err("Programma's worden nog niet ondersteund".to_string());

        }
    }

    //Sleutelwoord?
    match Sleutelwoord::from_string(tokens[0].as_str()) {
        Some(sleutelwoord)=> {
            match sleutelwoord {
                Sleutelwoord::SCHRIJF => {
                    if tokens.len() < 2 {
                        return Err("Geen expressie opgegeven voor SCHRIJF".to_string())
                    }

                    let expressie = tokens[1..].join("");
                    Ok(Line::new(0, LineInhoud::Schrijf { expressie }))
                }
                Sleutelwoord::TEKST | Sleutelwoord::NR => {
                    if tokens.len() < 4 {
                        return Err("Incomplete syntax voor de toekenning".to_string());
                    } else if !is_geldige_variabele_naam(&tokens[1]) {
                        return Err("Ongeldige naam voor de variabele".to_string());
                    } else if tokens[2] != TOEKENNING_TEKEN {
                        return Err("Onjuist toekenning teken".to_string());
                    }

                    let expressie = tokens[3..].join("");
                    let variabele_naam = tokens[1].to_string();
                    if sleutelwoord == Sleutelwoord::TEKST {
                        Ok(Line::new(0, LineInhoud::Tekst { variabele_naam, expressie }))
                    } else {
                        Ok(Line::new(0, LineInhoud::Nr { variabele_naam, expressie }))
                    }

                }
                Sleutelwoord::HELP => {
                    Ok(Line::new(0, LineInhoud::Help {}))
                }
            }

        }
        None => { Err(format!("Onbekend sleutelwoord: {}", tokens[0])) }

    }


}

/// ```rust
/// Parses the input string into a vector of tokens based on specific rules.
///
/// This function processes a given string and tries to tokenize it by analyzing
/// characters, handling special cases such as quoted text, escaped characters,
/// operators, and assignment symbols. The rules for parsing are as follows:
///
/// 1. Text enclosed in double quotes (`"`) is considered a single token, including spaces inside.
///    Escaped quotes (`\"`) are treated as part of the token and disregard the quote-ending semantics.
/// 2. Operators (determined by `Operator::is_operator_char`) are treated as individual tokens.
/// 3. The assignment operator is detected as `:=`. It must be placed explicitly in the input. Any
///    mismatch in syntax results in a failure to parse (returning `None`).
/// 4. Spaces (` `) serve as token delimiters when outside of quotes.
/// 5. Escaped characters (preceded by `\`) appear as-is in the resulting tokens.
///
/// # Parameters:
///
/// - `input`: A string slice (`&str`) that represents the input to be tokenized.
///
/// # Returns:
///
/// - `Some(Vec<String>)`: A vector of tokens if the input was successfully parsed.
/// - `None`: If the input fails to be parsed due to syntax errors (e.g., unclosed quotes,
///   incomplete escape sequences, improperly placed assignment symbols).
///
/// # Example:
///
/// ```rust
/// let input = r#"hello "world with spaces" := operator_example \\"#;
/// let tokens = parseer_regel(input);
/// assert_eq!(
///     tokens,
///     Some(vec![
///         "hello".to_string(),
///         "\"world with spaces\"".to_string(),
///         ":=".to_string(),
///         "operator_example".to_string(),
///         "\\".to_string()
///     ])
/// );
///
/// let invalid_input = r#"hello "unterminated quote"#;
/// assert_eq!(parseer_regel(invalid_input), None);
/// ```
///
/// # Notes:
///
/// - The `Operator::is_operator_char` function is used to determine if a character is part of an operator.
///   Ensure this function is implemented and adheres to the expected rules for operators.
/// - The assignment operator `:=` is validated against a predefined constant `TOEKENNING_TEKEN`. Ensure
///   that `TOEKENNING_TEKEN` is defined elsewhere in the code as `":="`.
/// - Specific edge cases such as consecutive operators, malformed input, and unexpected assignment symbols
///   are handled explicitly to prevent invalid parsing.
///
/// # Errors:
///
/// - Returns `None` if:
///   - A quoted string is left unclosed.
///   - Invalid escape sequences are found.
///   - The assignment operator syntax is violated.
///   - Input fails to form valid tokens.
///
/// fn
fn parseer_regel(input: &str) -> Option<Vec<String>> {
    let mut tokens: Vec<String> = Vec::new();
    let mut tussen_quotes = false;
    let mut escaped = false;
    let mut werk_string = String::new();
    let mut toekenning_teken = false;

    for c in input.chars() {
        if !tussen_quotes {
            if c == '"' {
                tussen_quotes = !tussen_quotes;
                werk_string.push(c);
                continue;
            } else if c == ' ' {
                if !werk_string.is_empty() {
                    tokens.push(std::mem::take(&mut werk_string));
                    werk_string.clear();
                }
                continue;
            } else if Operator::is_operator_char(c) {
                if !werk_string.is_empty() {
                    tokens.push(std::mem::take(&mut werk_string));
                    werk_string.clear();
                }
                tokens.push(c.to_string());
                continue;
            } else if c == ':' {
                toekenning_teken = true;

                if !werk_string.is_empty() {
                    return None;
                }
                werk_string.push(c);
                continue
            } else if toekenning_teken {
                toekenning_teken = false;
                if c != '=' {
                    return None;
                }
                werk_string.push(c);
                if werk_string.to_string() == TOEKENNING_TEKEN {
                    tokens.push(std::mem::take(&mut werk_string));
                } else {
                    return None
                }
                continue;
            }
            werk_string.push(c);
            continue;
        } else {
            if c == '\\' {
                escaped = true;
                werk_string.push(c);
                continue;
            }
            if escaped {
                werk_string.push(c);
                escaped = false;
                continue;
            }
            if c == '"' {
                tussen_quotes = !tussen_quotes;
                werk_string.push(c);
                tokens.push(std::mem::take(&mut werk_string));
                werk_string.clear();
                continue;
            }
            werk_string.push(c);
            continue;
        }
    }

    if tussen_quotes || escaped || toekenning_teken {
        return None;
    }

    if !werk_string.is_empty() {
        tokens.push(werk_string);
    }

    if tokens.len() == 0 {
        return None;
    }

    Some(tokens)
}

/// ```rust
/// Parses a comma-separated string into a vector of arguments with support for quoted and escaped values.
///
/// # Parameters
/// - `argumenten`: A string slice containing the comma-separated arguments. Quoted arguments are supported, and quotes must be properly closed.
/// - `aantal_argumenten`: The expected number of arguments to be parsed. If the actual number of parsed arguments does not match, an error will be returned.
///
/// # Returns
/// - `Ok(Vec<String>)`: A vector containing the parsed arguments as strings if parsing is successful and the number of arguments matches `aantal_argumenten`.
/// - `Err(String)`: An error describing why parsing failed (e.g., invalid quotes, wrong number of arguments).
///
/// # Behavior
/// - Values surrounded by quotes (`"`) are treated as single arguments, even if they contain commas.
/// - Escaped characters (using `\`) within quoted arguments are ignored when determining argument boundaries.
/// - Trailing and leading whitespace around arguments is trimmed.
/// - The function ensures proper quotes and escapes are used and verifies the exact number of parsed arguments.
///
/// # Errors
/// - If the quotes in the input string are not properly closed, returns an error with the message:
///   `"Ongeldige argumentlijst: quotes zijn niet correct afgesloten"`.
/// - If the number of parsed arguments does not match `aantal_argumenten`, returns an error stating:
///   `"Verkeerd aantal argumenten: verwacht X, kreeg Y"` where `X` is the expected count, and `Y` is the actual count.
///
/// # Examples
/// ```rust
/// // Parsing with valid arguments
/// let input = "arg1, \"arg, 2\", arg3";
/// let parsed = parseer_argumenten(input, 3);
/// assert_eq!(parsed.unwrap(), vec!["arg1", "arg, 2", "arg3"]);
///
/// // Parsing with mismatched argument count
/// let result = parseer_argumenten("arg1, arg2", 3);
/// assert!(result.is_err());
///
/// // Parsing with unclosed quotes
/// let result = parseer_argumenten("\"arg1, arg2", 2);
/// assert!(result.is_err());
///
/// // Empty input with expected zero arguments
/// let result = parseer_argumenten("", 0);
/// assert_eq!(result.unwrap(), vec![]);
/// ```
/// ```
fn parseer_argumenten(argumenten: &str, aantal_argumenten: usize) -> Result<Vec<String>, String> {
    let mut result = Vec::new();
    let mut tussen_quotes = false;
    let mut escape = false;
    let mut start = 0;

    for (i, c) in argumenten.char_indices() {
        if escape {
            escape = false;
            continue;
        }

        if tussen_quotes && c == '\\' {
            escape = true;
            continue;
        }

        if c == '"' {
            tussen_quotes = !tussen_quotes;
            continue;
        }

        if !tussen_quotes && c == ',' {
            result.push(argumenten[start..i].trim().to_string());
            start = i + c.len_utf8();
        }
    }

    if escape || tussen_quotes {
        return Err("Ongeldige argumentlijst: quotes zijn niet correct afgesloten".to_string());
    }

    if argumenten.is_empty() {
        if aantal_argumenten == 0 {
            return Ok(Vec::new());
        }
        return Err(format!(
            "Verkeerd aantal argumenten: verwacht {}, kreeg 0",
            aantal_argumenten
        ));
    }

    result.push(argumenten[start..].trim().to_string());

    if result.len() != aantal_argumenten {
        return Err(format!(
            "Verkeerd aantal argumenten: verwacht {}, kreeg {}",
            aantal_argumenten,
            result.len()
        ));
    }

    Ok(result)
}

/// ```rust
/// Parses the given string token into a 32-bit signed integer (`i32`).
///
/// The function attempts to parse the input in the following order:
/// 1. Tries to directly parse the input string as an `i32`.
/// 2. If that fails, it attempts to parse the input string as a `f32`
///    and converts the resulting value to an `i32` using truncation.
/// 3. If both attempts fail, returns `0` as the default fallback value.
///
/// # Arguments
///
/// * `token` - A string slice (`&str`) representing the value to parse.
///
/// # Returns
///
/// * An `i32` value parsed from the input string, or `0` if parsing fails.
///
/// # Examples
///
/// ```
/// let result = parse_i32("42");
/// assert_eq!(result, 42);
///
/// let result = parse_i32("42.7");
/// assert_eq!(result, 42);
///
/// let result = parse_i32("abc");
/// assert_eq!(result, 0);
/// ```
/// fn
fn parse_i32(token: &str) -> i32 {
    if let Ok(value) = token.parse::<i32>() {
        value
    } else if let Ok(value) = token.parse::<f32>() {
        value as i32
    } else {
        0
    }
}

/// ```rust
/// Attempts to parse a string slice (`&str`) into a `f32` (32-bit floating-point number).
///
/// The function follows these steps:
/// 1. It first tries to directly parse the input string into a `f32`.
/// 2. If the above parsing fails, it then tries to parse the string into an `i32`
///    (32-bit integer) and converts the result into a `f32`.
/// 3. If both parsing attempts fail, it defaults to returning `0.0`.
///
/// # Arguments
///
/// * `token` - A string slice that represents the input to be parsed.
///
/// # Returns
///
/// * A `f32` value parsed from the input string if successful.
/// * Returns `0.0` if the string cannot be parsed into either a `f32` or `i32`.
///
/// # Examples
///
/// ```
/// let num = parse_f32("3.14");
/// assert_eq!(num, 3.14);
///
/// let num = parse_f32("42");
/// assert_eq!(num, 42.0);
///
/// let num = parse_f32("invalid");
/// assert_eq!(num, 0.0);
/// ```
/// ```
fn parse_f32(token: &str) -> f32 {
    if let Ok(value) = token.parse::<f32>() {
        value
    } else if let Ok(value) = token.parse::<i32>() {
        value as f32
    } else {
        0.0
    }
}

/// ```rust
/// Parses a given string literal into an `EcolString`.
///
/// # Arguments
///
/// * `token` - A string slice (`&str`) representing the string literal to be parsed.
///
/// # Returns
///
/// A `Result` enum:
/// - `Ok(EcolString)` if the parsing is successful.
/// - `Err(String)` if there is an error during parsing, containing a description of the error.
///
/// # Example
///
/// ```rust
/// let token = "example";
/// match parse_string(token) {
///     Ok(ecol_string) => println!("Parsed successfully: {:?}", ecol_string),
///     Err(err) => eprintln!("Parsing failed: {}", err),
/// }
/// ```
///
/// # Errors
///
/// Returns an `Err` if `EcolString::from_literal` fails to parse the provided `token`.
///
/// # Note
///
/// This function relies on the implementation of `EcolString::from_literal`.
/// ```
fn parse_string(token: &str) -> Result<EcolString, String> {
    EcolString::from_literal(token)

}


/// ```rust
/// Checks whether the given string is a valid variable name.
///
/// This function verifies if the provided `naam` satisfies the following conditions:
/// 1. The name adheres to the syntax rules for valid variable names, determined by the
///    `heeft_geldige_variabele_syntax` function.
/// 2. The name is neither a reserved function name (`Functie::is_functie`) nor a reserved
///    keyword (`Sleutelwoord::is_sleutelwoord`).
///
/// # Parameters
/// - `naam`: A string slice representing the name to be validated.
///
/// # Returns
/// - `true` if the provided name is valid and not reserved.
/// - `false` otherwise.
///
/// # Example
/// ```rust
/// assert!(is_geldige_variabele_naam("my_variable"));
/// assert!(!is_geldige_variabele_naam("fn")); // `fn` is a reserved keyword
/// assert!(!is_geldige_variabele_naam("123abc")); // Invalid syntax
/// ```
/// ```
pub fn is_geldige_variabele_naam(naam: &str) -> bool {
    heeft_geldige_variabele_syntax(naam)


}


/// ```rust
/// Checks whether a given string is a valid variable name according to specific syntax rules.
///
/// # Syntax Rules:
/// 1. The first character must be a lowercase ASCII letter.
/// 2. Subsequent characters (if any) must either be lowercase ASCII letters,
///    digits, or underscores (`_`), except the last character, which must be
///    either a lowercase ASCII letter or a digit.
///
/// # Arguments:
/// * `naam` - A string slice containing the variable name to validate.
///
/// # Returns:
/// * `true` if the string conforms to the syntax rules for a valid variable name.
/// * `false` otherwise.
///
/// # Examples:
/// ```
/// assert_eq!(heeft_geldige_variabele_syntax("var_1"), true);  // Valid: starts with lowercase letter and follows rules.
/// assert_eq!(heeft_geldige_variabele_syntax("9variable"), false); // Invalid: starts with digit.
/// assert_eq!(heeft_geldige_variabele_syntax("variable_"), false); // Invalid: ends with an underscore.
/// assert_eq!(heeft_geldige_variabele_syntax("a"), true);        // Valid: single lowercase letter.
/// assert_eq!(heeft_geldige_variabele_syntax(""), false);        // Invalid: empty string.
/// ```
/// ```
fn heeft_geldige_variabele_syntax(naam: &str) -> bool {
    let mut chars = naam.chars();

    let Some(eerste) = chars.next() else {
        return false;
    };

    if !eerste.is_ascii_lowercase() {
        return false;
    }

    let rest: Vec<char> = chars.collect();

    if rest.is_empty() {
        return true;
    }

    for (i, &c) in rest.iter().enumerate() {
        let is_laatste = i == rest.len() - 1;

        if is_laatste {
            if !(c.is_ascii_lowercase() || c.is_ascii_digit()) {
                return false;
            }
        } else if !(c.is_ascii_lowercase() || c == '_' || c.is_ascii_digit()) {
            return false;
        }
    }

    true
}

/// ```rust
/// Returns the first word from the given input string, stopping at the first
/// occurrence of an operator character or an open parenthesis.
///
/// This function iterates over the characters in the input string along with
/// their byte indices. When it encounters a character that is either an operator
/// character (as determined by `Operator::is_operator_char`) or an open parenthesis
/// `(`, it slices the string up to (but not including) that character and returns it.
/// If no such character is found, the entire input string is returned.
///
/// # Arguments
///
/// * `input` - A string slice (`&str`) from which the first word will be extracted.
///
/// # Returns
///
/// * A string slice containing the first word from the input string.
///
/// # Examples
///
/// ```
/// struct Operator;
/// impl Operator {
///     fn is_operator_char(c: char) -> bool {
///         "+-*/".contains(c)
///     }
/// }
///
/// let input = "hello+world";
/// let result = first_word(input);
/// assert_eq!(result, "hello");
///
/// let input = "(test+data)";
/// let result = first_word(input);
/// assert_eq!(result, "");
///
/// let input = "no_operator_here";
/// let result = first_word(input);
/// assert_eq!(result, "no_operator_here");
/// ```
///
/// # Note
///
/// Ensure that the `Operator` struct and its `is_operator_char` method are defined,
/// as they are prerequisites for this function to work as expected.
/// ```
fn first_word(input: &str) -> &str {
    for (i, c) in input.char_indices() {
        if Operator::is_operator_char(c) || c == '(' {
            return &input[..i];
        }
    }
    input
}

/// ```rust
/// Removes spaces from a given input string, except for spaces that are inside string literals.
/// A string literal is defined as text enclosed in double quotes (`"`) in the input.
///
/// - Spaces outside of string literals are entirely removed.
/// - Escape sequences (e.g., `\"`) within string literals are properly handled, ensuring
///   quotes inside strings are not interpreted incorrectly.
///
/// # Parameters
/// - `input`: A string slice (`&str`) representing the input text to process.
///
/// # Returns
/// A new `String` with spaces removed unless they appear inside string literals.
///
/// # Examples
/// ```
/// let input = r#" This is "a test" string with " multiple spaces " outside "#;
/// let result = geen_spaties_buiten_literals(input);
/// assert_eq!(result, r#"Thisis"a test"stringwith" multiple spaces "outside"#);
/// ```
///
/// ```
/// let input = r#"" Keep spaces   inside", but remove   outside "#;
/// let result = geen_spaties_buiten_literals(input);
/// assert_eq!(result, r#"" Keep spaces   inside",butremoveoutside"#);
/// ```
/// fn
fn geen_spaties_buiten_literals(input: &str) -> String {
    let mut result = String::new();
    let mut in_string = false;
    let mut escape = false;

    for c in input.chars() {
        if !in_string {
            if c == '"' {
                in_string = true;
                result.push(c);
                continue;
            }
            if c.is_whitespace() {
                continue;
            }
            result.push(c);
        } else {
            if escape {
                escape = false;
                result.push(c);
                continue;
            }
            if c == '\\' {
                escape = true;
                result.push(c);
                continue;
            }
            if c == '"' {
                in_string = false;
                result.push(c);
                continue;

            }
            result.push(c);
        }
    }
    result
}


