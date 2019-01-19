#[macro_use]
extern crate neon;
extern crate localnative_core;
extern crate localnative_ssb;
use localnative_core::exe::run as ln_run;
use localnative_core::serde_json;
use localnative_core::Cmd;
use localnative_ssb as ssb;

use neon::prelude::*;

fn run(mut cx: FunctionContext) -> JsResult<JsString> {
    let text = cx.argument::<JsString>(0)?.value();

    if let Ok(cmd) = serde_json::from_str::<Cmd>(&text) {
        match cmd.action.as_ref() {
            "ssb-sync" => {
                ssb::run_sync();
                return Ok(cx.string(r#"{"run_sync": "done"}"#));
            }
            _ => {
                let response = ln_run(&text);
                eprintln!("response {:?}", response);
                return Ok(cx.string(response));
            }
        }
    }
    Ok(cx.string(r#"{"error":"cmd"}"#))
}

#[no_mangle]
pub extern "C" fn __cxa_pure_virtual() {
    //https://users.rust-lang.org/t/neon-electron-undefined-symbol-cxa-pure-virtual/21223
    loop {}
}

register_module!(mut cx, { cx.export_function("run", run) });
