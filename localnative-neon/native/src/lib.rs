#[macro_use]
extern crate neon;
extern crate localnative_core;

use neon::prelude::*;

fn run(mut cx: FunctionContext) -> JsResult<JsString> {
    let input = cx.argument::<JsString>(0)?.value();
    Ok(cx.string(localnative_core::exe::run(&input)))
}

#[no_mangle]
pub extern fn __cxa_pure_virtual() {
//https://users.rust-lang.org/t/neon-electron-undefined-symbol-cxa-pure-virtual/21223
    loop{};
}

register_module!(mut cx, { cx.export_function("run", run) });
