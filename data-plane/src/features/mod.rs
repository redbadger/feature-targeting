use std::collections::HashMap;

pub mod explicit;
pub mod implicit;

pub fn target<'a>(
    request: &HashMap<&str, &'a str>,
    explicit_config: &explicit::Config,
    implicit_config: &implicit::Config,
) -> String {
    union(
        target_explicit(request, explicit_config).as_ref(),
        target_implicit(request, implicit_config).as_ref(),
    )
}

pub fn union(existing: &[&str], new: &[&str]) -> String {
    let mut result: Vec<&str> = Vec::new();
    result.extend_from_slice(existing);
    result.extend_from_slice(new);
    result.sort();
    result.dedup();
    result.join(" ")
}

pub fn target_explicit<'a>(
    request: &HashMap<&str, &'a str>,
    config: &explicit::Config,
) -> Vec<&'a str> {
    explicit::from_request(request, config)
}

pub fn target_implicit<'a>(
    request: &HashMap<&str, &'a str>,
    config: &'a implicit::Config,
) -> Vec<&'a str> {
    implicit::from_request(request, config)
}

#[cfg(test)]
mod test {
    use super::*;
    use test_case::test_case;

    #[test_case(&["x y"], &["z"], "x y z" ; "add one")]
    #[test_case(&["x"], &["y","z"], "x y z" ; "add two")]
    #[test_case(&["r"], &["r","s","t"], "r s t" ; "add multiple")]
    #[test_case(&["x", "y", "z"], &["z"], "x y z" ; "already exists")]
    #[test_case(&["x", "z", "y"], &[], "x y z" ; "sort")]
    #[test_case(&["x", "z", "z", "y"], &[], "x y z" ; "dedup")]
    fn union_feature(existing: &[&str], new: &[&str], result: &str) {
        assert_eq!(union(existing, new), result);
    }
}
