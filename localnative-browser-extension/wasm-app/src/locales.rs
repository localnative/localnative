pub mod tr {
    #[allow(unused_imports)]
    use fluent_static::fluent_bundle::{
        FluentArgs, FluentBundle, FluentError, FluentResource, FluentValue,
    };
    use fluent_static::once_cell::sync::Lazy;
    use fluent_static::Message;
    static SUPPORTED_LANGUAGES: &[&str] = &["en", "zh"];
    static EN_RESOURCE : & str = "# Local Native v0.7.0\napp-title = Local Native\n\n# Public\ntoggle-label = Public\n\n# Language\nselect-label = Language\noption-en-US = en-US English - US\noption-zh-CN = zh-CN 中文简体 - 大陆\n\n# Input Section\ntitle = Title\nurl = URL\ndescription = Description\ntags = Tags (comma or space separated)\ninsert-btn = Insert\n\n# Search Section\nsearch = Search\nclear-search = Clear Search Term\nprevious-page = Previous Page\nnext-page = Next Page\n\ncollapse = collapse\nexpand = expand\n" ;
    static EN_BUNDLE: Lazy<FluentBundle<FluentResource>> = Lazy::new(|| {
        let lang_id = fluent_static::unic_langid::langid!("en");
        let mut bundle: FluentBundle<FluentResource> = FluentBundle::new_concurrent(vec![lang_id]);
        bundle
            .add_resource(FluentResource::try_new(EN_RESOURCE.to_string()).unwrap())
            .unwrap();
        bundle
    });
    static ZH_RESOURCE : & str = "# Local Native v0.7.0\napp-title = Local Native\n\n# Public\ntoggle-label = 公开\n\n# Language\nselect-label = 语言\noption-en-US = en-US English - US\noption-zh-CN = zh-CN 中文简体 - 大陆\n\n# Input Section\ntitle = 标题\nurl = 网址\ndescription = 描述\ntags = 标签 (逗号或空格分隔)\ninsert-btn = 插入\n\n# Search Section\nsearch = 搜索\nclear-search = 清除搜索词\nprevious-page = 上一页\nnext-page = 下一页\n\n\ncollapse = 收回\nexpand = 展开\n" ;
    static ZH_BUNDLE: Lazy<FluentBundle<FluentResource>> = Lazy::new(|| {
        let lang_id = fluent_static::unic_langid::langid!("zh");
        let mut bundle: FluentBundle<FluentResource> = FluentBundle::new_concurrent(vec![lang_id]);
        bundle
            .add_resource(FluentResource::try_new(ZH_RESOURCE.to_string()).unwrap())
            .unwrap();
        bundle
    });
    fn get_bundle(lang: &str) -> &'static FluentBundle<FluentResource> {
        for common_lang in fluent_static::accept_language::intersection(lang, SUPPORTED_LANGUAGES) {
            match common_lang.as_str() {
                "en" => return &EN_BUNDLE,
                "zh" => return &ZH_BUNDLE,
                _ => continue,
            }
        }
        &EN_BUNDLE
    }
    fn format_message(
        lang_id: &str,
        message_id: &str,
        attr: Option<&str>,
        args: Option<&FluentArgs>,
    ) -> Result<Message<'static>, FluentError> {
        let bundle = get_bundle(lang_id.as_ref());
        let msg = bundle.get_message(message_id).expect("Message not found");
        let pattern = if let Some(attr) = attr {
            msg.get_attribute(attr).unwrap().value()
        } else {
            msg.value().unwrap()
        };
        let mut errors = vec![];
        let result = Message::new(bundle.format_pattern(pattern, args, &mut errors));
        if errors.is_empty() {
            Ok(result)
        } else {
            Err(errors.into_iter().next().unwrap())
        }
    }
    pub fn app_title(lang_id: impl AsRef<str>) -> Message<'static> {
        format_message(lang_id.as_ref(), "app-title", None, None)
            .expect("Not fallible without variables; qed")
    }
    pub fn toggle_label(lang_id: impl AsRef<str>) -> Message<'static> {
        format_message(lang_id.as_ref(), "toggle-label", None, None)
            .expect("Not fallible without variables; qed")
    }
    pub fn select_label(lang_id: impl AsRef<str>) -> Message<'static> {
        format_message(lang_id.as_ref(), "select-label", None, None)
            .expect("Not fallible without variables; qed")
    }
    pub fn option_en_us(lang_id: impl AsRef<str>) -> Message<'static> {
        format_message(lang_id.as_ref(), "option-en-US", None, None)
            .expect("Not fallible without variables; qed")
    }
    pub fn option_zh_cn(lang_id: impl AsRef<str>) -> Message<'static> {
        format_message(lang_id.as_ref(), "option-zh-CN", None, None)
            .expect("Not fallible without variables; qed")
    }
    pub fn title(lang_id: impl AsRef<str>) -> Message<'static> {
        format_message(lang_id.as_ref(), "title", None, None)
            .expect("Not fallible without variables; qed")
    }
    pub fn url(lang_id: impl AsRef<str>) -> Message<'static> {
        format_message(lang_id.as_ref(), "url", None, None)
            .expect("Not fallible without variables; qed")
    }
    pub fn description(lang_id: impl AsRef<str>) -> Message<'static> {
        format_message(lang_id.as_ref(), "description", None, None)
            .expect("Not fallible without variables; qed")
    }
    pub fn tags(lang_id: impl AsRef<str>) -> Message<'static> {
        format_message(lang_id.as_ref(), "tags", None, None)
            .expect("Not fallible without variables; qed")
    }
    pub fn insert_btn(lang_id: impl AsRef<str>) -> Message<'static> {
        format_message(lang_id.as_ref(), "insert-btn", None, None)
            .expect("Not fallible without variables; qed")
    }
    pub fn search(lang_id: impl AsRef<str>) -> Message<'static> {
        format_message(lang_id.as_ref(), "search", None, None)
            .expect("Not fallible without variables; qed")
    }
    pub fn clear_search(lang_id: impl AsRef<str>) -> Message<'static> {
        format_message(lang_id.as_ref(), "clear-search", None, None)
            .expect("Not fallible without variables; qed")
    }
    pub fn previous_page(lang_id: impl AsRef<str>) -> Message<'static> {
        format_message(lang_id.as_ref(), "previous-page", None, None)
            .expect("Not fallible without variables; qed")
    }
    pub fn next_page(lang_id: impl AsRef<str>) -> Message<'static> {
        format_message(lang_id.as_ref(), "next-page", None, None)
            .expect("Not fallible without variables; qed")
    }
    pub fn collapse(lang_id: impl AsRef<str>) -> Message<'static> {
        format_message(lang_id.as_ref(), "collapse", None, None)
            .expect("Not fallible without variables; qed")
    }
    pub fn expand(lang_id: impl AsRef<str>) -> Message<'static> {
        format_message(lang_id.as_ref(), "expand", None, None)
            .expect("Not fallible without variables; qed")
    }
}
