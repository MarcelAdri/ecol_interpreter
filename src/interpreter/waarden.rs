/// ```rust
/// The `EcolString` struct is a simple wrapper around the standard `String` type,
/// designed with additional traits derived for ease of use. 
///
/// # Attributes
/// - `inhoud` (`String`): The inner string value encapsulated by the `EcolString`.
///
/// # Derives
/// - `Debug`: Allows instances of `EcolString` to be formatted using the {:?} formatter.
/// - `Clone`: Enables the creation of an exact copy of an `EcolString` instance.
/// - `Default`: Provides a default value for `EcolString`, which is an instance with an empty `String`.
/// - `PartialEq` / `Eq`: Makes `EcolString` comparable for equality, both partially and fully.
///
/// # Visibility
/// This struct is scoped as `pub(super)` and is accessible only within the current module and its children.
///
/// # Example
/// ```
/// let mut custom_string = EcolString::default();
/// custom_string.inhoud = "Hello, world!".to_owned();
/// println!("{:?}", custom_string); // Outputs: EcolString { inhoud: "Hello, world!" }
/// ```
/// ```
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(super) struct EcolString {
    inhoud: String,
}

impl std::fmt::Display for EcolString {
    /// ```rust
    /// Formats the contents of the `inhoud` field for output.
    ///
    /// This method is an implementation of the `fmt` method from the `Display` trait, 
    /// enabling custom string representation for the struct containing the `inhoud` field.
    /// It writes the value of the `inhoud` field to the provided formatter.
    ///
    /// # Parameters
    /// - `f`: A mutable reference to a `std::fmt::Formatter` that provides 
    ///   the interface for formatting output and strings.
    ///
    /// # Returns
    /// - `std::fmt::Result`: This result indicates whether the formatting operation
    ///   was successful or if it encountered an error.
    ///
    /// # Example
    /// ```rust
    /// use std::fmt;
    ///
    /// struct Example {
    ///     inhoud: String,
    /// }
    ///
    /// impl fmt::Display for Example {
    ///     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    ///         f.write_str(&self.inhoud)
    ///     }
    /// }
    ///
    /// let example = Example {
    ///     inhoud: String::from("Hello, world!"),
    /// };
    /// println!("{}", example);
    /// ```
    /// ```
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.inhoud)
    }
}
impl EcolString {
    /// ```rust
    /// Creates a new instance of `EcolString` with the provided content.
    ///
    /// # Parameters
    /// - `inhoud`: A value that can be converted into a `String`. This represents
    ///   the content that will be stored in the `EcolString` instance.
    ///
    /// # Returns
    /// A new instance of `EcolString` containing the provided content.
    ///
    /// # Visibility
    /// This function is visible only within the current module and its submodules
    /// due to the `pub(super)` visibility modifier.
    ///
    /// # Example
    /// ```
    /// let my_string = EcolString::new("Hello, world!");
    /// ```
    /// ```
    pub(super) fn new(inhoud: impl Into<String>) -> Self {
        EcolString { inhoud: inhoud.into() }
    }

