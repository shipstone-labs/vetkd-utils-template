
// NAPI Module lib.rs
use napi::{bindgen_prelude::*, JsObject};

#[module_exports]
fn init(mut exports: JsObject) -> napi::Result<()> {
    exports.set_named_property("greet", napi::JsFunction::from_fn(greet)?)?;
    Ok(())
}

#[js_function]
fn greet(ctx: napi::CallContext) -> napi::Result<napi::JsString> {
    ctx.env.create_string("Hello from Rust!")
}
