use {
    mlua::Compiler,
    anyhow::Error,
    cutedogs::document,
};

#[document(summary = "Gets the compiler", returns = "The compiler")]
pub fn get_compiler() -> Compiler 
{
    return Compiler::new()
        .set_debug_level(1)
        .set_optimization_level(1);
}

#[document(summary = "Compiles a code", params = {
    "code": "The code to compile"
}, returns = "The compiled code")]
pub fn compile(code: &str) -> Result<Vec<u8>, Error> {
    get_compiler().compile(code).map_err(|e| Error::from(e))
}