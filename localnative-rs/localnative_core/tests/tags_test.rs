/*
    Local Native
    Copyright (C) 2018-2019  Yi Wang

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU Affero General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU Affero General Public License for more details.

    You should have received a copy of the GNU Affero General Public License
    along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/
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
