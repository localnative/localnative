use fluent_static::{message_bundle, MessageBundle};

#[message_bundle(
    resources = [
        ("locales/en/tr.ftl", "en"),
        ("locales/zh/tr.ftl", "zh"),
    ],
    default_language = "en"
)]
pub struct Tr;

/// Look up the message bundle for a UI language tag. `state.language` is
/// normally "en" or "zh" (the values of the settings select); tags such as
/// "en-US" or "zh-CN" match on the primary subtag, and anything unsupported
/// falls back to the default language.
pub fn tr_for(lang: &str) -> Tr {
    let primary = lang.split(['-', '_']).next().unwrap_or(lang);
    Tr::get(primary).unwrap_or_default()
}
