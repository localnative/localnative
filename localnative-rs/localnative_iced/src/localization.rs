use std::{borrow::Cow, fmt::Display, sync::Arc};

use elsa::sync::FrozenMap;
use fluent_bundle::FluentResource;
use fluent_bundle::{bundle::FluentBundle, FluentArgs};
use intl_memoizer::concurrent::IntlLangMemoizer;
use serde::{Deserialize, Serialize};
use tokio::sync::OnceCell;
use unic_langid::langid;
use unic_langid::LanguageIdentifier;

static BUNDLES: OnceCell<FrozenMap<Language, Arc<FluentBundle<FluentResource, IntlLangMemoizer>>>> =
    OnceCell::const_new();

pub static mut BUNDLE: OnceCell<Option<&FluentBundle<FluentResource, IntlLangMemoizer>>> =
    OnceCell::const_new();

async fn read_file(path: &str) -> anyhow::Result<String> {
    tokio::fs::read_to_string(path)
        .await
        .map_err(|e| anyhow::anyhow!("read file fail:{:?}", e))
}

async fn create_resource(path: &str) -> anyhow::Result<FluentResource> {
    let content = read_file(path).await?;
    match FluentResource::try_new(content) {
        Ok(res) => Ok(res),
        Err((res, err)) => {
            for e in err {
                log::error!("fluent resource error:{:?}", e);
            }
            Ok(res)
        }
    }
}
async fn create_bundle(
    locale: Language,
) -> anyhow::Result<FluentBundle<FluentResource, IntlLangMemoizer>> {
    let path = locale.path();
    let res = create_resource(&path).await?;
    let mut bundle = FluentBundle::new_concurrent(vec![locale.locale()]);
    bundle.set_use_isolating(false);
    if let Err(e) = bundle.add_resource(res) {
        for e in e {
            log::error!("add resource error:{:?}", e);
        }
    }
    Ok(bundle)
}

async fn init_bundles(
    locale: Language,
) -> anyhow::Result<&'static FluentBundle<FluentResource, IntlLangMemoizer>> {
    let ress = BUNDLES.get_or_init(|| async { FrozenMap::new() }).await;
    ress.get(&locale)
        .ok_or_else(|| anyhow::anyhow!("ress not have this."))
        .or({
            let bundle = create_bundle(locale).await?;
            Ok(ress.insert(locale, Arc::new(bundle)))
        })
}
pub async fn init_bundle(locale: Language) -> anyhow::Result<()> {
    let bundle = init_bundles(locale).await?;
    unsafe {
        if BUNDLE.initialized() {
            let bundle_inner = BUNDLE.get_mut().unwrap();
            bundle_inner.replace(bundle);
        } else {
            BUNDLE
                .set(Some(bundle))
                .map_err(|_| anyhow::anyhow!("bundle set error"))?;
        }
    }
    Ok(())
}

#[inline]
pub fn tr_with_args<'a, 'arg: 'a>(
    key: &'a str,
    args: Option<&'a FluentArgs<'arg>>,
) -> Cow<'a, str> {
    let res = unsafe {
        BUNDLE.get().and_then(|bundle| {
            bundle.and_then(|bundle| {
                bundle
                    .get_message(key)
                    .and_then(|msg| msg.value())
                    .map(|p| {
                        let mut errors = vec![];
                        let res = bundle.format_pattern(p, args, &mut errors);
                        for e in errors {
                            log::error!("fluent get error:{:?}", e);
                        }
                        res
                    })
            })
        })
    };
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
