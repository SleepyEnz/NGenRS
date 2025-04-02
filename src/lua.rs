use mlua::{Lua, Result, Function};
use std::path::Path;

pub struct LuaBridge {
    lua: Lua,
}

impl LuaBridge {
    pub fn new() -> Result<Self> {
        Ok(LuaBridge {
            lua: Lua::new()
        })
    }

    pub fn load_file(&self, path: &str) -> Result<()> {
        let path = Path::new(path);
        self.lua.load(path).exec()
    }

    pub fn load_string(&self, script: &str) -> Result<()> {
        self.lua.load(script).exec()
    }

    pub fn call_function<A, R>(&self, func_name: &str, args: A) -> Result<R>
    where
        A: for<'lua> mlua::ToLuaMulti<'lua>,
        R: for<'lua> mlua::FromLuaMulti<'lua>,
    {
        let func: Function = self.lua.globals().get(func_name)?;
        func.call(args)
    }

    // 4. Export Rust function to Lua context
    pub fn export_function<'a, F, R>(&self, name: &str, func: F) -> Result<()>
    where
        F: Fn(&Lua, mlua::Value) -> Result<R> + 'static,
        R: for<'lua> mlua::ToLuaMulti<'lua>,
    {
        let lua_func = self.lua.create_function(func)?;
        self.lua.globals().set(name, lua_func)
    }

    // Generic version that works with any Rust function
    pub fn export_rust_fn<F, A, R>(&self, name: &str, func: F) -> Result<()>
    where
        F: Fn(A) -> R + 'static,
        A: for<'lua> mlua::FromLuaMulti<'lua>,
        R: for<'lua> mlua::ToLuaMulti<'lua>,
    {
        let lua_func = self.lua.create_function(move |_, args| Ok(func(args)))?;
        self.lua.globals().set(name, lua_func)
    }
}