pub fn auth_header(jwt: &str) -> String {
    format!("bearer {}", jwt)
}
