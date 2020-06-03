use serde::{Deserialize, Serialize};
use std::{
    collections::{hash_map::DefaultHasher, HashMap, HashSet},
    hash::{Hash, Hasher},
};

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
#[serde(rename_all = "snake_case")]
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
            BoolExpr::Constant(c) => Ok(*c),
            BoolExpr::Attribute(name) => request
                .get::<str>(name.as_ref())
                .map_or(Err(format!("Attribute '{}' not found.", name)), |_| {
                    Ok(true)
                }),
            BoolExpr::In { list, value } => list
                .eval(request)
                .and_then(|haystack| value.eval(request).map(|needle| haystack.contains(&needle))),
            BoolExpr::AnyIn { list, values } => list.eval(request).and_then(|haystack| {
                values.eval(request).map(|needles| {
                    let a: HashSet<_> = haystack.iter().collect();
                    let b: HashSet<_> = needles.iter().collect();

                    a.intersection(&b).next().is_some()
                })
            }),
            BoolExpr::AllIn { list, values } => list.eval(request).and_then(|haystack| {
                values.eval(request).map(|needles| {
                    let a: HashSet<_> = haystack.iter().collect();
                    let b: HashSet<_> = needles.iter().collect();

                    a.intersection(&b).collect::<Vec<_>>().len() == a.len()
                })
            }),
            BoolExpr::JsonPointer {
                pointer: _,
                value: _,
            } => todo!(),
            BoolExpr::Matches(_regex, _value) => todo!(),
            BoolExpr::StrEq(_left, _right) => todo!(),
            BoolExpr::NumEq(_left, _right) => todo!(),
            BoolExpr::Gt(_left, _right) => todo!(),
            BoolExpr::Lt(_left, _right) => todo!(),
            BoolExpr::Gte(_left, _right) => todo!(),
            BoolExpr::Lte(_left, _right) => todo!(),
            BoolExpr::Not(value) => value.eval(request).map(|v| !v),
            BoolExpr::And(values) => values
                .iter()
                .map(|v| v.eval(request))
                .collect::<Result<Vec<_>, _>>()
                .map(|it| it.iter().all(|v| *v == true)),
            BoolExpr::Or(values) => values
                .iter()
                .map(|v| v.eval(request))
                .collect::<Result<Vec<_>, _>>()
                .map(|it| it.iter().any(|v| *v == true)),
        }
    }
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StringListExpr {
    Constant(Vec<String>),
    Split {
        // Split a string value using a separator
        separator: String,
        value: StringExpr,
    },
    // Parse a HTTP header with q-values,
    // i.e. Accept, Accept-Charset, Accept-Language, Accept-Encoding
    HttpQualityValue(StringExpr),
}

impl StringListExpr {
    fn eval(&self, request: &HashMap<&str, &str>) -> Result<Vec<String>, String> {
        match &self {
            StringListExpr::Constant(c) => Ok(c.clone()),
            StringListExpr::Split { separator, value } => value.eval(request).map(|s| {
                s.split(separator)
                    .map(|item| item.to_string())
                    .collect::<Vec<_>>()
            }),
            StringListExpr::HttpQualityValue(value) => value.eval(request).map(parse_q_value),
        }
    }
}

// Parse a HTTP q-value of the form '*/*;q=0.3, text/plain;q=0.7, text/html, text/*;q=0.5'
fn parse_q_value(value: String) -> Vec<String> {
    let mut list: Vec<(&str, f32)> = value
        .split(',')
        .map(|q_val| {
            let mut parts = q_val.split(";q=").map(|it| it.trim());
            let v = parts.next().unwrap();
            let q = parts
                .next()
                .and_then(|q| q.parse::<f32>().ok())
                .or(Some(1.0))
                .unwrap();

            (v, q)
        })
        .collect();

    list.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    list.iter().map(|(v, _)| v.to_string()).collect::<Vec<_>>()
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
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
            StringExpr::JsonPointer {
                pointer: _pointer,
                value: _value,
            } => todo!(),
            StringExpr::First(list) => list
                .eval(request)
                .and_then(|v| v.first().cloned().ok_or_else(|| "List is empty.".into())),
            StringExpr::Last(list) => list
                .eval(request)
                .and_then(|v| v.last().cloned().ok_or_else(|| "List is empty.".into())),
        }
    }
}

