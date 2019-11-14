use lazy_static::lazy_static;
use regex::{Captures, Regex};
use std::collections::HashSet;

#[derive(Debug, PartialEq)]
struct Error {}

#[derive(Debug)]
struct Step {
    s: String,
    generalized: String,
    parts: Vec<String>,
    names: Vec<String>,
    variables_re: Regex,
}

impl Step {
    fn new(s: &str) -> Result<Step, Error> {
        lazy_static! {
            static ref PATH_VARIABLE: Regex = Regex::new(r"\{([^}]*)\}").unwrap();
        }
        let generalized = PATH_VARIABLE.replace_all(s, "{}").to_string();

        let parts = get_parts(&generalized)?;
        let names = get_names(&PATH_VARIABLE, &s)?;
        let variables_re = get_variables_re(&PATH_VARIABLE, &s);
        Ok(Step {
            s: s.to_owned(),
            generalized,
            parts,
            names,
            variables_re,
        })
    }

    /// match path segment, return names
    fn match_segment<'a>(&self, s: &'a str) -> Option<Vec<&'a str>> {
        // XXX how to make converter-driven matching work?
        self.variables_re.captures(s).map(|c| {
            c.iter()
                .skip(1)
                .map(|entry| entry.expect("match not matched").as_str())
                .collect()
        })
    }
}

/// Check whether a variable name is a proper identifier.
fn is_identifier(s: &str) -> bool {
    lazy_static! {
        static ref IDENTIFIER: Regex = Regex::new(r"^[^\d\W]\w*$").unwrap();
    }
    IDENTIFIER.is_match(s)
}

fn get_parts(generalized: &str) -> Result<Vec<String>, Error> {
    let parts: Vec<String> = generalized.split("{}").map(String::from).collect();

    if parts.len() > 1 {
        for part in &parts[1..parts.len() - 1] {
            if part == "" {
                // Cannot have consecutive variables
                return Err(Error {});
            }
        }
    }

    for part in &parts {
        if part.contains("{") || part.contains("}") {
            // Invalid step
            return Err(Error {});
        }
    }
    Ok(parts)
}

fn get_names(variable_regex: &Regex, s: &str) -> Result<Vec<String>, Error> {
    let names: Vec<String> = variable_regex
        .find_iter(s)
        .map(|m| m.as_str())
        .map(|s| s[1..s.len() - 1].to_string())
        .collect();

    let mut name_set = HashSet::new();
    for name in &names {
        if !is_identifier(&name) {
            // illegal variable identifier
            return Err(Error {});
        }
        if !name_set.insert(name) {
            // duplicate variable
            return Err(Error {});
        }
    }
    Ok(names)
}

fn get_variables_re(variable_regex: &Regex, s: &str) -> Regex {
    let variables_re = variable_regex
        .replace_all(s, |caps: &Captures| {
            format!("(?P<{}>.+)", &caps[0][1..caps[0].len() - 1])
        })
        .to_string();
    Regex::new(&variables_re).unwrap()
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

    #[test]
    fn test_step_duplicate_variable() {
        let step = Step::new("foo{bar}baz{bar}");
        assert!(step.is_err());
    }

    #[test]
    fn test_step_consecutive_variables() {
        let step = Step::new("{bar}{baz}");
        assert!(step.is_err());
    }

    #[test]
    fn test_invalid_step_only_open() {
        let step = Step::new("{bar");
        assert!(step.is_err());
    }

    #[test]
    fn test_invalid_step_only_close() {
        let step = Step::new("bar}");
        assert!(step.is_err());
    }

    #[test]
    fn test_match_segment_no_variables() {
        let step = Step::new("foo").unwrap();
        assert!(step.match_segment("foo").is_some());
        assert!(step.match_segment("bar").is_none());
    }

    #[test]
    fn test_match_segment_one_variable() {
        let step = Step::new("{bar}").unwrap();
        assert_eq!(step.match_segment("foo").unwrap(), vec!["foo"]);
    }

    #[test]
    fn test_match_segment_two_variables() {
        let step = Step::new("start{a}middle{b}end").unwrap();
        assert_eq!(
            step.match_segment("startAmiddleBend").unwrap(),
            vec!["A", "B"]
        );
    }

    // proptest! {
    //     #[test]
    //     fn doesnt_crash(s in "\\PC*") {
    //         Step::new(&s).unwrap();
    //     }
    // }
}
