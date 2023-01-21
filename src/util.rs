pub(crate) fn safe_env(key: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| panic!("could not get env var '{}'", key))
}
