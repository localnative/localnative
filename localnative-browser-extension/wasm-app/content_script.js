() => {
  function getMetaContent(name) {
    const meta =
      document.querySelector(`meta[name="${name}"]`) ||
      document.querySelector(`meta[property="${name}"]`);
    return meta ? meta.getAttribute("content") : "";
  }

  function getTags() {
    const keywordsMeta = getMetaContent("keywords");
    return keywordsMeta ? keywordsMeta.split(",").map((tag) => tag.trim()) : [];
  }

  function getAnnotations() {
    // 这里可以根据需要实现获取注释的逻辑
    // 例如，可以获取所有被高亮的文本
    return window.getSelection().toString();
  }

  return {
    title: document.title,
    url: window.location.href,
    description:
      getMetaContent("description") || getMetaContent("og:description"),
    tags: getTags(),
    annotations: getAnnotations(),
  };
};
