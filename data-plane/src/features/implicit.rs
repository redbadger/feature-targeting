use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::Hash;
use std::hash::Hasher;

/// A set of features and their matching rules
#[derive(Deserialize, Serialize)]
pub struct Config(pub Vec<Feature>);

/// Feature represents implicit targeting configuration for a single feature flag
#[derive(Deserialize, Serialize)]
pub struct Feature {
    name: String,
    rule: BoolExpr,
}

pub fn from_request<'a>(request: HashMap<&str, &str>, config: &'a Config) -> Vec<&'a str> {
    config
        .0
        .iter()
        .filter_map(|Feature { name, rule }| match rule.eval(&request) {
            Ok(true) => Some(name.as_ref()),
            _ => None, // Ignore features whose rules fail to evaluate
        })
        .collect()
}

#[derive(Deserialize, Serialize)]
pub enum BoolExpr {
    Constant(bool),
    Attribute(String), // Request attribute of name is present
    // Value contained in the list
    In {
        list: StringListExpr,
        value: StringExpr,
    },
    // Any of the values contained in the list
    AnyIn {
        list: StringListExpr,
        values: StringListExpr,
    },
    // All of the values contained in the list
    AllIn {
        list: StringListExpr,
        values: StringListExpr,
    },
    JsonPointer {
        pointer: String,
        value: StringExpr,
    },
    Matches(String, StringExpr),   // Matches regex
    StrEq(StringExpr, StringExpr), // == for strings
    NumEq(NumExpr, NumExpr),       // == for numbers
    Gt(NumExpr, NumExpr),          // >
    Lt(NumExpr, NumExpr),          // <
    Gte(NumExpr, NumExpr),         // >=
    Lte(NumExpr, NumExpr),         // <=
    Not(Box<BoolExpr>),
    And(Vec<BoolExpr>),
    Or(Vec<BoolExpr>),
}

impl BoolExpr {
    fn eval(&self, request: &HashMap<&str, &str>) -> Result<bool, String> {
        match &self {
            BoolExpr::Constant(c) => Ok(c.clone()),
            BoolExpr::Attribute(name) => request
                .get::<str>(name.as_ref())
                .map_or(Err(format!("Attribute '{}' not found.", name)), |_| {
                    Ok(true)
                }),
            BoolExpr::In { list, value } => todo!(),
            BoolExpr::AnyIn { list, values } => todo!(),
            BoolExpr::AllIn { list, values } => todo!(),
            BoolExpr::JsonPointer { pointer, value } => todo!(),
            BoolExpr::Matches(regex, value) => todo!(),
            BoolExpr::StrEq(left, right) => todo!(),
            BoolExpr::NumEq(left, right) => todo!(),
            BoolExpr::Gt(left, right) => todo!(),
            BoolExpr::Lt(left, right) => todo!(),
            BoolExpr::Gte(left, right) => todo!(),
            BoolExpr::Lte(left, right) => todo!(),
            BoolExpr::Not(value) => todo!(),
            BoolExpr::And(values) => todo!(),
            BoolExpr::Or(values) => todo!(),
        }
    }
}

#[derive(Deserialize, Serialize)]
pub enum StringListExpr {
    Constant(Vec<String>),
    Split {
        // Split a string value using a separator
        separator: String,
        value: StringExpr,
    },
    Accept,         // Accepted mime types in weight order
    AcceptLanguage, // Accepted languages in weight order
}

impl StringListExpr {
    fn eval(&self, request: &HashMap<&str, &str>) -> Result<Vec<String>, String> {
        match &self {
            StringListExpr::Constant(c) => Ok(c.clone()),
            StringListExpr::Split { separator, value } => todo!(),
            StringListExpr::AcceptLanguage => todo!(),
        }
    }
}

#[derive(Deserialize, Serialize)]
pub enum StringExpr {
    Constant(String),
    Attribute(String), // Request attribute value
    Browser,           // Derives browser from User-Agent
    OperatingSystem,   // Derives operating system from User-Agent
    // Extracts a string value from a JSON encoded StringExpr
    JsonPointer {
        pointer: String,
        value: Box<StringExpr>,
    },
    First(Box<StringListExpr>), // First item of a list
    Last(Box<StringListExpr>),  // Last item of a list
}

impl StringExpr {
    fn eval(&self, request: &HashMap<&str, &str>) -> Result<String, String> {
        match &self {
            StringExpr::Constant(c) => Ok(c.clone()),
            StringExpr::Attribute(name) => request
                .get::<str>(name.as_ref())
                .map_or(Err(format!("Attribute '{}' not found.", name)), |s| {
                    Ok((*s).to_string())
                }),
            StringExpr::Browser => todo!(),
            StringExpr::OperatingSystem => todo!(),
            StringExpr::JsonPointer { pointer, value } => todo!(),
            StringExpr::First(list) => list.eval(request).and_then(|v| {
                v.first()
                    .map(|it| it.clone())
                    .ok_or("List is empty.".into())
            }),
            StringExpr::Last(list) => list
                .eval(request)
                .and_then(|v| v.last().map(|it| it.clone()).ok_or("List is empty.".into())),
        }
    }
}

// Numerical value extractors
#[derive(Deserialize, Serialize)]
pub enum NumExpr {
    Constant(f64),
    Attribute(String), // Request attribute value
    // Randomly assigns a uniformly distributed stable number between 0.0 and 100.0
    Rank(StringExpr),
    JsonPointer { pointer: String, value: StringExpr },
}

