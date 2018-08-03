extern crate localnative_core;
use localnative_core::ssb::{ssbify_string};


#[test]
fn test_ssbify_string() {
    let link = ssbify_string("<html><body><h1>testing</h1><p>this is a test of ssbify-string</p></body></html>",
                  "test html blob",
                  "http://some.website");
    assert_eq!(link, "&nUNxeZTJkqw0q6yoUqUdlwjz22Pu0XITnhVDiIelEoM=.sha256");
}

