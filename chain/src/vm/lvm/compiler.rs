use {
    mlua::Compiler,
    anyhow::Error,
    cutedogs::document,
};

#[document(summary = "Gets the compiler", returns = "The compiler")]
pub fn get_compiler(debug_level: Option<u8>, optimization_level: Option<u8>, type_info_level: Option<u8>) -> Compiler 
{
    return Compiler::new()
        .set_debug_level(debug_level.unwrap_or(1))
        .set_optimization_level(optimization_level.unwrap_or(1))
        .set_type_info_level(type_info_level.unwrap_or(2));
}

#[document(summary = "Compiles a code", params = {
    "code": "The code to compile"
}, returns = "The compiled code")]
pub fn compile(code: &str, debug_level: Option<u8>, optimization_level: Option<u8>, type_info_level: Option<u8>) -> Result<Vec<u8>, Error> {
    get_compiler(debug_level, optimization_level, type_info_level).compile(code).map_err(Error::from)
}