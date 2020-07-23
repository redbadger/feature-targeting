use crate::features::expression::Bool;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A set of features and their matching rules
#[derive(Deserialize, Serialize, Debug)]
pub struct Config(pub Vec<Feature>);

impl Default for Config {
    fn default() -> Self {
        Self(vec![])
    }
}

/// Feature represents implicit targeting configuration for a single feature flag
#[derive(Deserialize, Serialize, Debug)]
pub struct Feature {
    name: String,
    rule: Bool,
}

pub fn from_request<'a>(request: &HashMap<&str, &str>, config: &'a Config) -> Vec<&'a str> {
    config
        .0
        .iter()
        .filter_map(|Feature { name, rule }| match rule.eval(&request) {
            Ok(true) => Some(name.as_ref()),
            _ => None, // Ignore features whose rules fail to evaluate
        })
        .collect()
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::features::expression::*;
    use pretty_assertions::assert_eq as assert_eq_diff;
    use serde_json::json;

    #[test]
    fn matches_a_complex_expression() {
        let req = [("accept-language", "en-GB,en;q=0.9,cs;q=0.8")]
            .iter()
            .cloned()
            .collect();

        let config = Config(vec![
            Feature {
                name: "english".into(),
                rule: Bool::AnyIn {
                    list: StrList::Constant(vec!["en".into(), "en-US".into(), "en-GB".into()]),
                    values: StrList::HttpQualityValue(Str::Attribute("accept-language".into())),
                },
            },
            Feature {
                name: "other-english".into(),
                rule: Bool::Or(vec![
                    Bool::In {
                        list: StrList::HttpQualityValue(Str::Attribute("accept-language".into())),
                        value: Str::Constant("en".into()),
                    },
                    Bool::In {
                        list: StrList::HttpQualityValue(Str::Attribute("accept-language".into())),
                        value: Str::Constant("en-US".into()),
                    },
                    Bool::In {
                        list: StrList::HttpQualityValue(Str::Attribute("accept-language".into())),
                        value: Str::Constant("en-GB".into()),
                    },
                ]),
            },
            Feature {
                name: "british".into(),
                rule: Bool::In {
                    list: StrList::HttpQualityValue(Str::Attribute("accept-language".into())),
                    value: Str::Constant("en-GB".into()),
                },
            },
            Feature {
                name: "german".into(),
                rule: Bool::In {
                    list: StrList::HttpQualityValue(Str::Attribute("accept-language".into())),
                    value: Str::Constant("de".into()),
                },
            },
        ]);

        assert_eq!(
            from_request(&req, &config),
            vec!["english", "other-english", "british"]
        );
    }

    #[test]
    fn serialises_to_json() {
        let config = Config(vec![
            Feature {
                name: "english".into(),
                rule: Bool::AnyIn {
                    list: StrList::Constant(vec!["en".into(), "en-US".into(), "en-GB".into()]),
                    values: StrList::HttpQualityValue(Str::Attribute("accept-language".into())),
                },
            },
            Feature {
                name: "other-english".into(),
                rule: Bool::Or(vec![
                    Bool::In {
                        list: StrList::HttpQualityValue(Str::Attribute("accept-language".into())),
                        value: Str::Constant("en".into()),
                    },
                    Bool::In {
                        list: StrList::HttpQualityValue(Str::Attribute("accept-language".into())),
                        value: Str::Constant("en-US".into()),
                    },
                    Bool::In {
                        list: StrList::HttpQualityValue(Str::Attribute("accept-language".into())),
                        value: Str::Constant("en-GB".into()),
                    },
                ]),
            },
            Feature {
                name: "british".into(),
                rule: Bool::In {
                    list: StrList::HttpQualityValue(Str::Attribute("accept-language".into())),
                    value: Str::Constant("en-GB".into()),
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
