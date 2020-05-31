use std::collections::HashMap;

// A set of pairs (feature-name, rule)
pub struct Config(pub Vec<Feature>);

// Rule is a logical expression evaluated on a HashMap<&str,&str>

pub struct Feature {
    name: String,
    rule: Rule,
}

pub enum Rule {
    Pred(String, Predicate),
    Not(Box<Rule>),
    And(Vec<Rule>),
    Or(Vec<Rule>),
}

// Predicatea to evaluate on attribute values
pub enum Predicate {
    Eq(String), // ==
    Gt(f64),    // >
    Lt(f64),    // <
    Gte(f64),   // >=
    Lte(f64),   // <=
}

pub fn from_request<'a>(request: HashMap<&str, &str>, config: &'a Config) -> Vec<&'a str> {
    config
        .0
        .iter()
        .filter_map(|Feature { name, rule }| {
            if rule.matches(&request) {
                Some(name.as_ref())
            } else {
                None
            }
        })
        .collect()
}

impl Rule {
    pub fn matches(&self, request: &HashMap<&str, &str>) -> bool {
        match self {
            Rule::Pred(attribute, rule) => request
                .get::<str>(attribute.as_ref())
                .map_or(rule.eval(""), |value| rule.eval(*value)), // FIXME consider separating missing and empty value
            Rule::And(rules) => rules.iter().all(|rule| rule.matches(request)),
            Rule::Or(rules) => rules.iter().any(|rule| rule.matches(request)),
            Rule::Not(rule) => !rule.matches(request),
        }
    }
}

impl Predicate {
    pub fn eval(&self, value: &str) -> bool {
        match self {
            Predicate::Eq(v) => value == v,
            Predicate::Gt(n) => value.parse().map_or(false, |num: f64| num > *n),
            Predicate::Gte(n) => value.parse().map_or(false, |num: f64| num >= *n),
            Predicate::Lt(n) => value.parse().map_or(false, |num: f64| num < *n),
            Predicate::Lte(n) => value.parse().map_or(false, |num: f64| num <= *n),
        }
    }
}

#[cfg(test)]
mod test {
    use super::Predicate::{Eq, Gt, Gte, Lt, Lte};
    use super::*;
    use test_case::test_case;

    #[test_case(Eq("foo".to_owned()), "foo", true)]
    #[test_case(Eq("foo".to_owned()), "bar", false)]
    #[test_case(Gt(10.0), "10.3", true)]
    #[test_case(Gt(10.31), "10.3", false)]
    #[test_case(Gt(10.3), "10.3", false)]
    #[test_case(Gte(10.0), "10.3", true)]
    #[test_case(Gte(10.31), "10.3", false)]
    #[test_case(Gte(10.3), "10.3", true)]
    #[test_case(Lt(10.0), "10.3", false)]
    #[test_case(Lt(10.31), "10.3", true)]
    #[test_case(Lt(10.3), "10.3", false)]
    #[test_case(Lte(10.0), "10.3", false)]
    #[test_case(Lte(10.31), "10.3", true)]
    #[test_case(Lte(10.3), "10.3", true)]
    fn predicate(rule: Predicate, value: &str, outcome: bool) {
        assert_eq!(rule.eval(value), outcome);
    }

    #[test]
    fn matches_a_simple_predicate_rule() {
        let req = [
            (":auth", "viktor@email.com"),
            (":region", "uk"),
            ("accept-language", "cs"),
            ("x-session-id", "21527782"),
        ]
        .iter()
        .cloned()
        .collect();

        let config = Config(vec![
            Feature {
                name: "english".into(),
                rule: Rule::Pred("accept-language".into(), Eq("en".into())),
            },
            Feature {
                name: "british".into(),
                rule: Rule::Pred(":region".into(), Eq("uk".into())),
            },
        ]);

        assert_eq!(from_request(req, &config), vec!["british"]);
    }

    #[test]
    fn matches_a_complex_rule() {
        let british_req = [
            (":auth", "viktor@email.com"),
            (":region", "uk"),
            ("accept-language", "cs"),
            ("x-session-id", "21527782"),
        ]
        .iter()
        .cloned()
        .collect();

        let english_req = [
            (":auth", "viktor@email.com"),
            (":region", "de"),
            ("accept-language", "en"),
            ("x-session-id", "21527782"),
        ]
        .iter()
        .cloned()
        .collect();

        let anonymous_req = [
            (":region", "uk"),
            ("accept-language", "cs"),
            ("x-session-id", "21527782"),
        ]
        .iter()
        .cloned()
        .collect();

        let config = Config(vec![
            Feature {
                name: "english".into(),
                // (accept-language == en OR :region == uk) AND NOT(:auth == "")
                rule: Rule::And(vec![
                    Rule::Or(vec![
                        Rule::Pred("accept-language".into(), Eq("en".into())),
                        Rule::Pred(":region".into(), Eq("uk".into())),
                    ]),
                    Rule::Not(Box::new(Rule::Pred(":auth".into(), Eq("".into())))),
                ]),
            },
            Feature {
                name: "british".into(),
                // :region == uk AND NOT(:auth == "")
                rule: Rule::And(vec![
                    Rule::Pred(":region".into(), Eq("uk".into())),
                    Rule::Not(Box::new(Rule::Pred(":auth".into(), Eq("".into())))),
                ]),
            },
        ]);

        assert_eq!(
            from_request(british_req, &config),
            vec!["english", "british"]
        );
        assert_eq!(from_request(english_req, &config), vec!["english"]);
        assert_eq!(from_request(anonymous_req, &config), Vec::<&str>::new());
    }
}
