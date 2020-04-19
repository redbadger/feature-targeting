use std::collections::HashMap;

use serde::Deserialize;

pub fn from_request<'a>(request: &HashMap<&str, &'a str>, config: &Config) -> Vec<&'a str> {
    let mut features = config
        .0
        .iter()
        .flat_map(|x| x.extract(request))
        .collect::<Vec<&str>>();

    features.sort();

    features
}

// Configuration
#[derive(Debug, Deserialize)]
pub struct Config(pub Vec<Extract>);

impl Default for Config {
    fn default() -> Self {
        Self(vec![Extract::List(List {
            attribute: "x-features".to_owned(),
        })])
    }
}

// Ways of extracting feature names from a request

#[derive(Debug, Deserialize)]
#[serde(tag = "_extract", rename_all = "snake_case")]
pub enum Extract {
    List(List),
    Pattern(Pattern),
}

impl Extract {
    fn extract<'a>(&self, request: &HashMap<&str, &'a str>) -> Vec<&'a str> {
        match self {
            Extract::List(l) => l.extract(request),
            Extract::Pattern(p) => p.extract(request),
        }
    }
}

// List of features in an attribute
#[derive(Debug, Deserialize)]
pub struct List {
    pub attribute: String,
}

impl List {
    fn extract<'a>(&self, request: &HashMap<&str, &'a str>) -> Vec<&'a str> {
        if let Some(value) = request.get::<str>(self.attribute.as_ref()) {
            return value.split_whitespace().collect();
        }

        vec![]
    }
}

// Pattern matching on an attribute

#[derive(Debug, Deserialize)]
pub struct Pattern {
    pub attribute: String,
    pub pattern: String,
}

impl Pattern {
    fn extract<'a>(&self, request: &HashMap<&str, &'a str>) -> Vec<&'a str> {
        if let Some(value) = request.get::<str>(self.attribute.as_ref()) {
            return match_pattern(value, self.pattern.as_ref()).map_or(vec![], |v| vec![v]);
        }

        vec![]
    }
}

fn match_pattern<'a, 'b>(value: &'a str, pattern: &'b str) -> Option<&'a str> {
    let tokens = pattern.split("*").collect::<Vec<&str>>();

    let (prefix, postfix) = match tokens.as_slice() {
        [prefix, postfix] => (prefix, postfix),
        _ => return None,
    };

    if !value.starts_with(prefix) || !value.ends_with(postfix) {
        return None;
    }

    Some(value.trim_start_matches(prefix).trim_end_matches(postfix))
}

#[cfg(test)]
mod test {
    use super::*;
    use lazy_static::lazy_static;
    use std::collections::HashMap;
    use test_case::test_case;

    lazy_static! {
        static ref CONFIG: Config = {
            Config(vec![
                Extract::Pattern(Pattern {
                    attribute: "host".to_owned(),
                    pattern: "f-*.echo.localhost".to_owned(),
                }),
                Extract::List(List {
                    attribute: "x-features".to_owned(),
                }),
            ])
        };
    }

    #[test_case(msg(vec![("x-features", ""), ("host", "echo.localhost")]), &*CONFIG, vec![]; "none")]
    #[test_case(msg(vec![("x-features", "one"), ("host", "echo.localhost")]), &*CONFIG, vec!["one"]; "header")]
    #[test_case(msg(vec![("x-features", "one two"), ("host", "echo.localhost")]), &*CONFIG, vec!["one", "two"]; "multiple")]
    #[test_case(msg(vec![("x-features", ""), ("host", "f-one.echo.localhost")]), &*CONFIG, vec!["one"]; "host")]
    #[test_case(msg(vec![("x-features", "one"), ("host", "f-two.echo.localhost")]), &*CONFIG, vec!["one", "two"]; "combo")]
    fn targeting(request: HashMap<&str, &str>, config: &Config, features: Vec<&str>) {
        assert_eq!(from_request(&request, config), features);
    }

    fn msg<'a, 'b>(headers: Vec<(&'a str, &'b str)>) -> HashMap<&'a str, &'b str> {
        let mut map = HashMap::with_capacity(3 + headers.len());

        map.insert("method", "GET");
        map.insert("path", "/");

        for (name, value) in headers {
            map.insert(name, value);
        }

        map
    }

    #[test_case("*", "foo", Some("foo"); "only match")]
    #[test_case("a-*", "a-foo", Some("foo"); "prefix match")]
    #[test_case("*-a", "foo-a", Some("foo"); "postfix match")]
    #[test_case("aa-*-ab", "aa-foo-ab", Some("foo"); "infix match")]
    #[test_case("aa-*", "foo", None; "prefix not match")]
    #[test_case("aa-*-*-bb", "foo", None; "multiple wildcards")]
    #[test_case("aa", "foo", None; "no wildcard")]
    fn substring_matching(pattern: &str, value: &str, mtch: Option<&str>) {
        assert_eq!(match_pattern(value, pattern), mtch)
    }
}
