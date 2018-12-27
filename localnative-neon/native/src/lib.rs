#[macro_use]
extern crate neon;
extern crate localnative_core;

use neon::prelude::*;

fn run(mut cx: FunctionContext) -> JsResult<JsString> {
    let input = cx.argument::<JsString>(0)?.value();
    Ok(cx.string(localnative_core::exe::run(&input)))
}

register_module!(mut cx, { cx.export_function("run", run) });
