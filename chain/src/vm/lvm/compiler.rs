use {
    mlua::Compiler,
    anyhow::Error,
};

pub fn get_compiler() -> Compiler 
{
    return Compiler::new()
        .set_debug_level(1)
        .set_optimization_level(1);
}

pub fn compile(code: &str) -> Result<Vec<u8>, Error> {
    get_compiler().compile(code).map_err(|e| Error::from(e))
}