use std::collections::HashMap;

// A set of pairs (feature-name, rule)
pub struct Config(pub Vec<(String, Rule)>);

// Rule is a logical expression evaluated on a HashMap<&str,&str>

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
        .filter_map(|(name, rule)| {
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
            Rule::Pred(attribute, rule) => {
                let key: &str = attribute.as_ref(); // FIXME not sure why this is needed
                request.get(key).map_or(false, |value| rule.eval(*value))
            }
            _ => false,
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
    use std::iter::FromIterator;
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
        let req = HashMap::from_iter(
            vec![
                (":auth", "viktor@email.com"),
                (":region", "uk"),
                ("accept-language", "cs"),
                ("x-session-id", "21527782"),
            ]
            .iter()
            .map(|it| *it),
        );

        let config = Config(vec![
            (
                "english".to_owned(),
                Rule::Pred("accept-language".to_owned(), Eq("en".to_owned())),
            ),
            (
                "british".to_owned(),
                Rule::Pred(":region".to_owned(), Eq("uk".to_owned())),
            ),
        ]);

        let expected = vec!["british"];

        assert_eq!(from_request(req, &config), expected);
    }
}
