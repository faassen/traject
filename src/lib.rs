use lazy_static::lazy_static;
use regex::Regex;

#[derive(Debug, PartialEq)]
struct Error {}

/// Check whether a variable name is a proper identifier.
fn is_identifier(s: &str) -> bool {
    lazy_static! {
        static ref IDENTIFIER: Regex = Regex::new(r"^[^\d\W]\w*$").unwrap();
    }
    IDENTIFIER.is_match(s)
}

#[derive(Debug)]
struct Step {
    s: String,
    generalized: String,
    parts: Vec<String>,
    names: Vec<String>,
    // variables_re: Regex,
}

impl Step {
    fn new(s: &str) -> Result<Step, Error> {
        lazy_static! {
            static ref PATH_VARIABLE: Regex = Regex::new(r"\{([^}]*)\}").unwrap();
        }
        let generalized = PATH_VARIABLE.replace_all(s, "{}").to_string();

        let parts: Vec<String> = generalized.split("{}").map(String::from).collect();

        let names: Vec<String> = PATH_VARIABLE
            .find_iter(s)
            .map(|m| m.as_str())
            .map(|s| s[1..s.len() - 1].to_string())
            .collect();

        for name in &names {
            if !is_identifier(&name) {
                return Err(Error {});
            }
        }
        Ok(Step {
            s: s.to_owned(),
            generalized: generalized,
            parts,
            names,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // use proptest::prelude::*;

    #[test]
    fn test_is_identifier() {
        assert!(is_identifier("foo"));
        assert!(is_identifier("foo123"));
        assert!(is_identifier("foo_bar"));
        assert!(is_identifier("fooBar"));
        assert!(!is_identifier("123"));
        assert!(!is_identifier("$foo"));
    }

    #[test]
    fn test_step_new_no_variables() {
        let step = Step::new("foo").unwrap();
        assert_eq!(step.s, "foo");
        assert_eq!(step.generalized, "foo");
        assert_eq!(step.parts, vec!["foo"]);
        assert_eq!(step.names, vec![] as Vec<String>);
    }

    #[test]
    fn test_step_new_one_variable_start() {
        let step = Step::new("{bar}baz").unwrap();
        assert_eq!(step.s, "{bar}baz");
        assert_eq!(step.generalized, "{}baz");
        assert_eq!(step.parts, vec!["", "baz"]);
        assert_eq!(step.names, vec!["bar"]);
    }

    #[test]
    fn test_step_new_one_variable_middle() {
        let step = Step::new("foo{bar}baz").unwrap();
        assert_eq!(step.s, "foo{bar}baz");
        assert_eq!(step.generalized, "foo{}baz");
        assert_eq!(step.parts, vec!["foo", "baz"]);
        assert_eq!(step.names, vec!["bar"]);
    }

    #[test]
    fn test_step_new_one_variable_end() {
        let step = Step::new("foo{bar}").unwrap();
        assert_eq!(step.s, "foo{bar}");
        assert_eq!(step.generalized, "foo{}");
        assert_eq!(step.parts, vec!["foo", ""]);
        assert_eq!(step.names, vec!["bar"]);
    }

    #[test]
    fn test_step_new_one_variable_only() {
        let step = Step::new("{bar}").unwrap();
        assert_eq!(step.s, "{bar}");
        assert_eq!(step.generalized, "{}");
        assert_eq!(step.parts, vec!["", ""]);
        assert_eq!(step.names, vec!["bar"]);
    }

    #[test]
    fn test_step_multiple_variables() {
        let step = Step::new("foo{bar}baz{qux}frub").unwrap();
        assert_eq!(step.s, "foo{bar}baz{qux}frub");
        assert_eq!(step.generalized, "foo{}baz{}frub");
        assert_eq!(step.parts, vec!["foo", "baz", "frub"]);
        assert_eq!(step.names, vec!["bar", "qux"]);
    }

    #[test]
    fn test_step_bad_variable() {
        let step = Step::new("foo{%$}baz");
        assert!(step.is_err());
    }

    // proptest! {
    //     #[test]
    //     fn doesnt_crash(s in "\\PC*") {
    //         Step::new(&s).unwrap();
    //     }
    // }
}
