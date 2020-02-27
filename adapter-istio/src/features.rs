use crate::server::adapter_istio::InstanceMsg;
use regex::Regex;

#[derive(Debug)]
pub struct ExplicitMatchingConfig {
    // TODO this can probably be generalised to allow different types of matching
    // on different inputs, but I can't be bothered to figure that out right now
    pub host: Regex,
    pub header: String,
}

pub fn union(existing: &[&str], new: &[&str]) -> String {
    let mut result: Vec<&str> = Vec::new();
    result.extend_from_slice(existing);
    result.extend_from_slice(new);
    result.sort();
    result.dedup();
    result.join(" ")
}

pub fn explicit<'a>(msg: &'a InstanceMsg, config: &ExplicitMatchingConfig) -> Vec<&'a str> {
    let mut requested_features = match msg.headers.get(&config.header) {
        Some(s) => s.split_whitespace().collect(),
        None => Vec::new(),
    };

    if let Some(s) = msg
        .headers
        .get("host")
        .and_then(|it| match_host(it, &config.host))
    {
        requested_features.push(s);
    };

    requested_features
}

pub fn implicit(msg: &InstanceMsg) -> Vec<&str> {
    vec!["new_feature"]
}

fn match_host<'a>(host: &'a str, regex: &Regex) -> Option<&'a str> {
    println!("{:?}", regex);

    regex
        .captures(host)
        .and_then(|it| it.get(1))
        .map(|it| it.as_str())
}

#[cfg(test)]
mod test {
    use super::*;
    use lazy_static::lazy_static;
    use test_case::test_case;

    lazy_static! {
        static ref CONFIG: ExplicitMatchingConfig = {
            ExplicitMatchingConfig {
                host: Regex::new("^f-([a-z0-9-]+)\\..+$").unwrap(),
                header: "x-features".to_owned(),
            }
        };
    }

    #[test_case(&["x y"], &["z"], "x y z" ; "add one")]
    #[test_case(&["x"], &["y","z"], "x y z" ; "add two")]
    #[test_case(&["r"], &["r","s","t"], "r s t" ; "add multiple")]
    #[test_case(&["x", "y", "z"], &["z"], "x y z" ; "already exists")]
    #[test_case(&["x", "z", "y"], &[], "x y z" ; "sort")]
    #[test_case(&["x", "z", "z", "y"], &[], "x y z" ; "dedup")]
    fn union_feature(existing: &[&str], new: &[&str], result: &str) {
        assert_eq!(union(existing, new), result);
    }

    #[test_case(msg(vec![("x-features", ""), ("host", "echo.localhost")]), &*CONFIG, vec![]; "none")]
    #[test_case(msg(vec![("x-features", "one"), ("host", "echo.localhost")]), &*CONFIG, vec!["one"]; "header")]
    #[test_case(msg(vec![("x-features", "one two"), ("host", "echo.localhost")]), &*CONFIG, vec!["one", "two"]; "multiple")]
    #[test_case(msg(vec![("x-features", ""), ("host", "f-one.echo.localhost")]), &*CONFIG, vec!["one"]; "host")]
    #[test_case(msg(vec![("x-features", "one"), ("host", "f-two.echo.localhost")]), &*CONFIG, vec!["one", "two"]; "combo")]
    fn explicit_targeting(msg: InstanceMsg, config: &ExplicitMatchingConfig, features: Vec<&str>) {
        assert_eq!(explicit(&msg, config), features);
    }

    fn msg(headers: Vec<(&str, &str)>) -> InstanceMsg {
        InstanceMsg {
            name: "xyz".to_owned(),
            method: "GET".to_owned(),
            path: "/".to_owned(),
            headers: headers
                .into_iter()
                .map(|(k, v)| (k.to_owned(), v.to_owned()))
                .collect(),
        }
    }
}
