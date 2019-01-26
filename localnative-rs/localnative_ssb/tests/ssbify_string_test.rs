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
