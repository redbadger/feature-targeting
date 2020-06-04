use base64::decode as base64decode;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{
    collections::{hash_map::DefaultHasher, HashMap, HashSet},
    hash::{Hash, Hasher},
};
use woothee::parser::{Parser as UserAgentParser, WootheeResult};

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
    /// The identity expression
    Constant(bool),
    /// Request attribute of name is present
    Attribute(String),
    /// Value contained in the list
    In {
        list: StringListExpr,
        value: StringExpr,
    },
    /// Any of the values contained in the list
    AnyIn {
        list: StringListExpr,
        values: StringListExpr,
    },
    /// All of the values contained in the list
    AllIn {
        list: StringListExpr,
        values: StringListExpr,
    },
    /// Looks up a boolean value by a JSON Pointer
    ///
    /// JSON Pointer defines a string syntax for identifying a specific value
    /// within a JavaScript Object Notation (JSON) document.
    ///
    /// A Pointer is a Unicode string with the reference tokens separated by `/`.
    /// Inside tokens `/` is replaced by `~1` and `~` is replaced by `~0`. The
    /// addressed value is returned and if there is no such value `None` is
    /// returned.
    ///
    /// For more information read [RFC6901](https://tools.ietf.org/html/rfc6901).
    JsonPointer { pointer: String, value: StringExpr },
    /// Matches a Regular Expression
    Matches(String, StringExpr),
    /// == for strings
    StrEq(StringExpr, StringExpr),
    /// == for numbers
    NumEq(NumExpr, NumExpr),
    /// > for numbers
    Gt(NumExpr, NumExpr),
    /// < for numbers
    Lt(NumExpr, NumExpr),
    /// >= for numbers
    Gte(NumExpr, NumExpr),
    /// <= for numbers
    Lte(NumExpr, NumExpr),
    /// Logical NOT
    Not(Box<BoolExpr>),
    /// Logical AND
    And(Vec<BoolExpr>),
    /// Logical OR
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

                    a.intersection(&b).count() == a.len()
                })
            }),
            BoolExpr::JsonPointer { pointer, value } => value
                .eval(request)
                .and_then(|json| json_pointer(pointer, json, "boolean", |v| v.as_bool())),
            BoolExpr::Matches(regex, value) => value.eval(request).and_then(|v| {
                Regex::new(regex)
                    .map(|r| r.is_match(v.as_ref()))
                    .map_err(|e| format!("{}", e))
            }),
            BoolExpr::StrEq(left, right) => left
                .eval(request)
                .and_then(|l| right.eval(request).map(|r| l == r)),
            BoolExpr::NumEq(left, right) => left.eval(request).and_then(|l| {
                right
                    .eval(request)
                    .map(|r| (l - r).abs() < std::f64::EPSILON)
            }),
            BoolExpr::Gt(left, right) => left
                .eval(request)
                .and_then(|l| right.eval(request).map(|r| l > r)),
            BoolExpr::Lt(left, right) => left
                .eval(request)
                .and_then(|l| right.eval(request).map(|r| l < r)),
            BoolExpr::Gte(left, right) => left
                .eval(request)
                .and_then(|l| right.eval(request).map(|r| l >= r)),
            BoolExpr::Lte(left, right) => left
                .eval(request)
                .and_then(|l| right.eval(request).map(|r| l <= r)),
            BoolExpr::Not(value) => value.eval(request).map(|v| !v),
            BoolExpr::And(values) => values
                .iter()
                .map(|v| v.eval(request))
                .collect::<Result<Vec<_>, _>>()
                .map(|it| it.iter().all(|v| *v)),
            BoolExpr::Or(values) => values
                .iter()
                .map(|v| v.eval(request))
                .collect::<Result<Vec<_>, _>>()
                .map(|it| it.iter().any(|v| *v)),
        }
    }
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StringListExpr {
    /// The identity expression
    Constant(Vec<String>),
    /// Split a string value using a separator
    Split {
        separator: String,
        value: StringExpr,
    },
    /// Parse a HTTP header with q-values,
    /// i.e. Accept, Accept-Charset, Accept-Language, Accept-Encoding
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

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StringExpr {
    /// The identity expression
    Constant(String),
    /// Request attribute value
    Attribute(String),
    /// Base 64 decode value
    Base64(Box<StringExpr>),
    /// Derives browser from User-Agent
    Browser,
    /// Derives bowser version from User-Agent
    BrowserVersion,
    /// Derives operating system from User-Agent
    OperatingSystem,
    /// Looks up a string value by a JSON Pointer
    ///
    /// JSON Pointer defines a string syntax for identifying a specific value
    /// within a JavaScript Object Notation (JSON) document.
    ///
    /// A Pointer is a Unicode string with the reference tokens separated by `/`.
    /// Inside tokens `/` is replaced by `~1` and `~` is replaced by `~0`. The
    /// addressed value is returned and if there is no such value `None` is
    /// returned.
    ///
    /// For more information read [RFC6901](https://tools.ietf.org/html/rfc6901).
    JsonPointer {
        pointer: String,
        value: Box<StringExpr>,
    },
    /// First item of a list
    First(Box<StringListExpr>),
    /// Last item of a list
    Last(Box<StringListExpr>),
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
            StringExpr::Base64(value) => value.eval(request).and_then(|v| {
                base64decode(v)
                    .map_err(|e| format!("{}", e))
                    .and_then(|it| {
                        std::str::from_utf8(&it[..])
                            .map(|it| it.into())
                            .map_err(|e| format!("{}", e))
                    })
            }),
            StringExpr::Browser => map_user_agent(request, |ua| ua.name.into()),
            StringExpr::BrowserVersion => map_user_agent(request, |ua| ua.version.into()),
            StringExpr::OperatingSystem => map_user_agent(request, |ua| ua.os.into()),
            StringExpr::JsonPointer { pointer, value } => value.eval(request).and_then(|json| {
                json_pointer(pointer, json, "string", |v| {
                    v.as_str().map(|s| s.to_string())
                })
            }),
            StringExpr::First(list) => list
                .eval(request)
                .and_then(|v| v.first().cloned().ok_or_else(|| "List is empty.".into())),
            StringExpr::Last(list) => list
                .eval(request)
                .and_then(|v| v.last().cloned().ok_or_else(|| "List is empty.".into())),
        }
    }
}

