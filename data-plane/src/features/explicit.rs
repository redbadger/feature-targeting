use crate::features::expression::{Str, StrList};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub fn from_request(request: &HashMap<&str, &str>, config: &Config) -> Vec<String> {
    let mut features = config
        .0
        .iter()
        .flat_map(|x| x.eval(request).unwrap_or_else(|_| vec![]))
        .map(|s| s)
        .collect::<Vec<_>>();

    features.sort();

    features
}

/// Configuration
#[derive(Debug, Serialize, Deserialize)]
pub struct Config(pub Vec<StrList>);

impl Default for Config {
    fn default() -> Self {
        Self(vec![StrList::Split {
            separator: " ".to_owned(),
            value: Str::Attribute("x-feature-overrides".to_owned()),
        }])
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use lazy_static::lazy_static;
    use pretty_assertions::assert_eq as assert_eq_diff;
    use serde_json::json;
    use std::collections::HashMap;
    use test_case::test_case;

    lazy_static! {
        static ref CONFIG: Config = {
            Config(vec![
                StrList::Extract {
                    value: Box::new(Str::Attribute("host".to_owned())),
                    regex: r#"f-([a-z]+)\.echo\.localhost"#.to_owned(),
                },
                StrList::Split {
                    separator: " ".to_owned(),
                    value: Str::Attribute("x-features".to_owned()),
                },
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

    #[test]
    fn serialises_to_json() {
        let expected = json!([
            {
                "extract": {
                    "regex": r#"f-([a-z]+)\.echo\.localhost"#,
                    "value": { "attribute": "host" }
                }
            },
            {
                "split": {
                    "separator": " ",
                    "value": { "attribute": "x-features" }
                }
            }
        ])
        .to_string();
        let actual = serde_json::to_string(&*CONFIG).unwrap();

        assert_eq_diff!(actual, expected);
    }
}