    /// ```rust
    /// Converts a string literal into an instance of the implementor type, handling escape sequences and ensuring
    /// the string is properly enclosed in double quotation marks.
    ///
    /// # Parameters
    /// - `literal`: A string slice expected to be enclosed in double quotes (e.g., `"example"`).
    ///
    /// # Returns
    /// - `Ok(Self)`: If the string literal is valid, the function returns an instance of the type,
    ///   constructed from the processed string with escape sequences resolved.
    /// - `Err(String)`: If the string literal is not properly enclosed in double quotes, an error
    ///   message is returned.
    ///
    /// # Escape Sequences
    /// The function processes the following escape sequences within the string:
    /// - `\"` → `"`
    /// - `\\` → `\`
    /// - `\n` → newline (`\n`)
    /// - `\r` → carriage return (`\r`)
    /// - `\t` → tab (`\t`)
    /// - `\0` → null byte (`\0`)
    ///
    /// Any unrecognized escape sequences are preserved as-is, prefixed by a backslash.
    ///
    /// # Examples
    /// ```rust
    /// // Assuming `MyType` implements `from_literal`
    /// let result = MyType::from_literal(r#""Hello\nWorld""#);
    /// assert!(result.is_ok());
    /// assert_eq!(result.unwrap(), MyType::new("Hello\nWorld".to_string()));
    ///
    /// let result = MyType::from_literal("Hello World");
    /// assert!(result.is_err()); // Missing double quotes
    ///
    /// let result = MyType::from_literal(r#""Invalid\qEscape""#);
    /// assert!(result.is_ok()); // Escape sequence '\q' is retained as-is
    /// ```
    ///
    /// # Errors
    /// - If the input does not start and end with double quotes, an error message is returned
    ///   stating: `"String moet binnen dubbele aanhalingstekens worden geplaatst"`.
    /// ```
    pub(super) fn from_literal(literal: &str) -> Result<Self, String> {

        let Some(zonder_begin) = literal.strip_prefix('"') else {
            return Err("String moet binnen dubbele aanhalingstekens worden geplaatst".to_string());
        };

        let Some(inhoud) = zonder_begin.strip_suffix('"') else {
            return Err("String moet binnen dubbele aanhalingstekens worden geplaatst".to_string());
        };

        let mut werk = String::new();
        let mut escaped = false;

        for c in inhoud.chars() {
            if !escaped && c == '\\' {
                escaped = true;
                continue;
            }

            if escaped {
                match c {
                    '"' => werk.push('"'),
                    '\\' => werk.push('\\'),
                    'n' => werk.push('\n'),
                    'r' => werk.push('\r'),
                    't' => werk.push('\t'),
                    '0' => werk.push('\0'),
                    _ => {
                        werk.push('\\');
                        werk.push(c);
                    }
                }
                escaped = false;
                continue;
            }

            werk.push(c);
        }

        if escaped {
            werk.push('\\');
        }

        Ok(Self::new(werk))
    }

    /// ```rust
    ///     /**
    ///      * Converts the characters of the current object to a string expression,
    ///      * escaping any special characters in the process. The resulting string
    ///      * is wrapped in double quotes.
    ///      * 
    ///      * # Functionality
    ///      * - Iterates through the characters of the object.
    ///      * - Escapes characters as needed using the `escape_char` function.
    ///      * - Appends the escaped or unescaped characters to a `String` object.
    ///      * - Returns the final composed string wrapped in double quotes.
    ///      * 
    ///      * # Returns
    ///      * `String`: A string representation of the object's characters 
    ///      * with appropriate escaping and surrounded by double quotes.
    ///      * 
    ///      * # Example
    ///      * ```
    ///      * let example = MyType::new("Hello\nWorld");
    ///      * let result = example.to_expressions();
    ///      * assert_eq!(result, "\"Hello\\nWorld\"");
    ///      * ```
    ///      * 
    ///      * # Note
    ///      * The method uses a helper function `escape_char` which
    ///      * determines how specific characters are escaped.
    ///      */
    /// ```
    pub(super) fn to_expressions(&self) -> String {
        let mut werk = String::new();

        for c in self.chars(){
            if let Some(escaped) = escape_char(c) {
                werk.push_str(escaped);
            } else {
                werk.push(c);
            }
        }

        format!("\"{}\"", werk)
    }