fn map_user_agent<V, F: FnOnce(WootheeResult) -> V>(
    request: &HashMap<&str, &str>,
    map: F,
) -> Result<V, String> {
    request
        .get("user-agent")
        .map_or(Err("User-Agent header not found".into()), |ua| {
            UserAgentParser::new()
                .parse(ua)
                .map(map)
                .map_or(Err(format!("Malformed User-Agent string: {}", ua)), |it| {
                    Ok(it)
                })
        })
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum NumExpr {
    /// The identity expression
    Constant(f64),
    /// Request attribute value
    Attribute(String),
    /// Randomly assigns a uniformly distributed stable number between 0.0 and 100.0
    Rank(StringExpr),
    /// Looks up a number value by a JSON Pointer
    ///
    /// JSON Pointer defines a string syntax for identifying a specific value
    /// within a JavaScript Object Notation (JSON) document.
    ///
    /// A Pointer is a Unicode string with the reference tokens separated by `/`.
    /// Inside tokens `/` is replaced by `~1` and `~` is replaced by `~0`. The
    /// addressed value is returned and if there is no such value `None` is
    /// returned.
    ///
    /// For more information read [RFC6901](https://tools.ietf.org/html/rfc6901).
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
            NumExpr::JsonPointer { pointer, value } => value
                .eval(request)
                .and_then(|json| json_pointer(pointer, json, "number", |v| v.as_f64())),
        }
    }
}

// Helpers

/// Parse a HTTP q-value of the form '*/*;q=0.3, text/plain;q=0.7, text/html, text/*;q=0.5'
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

    list.iter()
        .map(|(v, _)| (*v).to_string())
        .collect::<Vec<_>>()
}

