extern crate localnative_core;
use localnative_core::ssb::ssbify_string;

#[test]
fn test_ssbify_string() {
    let link = ssbify_string(
        "<html><body><h1>testing</h1><p>this is a test of ssbify-string</p></body></html>",
        "test html blob",
        "http://some.website",
    );
    assert_eq!(link, "&Z05BGxF7EKKGSv2vMtSN/WWMmHJjI4KOqpskAbhQjaM=.sha256");
}

#[test]
fn test_ssbify_string_cn() {
    let link = ssbify_string(
        "<html><body><h1>测试</h1><p>这是一个中文测试</p></body></html>",
        "测试 html blob",
        "http://some.website",
    );
    println!("{}", link);
    assert_eq!(link, "&vNyMLlGhTjfSuTtdWLD3cz4+pd6OS3RYFCm+zk1BszM=.sha256");
}
