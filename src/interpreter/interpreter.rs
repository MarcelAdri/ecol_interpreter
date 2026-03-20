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

    /// ```rust
    /// Retrieves the variable type associated with a given name from the symbol table.
    ///
    /// This function looks up the provided variable name (`naam`) in the symbol table (`symbolen`).
    /// If the name exists in the table, it retrieves its index in the data pool (`data_pool`) and
    /// returns the associated variable type. If the name does not exist in the symbol table, the
    /// function returns `None`.
    ///
    /// # Parameters
    ///
    /// - `naam`: A string slice (`&str`) representing the name of the variable to look up.
    ///
    /// # Returns
    ///
    /// - `Option<VariabeleType>`:
    ///     - `Some(VariabeleType)` if the name exists in the symbol table and the variable type
    ///       is successfully retrieved from the data pool.
    ///     - `None` if the name does not exist in the symbol table.
    ///
    /// # Example
    /// ```rust
    /// let mut mijn_struct = MijnStruct::new();
    /// mijn_struct.voeg_toe("x", VariabeleType::Integer);
    ///
    /// if let Some(type_van) = mijn_struct.haal_variabele_type("x") {
    ///     println!("Het type van 'x' is: {:?}", type_van);
    /// } else {
    ///     println!("'x' bestaat niet in de symbolentabel.");
    /// }
    /// ```
    /// //
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
                    LineInhoud::Toekennen {variabele_naam, expressie} => {
                        reply = self.execute_toekennen(variabele_naam, expressie);
                    },
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

    /// ```
    /// Executes the assignment of a value (defined by an expression) to a variable.
    ///
    /// This function handles assigning the result of an expression to a variable, dynamically determining
    /// the variable type if it has not been previously declared. It involves the following steps:
    /// 1. Determines the type of the variable based on prior declarations or evaluates the type from the expression.
    /// 2. Finds or creates the associated index for the variable in the `data_pool`.
    /// 3. Evaluates the given expression and assigns the result to the variable.
    ///
    /// # Arguments
    /// - `variabele_naam` (`&str`): The name of the variable to which the value will be assigned.
    /// - `expressie` (`&str`): The expression whose result will be assigned to the variable.
    ///
    /// # Returns
    /// A `String` indicating the result of executing the assignment:
    /// - If successful, an empty string (`""`) is returned.
    /// - If an error occurs (e.g., during type handling, index creation, or expression solving), an error message is returned.
    ///
    /// # Errors
    /// - Returns an error message if:
    ///   - The variable index creation (`pak_of_maak_index`) fails.
    ///   - The expression evaluation (`solve_expression`) fails.
    ///
    /// # Example
    /// ```
    /// let mut executor = MyExecutor::new();
    /// let result = executor.execute_toekennen("my_var", "10 + 20");
    /// assert_eq!(result, ""); // Successfully assigned value.
    /// ```
    /// ```
    fn execute_toekennen(&mut self, variabele_naam: &str, expressie: &str) -> String {
        let mut var_type: VariabeleType;

        if let Some(variabeleType) = self.haal_variabele_type(variabele_naam) {
            var_type = variabeleType;
        } else {
            var_type = self.expressie_type(expressie);
        }

        let variabele_index = match self.pak_of_maak_index(variabele_naam, Some(var_type)) {
            Ok(index) => index,
            Err(e) => return e,
        };

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

    /// ```rust
    /// Determines the type of an expression as either a string or a number.
    ///
    /// # Parameters
    /// - `&mut self`: A mutable reference to the current object, which is used to access
    ///   stored variable types and corresponding helper methods.
    /// - `expressie: &str`: A string reference representing the expression to evaluate.
    ///   The expression is analyzed to determine its type.
    ///
    /// # Returns
    /// A `VariabeleType` enumerator:
    /// - Returns `VariabeleType::Tekst` if the expression is determined to be a string.
    /// - Returns `VariabeleType::Float` if the expression is determined to be a numeric value.
    ///
    /// # Process
    /// 1. The function removes spaces outside literals from the given expression by calling
    ///    `geen_spaties_buiten_literals`.
    /// 2. It determines whether the expression represents a string:
    ///     - If the expression starts with a double quote (`"`) it is identified as a string.
    ///     - If the expression starts with an opening parenthesis (`(`), it is assumed not to be a string.
    ///     - If the first word of the expression is a valid variable name:
    ///       - It checks if the variable is of type `Tekst` by querying `self.haal_variabele_type`.
    ///     - If the first word of the expression is a string function, verified by
    ///       `Functie::is_string_functie`, the expression is identified as a string.
    /// 3. Based on the analysis, the function returns `VariabeleType::Tekst` for string-like
    ///    expressions, or `VariabeleType::Float` for numeric expressions.
    ///
    /// # Assumptions
    /// - The helper functions including `geen_spaties_buiten_literals`, `first_word`,
    ///   `is_geldige_variabele_naam`, `self.haal_variabele_type`, and `Functie::is_string_functie`
    ///   are assumed to function correctly and provide valid outputs.
    ///
    /// # Note
    /// This method may evaluate expressions in a simplified manner, and the logic may need to
    /// be extended if more nuanced expression evaluation is needed in the future.
    /// ```
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


    /// ```rust
    /// Solves a string-based mathematical or logical expression by processing variables
    /// and function calls within the expression and returns the resulting value.
    ///
    /// # Parameters
    /// - `expressie`: A string slice representing the initial expression to be solved.
    ///   This expression may contain variables and functions that need to be processed.
    ///
    /// # Returns
    /// - `Result<Waarde, String>`:
    ///   - On success, it returns a `Waarde::Tekst` containing the final result of the evaluated expression as a string.
    ///   - On failure, it returns a `String` with the error message detailing what went wrong.
    ///
    /// # Process
    /// 1. A mutable copy of the input expression (`werk_expressie`) is created.
    /// 2. Calls `vervang_variabelen_in_tekst_expressie` to replace variables within the expression.
    /// 3. Calls `vervang_functies_in_tekst_expressie` to replace function calls within the expression.
    /// 4. Combines the processed expression into the final string result via
    ///    `samenstellen_tekst_resultaat`.
    ///
    /// # Errors
    /// This function may return an error in the following cases:
    /// - If replacing variables within the expression fails during the call to `vervang_variabelen_in_tekst_expressie`.
    /// - If replacing function calls within the expression fails during the call to `vervang_functies_in_tekst_expressie`.
    /// - If constructing the final result from the processed expression fails during the call to `samenstellen_tekst_resultaat`.
    ///
    /// # Examples
    /// ```
    /// let mut solver = MySolver::new();
    /// let expression = "2 + x * my_function(y)";
    ///
    /// match solver.solve_string_expression(expression) {
    ///     Ok(result) => println!("Solved Expression: {:?}", result),
    ///     Err(error) => println!("Error Solving Expression: {}", error),
    /// }
    /// ```
    /// ```
    fn solve_string_expression(&mut self, expressie: &str) -> Result<Waarde, String> {
        let mut werk_expressie = expressie.to_string();

        self.vervang_variabelen_in_tekst_expressie(&mut werk_expressie)?;
        self.vervang_functies_in_tekst_expressie(&mut werk_expressie)?;

        let tekst_resultaat = self.samenstellen_tekst_resultaat(&werk_expressie)?;
        Ok(Waarde::Tekst(EcolString::new(tekst_resultaat)))
    }

    /// ```rust
    /// /**
    ///  * Replaces variables in a text expression with their corresponding values.
    ///  *
    ///  * This function iterates through the provided textual expression, identifies variables,
    ///  * and substitutes them with their evaluated values. Variables are evaluated using data
    ///  * stored in the `data_pool`.
    ///  *
    ///  * # Parameters
    ///  *
    ///  * - `&mut self`: A mutable reference to the current instance of the object.
    ///  * - `werk_expressie: &mut String`: A mutable reference to the string containing the
    ///  *   expression in which variables will be substituted.
    ///  *
    ///  * # Returns
    ///  *
    ///  * - `Result<(), String>`: Returns `Ok(())` if all variables are successfully replaced.
    ///  *   Returns an `Err(String)` with an error message if any error occurs during processing.
    ///  *
    ///  * # Process
    ///  *
    ///  * 1. The function continuously parses the `werk_expressie` to identify variables using
    ///  *    the `parseer_variabele` function.
    ///  * 2. For each identified variable:
    ///  *    - Retrieves or creates the corresponding index in the `data_pool` using `pak_of_maak_index`.
    ///  *    - Extracts the complete value (`Waarde::Tekst`) from `data_pool` using the `haal_data` function.
    ///  *    - Converts the value into a text-compatible format via `waarde_naar_expressie`.
    ///  *    - Replaces the variable placeholder in the `werk_expressie` with the calculated value.
    ///  * 3. If no errors are encountered, the function returns `Ok(())`.
    ///  *
    ///  * # Errors
    ///  *
    ///  * The function returns an `Err(String)` in case of any processing errors, such as:
    ///  * - Failing to parse variables.
    ///  * - Issues with accessing or retrieving data from `data_pool`.
    ///  * - Errors in the value conversion process (`waarde_naar_expressie`).
    ///  *
    ///  * # Example
    ///  *
    ///  * ```rust
    ///  * let mut processor = Processor::new(data_pool);
    ///  * let mut expression = "Hallo, {{naam}}!".to_string();
    ///  *
    ///  * match processor.vervang_variabelen_in_tekst_expressie(&mut expression) {
    ///  *     Ok(()) => println!("Processed expression: {}", expression),
    ///  *     Err(err) => eprintln!("Error: {}", err),
    ///  * }
    ///  * ```
    ///  *
    ///  * In this example, if `data_pool` contains a variable `naam` with the value `"John"`,
    ///  * the resulting `expression` will become `"Hallo, John!"`.
    ///  */
    /// ```
    fn vervang_variabelen_in_tekst_expressie(&mut self, werk_expressie: &mut String) -> Result<(), String> {
        while let Some(werk_variabele) = parseer_variabele(werk_expressie) {
            let idx = self.pak_of_maak_index(&werk_variabele.variabele_naam, Some(VariabeleType::Tekst))?;
            let complete_waarde = Waarde::Tekst(EcolString::new(&haal_data(&self.data_pool[idx])));
            let result = waarde_naar_expressie(&complete_waarde);

            werk_expressie.replace_range(werk_variabele.start..werk_variabele.einde, &result);
        }

        Ok(())
    }

    /// ```rust
    /// Replaces function calls in the given string expression with their evaluated results.
    ///
    /// This method repeatedly parses and executes functions found within the provided string expression.
    /// Each identified function is evaluated, and its result replaces the function call in the string.
    ///
    /// # Arguments
    ///
    /// * `werk_expressie` - A mutable reference to the string containing the expression where
    ///   function calls will be replaced with their results.
    ///
    /// # Returns
    ///
    /// * `Result<(), String>` - Returns `Ok(())` if all function calls are successfully evaluated and replaced.
    ///   Returns `Err(String)` if an error occurs during the execution of any function.
    ///
    /// # Behavior
    ///
    /// 1. Uses `parseer_functie` to identify the next function call in the expression.
    /// 2. Executes the parsed function call using `self.execute_string_function`.
    /// 3. Converts the result of the execution to a string representation using `waarde_naar_expressie`.
    /// 4. Replaces the function call in `werk_expressie` with the evaluated result.
    /// 5. Repeats the process until no more function calls are found in `werk_expressie`.
    ///
    /// # Errors
    ///
    /// This function returns an error in the following cases:
    /// - If `parseer_functie` or `self.execute_string_function` encounters an error.
    /// - If any other unexpected behavior occurs during parsing or execution.
    ///
    /// # Example
    ///
    /// ```rust
    /// let mut calculator = Calculator::new();
    /// let mut expression = String::from("sin(0) + cos(0)");
    /// calculator.vervang_functies_in_tekst_expressie(&mut expression).unwrap();
    /// assert_eq!(expression, "0 + 1"); // Assuming appropriate implementations.
    /// ```
    /// ```
    fn vervang_functies_in_tekst_expressie(&mut self, werk_expressie: &mut String) -> Result<(), String> {
        while let Some(werk_functie) = parseer_functie(werk_expressie) {
            let uitkomst = self.execute_string_function(&werk_functie)?;
            let result = waarde_naar_expressie(&uitkomst);

            werk_expressie.replace_range(werk_functie.start..werk_functie.einde, &result);
        }

        Ok(())
    }

    /// ```rust
    /// Executes a string manipulation function based on the given `werk_functie`.
    ///
    /// This method determines which string-related function (`LinksString`,
    /// `RechtsString`, or `MiddenString`) should be executed based on the
    /// `werk_functie.functie` field. It delegates the execution to specialized
    /// helper functions. If the function type is unsupported, an error is returned.
    ///
    /// # Parameters
    /// - `werk_functie`: A reference to a `FunctieAanroep` struct that contains
    ///   the function to be executed and its associated arguments.
    ///
    /// # Returns
    /// A `Result` with the following:
    /// - `Ok(Waarde)`: The result of the executed function if the operation succeeds.
    /// - `Err(String)`: An error message if the execution fails or if the
    ///   specified function is not supported.
    ///
    /// # Errors
    /// Returns an error if:
    /// - The function specified in `werk_functie.functie` is not supported.
    /// - An error occurred while executing the specific string manipulation function.
    ///
    /// # Examples
    /// ```rust
    /// let mut executor = Executor::new();
    /// let werk_functie = FunctieAanroep {
    ///     functie: Functie::LinksString,
    ///     argumenten: String::from("example argument"),
    /// };
    ///
    /// match executor.execute_string_function(&werk_functie) {
    ///     Ok(result) => println!("Result: {:?}", result),
    ///     Err(err) => println!("Error: {}", err),
    /// }
    /// ```
    ///
    /// # Notes
    /// - This function ensures error messages are descriptive by including the type
    ///   of the function and the underlying error details.
    ///
    /// # Related Functions
    /// - `execute_function_links`
    /// - `execute_function_rechts`
    /// - `execute_function_midden`
    /// ```
    fn execute_string_function(&mut self, werk_functie: &FunctieAanroep) -> Result<Waarde, String> {
        let uitkomst = match werk_functie.functie {
            Functie::LinksString => self.execute_function_links(werk_functie.argumenten.as_str()),
            Functie::RechtsString => self.execute_function_rechts(werk_functie.argumenten.as_str()),
            Functie::MiddenString => self.execute_function_midden(werk_functie.argumenten.as_str()),
            _ => return Err(format!("Functie {:?} niet ondersteund", werk_functie.functie)),
        };

        uitkomst.map_err(|e| format!("Fout bij uitvoeren van functie: {:?}: {}", werk_functie.functie, e))
    }

    /// ```rust
    /// Verwerkt een werk-expressie naar een resultaatstring of geeft een foutmelding terug.
    ///
    /// Deze functie analyseert de input 'werk_expressie' op basis van een aangepaste
    /// parserlogica waarin een tekst-expressie wordt verwerkt. De expressie ondersteunt
    /// strings tussen dubbele aanhalingstekens en bevat validatieregels om syntaxfouten
    /// te voorkomen.
    ///
    /// # Parameters
    /// - `&self`: Een referentie naar de instantie van de struct waarin deze functie is gedefinieerd.
    /// - `werk_expressie`: Een referentie naar een string slice die de werk-expressie bevat.
    ///
    /// # Returns
    /// - `Ok(String)`: Geeft een geconstrueerde string terug waarin de gelezen tekst
    ///   binnen quotes is samengevoegd.
    /// - `Err(String)`: Geeft een foutmelding terug als de expressie een ongeldig formaat heeft.
    ///
    /// # Gedragsregels
    /// - Tekstensnippers moeten tussen dubbele aanhalingstekens (`"`) staan.
    /// - Escape-karakters (`\`) worden ondersteund binnen quotes.
    /// - Het plus-teken (`+`) mag niet worden gebruikt buiten de context van geldige tekstsamenstellingen.
    /// - Spaties zijn toegestaan maar mogen de validatie van het plus-teken niet verstoren.
    ///
    /// # Foutgevallen
    /// De functie retourneert een foutmelding als:
    /// - Een dubbele aanhalingstekengroep niet correct wordt afgesloten.
    /// - Het plus-teken (`+`) wordt gebruikt in een foutieve volgorde.
    /// - Andere syntaxfouten worden aangetroffen.
    ///
    /// # Voorbeeld
    /// ```rust
    /// let werk_expressie = r#""Hallo" + "wereld""#;
    /// let resultaat = my_instance.samenstellen_tekst_resultaat(werk_expressie);
    /// assert_eq!(resultaat, Ok("Hallo".to_string()));
    ///
    /// let fout_expressie = r#""Hallo" + wereld"#;
    /// let resultaat = my_instance.samenstellen_tekst_resultaat(fout_expressie);
    /// assert!(resultaat.is_err());
    /// ```
    ///
    /// # Opmerkingen
    /// Tekst-expressies moeten voldoen aan specifieke syntactische regels
    /// zoals hierboven beschreven. Onjuiste constructies genereren een fout om
    /// consistentie te garanderen.
    ///
    /// # Constanten
    /// - `ONGELDIGE_TEKST_EXPRESSIE`: Bevat het bericht dat wordt geretourneerd in
    ///   geval van een syntactische fout.
    /// ```
    fn samenstellen_tekst_resultaat(&self, werk_expressie: &str) -> Result<String, String> {
        /// ```rust
        /// A constant string used to represent an invalid text expression.
        ///
        /// This constant can be utilized to provide standardized error messages
        /// or validations when the text expression provided does not meet certain criteria.
        ///
        /// # Value
        /// "Ongeldige tekst-expressie" (Dutch for "Invalid text expression")
        ///
        /// # Usage
        /// This constant can be used in scenarios where invalid input or data
        /// needs to be flagged or processed consistently throughout the application.
        ///
        /// Example:
        /// ```rust
        /// if is_text_invalid(input_text) {
        ///     println!("{}", ONGELDIGE_TEKST_EXPRESSIE);
        /// }
        /// ```
        ///
        /// # Notes
        /// - Ensure proper validation logic when using this constant to avoid false positives.
        /// ```
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




    /// ```rust
    /// Executes the `execute_function_links` method, which processes a string input and extracts a substring
    /// based on the specified length.
    ///
    /// # Parameters
    /// - `argumenten`: A string slice containing the arguments to be parsed. This should consist of two
    ///     components separated by a delimiter:
    ///     1. A string expression to be solved.
    ///     2. An integer specifying the number of characters to extract from the start of the string.
    ///
    /// # Returns
    /// A `Result`:
    /// - `Ok(Waarde::Tekst(EcolString))`: If the operation succeeds, it returns a `Waarde` variant containing
    ///     the substring wrapped in an `EcolString`.
    /// - `Err(String)`: If any parsing or processing error occurs, it returns an error message describing
    ///     the issue.
    ///
    /// # Workflow
    /// 1. The `argumenten` string is parsed into two components using the `parseer_argumenten` function
    ///     with an expected argument count of 2.
    /// 2. The first component is interpreted as a string expression and solved using `self.solve_string_expression`.
    /// 3. The second component is interpreted as an integer, representing the number of characters to extract,
    ///     which is retrieved through the `self.lees_integer_argument` method.
    /// 4. The resulting string expression is processed through `haal_data`, and characters up to the specified length
    ///     are collected into a new string.
    /// 5. The resulting substring is wrapped in an `EcolString` and returned.
    ///
    /// # Errors
    /// This function may return an error in the following cases:
    /// - If the `argumenten` string cannot be parsed or does not have exactly 2 arguments.
    /// - If the string expression provided cannot be solved.
    /// - If the second argument cannot be parsed as a valid integer.
    /// - If other unexpected errors occur during processing.
    ///
    /// # Examples
    /// ```rust
    /// // Example usage
    /// let mut instance = MyStruct::new();
    /// match instance.execute_function_links("input_string,5") {
    ///     Ok(result) => {
    ///         println!("Result: {:?}", result);
    ///     }
    ///     Err(e) => {
    ///         eprintln!("Error: {}", e);
    ///     }
    /// }
    /// ```
    /// ```
    fn execute_function_links(&mut self, argumenten: &str) -> Result<Waarde, String> {
        let arguments = parseer_argumenten(argumenten, 2)?;
        let tekst = self.solve_string_expression(&arguments[0])?;
        let lengte = self.lees_integer_argument(&arguments[1])?;
        let mut reply = String::new();

        let werk_tekst = haal_data(&tekst);
        reply = werk_tekst.chars().take(lengte).collect::<String>();

        Ok(Waarde::Tekst(EcolString::new(reply)))
    }



    /// ```rust
    /// Executes the `rechts` function, extracting a specified number of characters
    /// from the end of a given string.
    ///
    /// # Parameters
    /// - `argumenten`: A `&str` containing the arguments for the function,
    ///   which should include two values:
    ///   1. A string expression to be evaluated.
    ///   2. An integer specifying the number of characters to extract from the end of the string.
    ///
    /// # Returns
    /// - `Result<Waarde, String>`:
    ///   - On success, returns `Waarde::Tekst`, which contains the resulting string extracted
    ///     from the end of the input string.
    ///   - On failure, returns an error message as a `String`.
    ///
    /// # Behavior
    /// - The function first parses the arguments to validate and retrieve the required input values.
    /// - It evaluates the first argument to obtain a string (`tekst`).
    /// - It processes the second argument to get the integer (`lengte`) representing the number of
    ///   characters to extract.
    /// - If the length of the string is less than the specified length (`lengte`), the function
    ///   safely handles this by extracting as much of the string as possible without error.
    /// - The string extraction is performed by analyzing the character positions and skipping the
    ///   excess characters from the beginning.
    ///
    /// # Errors
    /// - Returns an error if:
    ///   - The arguments are invalid or less/more than expected.
    ///   - The string expression cannot be resolved.
    ///   - The integer argument is invalid or not parsable.
    ///
    /// # Examples
    /// ```
    /// let mut context = ...; // Assume this is the initialized context with `solve_string_expression` implementation.
    /// let result = context.execute_function_rechts("\"Hello, World!\", 5");
    /// assert_eq!(result.unwrap(), Waarde::Tekst(EcolString::new(String::from("orld!"))));
    /// ```
    ///
    /// In this example, the function extracts the last 5 characters ("orld!") from the string
    /// `"Hello, World!"`.
    /// ```
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



    /// ```rust
    /// Executes the `midden` function, which extracts a substring from a given string based on
    /// specified start and length parameters. The function performs validations and computations
    /// to ensure proper slicing of the input string.
    ///
    /// # Arguments
    /// * `argumenten` - A string slice containing the arguments for the function, separated by delimiters
    ///   (e.g., commas). This should be in the form: "<tekst>,<start>,<lengte>", where:
    ///   - `<tekst>` is the input string from which the substring will be extracted.
    ///   - `<start>` is the 1-based starting position for the substring.
    ///   - `<lengte>` is the length of the substring to extract.
    ///
    /// # Returns
    /// * `Ok(Waarde::Tekst(EcolString))` - A result containing the extracted substring encapsulated
    ///   in the `Waarde::Tekst` enum variant if successful.
    /// * `Err(String)` - A result containing an error message string if an error occurs during
    ///   processing (e.g., invalid arguments, parsing errors, or out-of-bounds access).
    ///
    /// # Errors
    /// Returns an error in the following scenarios:
    /// * The `argumenten` string cannot be parsed into exactly three components.
    /// * Resolving the string expression for `<tekst>` fails.
    /// * Parsing the integer values for `<start>` or `<lengte>` fails.
    /// * If the slicing indexes (<start> or <lengte>) lead to invalid or out-of-bounds positions.
    ///
    /// # Details
    /// * Extraction is 1-based for the `start` argument, meaning a `start` value of 1 corresponds
    ///   to the first character of the string.
    /// * If the `start` position is less than 1, it will be clamped to begin extraction from the
    ///   first character of the `werk_tekst`.
    /// * The slicing operation ensures that characters are taken only within the valid bounds of
    ///   the string to avoid panics.
    ///
    /// # Example
    /// ```rust
    /// let mut context = Context::new();
    /// let result = context.execute_function_midden("hello world,7,5");
    /// assert_eq!(result.unwrap(), Waarde::Tekst(EcolString::new("world".to_string())));
    /// ```
    /// ```
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


    /// ```rust
    /// Parses an integer argument, either by interpreting it as a variable name
    /// or directly as a raw integer value provided in string format.
    ///
    /// # Arguments
    /// - `argument`: A string slice that represents either a variable name or a string containing an integer.
    ///
    /// # Returns
    /// - `Ok(usize)`: Returns the parsed integer as `usize` if successful.
    /// - `Err(String)`: Returns an error message in case of failure to parse the integer.
    ///
    /// # Behavior
    /// 1. If the provided `argument` is a valid variable name (as determined by `is_geldige_variabele_naam`):
    ///     - Checks whether its data type is not `Float`, and if so, returns an error "Argument moet een getal zijn".
    ///     - Otherwise, it retrieves or creates the index for this variable (via `pak_of_maak_index`) and
    ///       fetches the complete data through the `data_pool`. The retrieved value is parsed as a 32-bit integer
    ///       and then cast to `usize`.
    /// 2. If `argument` is not a valid variable name:
    ///     - Attempts to parse it directly as a `usize`.
    ///     - If parsing fails, returns an error "Ongeldige integerwaarde".
    ///
    /// # Errors
    /// This function will return an error in the following scenarios:
    /// - The variable name exists, but its type is not compatible (e.g., it is not a numeric type like `Float`).
    /// - `argument` is not a valid integer or variable name.
    /// - Other failures related to parsing or data retrieval.
    ///
    /// # Example Usage
    /// ```rust
    /// let mut interpreter = Interpreter::new();
    ///
    /// // Assuming `x` is a valid variable with a numeric value
    /// let result = interpreter.lees_integer_argument("x");
    /// assert!(result.is_ok());
    ///
    /// // Parsing a raw integer string directly
    /// let result = interpreter.lees_integer_argument("42");
    /// assert_eq!(result.unwrap(), 42);
    ///
    /// // Invalid case
    /// let result = interpreter.lees_integer_argument("invalid");
    /// assert!(result.is_err());
    /// ```
    /// ```
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




/// ```rust
/// Extracts the arguments from a given function-like expression.
///
/// This function takes a string slice that represents a function or method
/// expression (e.g., "functie(arg1, arg2)"). It identifies and extracts the
/// arguments enclosed within parentheses, returning them as a `String`. If the
/// input does not contain parentheses, or if they are improperly formatted,
/// the function returns an empty string.
///
/// # Arguments
///
/// * `werk_expressie` - A string slice containing the expression to parse.
///
/// # Returns
///
/// A `String` containing the arguments extracted from the input expression.
/// If no arguments are found or the input is improperly formatted, an empty
/// string is returned.
///
/// # Examples
///
/// ```
/// let input = "functie(waarde1, waarde2)";
/// let result = verbijzonder_argumenten(input);
/// assert_eq!(result, "waarde1, waarde2");
///
/// let no_arguments = "andere_functie()";
/// let result_empty = verbijzonder_argumenten(no_arguments);
/// assert_eq!(result_empty, "");
///
/// let malformed = "foutieve_syntax)";
/// let result_malformed = verbijzonder_argumenten(malformed);
/// assert_eq!(result_malformed, "");
/// ```
/// ```
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

/// ```
/// Parses a function call expression from the given string.
///
/// This function iterates through the characters of the provided expression string (`expressie`)
/// and attempts to identify and parse valid function calls. The parser supports alphanumeric function names
/// as well as names containing the characters `_`, `$`, and `%`. It also considers quoted parts of the string
/// to ensure correct parsing of arguments or strings within the expression.
///
/// # Parameters
/// - `expressie`: A string slice containing the expression to parse.
///
/// # Returns
/// - `Option<FunctieAanroep>`:
///     - `Some(FunctieAanroep)` if a valid function call is identified and successfully parsed.
///     - `None` if no valid function call is found in the given expression.
///
/// # Behavior
/// 1. Handles quoted strings:
///    - Characters within quotes (`"`) are ignored during the parsing process.
/// 2. Detects function start and end:
///    - Identifies potential function names, starting from alphabetic characters.
///    - Confirms function names when encountering an opening parenthesis (`(`).
/// 3. Manages a stack to handle nested function calls:
///    - Each detected function is stored on the stack with its start position and name.
///    - On encountering a closing parenthesis (`)`), attempts to complete the innermost function call.
/// 4. Extracts arguments using `verbijzonder_argumenten` (presumed to be an external helper function).
///
/// # Example
/// ```rust
/// // Example usage of the `parseer_functie` function:
/// let expressie = "example_func(arg1, \"string with (parentheses)\")";
/// if let Some(functie_aanroep) = parseer_functie(expressie) {
///     println!("Parsed function call: {:?}", functie_aanroep);
/// } else {
///     println!("No valid function call found.");
/// }
/// ```
///
/// # Limitations
/// - Assumes the existence of the `Functie` type and its methods `from_str` and `is_string_functie`.
/// - Relies on an external helper function `verbijzonder_argumenten` for processing arguments.
/// - Does not handle invalid syntax such as mismatched parentheses or malformed function names.
///
/// # Panics
/// This function should not panic under normal circumstances, as it handles all parsing
/// gracefully and returns `None` for invalid input.
///
/// # Dependencies
/// - `Functie`: Must implement `from_str` for function name matching.
/// - `FunctieAanroep`: Must provide a `new` method to construct the function call with parsed data.
/// - `verbijzonder_argumenten`: Presumed to extract arguments from the parsed function string.
///
/// # Notes
/// - The function is designed to detect functions with specific naming conventions. If your use case
///   requires a different set of valid characters for function names, you may need to update this implementation.
///
/// - Improvements to error reporting or the handling of edge cases may make the function more robust.
/// ```
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

/// ```
/// Parses a variable from a string expression if it meets certain criteria.
///
/// This function scans the given expression string to identify variable names
/// that start with an uppercase ASCII letter and, optionally, contain uppercase
/// letters, digits, or underscores. Variables may also optionally terminate with
/// special characters `$` or `%`.
///
/// # Parameters
/// - `expressie`: A string slice containing the expression to parse. It may contain
///   characters, quoted segments, and potential variable names.
///
/// # Returns
/// - `Option<VariabeleAanroep>`: Returns `Some(VariabeleAanroep)` if a valid variable
///   name is found. Returns `None` if no valid variable is identified.
///
/// # Process
/// 1. Iterate through the characters in the expression.
/// 2. Identify the beginning of a potential variable name using its leading uppercase
///    ASCII character.
/// 3. Validate the subsequent characters for compliance with variable naming rules:
///    - Can only contain uppercase ASCII letters, digits, or underscores.
///    - Can optionally terminate in `$` or `%`.
/// 4. Ensure the parsing skips text enclosed in quotes (`"`).
/// 5. Return the variable name and its position within the expression if valid.
///
/// # Example
/// ```rust
/// let expression = r#"VAR_NAME_123%"#;
/// if let Some(variabele) = parseer_variabele(expression) {
///     println!("Geldige variabele gevonden: {}", variabele.naam);
/// } else {
///     println!("Geen geldige variabele gevonden.");
/// }
/// ```
///
/// In this example, the function identifies the variable `VAR_NAME_123%`.
///
/// # Notes
/// - The function assumes the presence of:
///   - A `VariabeleAanroep` struct with a `new` constructor.
///   - An auxiliary function `is_geldige_variabele_naam` to validate the variable names.
///
/// - Quoted text within the expression is ignored during parsing. The `tussen_quotes`
///   flag tracks whether the current context is within quotes.
///
/// - The input expression is processed using `char_indices`, allowing access to both
///   character values and their byte index positions.
///
/// # Edge Cases
/// - If the expression ends while a variable is being parsed, the function performs
///   a final check to validate the partially parsed variable.
/// - Consecutive variables or malformed input are gracefully handled by transitioning
///   to new potential parsing segments.
///
/// # Dependencies
/// This function relies on the following components:
/// - `VariabeleAanroep`: A struct that represents a variable call, which should
///   support the creation of an instance using `VariabeleAanroep::new(String, usize, usize)`.
/// - `is_geldige_variabele_naam`: A utility function that verifies the validity of
///   a variable name based on custom rules.
///
/// # Limitations
/// - The function is designed to handle single-variable parsing only; it stops processing
///   once the first valid variable is identified.
/// - Special rules for variable naming are strictly enforced, which may result in skipping
///   variables with unconventional formats.
///
/// # Performance
/// The function has a time complexity of O(n), where `n` is the length of the input string,
/// as it iterates over the string once without nested loops.
/// ```
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

/// ```rust
/// This function implements a lexer that parses an input string and generates a corresponding
/// representation in the form of a `Line` structure. It handles simple commands, such as
/// recognizing specific keywords (`SCHRIJF`, `ZET`) and validating their syntax.
///
/// # Parameters:
/// - `input: &str` - A string slice containing the input code to be parsed.
///
/// # Returns:
/// - `Result<Line, String>`:
///   - `Ok(Line)` on successful parsing, containing the necessary representation of the line.
///   - `Err(String)` if there is a syntax error or an unsupported operation.
///
/// # Supported Keywords:
/// - `SCHRIJF`: Used to output an expression to some output medium.
///   - Usage Syntax: `SCHRIJF <expression>`
///   - Errors:
///     - Fails if no expression is provided after the keyword.
/// - `ZET`: Used to assign a value to a variable.
///   - Usage Syntax: `ZET <variabele_naam> <toekennings_teken> <expression>`
///   - Errors:
///     - Fails if the variable name is invalid.
///     - Fails if the assignment operator is missing or invalid.
///     - Fails if the command structure is incomplete.
///
/// # Notes:
/// - Full program parsing is not yet supported. If the first token is a number, the
///   function will return an error stating `"Programma's worden nog niet ondersteund"`.
/// - Syntax verification relies on helper functions such as `parseer_regel`,
///   `is_geldige_variabele_naam`, and others for token parsing and validation.
///
/// # Errors:
/// - `"Syntax fout: regel wordt niet herkend"` if the input does not match any recognizable structure.
/// - `"Programma's worden nog niet ondersteund"` for program-level parsing.
/// - `"Geen expressie opgegeven voor SCHRIJF"` if no expression is provided after `SCHRIJF`.
/// - `"Incomplete syntax voor ZET"` if the `ZET` command lacks the needed structure.
/// - `"Ongeldige naam voor de variabele"` if the variable name is invalid.
/// - `"Onjuist toekenning teken"` if the assignment operator is invalid.
/// - `"Onbekend sleutelwoord"` if the keyword in the input is not recognized.
///
/// # Examples:
/// ```rust
/// // Valid `SCHRIJF` example
/// let result = lexer("SCHRIJF Hallo Wereld");
/// assert!(result.is_ok());
///
/// // Valid `ZET` example
/// let result = lexer("ZET x = 42");
/// assert!(result.is_ok());
///
/// // Syntax error example
/// let result = lexer("UNKNOWN_COMMAND");
/// assert!(result.is_err());
/// ```
///
/// # Dependencies:
/// - This function relies on:
///   - `parseer_regel`: For extracting and tokenizing the input.
///   - `is_geldige_variabele_naam`: To validate variable names against expected rules.
///   - Structures like `Line` and `Sleutelwoord` for organizing parsed data.
///
/// # Limitations:
/// - Commands at the program level (e.g., bulk parsing) are currently not supported.
/// - Complex expressions are treated as concatenated tokens. Advanced expression parsing may
///   require additional implementation.
///
/// # Future Considerations:
/// - Adding support for full program parsing.
/// - Improving expression parsing for complex scenarios and edge cases.
/// - Localization of error messages if needed for multilingual support.
/// ```
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

    //Toekenning heeft geen sleutelwoord
    if tokens[0].chars().next().is_some_and(|c| c.is_ascii_lowercase())  {
        tokens.insert(0, "TOEKENNEN".to_string());
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
                Sleutelwoord::TOEKENNEN => {
                    if tokens.len() < 4 {
                        return Err("Incomplete syntax voor de toekenning".to_string());
                    } else if !is_geldige_variabele_naam(&tokens[1]) {
                        return Err("Ongeldige naam voor de variabele".to_string());
                    } else if tokens[2] != TOEKENNING_TEKEN {
                        return Err("Onjuist toekenning teken".to_string());
                    }

                    let expressie = tokens[3..].join("");
                    let variabele_naam = tokens[1].to_string();
                    Ok(Line::new(0, LineInhoud::Toekennen { variabele_naam, expressie }))
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