    /// ```rust
    /// Returns an iterator over the characters of the `inhoud` field.
    ///
    /// # Returns
    ///
    /// An iterator that yields each character (`char`) within the `inhoud` string slice.
    ///
    /// # Notes
    ///
    /// - This function has a visibility restriction of `pub(super)`, meaning it is only accessible
    ///   within the current module and its parent module.
    /// - The returned iterator has the same lifetime as `self`, as denoted by the `'_` lifetime used.
    ///
    /// # Example
    ///
    /// ```rust
    /// let my_struct = MyStruct {
    ///     inhoud: String::from("hello"),
    /// };
    ///
    /// let mut chars_iter = my_struct.chars();
    /// assert_eq!(chars_iter.next(), Some('h'));
    /// assert_eq!(chars_iter.next(), Some('e'));
    /// assert_eq!(chars_iter.next(), Some('l'));
    /// assert_eq!(chars_iter.next(), Some('l'));
    /// assert_eq!(chars_iter.next(), Some('o'));
    /// assert_eq!(chars_iter.next(), None);
    /// ```
    /// ```
    pub(super) fn chars(&self) -> impl Iterator<Item = char> + '_ {
        self.inhoud.chars()
    }

    /// ```rust
    /// Appends a character to the internal `inhoud` field.
    ///
    /// # Parameters
    /// - `c`: The character to be appended to the `inhoud`.
    ///
    /// # Behavior
    /// This method takes a mutable reference to the instance (`self`)
    /// and adds the provided character `c` to the end of the internal 
    /// `inhoud` field, which is presumed to be a `String` or similar 
    /// collection supporting the `push` method.
    ///
    /// # Visibility
    /// This method is `pub(super)`, meaning it is publicly accessible
    /// within the current module and its parent but not outside of 
    /// this restricted scope.
    /// ```
    pub(super) fn push(&mut self, c: char) {
        self.inhoud.push(c);
    }

    /// ```rust
    /// Returns a string slice (`&str`) that references the contents of the `inhoud` field.
    ///
    /// # Example
    /// ```rust
    /// struct Example {
    ///     inhoud: String,
    /// }
    ///
    /// impl Example {
    ///     pub(super) fn as_str(&self) -> &str {
    ///         &self.inhoud
    ///     }
    /// }
    ///
    /// let example = Example { inhoud: String::from("Hello, world!") };
    /// assert_eq!(example.as_str(), "Hello, world!");
    /// ```
    ///
    /// # Notes
    /// - This function provides read-only access to the `inhoud` field as a string slice.
    /// - The `pub(super)` visibility modifier means this method is accessible only within the parent module or its submodules.
    /// ```
    pub(super) fn as_str(&self) -> &str {
        &self.inhoud
    }
}

#[derive(Debug, Clone)]
pub(super) enum Waarde {
    Float(f32),
    Tekst(EcolString),
}
impl Waarde {
    pub(super) fn standaard_voor_type(var_type: VariabeleType) -> Self {
        match var_type {
            VariabeleType::Float => Waarde::Float(0.0),
            VariabeleType::Tekst => Waarde::Tekst(EcolString::default()),
        }
    }

