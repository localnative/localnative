extern crate localnative_ssb;
use localnative_ssb::ssbify;

#[test]
fn test_ssbify_bom() {
    if let Some(s) = ssbify(
        "<html><body><h1>testing</h1><p>this is a test of ssbify-string</p></body></html>",
        "test html blob",
        "http://some.website",
    ) {
        println!("{:?}", s);
        assert_eq!(
            s.hash,
            "&Z05BGxF7EKKGSv2vMtSN/WWMmHJjI4KOqpskAbhQjaM=.sha256"
        );
    }
}

#[test]
fn test_ssbify_bom_zh() {
    if let Some(s) = ssbify(
        "<html><body><h1>测试</h1><p>这是一个中文测试</p></body></html>",
        "测试 html blob",
        "http://some.website",
    ) {
        println!("{:?}", s);
        assert_eq!(
            s.hash,
            "&vNyMLlGhTjfSuTtdWLD3cz4+pd6OS3RYFCm+zk1BszM=.sha256"
        );
    }
}
