pub fn add_features<'a>(existing: String, new: &[&str]) -> String {
    let mut result = Vec::new();
    for s in existing.split_whitespace() {
        result.push(s);
    }
    for s in new {
        result.push(s);
    }
    result.sort();
    result.dedup();
    result.join(" ")
}

#[cfg(test)]
mod test {
    use super::*;
    use test_case::test_case;

    #[test_case("x y", &["z"], "x y z" ; "add one")]
    #[test_case("x", &["y","z"], "x y z" ; "add two")]
    #[test_case("r", &["r","s","t"], "r s t" ; "add multiple")]
    #[test_case("x y z", &["z"], "x y z" ; "already exists")]
    #[test_case("x z y", &[], "x y z" ; "sort")]
    #[test_case("x z z y", &[], "x y z" ; "dedup")]
    fn add_new_feature(existing: &str, new: &[&str], result: &str) {
        assert_eq!(add_features(existing.to_string(), &new), result);
    }
}