    pub(super) fn type_van(&mut self) -> Option<VariabeleType> {
        match self {
            Waarde::Float(_) => Some(VariabeleType::Float),
            Waarde::Tekst(_) => Some(VariabeleType::Tekst),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum VariabeleType {
    Float,
    Tekst,
}



// pub(super) fn type_van_naam(naam: &str) -> VariabeleType {
//     if naam.ends_with('$') {
//         VariabeleType::Tekst
//     } else if naam.ends_with('%') {
//         VariabeleType::Integer
//     } else {
//         VariabeleType::Float
//     }
// }

/// ```rust
/// Retrieves data from a `Waarde` enum and converts it into a `String`.
///
/// This function takes a reference to a `Waarde` enum and pattern matches on its
/// variants to extract the underlying value. The extracted value is then formatted
/// into a `String`.
///
/// # Parameters
/// - `token`: A reference to a `Waarde` enum instance, which can hold one of the following:
///   - `Waarde::Integer(x)` - An integer value.
///   - `Waarde::Float(x)` - A floating-point value. This value is formatted using the `format_float` function.
///   - `Waarde::Tekst(x)` - A string slice (`&str`) value.
///
/// # Returns
/// A `String` representing the formatted value of the `token`.
///
/// # Example
/// ```rust
/// let integer_token = Waarde::Integer(42);
/// let result = haal_data(&integer_token);
/// assert_eq!(result, "42");
///
/// let float_token = Waarde::Float(3.14);
/// let result = haal_data(&float_token);
/// assert_eq!(result, "3.14");
///
/// let tekst_token = Waarde::Tekst(String::from("Hello"));
/// let result = haal_data(&tekst_token);
/// assert_eq!(result, "Hello");
/// ```
/// ```
pub(super) fn haal_data(token: &Waarde) -> String {

    match token {
        Waarde::Float(x) => format_float(*x),
        Waarde::Tekst(x) => format!("{}", x),
    }
}

fn format_float(x: f32) -> String {
    let mut s = format!("{:.6}", x);

    while s.contains('.') && s.ends_with('0') {
        s.pop();
    }

    if s.ends_with('.') {
        s.push('0');
    }

    s
}

/// ```rust
/// Converts a `Waarde` enum variant to its corresponding string representation.
///
/// This function takes a reference to a `Waarde` instance and matches its variant to generate
/// a `String` representation. The supported variants and their conversions are:
///
/// - `Waarde::Integer(x)` -> Converts the integer value `x` to a `String`.
/// - `Waarde::Float(x)` -> Converts the floating-point value `x` to a formatted `String` using the `format_float` function.
/// - `Waarde::Tekst(x)` -> Wraps the string value `x` in double quotes without escaping any special characters.
///
/// # Arguments
///
/// * `waarde` - A reference to the `Waarde` value to be converted.
///
/// # Returns
///
/// A `String` representation of the given `Waarde` value.
///
/// # Example
///
/// ```rust
/// let integer_value = Waarde::Integer(42);
/// assert_eq!(waarde_naar_expressie(&integer_value), "42");
///
/// let float_value = Waarde::Float(3.14);
/// assert_eq!(waarde_naar_expressie(&float_value), "3.14");
///
/// let text_value = Waarde::Tekst(String::from("Hello, world"));
/// assert_eq!(waarde_naar_expressie(&text_value), "\"Hello, world\"");
/// ```
/// ```
pub(super) fn waarde_naar_expressie(waarde: &Waarde) -> String {
    match waarde {
        Waarde::Float(x) => format_float(*x),
        Waarde::Tekst(x) => {
            x.to_expressions()
        }
    }
}

/// ```rust
/// Converts a special character into its corresponding escaped string representation.
///
/// This function takes a single character as input and returns an `Option<&'static str>`.
/// If the character is a special character that requires escaping (e.g., '"' or '\n'),
/// the function returns the escaped representation of the character as a `Some(&str)`.
/// Otherwise, it returns `None` if the character does not require escaping.
///
/// # Parameters
/// - `c`: A `char` to be checked and potentially escaped.
///
/// # Returns
/// An `Option<&'static str>`:
/// - `Some(&str)`: The escaped string representation of the input character, if it is a special character.
/// - `None`: If the input character does not require escaping.
///
/// # Escaped Characters
/// The function recognizes the following special characters and outputs their respective escape sequences:
/// - `'"'`: `Some("\\\"")`
/// - `'\\'`: `Some("\\\\")`
/// - `'\n'`: `Some("\\n")`
/// - `'\r'`: `Some("\\r")`
/// - `'\t'`: `Some("\\t")`
/// - `'\0'`: `Some("\\0")`
///
/// # Examples
/// ```
/// assert_eq!(escape_char('"'), Some("\\\""));
/// assert_eq!(escape_char('\n'), Some("\\n"));
/// assert_eq!(escape_char('a'), None);
/// ```
/// //
fn escape_char(c: char) -> Option<&'static str> {
    match c {
        '"' => Some("\\\""),
        '\\' => Some("\\\\"),
        '\n' => Some("\\n"),
        '\r' => Some("\\r"),
        '\t' => Some("\\t"),
        '\0' => Some("\\0"),
        _ => None,
    }
}

