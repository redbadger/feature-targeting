use std::collections::HashMap;

#[derive(Debug)]
pub struct ExplicitMatchingConfig {
    // pattern to match hostname against. A string with a single `*` character
    // in it as a placeholer for the feature name
    pub host: String,
    // the header which can be used to explicitly enable features
    pub header: String,
}

impl Default for ExplicitMatchingConfig {
    fn default() -> Self {
        Self {
            host: "".to_owned(),
            header: "x-features".to_owned(),
        }
    }
}

pub fn union(existing: &[&str], new: &[&str]) -> String {
    let mut result: Vec<&str> = Vec::new();
    result.extend_from_slice(existing);
    result.extend_from_slice(new);
    result.sort();
    result.dedup();
    result.join(" ")
}

pub fn explicit<'a>(msg: &HashMap<&str, &'a str>, config: &ExplicitMatchingConfig) -> Vec<&'a str> {
    let mut requested_features = match msg.get(&config.header.as_ref()) {
        Some(s) => s.split_whitespace().collect(),
        None => Vec::new(),
    };

    if let Some(s) = msg.get("host").and_then(|it| match_host(it, &config.host)) {
        requested_features.push(s);
    };

    requested_features
}

pub fn implicit<'a>(_msg: &HashMap<&str, &'a str>) -> Vec<&'a str> {
    vec!["new_feature"]
}

fn match_host<'a, 'b>(host: &'a str, pattern: &'b str) -> Option<&'a str> {
    let tokens = pattern.split("*").collect::<Vec<&str>>();

    let (prefix, postfix) = match tokens.as_slice() {
        [prefix, postfix] => (prefix, postfix),
        _ => return None,
    };

    if !host.starts_with(prefix) || !host.ends_with(postfix) {
        return None;
    }

    Some(host.trim_start_matches(prefix).trim_end_matches(postfix))
}

#[cfg(test)]
mod test {
    use super::*;
    use lazy_static::lazy_static;
    use std::collections::HashMap;
    use test_case::test_case;

    lazy_static! {
        static ref CONFIG: ExplicitMatchingConfig = {
            ExplicitMatchingConfig {
                host: "f-*.echo.localhost".to_owned(),
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
    fn explicit_targeting(
        msg: HashMap<&str, &str>,
        config: &ExplicitMatchingConfig,
        features: Vec<&str>,
    ) {
        assert_eq!(explicit(&msg, config), features);
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
    fn substring_matching(pattern: &str, host: &str, mtch: Option<&str>) {
        assert_eq!(match_host(host, pattern), mtch)
    }
}
