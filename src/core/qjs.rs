use crate::c::util::{cstr_to_rust, ngenrs_free_cstr, rust_to_cstr};
use libquickjs_ng_sys::{
    JS_Call, JS_Eval, JS_FreeValue, JS_GetException, JS_GetGlobalObject, JS_GetPropertyStr,
    JS_HasException, JS_NewContext, JS_NewRuntime, JS_NewStringLen, JS_SetPropertyStr, JS_ToString,
    JSContext, JSRuntime, JSValue,
};
use std::ffi::CString;
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};

pub struct JSBridge {
    rt: Arc<Mutex<*mut JSRuntime>>,
    ctx: Arc<Mutex<*mut JSContext>>,
}

impl JSBridge {
    pub fn new() -> Self {
        unsafe {
            let rt = JS_NewRuntime();
            let ctx = JS_NewContext(rt);

            JSBridge {
                rt: Arc::new(Mutex::new(rt)),
                ctx: Arc::new(Mutex::new(ctx)),
            }
        }
    }

    pub fn load_file(&self, path: &str) -> Result<(), String> {
        let content = fs::read_to_string(Path::new(path))
            .map_err(|e| format!("Failed to read file: {}", e))?;
        self.load_script(&content)
    }

    pub fn load_script(&self, script: &str) -> Result<(), String> {
        unsafe {
            let ctx = self.ctx.lock().unwrap();
            let cscript = CString::new(script).unwrap();
            let filename = CString::new("script.js").unwrap();

            let val = JS_Eval(
                *ctx,
                cscript.as_ptr(),
                script.len(),
                filename.as_ptr(),
                libquickjs_ng_sys::JS_EVAL_TYPE_GLOBAL as i32,
            );

            if JS_HasException(*ctx) {
                let exception = JS_GetException(*ctx);
                let string_val = JS_ToString(*ctx, exception);

                let mut len = 0;
                let ptr = libquickjs_ng_sys::JS_ToCStringLen2(*ctx, &mut len, string_val, false);
                let err_msg = cstr_to_rust(ptr).unwrap_or("Unknown error").to_string();

                if !ptr.is_null() {
                    libquickjs_ng_sys::JS_FreeCString(*ctx, ptr);
                }
                JS_FreeValue(*ctx, exception);
                JS_FreeValue(*ctx, string_val);
                return Err(format!("Script error: {}", err_msg));
            }

            JS_FreeValue(*ctx, val);
            Ok(())
        }
    }

    pub fn call_function(&self, func_name: &str, arg: &str) -> Result<String, String> {
        unsafe {
            let ctx = self.ctx.lock().unwrap();
            let global = JS_GetGlobalObject(*ctx);

            let cname = CString::new(func_name).unwrap();
            let func_val = JS_GetPropertyStr(*ctx, global, cname.as_ptr());

            if JS_HasException(*ctx) {
                JS_FreeValue(*ctx, global);
                return Err(format!("Function {} not found", func_name));
            }

            // Create JS string from input arg
            let arg_val =
                JS_NewStringLen(*ctx, arg.as_ptr() as *const i8, arg.len() as libc::size_t);

            let result = JS_Call(
                *ctx,
                func_val,
                global,
                1, // Single argument
                &arg_val as *const JSValue as *mut JSValue,
            );

            JS_FreeValue(*ctx, func_val);
            JS_FreeValue(*ctx, global);

            if JS_HasException(*ctx) {
                let exception = JS_GetException(*ctx);
                let mut len = 0;
                let ptr = libquickjs_ng_sys::JS_ToCStringLen2(*ctx, &mut len, exception, false);
                let err_msg = cstr_to_rust(ptr).unwrap_or("Unknown error").to_string();

                if !ptr.is_null() {
                    libquickjs_ng_sys::JS_FreeCString(*ctx, ptr);
                }
                JS_FreeValue(*ctx, exception);
                return Err(format!("Function call error: {}", err_msg));
            }

            // Convert result to string
            let mut len = 0;
            let ptr = libquickjs_ng_sys::JS_ToCStringLen2(*ctx, &mut len, result, false);
            let result_str = cstr_to_rust(ptr).unwrap_or("").to_string();

            if !ptr.is_null() {
                libquickjs_ng_sys::JS_FreeCString(*ctx, ptr);
            }
            JS_FreeValue(*ctx, result);

            Ok(result_str)
        }
    }

    pub fn export_function<F>(&self, name: &str, func: F) -> Result<(), String>
    where
        F: Fn(Vec<JSValue>) -> Result<JSValue, String> + 'static,
    {
        unsafe {
            let ctx = self.ctx.lock().unwrap();
            let global = JS_GetGlobalObject(*ctx);
            let cname = CString::new(name).unwrap();

            // Define a trait object type for our callback
            type Callback = dyn Fn(Vec<JSValue>) -> Result<JSValue, String> + 'static;

            // Box the closure as a trait object
            let func_box: Box<Callback> = Box::new(func);
            let func_ptr = Box::into_raw(func_box);

            // Store the pointer immediately after converting to raw
            libquickjs_ng_sys::JS_SetContextOpaque(*ctx, func_ptr as *mut libc::c_void);

            // Create a raw callback that uses the trait object
            extern "C" fn raw_callback(
                ctx: *mut JSContext,
                _this_val: JSValue,
                argc: i32,
                argv: *mut JSValue,
            ) -> JSValue {
                unsafe {
                    // Get the raw pointer and cast it to the concrete type
                    let func_ptr = libquickjs_ng_sys::JS_GetContextOpaque(ctx) as *mut ();
                    let func_ptr =
                        func_ptr as *mut Box<dyn Fn(Vec<JSValue>) -> Result<JSValue, String>>;
                    let func = &**func_ptr;

                    let args = if argc > 0 {
                        std::slice::from_raw_parts(argv, argc as usize).to_vec()
                    } else {
                        Vec::new()
                    };

                    match func(args) {
                        Ok(result) => result,
                        Err(err) => {
                            let err_cstr = rust_to_cstr(err);
                            let js_str = libquickjs_ng_sys::JS_NewStringLen(
                                ctx,
                                err_cstr,
                                libc::strlen(err_cstr),
                            );
                            // Use utility function to free the C string
                            ngenrs_free_cstr(err_cstr);
                            libquickjs_ng_sys::JS_Throw(ctx, js_str);
                            let global = libquickjs_ng_sys::JS_GetGlobalObject(ctx);
                            let undefined = libquickjs_ng_sys::JS_GetPropertyStr(
                                ctx,
                                global,
                                CString::new("undefined").unwrap().as_ptr(),
                            );
                            libquickjs_ng_sys::JS_FreeValue(ctx, global);
                            undefined
                        }
                    }
                }
            }

            // Create JS function with our raw callback
            let js_func = libquickjs_ng_sys::JS_NewCFunction2(
                *ctx,
                Some(raw_callback),
                cname.as_ptr(),
                0,
                0,
                0,
            );

            JS_SetPropertyStr(*ctx, global, cname.as_ptr(), js_func);
            JS_FreeValue(*ctx, global);
            Ok(())
        }
    }
}

impl Drop for JSBridge {
    fn drop(&mut self) {
        unsafe {
            let ctx = self.ctx.lock().unwrap();
            let rt = self.rt.lock().unwrap();
            libquickjs_ng_sys::JS_FreeContext(*ctx);
            libquickjs_ng_sys::JS_FreeRuntime(*rt);
        }
    }
}