impl NumExpr {
    fn eval(&self, request: &HashMap<&str, &str>) -> Result<f64, String> {
        match &self {
            NumExpr::Constant(c) => Ok(*c),
            NumExpr::Attribute(name) => request.get::<str>(name.as_ref()).map_or(
                Err(format!("Attribute '{}' not found.", name)),
                |s| {
                    (*s).parse()
                        .map_err(|_| format!("Cannot parse '{}' as number.", s))
                },
            ),
            NumExpr::Rank(str_exp) => str_exp.eval(request).map(|s| {
                let mut hasher = DefaultHasher::new();
                s.hash(&mut hasher);
                (hasher.finish() % 1000) as f64 / 10.0
            }),
            NumExpr::JsonPointer { pointer, value } => todo!(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq as assert_eq_diff;
    use serde_json::json;
    use test_case::test_case;

    #[test_case(NumExpr::Constant(10.0), Ok(10.0))]
    #[test_case(NumExpr::Attribute("number".into()), Ok(1.4))]
    #[test_case(NumExpr::Attribute("nope".into()), Err("Attribute 'nope' not found.".into()))]
    #[test_case(NumExpr::Attribute("not-number".into()), Err("Cannot parse 'hi' as number.".into()))]
    #[test_case(NumExpr::Rank(StringExpr::Attribute("not-number".into())), Ok(40.9))]
    fn evaluate_numerical_expressions(expr: NumExpr, expected: Result<f64, String>) {
        let request = [("number", "1.4"), ("not-number", "hi")]
            .iter()
            .cloned()
            .collect();

        assert_eq!(expr.eval(&request), expected)
    }

    #[test_case(StringExpr::Constant("hello".into()), Ok("hello".into()))]
    #[test_case(StringExpr::Attribute("hello".into()), Ok("world".into()))]
    #[test_case(StringExpr::First(Box::new(StringListExpr::Constant(vec!["a".into(), "b".into(), "c".into()]))), Ok("a".into()))]
    #[test_case(StringExpr::Last(Box::new(StringListExpr::Constant(vec!["a".into(), "b".into(), "c".into()]))), Ok("c".into()))]
    #[test_case(StringExpr::First(Box::new(StringListExpr::Constant(vec![]))), Err("List is empty.".into()))]
    #[test_case(StringExpr::Last(Box::new(StringListExpr::Constant(vec![]))), Err("List is empty.".into()))]
    fn evaluate_string_expressions(expr: StringExpr, expected: Result<String, String>) {
        let request = [("hello", "world")].iter().cloned().collect();

        assert_eq!(expr.eval(&request), expected)
    }

    #[test_case(BoolExpr::Constant(true), Ok(true))]
    fn evaluate_boolean_expressions(expr: BoolExpr, expected: Result<bool, String>) {
        let request = [("hello", "world")].iter().cloned().collect();

        assert_eq!(expr.eval(&request), expected)
    }

    #[test]
    fn matches_a_complex_expression() {
        let req = [("accept-language", "en-GB,en;q=0.9,cs;q=0.8")]
            .iter()
            .cloned()
            .collect();

        let config = Config(vec![
            Feature {
                name: "english".into(),
                rule: BoolExpr::AnyIn {
                    list: StringListExpr::Constant(vec![
                        "en".into(),
                        "en-US".into(),
                        "en-GB".into(),
                    ]),
                    values: StringListExpr::AcceptLanguage,
                },
            },
            Feature {
                name: "other-english".into(),
                rule: BoolExpr::Or(vec![
                    BoolExpr::In {
                        list: StringListExpr::AcceptLanguage,
                        value: StringExpr::Constant("en".into()),
                    },
                    BoolExpr::In {
                        list: StringListExpr::AcceptLanguage,
                        value: StringExpr::Constant("en-US".into()),
                    },
                    BoolExpr::In {
                        list: StringListExpr::AcceptLanguage,
                        value: StringExpr::Constant("en-GB".into()),
                    },
                ]),
            },
            Feature {
                name: "british".into(),
                rule: BoolExpr::In {
                    list: StringListExpr::AcceptLanguage,
                    value: StringExpr::Constant("en-GB".into()),
                },
            },
        ]);

        assert_eq!(from_request(req, &config), vec!["british"]);
    }

    #[test]
    fn serialises_to_json() {
        todo!();

        let config = Config(vec![
            Feature {
                name: "english".into(),
                rule: BoolExpr::AnyIn {
                    list: StringListExpr::Constant(vec![
                        "en".into(),
                        "en-US".into(),
                        "en-GB".into(),
                    ]),
                    values: StringListExpr::AcceptLanguage,
                },
            },
            Feature {
                name: "other-english".into(),
                rule: BoolExpr::Or(vec![
                    BoolExpr::In {
                        list: StringListExpr::AcceptLanguage,
                        value: StringExpr::Constant("en".into()),
                    },
                    BoolExpr::In {
                        list: StringListExpr::AcceptLanguage,
                        value: StringExpr::Constant("en-US".into()),
                    },
                    BoolExpr::In {
                        list: StringListExpr::AcceptLanguage,
                        value: StringExpr::Constant("en-GB".into()),
                    },
                ]),
            },
            Feature {
                name: "british".into(),
                rule: BoolExpr::In {
                    list: StringListExpr::AcceptLanguage,
                    value: StringExpr::Constant("en-GB".into()),
                },
            },
        ]);

        let expected = json!([]).to_string();
        let actual = serde_json::to_string(&config).unwrap();

        assert_eq_diff!(actual, expected);
    }
}
