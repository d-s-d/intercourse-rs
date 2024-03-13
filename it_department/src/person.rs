use once_cell::sync::OnceCell;
use phantom_newtype::Amount;
use regex::Regex;
use thiserror::Error;

/// Represent a person.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Person {
    /// First name.
    pub first: String,
    /// Last name.
    pub last: String,
    /// Email address.
    pub email: EmailAddr,
    /// Preferred language for communication.
    pub pref_lang: Option<PreferredLanguage>,
    /// What relations ship does this person have to the company.
    pub affiliation: Affiliation,
}

pub enum Chf {}
pub type ChfAmout = Amount<Chf, u64>;

// A note on constructors: Typically, it is good advice to always provide
// constructor functions for the types you define. Even if they just accept the
// fields as function parameters, it gives you, as the provider of the type, a
// controlled entry point for users of the type.
//
// For complex data structures, an often used approach is the builder pattern:
// It provides for an extensible, well-typed API which hides away default
// values. In the case of [PersonBuilder] it is a bit 'overkill', as the
// structure is rather simple. We do it here for demonstration purposes.
//
// Note: Here, we can derive default because the default for all options is always
// well-defined.
#[derive(Debug, Clone, Default)]
pub struct PersonBuilder {
    first: Option<String>,
    last: Option<String>,
    email: Option<EmailAddr>,
    pref_lang: Option<PreferredLanguage>,
    affiliation: Option<Affiliation>,
}

impl PersonBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_first_name<T: ToString>(self, s: T) -> Self {
        Self {
            first: Some(s.to_string()),
            ..self
        }
    }

    pub fn with_last_name<T: ToString>(self, s: T) -> Self {
        Self {
            last: Some(s.to_string()),
            ..self
        }
    }

    /// Set an email address for the person.
    ///
    /// # Panics
    ///
    /// Panics if the provided string is not a valid email address.
    pub fn with_email_address<T>(self, s: T) -> Self
    where
        // While it's not possible to overload methods in Rust, the language to
        // describe type constraints is quite powerful. Here, we accept a type
        // T, iff EmailAddr implements From<T>.
        EmailAddr: TryFrom<T>,
        // The following is an artificial constraint we put here, because we
        // want to make use of `expect` which requires the Error to implement
        // Debug.
        <EmailAddr as TryFrom<T>>::Error: std::fmt::Debug,
    {
        let email = Some(EmailAddr::try_from(s).expect("Could not parse email Address."));
        Self { email, ..self }
    }

    pub fn with_preferred_language(self, pref_lang: PreferredLanguage) -> Self {
        Self {
            pref_lang: Some(pref_lang),
            ..self
        }
    }

    pub fn with_affiliation(self, relationship: Affiliation) -> Self {
        Self {
            affiliation: Some(relationship),
            ..self
        }
    }

    pub fn build(self) -> Result<Person, BuildPersonError> {
        use BuildPersonError::*;
        let Some(first) = self.first else {
            return Err(FirstnameUnset);
        };
        let Some(last) = self.last else {
            return Err(LastnameUnset);
        };
        let Some(email) = self.email else {
            return Err(EmailUnset);
        };
        let Some(affiliation) = self.affiliation else {
            return Err(AffiliationUnset);
        };
        Ok(Person {
            first,
            last,
            email,
            pref_lang: self.pref_lang,
            affiliation,
        })
    }
}

#[derive(Debug, Error)]
pub enum BuildPersonError {
    #[error("Firstname not set")]
    FirstnameUnset,
    #[error("Lastname not set")]
    LastnameUnset,
    #[error("Email is not set")]
    EmailUnset,
    #[error("Affiliation not set")]
    AffiliationUnset,
}

// A const is symbol which has a constant value known already at compile time. A
// const is typically inlined.
const EMAIL_RGX_STR: &str = r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$";

// A static variable has a fixed memory location throughout the programs
// lifetime and hence, a 'static lifetime. Changing the value directly is unsafe
// due to possible data races. Here, we use an abstraction to safely share a
// value that is computed only once.
static EMAIL_REGEX: OnceCell<Regex> = OnceCell::new();

/// A syntactically valid EMail address.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct EmailAddr(String);

impl EmailAddr {
    /// Construct a new EmailAddr from `addr`.
    ///
    /// # Returns
    ///
    /// The [EmailAddr] if the given email address is valid, otherwise [Option::None].
    pub fn new<T: AsRef<str>>(addr: T) -> Option<Self> {
        if !EMAIL_REGEX
            .get_or_init(|| Regex::new(EMAIL_RGX_STR).expect("Should always compile"))
            .is_match(addr.as_ref())
        {
            return None;
        }
        Some(Self(addr.as_ref().to_owned()))
    }

    /// Construct a new EMailAddr from `addr`.
    ///
    /// # Safety
    ///
    /// This function should only be called with valid email addresses.
    pub unsafe fn new_unchecked<T: AsRef<str>>(addr: T) -> Self {
        Self(addr.as_ref().to_string())
    }
}

impl AsRef<str> for EmailAddr {
    fn as_ref(&self) -> &str {
        self.0.as_str()
    }
}

// It is common to implement canonical transformations between types using
// From/Into trait implementations (though From is preferred whenever possible).
// This has the benefit that the user does not need to search for a particular
// function call, but the transformation is statically inferred by the types.
//
// In this particular case, the transformation is only partial, as not all
// strings are valid email addresses. Thus, we implement the TryFrom trait.
impl TryFrom<&str> for EmailAddr {
    type Error = EmailParseError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        // Easy map a None to an error ... no "if err != nil" etc. etc. :-))
        EmailAddr::new(value).ok_or_else(EmailParseError)
        // [clippy] toggle comment above/below to see clippy in action
        // EmailAddr::new(value).ok_or_else(|| EmailParseError())
    }
}

// Define your custom error type
#[derive(Debug, Error)]
#[error("Invalid email address in string")]
pub struct EmailParseError();

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub enum PreferredLanguage {
    // The following is a nice way how cargo give you tips and tricks to improve
    // your code. If remove the #[default] below and uncomment the explicit
    // Default trait implementation below, it will suggest to do it otherwise.
    #[default]
    English,
    German,
    Spanish,
    Schwyzerduetsch,
}

/*
impl Default for PreferredLanguage {
    fn default() -> Self {
        PreferredLanguage::English
    }
}
*/

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone)]
pub enum Affiliation {
    Employee { annual_income: ChfAmout },
    Contractor { company_name: String },
    Intern,
}

#[cfg(test)]
mod tests {
    // This is a typical short-cut in test modules to make just everything
    // visible from the super module.
    use super::*;

    #[test]
    fn test_simple_successful_person_build() {
        let person = get_manuel()
            .with_email_address("manuel@udssr.com")
            .build()
            .unwrap();

        assert_eq!(person.first, "Manuel");
    }

    #[test]
    #[should_panic(expected = "Could not parse email Address.")]
    fn test_fail_email() {
        let _ = get_manuel()
            .with_email_address("teufel test@example.com")
            .build();
    }

    #[test]
    fn test_missing_email() {
        matches!(get_manuel().build(), Err(BuildPersonError::EmailUnset));
    }

    fn get_manuel() -> PersonBuilder {
        PersonBuilder::new()
            .with_first_name("Manuel")
            .with_last_name("Gorbatchov")
            .with_affiliation(Affiliation::Intern)
    }
}