// Extract a value out of JSON using a JSON pointer. Useful for JWT tokens for example
fn json_pointer<T, F>(pointer: &str, json: String, typename: &str, cast: F) -> Result<T, String>
where
    F: FnOnce(&serde_json::Value) -> Option<T>,
{
    serde_json::from_str(json.as_ref())
        .map_err(|e| format!("{}", e))
        .and_then(|v: serde_json::Value| {
            v.pointer(pointer).and_then(cast).ok_or_else(|| {
                format!(
                    "Cannot find a {} at pointer {} in JSON {}",
                    typename, pointer, v
                )
            })
        })
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
    #[test_case(NumExpr::JsonPointer { pointer: "/foo/0".into(), value: StringExpr::Constant(r#"{"foo":[0.3]}"#.into()) }, Ok(0.3))]
    #[test_case(NumExpr::JsonPointer { pointer: "/bar/0".into(), value: StringExpr::Constant(r#"{"foo":[0.3]}"#.into()) }, Err("Cannot find a number at pointer /bar/0 in JSON {\"foo\":[0.3]}".into()))]
    fn evaluate_numerical_expressions(expr: NumExpr, expected: Result<f64, String>) {
        let request = [("number", "1.4"), ("not-number", "hi")]
            .iter()
            .cloned()
            .collect();

        assert_eq!(expr.eval(&request), expected)
    }

    #[test_case(StringExpr::Constant("hello".into()), Ok("hello".into()))]
    #[test_case(StringExpr::Attribute("hello".into()), Ok("world".into()))]
    #[test_case(StringExpr::Base64(Box::new(StringExpr::Constant("aGVsbG8=".into()))), Ok("hello".into()))]
    #[test_case(StringExpr::First(Box::new(StringListExpr::Constant(vec!["a".into(), "b".into(), "c".into()]))), Ok("a".into()))]
    #[test_case(StringExpr::Last(Box::new(StringListExpr::Constant(vec!["a".into(), "b".into(), "c".into()]))), Ok("c".into()))]
    #[test_case(StringExpr::First(Box::new(StringListExpr::Constant(vec![]))), Err("List is empty.".into()))]
    #[test_case(StringExpr::Last(Box::new(StringListExpr::Constant(vec![]))), Err("List is empty.".into()))]
    #[test_case(StringExpr::Browser, Ok("Chrome".into()))]
    #[test_case(StringExpr::BrowserVersion, Ok("83.0.4103.61".into()))]
    #[test_case(StringExpr::OperatingSystem, Ok("Mac OSX".into()))]
    #[test_case(StringExpr::JsonPointer { pointer: "/foo/0".into(), value: Box::new(StringExpr::Constant(r#"{"foo":["bar"]}"#.into())) }, Ok("bar".into()))]
    #[test_case(StringExpr::JsonPointer { pointer: "/foo/0".into(), value: Box::new(StringExpr::Constant(r#"{"foo":[0.3]}"#.into())) }, Err("Cannot find a string at pointer /foo/0 in JSON {\"foo\":[0.3]}".into()))]
    fn evaluate_string_expressions(expr: StringExpr, expected: Result<String, String>) {
        let request = [
            ("hello", "world"),
            (
                "user-agent",
                "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_4) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/83.0.4103.61 Safari/537.36",
            ),
        ]
        .iter()
        .cloned()
        .collect();

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
    #[test_case(BoolExpr::JsonPointer { pointer: "/foo/0".into(), value: StringExpr::Constant(r#"{"foo":[true]}"#.into()) }, Ok(true))]
    #[test_case(BoolExpr::JsonPointer { pointer: "/foo/0".into(), value: StringExpr::Constant(r#"{"foo":[0.3]}"#.into()) }, Err("Cannot find a boolean at pointer /foo/0 in JSON {\"foo\":[0.3]}".into()))]
    #[test_case(BoolExpr::Matches(r#"\+440?[0-9]{10}"#.into(), StringExpr::Constant("+4407945123456".into())), Ok(true))]
    #[test_case(BoolExpr::Matches(r#"\+440?[0-9]{10}"#.into(), StringExpr::Constant("+47945123456".into())), Ok(false))]
    #[test_case(BoolExpr::StrEq(StringExpr::Constant("foo".into()), StringExpr::Constant("foo".into())), Ok(true))]
    #[test_case(BoolExpr::StrEq(StringExpr::Constant("foo".into()), StringExpr::Constant("bar".into())), Ok(false))]
    #[test_case(
        BoolExpr::NumEq(NumExpr::Constant(1.3), NumExpr::Constant(1.3)),
        Ok(true)
    )]
    #[test_case(
        BoolExpr::NumEq(NumExpr::Constant(1.0), NumExpr::Constant(1.3)),
        Ok(false)
    )]
    #[test_case(BoolExpr::Gt(NumExpr::Constant(1.3), NumExpr::Constant(1.2)), Ok(true))]
    #[test_case(
        BoolExpr::Gt(NumExpr::Constant(1.2), NumExpr::Constant(1.3)),
        Ok(false)
    )]
    #[test_case(
        BoolExpr::Gt(NumExpr::Constant(1.3), NumExpr::Constant(1.3)),
        Ok(false)
    )]
    #[test_case(
        BoolExpr::Gte(NumExpr::Constant(1.3), NumExpr::Constant(1.2)),
        Ok(true)
    )]
    #[test_case(
        BoolExpr::Gte(NumExpr::Constant(1.2), NumExpr::Constant(1.3)),
        Ok(false)
    )]
    #[test_case(
        BoolExpr::Gte(NumExpr::Constant(1.3), NumExpr::Constant(1.3)),
        Ok(true)
    )]
    #[test_case(
        BoolExpr::Lt(NumExpr::Constant(1.3), NumExpr::Constant(1.2)),
        Ok(false)
    )]
    #[test_case(BoolExpr::Lt(NumExpr::Constant(1.2), NumExpr::Constant(1.3)), Ok(true))]
    #[test_case(
        BoolExpr::Lt(NumExpr::Constant(1.3), NumExpr::Constant(1.3)),
        Ok(false)
    )]
    #[test_case(
        BoolExpr::Lte(NumExpr::Constant(1.3), NumExpr::Constant(1.2)),
        Ok(false)
    )]
    #[test_case(
        BoolExpr::Lte(NumExpr::Constant(1.2), NumExpr::Constant(1.3)),
        Ok(true)
    )]
    #[test_case(
        BoolExpr::Lte(NumExpr::Constant(1.3), NumExpr::Constant(1.3)),
        Ok(true)
    )]
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

        let expected = json!([
            {
                "name":"english",
                "rule": {
                    "any_in": {
                        "list": { "constant": ["en","en-US","en-GB"] },
                        "values": {
                            "http_quality_value": { "attribute": "accept-language" }
                        }
                    }
                }
            },
            {
                "name":"other-english",
                "rule": {
                    "or": [
                        {
                            "in": {
                                "list": {
                                    "http_quality_value": { "attribute": "accept-language" }
                                },
                                "value": { "constant": "en" }
                            }
                        },
                        {
                            "in": {
                                "list": {
                                    "http_quality_value": { "attribute": "accept-language" }
                                },
                                "value": { "constant": "en-US" }
                            }
                        },
                        {
                            "in": {
                                "list": {
                                    "http_quality_value": { "attribute": "accept-language" }
                                },
                                "value": { "constant": "en-GB" }
                            }
                        }
                    ]
                }
            },
            {
                "name":"british",
                "rule": {
                    "in": {
                        "list": {
                            "http_quality_value": { "attribute": "accept-language" }
                        },
                        "value": { "constant": "en-GB" }
                    }
                }
            }
        ])
        .to_string();
        let actual = serde_json::to_string(&config).unwrap();

        assert_eq_diff!(actual, expected);
    }
}
