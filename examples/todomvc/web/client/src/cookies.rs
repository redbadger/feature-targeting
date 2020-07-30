use super::browser::util::cookies;

pub fn get_cookie(name: &str) -> Option<String> {
    if let Some(jar) = cookies() {
        if let Some(cookie) = jar.get(name) {
            return Some(cookie.value().to_string());
        }
    }
    None
}

pub fn delete_cookie(name: &str) {
    if let Some(mut jar) = cookies() {
        let jar2 = jar.clone();
        if let Some(cookie) = jar2.get(name) {
            jar.force_remove(cookie.to_owned());
        }
    }
}

pub fn get_cookie_or_default(name: &str, default: &str) -> String {
    get_cookie(name).unwrap_or_else(|| default.to_owned())
}
