use std::{borrow::Cow, fmt::Display, sync::Arc};

use elsa::sync::FrozenMap;
use fluent_bundle::FluentResource;
use fluent_bundle::{bundle::FluentBundle, FluentArgs};
use intl_memoizer::concurrent::IntlLangMemoizer;
use serde::{Deserialize, Serialize};
use tokio::sync::{OnceCell, RwLock};
use unic_langid::langid;
use unic_langid::LanguageIdentifier;

use crate::error_handle;

static BUNDLE_CACHE: OnceCell<
    FrozenMap<Language, Arc<FluentBundle<FluentResource, IntlLangMemoizer>>>,
> = OnceCell::const_new();

pub static BUNDLE: OnceCell<RwLock<&FluentBundle<FluentResource, IntlLangMemoizer>>> =
    OnceCell::const_new();

async fn read_file(path: &str) -> Option<String> {
    tokio::fs::read_to_string(path)
        .await
        .map_err(error_handle)
        .ok()
}

async fn create_resource(path: &str) -> Option<FluentResource> {
    let content = read_file(path).await?;
    match FluentResource::try_new(content) {
        Ok(res) => Some(res),
        Err((res, err)) => {
            err.into_iter().for_each(error_handle);
            Some(res)
        }
    }
}
async fn create_bundle(locale: Language) -> Option<FluentBundle<FluentResource, IntlLangMemoizer>> {
    let path = locale.path();
    let res = create_resource(&path).await?;
    let mut bundle = FluentBundle::new_concurrent(vec![locale.locale()]);
    bundle.set_use_isolating(false);
    if let Err(e) = bundle.add_resource(res) {
        e.into_iter().for_each(error_handle);
    }
    Some(bundle)
}

async fn init_bundle_cache(
    locale: Language,
) -> Option<&'static FluentBundle<FluentResource, IntlLangMemoizer>> {
    let ress = BUNDLE_CACHE
        .get_or_init(|| async { FrozenMap::new() })
        .await;
    ress.get(&locale).or({
        let bundle = create_bundle(locale).await?;
        Some(ress.insert(locale, Arc::new(bundle)))
    })
}
pub async fn init_bundle(locale: Language) -> Option<()> {
    let bundle = init_bundle_cache(locale).await?;
    if BUNDLE.initialized() {
        let mut bundle_inner = BUNDLE.get()?.write().await;
        *bundle_inner = bundle;
    } else {
        BUNDLE
            .set(RwLock::new(bundle))
            //.map_err(error_handle)
            .ok()?;
    }
    Some(())
}

#[inline]
pub fn tr_with_args<'a, 'arg: 'a>(
    key: &'a str,
    args: Option<&'a FluentArgs<'arg>>,
) -> Cow<'a, str> {
    let res = BUNDLE
        .get()
        .and_then(|bundle| bundle.try_read().ok())
        .and_then(|bundle| {
            bundle
                .get_message(key)
                .and_then(|msg| msg.value())
                .map(|p| {
                    let mut errors = vec![];
                    let res = bundle.format_pattern(p, args, &mut errors);
                    errors.into_iter().for_each(error_handle);
                    res
                })
        });
    if let Some(res) = res {
        res
    } else {
        Cow::from(key)
    }
}

#[derive(Copy, Clone, Debug, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub enum Language {
    English,
    Chinese,
}
const EN_US: LanguageIdentifier = langid!("en-US");
const ZH_CN: LanguageIdentifier = langid!("zh-CN");

impl Language {
    pub fn locale(&self) -> LanguageIdentifier {
        match self {
            Language::English => EN_US,
            Language::Chinese => ZH_CN,
        }
    }
    pub fn path(&self) -> String {
        let locale = self.locale().to_string();
        format!("./locales/{}/tr.ftl", locale)
    }
}
impl Default for Language {
    fn default() -> Self {
        Language::English
    }
}
impl Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Language::English => write!(f, "english"),
            Language::Chinese => write!(f, "中文"),
        }
    }
}

#[macro_export]
macro_rules! tr {
    ($msg:expr) => {
        crate::translate::tr_with_args($msg, None)
    };
    ($msg:expr; $args:expr ) => {{
        crate::translate::tr_with_args($msg, Some($args))
    }};
}

#[macro_export]
macro_rules! args {
    ($($key:expr => $value:expr),+ ) => {
        {
            let mut args: fluent_bundle::FluentArgs = fluent_bundle::FluentArgs::new();
            $(
                args.set($key, $value);
            )+
            args
        }
    };
}
