use anyhow::{anyhow, Result};
use base64::decode as base64decode;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{
    collections::{hash_map::DefaultHasher, HashMap, HashSet},
    hash::{Hash, Hasher},
};
use woothee::parser::{Parser as UserAgentParser, WootheeResult};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum Bool {
    /// The identity expression
    Constant(bool),
    /// Request attribute of name is present
    Attribute(String),
    /// Value contained in the list
    In { list: StrList, value: Str },
    /// Any of the values contained in the list
    AnyIn { list: StrList, values: StrList },
    /// All of the values contained in the list
    AllIn { list: StrList, values: StrList },
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
    JsonPointer { pointer: String, value: Str },
    /// Matches a Regular ession
    Matches(String, Str),
    /// == for strings
    StrEq(Str, Str),
    /// == for numbers
    NumEq(Num, Num),
    /// > for numbers
    Gt(Num, Num),
    /// < for numbers
    Lt(Num, Num),
    /// >= for numbers
    Gte(Num, Num),
    /// <= for numbers
    Lte(Num, Num),
    /// Logical NOT
    Not(Box<Bool>),
    /// Logical AND
    And(Vec<Bool>),
    /// Logical OR
    Or(Vec<Bool>),
}

impl Bool {
    pub fn eval(&self, request: &HashMap<&str, &str>) -> Result<bool> {
        use Bool::*;
        match self {
            Constant(c) => Ok(*c),
            Attribute(name) => request
                .get::<str>(&name)
                .map(|_| true)
                .ok_or_else(|| anyhow!("Attribute '{}' not found.", name)),
            In { list, value } => list
                .eval(request)
                .and_then(|haystack| value.eval(request).map(|needle| haystack.contains(&needle))),
            AnyIn { list, values } => list.eval(request).and_then(|haystack| {
                values.eval(request).map(|needles| {
                    let a: HashSet<_> = haystack.iter().collect();
                    let b: HashSet<_> = needles.iter().collect();

                    a.intersection(&b).next().is_some()
                })
            }),
            AllIn { list, values } => list.eval(request).and_then(|haystack| {
                values.eval(request).map(|needles| {
                    let a: HashSet<_> = haystack.iter().collect();
                    let b: HashSet<_> = needles.iter().collect();

                    a.intersection(&b).count() == a.len()
                })
            }),
            JsonPointer { pointer, value } => value
                .eval(request)
                .and_then(|json| json_pointer(&pointer, &json, "boolean", |v| v.as_bool())),
            Matches(regex, value) => {
                let v = value.eval(request)?;
                let r = Regex::new(&regex)?;
                Ok(r.is_match(&v))
            }
            StrEq(left, right) => {
                let l = left.eval(request)?;
                let r = right.eval(request)?;
                Ok(l == r)
            }
            NumEq(left, right) => {
                let l = left.eval(request)?;
                let r = right.eval(request)?;
                Ok((l - r).abs() < std::f64::EPSILON)
            }
            Gt(left, right) => {
                let l = left.eval(request)?;
                let r = right.eval(request)?;
                Ok(l > r)
            }
            Lt(left, right) => {
                let l = left.eval(request)?;
                let r = right.eval(request)?;
                Ok(l < r)
            }
            Gte(left, right) => {
                let l = left.eval(request)?;
                let r = right.eval(request)?;
                Ok(l >= r)
            }
            Lte(left, right) => {
                let l = left.eval(request)?;
                let r = right.eval(request)?;
                Ok(l <= r)
            }
            Not(value) => value.eval(request).map(|v| !v),
            And(values) => values
                .iter()
                .map(|v| v.eval(request))
                .collect::<Result<Vec<_>, _>>()
                .map(|it| it.iter().all(|v| *v)),
            Or(values) => values
                .iter()
                .map(|v| v.eval(request))
                .collect::<Result<Vec<_>, _>>()
                .map(|it| it.iter().any(|v| *v)),
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum StrList {
    /// The identity expression
    Constant(Vec<String>),
    /// Split a string value using a separator
    Split { separator: String, value: Str },
    /// Extract using a regular expression
    ///
    /// provided regex string must have at least one capture group
    Extract { regex: String, value: Box<Str> },
    /// Parse a HTTP header with q-values,
    /// i.e. Accept, Accept-Charset, Accept-Language, Accept-Encoding
    HttpQualityValue(Str),
}

impl StrList {
    pub fn eval(&self, request: &HashMap<&str, &str>) -> Result<Vec<String>> {
        use StrList::*;
        match self {
            Constant(c) => Ok(c.clone()),
            Split { separator, value } => {
                let s = value.eval(request)?;
                if s == "" {
                    return Ok(vec![]);
                }

                Ok(s.split(separator).map(|s| s.to_string()).collect())
            }
            Extract { regex, value } => {
                let value = value.eval(request)?;
                let regex = Regex::new(regex)?;

                let captures = regex.captures(&value).ok_or(anyhow!(
                    "'{}' does not match '{}'",
                    value,
                    regex
                ))?;

                Ok(captures
                    .iter()
                    .filter_map(|m| m.map(|s| s.as_str().to_string()))
                    .skip(1)
                    .collect())
            }
            HttpQualityValue(value) => {
                let s = value.eval(request)?;
                Ok(parse_q_value(&s).iter().map(|s| s.to_string()).collect())
            }
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum Str {
    /// The identity expression
    Constant(String),
    /// Request attribute value
    Attribute(String),
    /// Base 64 decode value
    Base64(Box<Str>),
    /// Extract using a regular expression
    ///
    /// provided regex string must have one capture group
    Extract { regex: String, value: Box<Str> },
    /// Value of a cookie by name
    Cookie(String),
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
    JsonPointer { pointer: String, value: Box<Str> },
    /// First item of a list
    First(Box<StrList>),
    /// Last item of a list
    Last(Box<StrList>),
}

impl Str {
    pub fn eval(&self, request: &HashMap<&str, &str>) -> Result<String> {
        use Str::*;
        match self {
            Constant(c) => Ok(c.clone()),
            Attribute(name) => request.get::<str>(name).map_or_else(
                || Err(anyhow!("Attribute '{}' not found.", name)),
                |s| Ok((*s).to_string()),
            ),
            Base64(value) => {
                // Transformations like this one mean we can't use string slices
                // as output, as the return values are not simply substrings of
                // the request or the config
                let s = value.eval(request)?;
                let bytes = base64decode(s)?;
                let v = String::from_utf8(bytes)?;

                Ok(v)
            }
            Extract { regex, value } => {
                let value = value.eval(request)?;
                let regex = Regex::new(regex)?;

                regex
                    .captures(&value)
                    .and_then(|c| c.get(1).map(|m| m.as_str().to_string()))
                    .ok_or(anyhow!("'{}' does not match '{}'", value, regex))
            }
            Cookie(name) => get_cookie(request, name).map(|s| s.to_string()),
            Browser => map_user_agent(request, |ua| ua.name.to_string()),
            BrowserVersion => map_user_agent(request, |ua| ua.version.to_string()),
            OperatingSystem => map_user_agent(request, |ua| ua.os.to_string()),
            JsonPointer { pointer, value } => {
                let json = value.eval(request)?;
                json_pointer(pointer, &json, "string", |v| {
                    v.as_str().map(|s| s.to_string())
                })
            }
            First(list) => {
                let v = list.eval(request)?;
                if let Some(s) = v.first() {
                    Ok(s.clone())
                } else {
                    Err(anyhow!("List is empty."))
                }
            }
            Last(list) => {
                let v = list.eval(request)?;
                if let Some(s) = v.last() {
                    Ok(s.clone())
                } else {
                    Err(anyhow!("List is empty."))
                }
            }
        }
    }
}

fn map_user_agent<'a, V, F>(request: &HashMap<&str, &'a str>, map: F) -> Result<V>
where
    F: FnOnce(WootheeResult<'a>) -> V,
    V: 'a,
{
    if let Some(ua) = request.get("user-agent") {
        if let Some(ua) = UserAgentParser::new().parse(ua) {
            Ok(map(ua))
        } else {
            Err(anyhow!("Malformed User-Agent string: {}", ua))
        }
    } else {
        Err(anyhow!("User-Agent header not found"))
    }
}

fn get_cookie<'a>(request: &HashMap<&str, &'a str>, name: &str) -> Result<&'a str> {
    let cookie_header = request
        .get("cookie")
        .ok_or(anyhow!("No cookies found in request"))?;

    cookie_header
        .split("; ")
        .map(|pair| {
            let items: Vec<_> = pair.split("=").collect();

            (items[0], items[1])
        })
        .collect::<HashMap<_, _>>()
        .get(name)
        .map(|it| *it)
        .ok_or(anyhow!("Cookie {} not found", name))
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum Num {
    /// The identity expression
    Constant(f64),
    /// Request attribute value
    Attribute(String),
    /// Randomly assigns a uniformly distributed stable number between 0.0 and 100.0
    Rank(Str),
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
    JsonPointer { pointer: String, value: Str },
}

impl Num {
    fn eval(&self, request: &HashMap<&str, &str>) -> Result<f64> {
        use Num::*;
        match &self {
            Constant(c) => Ok(*c),
            Attribute(name) => {
                if let Some(s) = request.get::<str>(&name) {
                    return Ok((*s).parse::<f64>()?);
                }
                Err(anyhow!("Attribute '{}' not found.", name))
            }
            Rank(str_exp) => str_exp.eval(request).map(|s| {
                let mut hasher = DefaultHasher::new();
                s.hash(&mut hasher);
                (hasher.finish() % 1000) as f64 / 10.0
            }),
            JsonPointer { pointer, value } => value
                .eval(request)
                .and_then(|json| json_pointer(pointer, &json, "number", |v| v.as_f64())),
        }
    }
}

// Helpers

/// Parse a HTTP q-value of the form '*/*;q=0.3, text/plain;q=0.7, text/html, text/*;q=0.5'
fn parse_q_value(value: &str) -> Vec<&str> {
    let mut list: Vec<(&str, f32)> = value
        .split(',')
        .map(|q_val| {
            let mut parts = q_val.split(";q=").map(|it| it.trim());
            let v = parts.next().unwrap();
            let q = parts
                .next()
                .and_then(|q| q.parse::<f32>().ok())
                .or_else(|| Some(1.0))
                .unwrap();

            (v, q)
        })
        .collect();

    list.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    list.iter().map(|(v, _)| *v).collect()
}

// Extract a value out of JSON using a JSON pointer. Useful for JWT tokens for example
fn json_pointer<'a, T, F>(pointer: &str, json: &'a str, typename: &str, cast: F) -> Result<T>
where
    F: FnOnce(&serde_json::Value) -> Option<T>,
    T: 'a,
{
    let value: serde_json::Value = serde_json::from_str(json)?;

    if let Some(inner_value) = value.pointer(pointer) {
        if let Some(v) = cast(inner_value) {
            return Ok(v);
        }
    }

    Err(anyhow!(
        "Cannot find a {} at pointer {} in JSON {}",
        typename,
        pointer,
        value
    ))
}

#[cfg(test)]
mod test {
    use super::*;
    use test_case::test_case;

    #[test_case(Num::Constant(10.0), Ok(10.0))]
    #[test_case(Num::Attribute("number".into()), Ok(1.4))]
    #[test_case(Num::Attribute("nope".into()), Err(anyhow!("Attribute 'nope' not found.")))]
    #[test_case(Num::Attribute("not-number".into()), Err(anyhow!("invalid float literal")))]
    #[test_case(Num::Rank(Str::Attribute("not-number".into())), Ok(40.9))]
    #[test_case(Num::JsonPointer { pointer: "/foo/0".into(), value: Str::Constant(r#"{"foo":[0.3]}"#.into()) }, Ok(0.3))]
    #[test_case(Num::JsonPointer { pointer: "/bar/0".into(), value: Str::Constant(r#"{"foo":[0.3]}"#.into()) }, Err(anyhow!("Cannot find a number at pointer /bar/0 in JSON {\"foo\":[0.3]}")))]
    fn evaluate_numerical_expressions(expr: Num, expected: Result<f64>) {
        let request = [("number", "1.4"), ("not-number", "hi")]
            .iter()
            .cloned()
            .collect();

        let actual = expr.eval(&request);
        assert_eq!(format!("{:?}", actual), format!("{:?}", expected));
    }

    #[test_case(Str::Constant("hello".into()), Ok("hello".into()))]
    #[test_case(Str::Attribute("hello".into()), Ok("world".into()))]
    #[test_case(Str::Base64(Box::new(Str::Constant("aGVsbG8=".into()))), Ok("hello".into()))]
    #[test_case(Str::First(Box::new(StrList::Constant(vec!["a".into(), "b".into(), "c".into()]))), Ok("a".into()))]
    #[test_case(Str::Last(Box::new(StrList::Constant(vec!["a".into(), "b".into(), "c".into()]))), Ok("c".into()))]
    #[test_case(Str::First(Box::new(StrList::Constant(vec![]))), Err(anyhow!("List is empty.")))]
    #[test_case(Str::Last(Box::new(StrList::Constant(vec![]))), Err(anyhow!("List is empty.")))]
    #[test_case(Str::Browser, Ok("Chrome".into()))]
    #[test_case(Str::BrowserVersion, Ok("83.0.4103.61".into()))]
    #[test_case(Str::OperatingSystem, Ok("Mac OSX".into()))]
    #[test_case(Str::JsonPointer { pointer: "/foo/0".into(), value: Box::new(Str::Constant(r#"{"foo":["bar"]}"#.into())) }, Ok("bar".into()))]
    #[test_case(Str::JsonPointer { pointer: "/foo/0".into(), value: Box::new(Str::Constant(r#"{"foo":[0.3]}"#.into())) }, Err(anyhow!("Cannot find a string at pointer /foo/0 in JSON {\"foo\":[0.3]}")))]
    fn evaluate_string_expressions(expr: Str, expected: Result<String>) {
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

        let actual = expr.eval(&request);
        assert_eq!(format!("{:?}", actual), format!("{:?}", expected));
    }

    #[test_case(StrList::Constant(vec!["a".into(), "b".into()]), Ok(vec!["a".into(), "b".into()]))]
    #[test_case(StrList::Split { separator: " ".into(), value: Str::Constant("a b".into())}, Ok(vec!["a".into(), "b".into()]))]
    #[test_case(StrList::HttpQualityValue(Str::Attribute("accept".into())), Ok(vec!["text/html".into(), "text/plain".into(), "text/*".into(), "*/*".into()]))]
    fn evaluate_string_list_expressions(expr: StrList, expected: Result<Vec<String>>) {
        let mut request = HashMap::new();
        request.insert(
            "accept",
            "*/*;q=0.3, text/plain;q=0.7, text/html, text/*;q=0.5",
        );
        let actual = expr.eval(&request);
        assert_eq!(format!("{:?}", actual), format!("{:?}", expected));
    }

    #[test_case(Bool::Constant(true), Ok(true))]
    #[test_case(Bool::Attribute("hello".into()), Ok(true))]
    #[test_case(Bool::Attribute("world".into()), Err(anyhow!("Attribute 'world' not found.")))]
    #[test_case(Bool::In { list: StrList::Constant(vec!["a".into(), "b".into()]), value: Str::Constant("b".into()) }, Ok(true); "list contains value")]
    #[test_case(Bool::In { list: StrList::Constant(vec!["a".into(), "b".into()]), value: Str::Constant("c".into()) }, Ok(false); "list doesn't contain value")]
    #[test_case(Bool::AllIn { list: StrList::Constant(vec!["a".into(), "b".into()]), values: StrList::Constant(vec!["a".into(), "b".into()]) }, Ok(true); "list contains all values")]
    #[test_case(Bool::AllIn { list: StrList::Constant(vec!["a".into(), "b".into()]), values: StrList::Constant(vec!["a".into(), "c".into()]) }, Ok(false); "list doesn't contain all values")]
    #[test_case(Bool::AnyIn { list: StrList::Constant(vec!["a".into(), "b".into()]), values: StrList::Constant(vec!["a".into(), "c".into()]) }, Ok(true); "list contains any of the values")]
    #[test_case(Bool::AnyIn { list: StrList::Constant(vec!["a".into(), "b".into()]), values: StrList::Constant(vec!["c".into(), "d".into()]) }, Ok(false); "list doesn't contain any of the values")]
    #[test_case(Bool::JsonPointer { pointer: "/foo/0".into(), value: Str::Constant(r#"{"foo":[true]}"#.into()) }, Ok(true))]
    #[test_case(Bool::JsonPointer { pointer: "/foo/0".into(), value: Str::Constant(r#"{"foo":[0.3]}"#.into()) }, Err(anyhow!("Cannot find a boolean at pointer /foo/0 in JSON {\"foo\":[0.3]}")))]
    #[test_case(Bool::Matches(r#"\+440?[0-9]{10}"#.into(), Str::Constant("+4407945123456".into())), Ok(true))]
    #[test_case(Bool::Matches(r#"\+440?[0-9]{10}"#.into(), Str::Constant("+47945123456".into())), Ok(false))]
    #[test_case(Bool::StrEq(Str::Constant("foo".into()), Str::Constant("foo".into())), Ok(true))]
    #[test_case(Bool::StrEq(Str::Constant("foo".into()), Str::Constant("bar".into())), Ok(false))]
    #[test_case(Bool::NumEq(Num::Constant(1.3), Num::Constant(1.3)), Ok(true))]
    #[test_case(Bool::NumEq(Num::Constant(1.0), Num::Constant(1.3)), Ok(false))]
    #[test_case(Bool::Gt(Num::Constant(1.3), Num::Constant(1.2)), Ok(true))]
    #[test_case(Bool::Gt(Num::Constant(1.2), Num::Constant(1.3)), Ok(false))]
    #[test_case(Bool::Gt(Num::Constant(1.3), Num::Constant(1.3)), Ok(false))]
    #[test_case(Bool::Gte(Num::Constant(1.3), Num::Constant(1.2)), Ok(true))]
    #[test_case(Bool::Gte(Num::Constant(1.2), Num::Constant(1.3)), Ok(false))]
    #[test_case(Bool::Gte(Num::Constant(1.3), Num::Constant(1.3)), Ok(true))]
    #[test_case(Bool::Lt(Num::Constant(1.3), Num::Constant(1.2)), Ok(false))]
    #[test_case(Bool::Lt(Num::Constant(1.2), Num::Constant(1.3)), Ok(true))]
    #[test_case(Bool::Lt(Num::Constant(1.3), Num::Constant(1.3)), Ok(false))]
    #[test_case(Bool::Lte(Num::Constant(1.3), Num::Constant(1.2)), Ok(false))]
    #[test_case(Bool::Lte(Num::Constant(1.2), Num::Constant(1.3)), Ok(true))]
    #[test_case(Bool::Lte(Num::Constant(1.3), Num::Constant(1.3)), Ok(true))]
    #[test_case(Bool::Not(Box::new(Bool::Constant(true))), Ok(false))]
    #[test_case(Bool::And(vec![Bool::Constant(true), Bool::Constant(true)]), Ok(true))]
    #[test_case(Bool::And(vec![Bool::Constant(true), Bool::Constant(false)]), Ok(false))]
    #[test_case(Bool::Or(vec![Bool::Constant(true), Bool::Constant(false)]), Ok(true))]
    #[test_case(Bool::Or(vec![Bool::Constant(false), Bool::Constant(false)]), Ok(false))]
    fn evaluate_boolean_expressions(expr: Bool, expected: Result<bool>) {
        let request = [("hello", "world")].iter().cloned().collect();

        let actual = expr.eval(&request);
        assert_eq!(format!("{:?}", actual), format!("{:?}", expected));
    }
}
