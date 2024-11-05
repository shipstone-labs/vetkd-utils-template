// NAPI Module lib.rs
use napi::{bindgen_prelude::*, JsObject, JsString, NapiRaw};
use napi_derive::napi;
use std::ptr;

#[allow(dead_code)]
extern "C" fn greet_callback(env: sys::napi_env, info: sys::napi_callback_info) -> sys::napi_value {
    // Safely use `CallbackInfo` to extract arguments and avoid low-level handling
    let cb_info: CallbackInfo<1> =
        CallbackInfo::new(env, info, Some(1), false).expect("One argument required");
    let env = unsafe { Env::from_raw(env) };

    let name = if let Some(arg) = cb_info.args.get(0) {
        // Ensure the argument is a string by converting it to `JsString`
        if let Some(js_string) = unsafe { arg.cast::<JsString>().as_ref() } {
            match js_string.into_utf8() {
                Ok(utf8) => utf8.as_str().unwrap_or("").to_string(),
                Err(_) => "".to_string(),
            }
        } else {
            // If the argument is not a string, return a default empty string
            "".to_string()
        }
    } else {
        "".to_string()
    };

    // Call your higher-level Rust function
    let result = greet(name);

    // Create a JavaScript string to return
    match env.create_string(&result) {
        Ok(js_string) => unsafe { js_string.raw() }, // Use `.raw()` to get the underlying pointer
        Err(_) => ptr::null_mut(),                   // Return `null` on failure
    }
}

#[napi]
fn init(mut exports: JsObject, env: Env) -> Result<()> {
    // Create a closure to wrap the `greet` function.
    let js_greet = env.create_function("greet", greet_callback)?;

    // Set the `greet` function as a property on the exports object.
    exports.set_named_property("greet", js_greet)?;
    Ok(())
}

#[napi]
fn greet(name: String) -> String {
    format!("Hello, {}!", name)
}

// #[napi]
// fn return_greet_fn(env: Env) -> Result<JsFunction> {
//     env.create_function("greet", greet_callback)
// }
