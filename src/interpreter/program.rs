#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Sleutelwoord {
    SCHRIJF,
    TOEKENNEN,
    HELP,
}

impl Sleutelwoord {

    pub(super) fn from_string(input: &str) -> Option<Self> {
        match input {
            "SCHRIJF" => Some(Sleutelwoord::SCHRIJF),
            "TOEKENNEN" => Some(Sleutelwoord::TOEKENNEN),
            "HELP" => Some(Sleutelwoord::HELP),
            _ => None
        }
    }

    pub(super) fn is_sleutelwoord(woord: &str) -> bool {
        Self::from_string(woord).is_some()
    }

}




pub(super) const TOEKENNING_TEKEN: &str = ":=";

pub(super) struct Line {
    pub(super) regelnummer: usize,
    pub(super) inhoud: LineInhoud,
}

pub(super) enum LineInhoud {
    Schrijf {
        expressie: String,
    },
    Toekennen {
        variabele_naam: String,
        expressie: String,
    },
    Help {

    },
}

impl Line {
    pub(super) fn new(
        regelnummer: usize,
        inhoud: LineInhoud,
    ) -> Self {
        Line {
            regelnummer,
            inhoud,
        }
    }
    
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Operator {
    Plus,
    Min,
    Vermenigvuldig,
    Deel,
}

impl Operator {
    pub(super) fn from_char(c: char) -> Option<Self> {
        match c {
            '+' => Some(Self::Plus),
            '-' => Some(Self::Min),
            '*' => Some(Self::Vermenigvuldig),
            '/' => Some(Self::Deel),
            _ => None,
        }
    }

    pub(super) fn is_operator_char(c: char) -> bool {
        Self::from_char(c).is_some()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Functie {
    LinksString,
    RechtsString,
    MiddenString,
    Val,
}

impl Functie {
    pub(super) fn from_str(s: &str) -> Option<Self> {
        match s {
            "LINKS$" => Some(Self::LinksString),
            "RECHTS$" => Some(Self::RechtsString),
            "MIDDEN$" => Some(Self::MiddenString),
            "VAL" => Some(Self::Val),
            _ => None,
        }
    }

    pub(super) fn as_str(self) -> &'static str {
        match self {
            Self::LinksString => "LINKS$",
            Self::RechtsString => "RECHTS$",
            Self::MiddenString => "MIDDEN$",
            Self::Val => "VAL",
        }
    }
    
    /// ```rust
    /// Checks if the given string can be parsed into a valid enum variant or struct instance.
    ///
    /// # Parameters
    /// - `s`: A string slice (`&str`) that represents the input to be checked.
    ///
    /// # Returns
    /// - `true` if the input string can be successfully parsed into a valid instance
    ///   using the `from_str` method of the implementing type.
    /// - `false` otherwise.
    ///
    /// # Example
    /// ```
    /// let valid_input = "example_variant";
    /// let invalid_input = "invalid_variant";
    ///
    /// assert!(YourType::is_functie(valid_input));
    /// assert!(!YourType::is_functie(invalid_input));
    /// ```
    ///
    /// # Visibility
    /// - This method is visible only within the current module (`pub(super)`).
    ///
    /// # Notes
    /// The implementation depends on the `Self::from_str` method, which should return
    /// `Some` for valid inputs and `None` for invalid ones.
    /// ```
    pub(super) fn is_functie(s: &str) -> bool {
        Self::from_str(s).is_some()
    }
    
    /// ```rust
    /// Checks if the given string represents a valid "functie" and ends with a `$`.
    ///
    /// This method performs two checks:
    /// 1. It verifies if the input string `s` is considered a valid "functie" by calling the `is_functie` method.
    /// 2. It checks whether the input string ends with the `$` character.
    ///
    /// # Parameters
    /// - `s`: A string slice (`&str`) to validate.
    ///
    /// # Returns
    /// - `true` if the string is both a valid "functie" (as determined by `is_functie`) and ends with `$`.
    /// - `false` otherwise.
    ///
    /// # Visibility
    /// This method is marked as `pub(super)` and is therefore only accessible within the current module and its parent.
    ///
    /// # Example
    /// ```
    /// # struct Example;
    /// # impl Example {
    /// #     fn is_functie(s: &str) -> bool { /* Implementation omitted */ true }
    /// #     pub(super) fn is_string_functie(s: &str) -> bool {
    /// #         Self::is_functie(s) && s.ends_with('$')
    /// #     }
    /// # }
    /// let valid_string = "my_functie$";
    /// let invalid_string = "my_functie";
    /// let other_invalid_string = "not_a_functie";
    ///
    /// assert!(Example::is_string_functie(valid_string));
    /// assert!(!Example::is_string_functie(invalid_string));
    /// assert!(!Example::is_string_functie(other_invalid_string));
    /// ```
    /// ```
    pub(super) fn is_string_functie (s: &str) -> bool {
        Self::is_functie(s) && s.ends_with('$')
    }
    
    /// ```rust
    /// Checks if the given string `s` is a "nummer functie" (number function).
    ///
    /// A "nummer functie" is determined based on the following conditions:
    /// 1. The string `s` must satisfy the `is_functie` condition.
    /// 2. The string `s` must not end with the '$' character.
    ///
    /// # Parameters
    /// - `s`: A reference to the input string to be checked.
    ///
    /// # Returns
    /// - `true` if `s` qualifies as a "nummer functie".
    /// - `false` otherwise.
    ///
    /// # Requirements
    /// This function relies on the `Self::is_functie(s)` method to evaluate
    /// the first condition. Ensure that `is_functie` is correctly implemented
    /// and available within the context of the `Self` type.
    ///
    /// # Examples
    /// ```
    /// // Assuming `Self::is_functie("abc")` returns true:
    /// assert!(is_nummer_functie("abc"));
    ///
    /// // Assuming `Self::is_functie("abc$")` returns true:
    /// assert!(!is_nummer_functie("abc$"));
    ///
    /// // Assuming `Self::is_functie("xyz")` returns false:
    /// assert!(!is_nummer_functie("xyz"));
    /// ```
    /// ```
    pub(super) fn is_nummer_functie (s: &str) -> bool {
        Self::is_functie(s) && !s.ends_with('$')
    }
}