// Numerical value extractors
#[derive(Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
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
            NumExpr::JsonPointer {
                pointer: _pointer,
                value: _value,
            } => todo!(),
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

    #[test_case(StringListExpr::Constant(vec!["a".into(), "b".into()]), Ok(vec!["a".into(), "b".into()]))]
    #[test_case(StringListExpr::Split { separator: " ".into(), value: StringExpr::Constant("a b".into())}, Ok(vec!["a".into(), "b".into()]))]
    #[test_case(StringListExpr::HttpQualityValue(StringExpr::Attribute("accept".into())), Ok(vec!["text/html".into(), "text/plain".into(), "text/*".into(), "*/*".into()]))]
    fn evaluate_string_list_expressions(
        expr: StringListExpr,
        expected: Result<Vec<String>, String>,
    ) {
        let request = [(
            "accept",
            "*/*;q=0.3, text/plain;q=0.7, text/html, text/*;q=0.5",
        )]
        .iter()
        .cloned()
        .collect();

        assert_eq!(expr.eval(&request), expected)
    }

    #[test_case(BoolExpr::Constant(true), Ok(true))]
    #[test_case(BoolExpr::Attribute("hello".into()), Ok(true))]
    #[test_case(BoolExpr::Attribute("world".into()), Err("Attribute 'world' not found.".into()))]
    #[test_case(BoolExpr::In { list: StringListExpr::Constant(vec!["a".into(), "b".into()]), value: StringExpr::Constant("b".into()) }, Ok(true); "list contains value")]
    #[test_case(BoolExpr::In { list: StringListExpr::Constant(vec!["a".into(), "b".into()]), value: StringExpr::Constant("c".into()) }, Ok(false); "list doesn't contain value")]
    #[test_case(BoolExpr::AllIn { list: StringListExpr::Constant(vec!["a".into(), "b".into()]), values: StringListExpr::Constant(vec!["a".into(), "b".into()]) }, Ok(true); "list contains all values")]
    #[test_case(BoolExpr::AllIn { list: StringListExpr::Constant(vec!["a".into(), "b".into()]), values: StringListExpr::Constant(vec!["a".into(), "c".into()]) }, Ok(false); "list doesn't contain all values")]
    #[test_case(BoolExpr::AnyIn { list: StringListExpr::Constant(vec!["a".into(), "b".into()]), values: StringListExpr::Constant(vec!["a".into(), "c".into()]) }, Ok(true); "list contains any of the values")]
    #[test_case(BoolExpr::AnyIn { list: StringListExpr::Constant(vec!["a".into(), "b".into()]), values: StringListExpr::Constant(vec!["c".into(), "d".into()]) }, Ok(false); "list doesn't contain any of the values")]
    #[test_case(BoolExpr::Not(Box::new(BoolExpr::Constant(true))), Ok(false))]
    #[test_case(BoolExpr::And(vec![BoolExpr::Constant(true), BoolExpr::Constant(true)]), Ok(true))]
    #[test_case(BoolExpr::And(vec![BoolExpr::Constant(true), BoolExpr::Constant(false)]), Ok(false))]
    #[test_case(BoolExpr::Or(vec![BoolExpr::Constant(true), BoolExpr::Constant(false)]), Ok(true))]
    #[test_case(BoolExpr::Or(vec![BoolExpr::Constant(false), BoolExpr::Constant(false)]), Ok(false))]
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
                    values: StringListExpr::HttpQualityValue(StringExpr::Attribute(
                        "accept-language".into(),
                    )),
                },
            },
            Feature {
                name: "other-english".into(),
                rule: BoolExpr::Or(vec![
                    BoolExpr::In {
                        list: StringListExpr::HttpQualityValue(StringExpr::Attribute(
                            "accept-language".into(),
                        )),
                        value: StringExpr::Constant("en".into()),
                    },
                    BoolExpr::In {
                        list: StringListExpr::HttpQualityValue(StringExpr::Attribute(
                            "accept-language".into(),
                        )),
                        value: StringExpr::Constant("en-US".into()),
                    },
                    BoolExpr::In {
                        list: StringListExpr::HttpQualityValue(StringExpr::Attribute(
                            "accept-language".into(),
                        )),
                        value: StringExpr::Constant("en-GB".into()),
                    },
                ]),
            },
            Feature {
                name: "british".into(),
                rule: BoolExpr::In {
                    list: StringListExpr::HttpQualityValue(StringExpr::Attribute(
                        "accept-language".into(),
                    )),
                    value: StringExpr::Constant("en-GB".into()),
                },
            },
            Feature {
                name: "german".into(),
                rule: BoolExpr::In {
                    list: StringListExpr::HttpQualityValue(StringExpr::Attribute(
                        "accept-language".into(),
                    )),
                    value: StringExpr::Constant("de".into()),
                },
            },
        ]);

        assert_eq!(
            from_request(req, &config),
            vec!["english", "other-english", "british"]
        );
    }

    #[test]
    fn serialises_to_json() {
        let config = Config(vec![
            Feature {
                name: "english".into(),
                rule: BoolExpr::AnyIn {
                    list: StringListExpr::Constant(vec![
                        "en".into(),
                        "en-US".into(),
                        "en-GB".into(),
                    ]),
                    values: StringListExpr::HttpQualityValue(StringExpr::Attribute(
                        "accept-language".into(),
                    )),
                },
            },
            Feature {
                name: "other-english".into(),
                rule: BoolExpr::Or(vec![
                    BoolExpr::In {
                        list: StringListExpr::HttpQualityValue(StringExpr::Attribute(
                            "accept-language".into(),
                        )),
                        value: StringExpr::Constant("en".into()),
                    },
                    BoolExpr::In {
                        list: StringListExpr::HttpQualityValue(StringExpr::Attribute(
                            "accept-language".into(),
                        )),
                        value: StringExpr::Constant("en-US".into()),
                    },
                    BoolExpr::In {
                        list: StringListExpr::HttpQualityValue(StringExpr::Attribute(
                            "accept-language".into(),
                        )),
                        value: StringExpr::Constant("en-GB".into()),
                    },
                ]),
            },
            Feature {
                name: "british".into(),
                rule: BoolExpr::In {
                    list: StringListExpr::HttpQualityValue(StringExpr::Attribute(
                        "accept-language".into(),
                    )),
                    value: StringExpr::Constant("en-GB".into()),
                },
            },
        ]);

        let expected = json!([{"name":"english","rule":{"any_in":{"list":{"constant":["en","en-US","en-GB"]},"values":{"http_quality_value":{"attribute":"accept-language"}}}}},{"name":"other-english","rule":{"or":[{"in":{"list":{"http_quality_value":{"attribute":"accept-language"}},"value":{"constant":"en"}}},{"in":{"list":{"http_quality_value":{"attribute":"accept-language"}},"value":{"constant":"en-US"}}},{"in":{"list":{"http_quality_value":{"attribute":"accept-language"}},"value":{"constant":"en-GB"}}}]}},{"name":"british","rule":{"in":{"list":{"http_quality_value":{"attribute":"accept-language"}},"value":{"constant":"en-GB"}}}}]).to_string();
        let actual = serde_json::to_string(&config).unwrap();

        assert_eq_diff!(actual, expected);
    }
}
