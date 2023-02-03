use anyhow::Context as _;

pub(crate) fn safe_env(key: &str) -> anyhow::Result<String> {
    std::env::var(key).with_context(|| format!("could not get env var '{key}'"))
}
