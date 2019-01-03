# Changelog

## v0.3.3
- package web host inside electron and setup manifest
- change path to ~/LocalNative
- consolidate localnative-nodejs into only one binary

## v0.3.2-appimage
- electron gnu/linux desktop build

## v0.3.2-neon
- nodejs to rust bridge used in electron

## v0.3.2-rust
- same as v0.3.1-rust and v0.3.0-rust

## v0.3.2-npm
- upgrade package.json dependencies

## v0.3.2-mas-pkg
- electron mac app store build but got rejected

## v0.3.1-rust
- fix crate-type, all rust builds works on mac

## v0.3.0-rust
- extract localnative_ssb as new crate

## v0.2.4-ios
- move cancel and save button down
- upgrade rusqlite to 0.16.0 etc

## v0.2.4-rust
- format and dedup tags in rust 

## v0.2.4-android
- insert note on android for chrome

## v0.2.3-android
- search on android

## v0.2.3-ios
- insert and search on ios for safari

## v0.2.2
- release initial iOS App
- web extension
  - persist language choice
  - fix firefox language dropdown issue

## v0.2.1
- add i18n zh-CN

## v0.2.0
- allow public note be send to ssb
- breaking schema change of table note by adding is_public column, sql migration script added

## v0.1.5
- allow web-ext to function without ssb sync
- docs with screenshots

## v0.1.4
- remove annotations in ssb message
- fill all ssb fields after publish

## v0.1.3
- recursively reduce annotation markdown size
- ssbify None result handling 

## v0.1.2
- fix to not ssbify when empty annotations
- internalize ssbify code
- allow small sized markdown to be displayed in ssb message

## v0.1.1
- use sql transactions
- simplify rust to nodejs calls to global binaries

## v0.1.0
- ssb sync with other ids
- add ssbify option
- hide delete item operation

## v0.1.0
- hide delete item operation
- tune UI elements to be more instant responsive

## v0.0.2
- add delete item operation
- tune UI elements to be more instant responsive

## v0.0.1
- basic firefox and chrome extension

