extern crate localnative_core;
use localnative_core::ssb::ssbify_string;

#[test]
fn test_ssbify_string() {
    let link = ssbify_string(
        "<html><body><h1>testing</h1><p>this is a test of ssbify-string</p></body></html>",
        "test html blob",
        "http://some.website",
    );
    assert_eq!(link, "&nUNxeZTJkqw0q6yoUqUdlwjz22Pu0XITnhVDiIelEoM=.sha256");
}

#[test]
fn test_ssbify_string_cn() {
    let link = ssbify_string(
        "<html><body><h1>测试</h1><p>这是一个中文测试</p></body></html>",
        "测试 html blob",
        "http://some.website",
    );
    println!("{}", link);
    assert_eq!(link, "&kNkjGQvUkOEC1xbaqnRDrnUQ3daN1jnA0SaNoMatkuc=.sha256");
}
