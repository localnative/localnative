extern crate localnative_core;
use localnative_core::cmd::make_tags;

#[test]
fn test_make_tags() {
    let output = make_tags("this,is a tag");
    eprintln!("{:?}", output);
    assert_eq!("this,is,a,tag", output);
    let output2 = make_tags(" , , this,, is a tag,, that is another , ,tag, ");
    eprintln!("{:?}", output2);
    assert_eq!("this,a,that,is,another,tag", output2);
}
