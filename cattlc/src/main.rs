mod tokenizer;
mod compiler;
mod parser;

use crate::tokenizer::tokenize;
use crate::parser::parse;
use crate::compiler::compile_reader;
use crate::compiler::compile_struct;
use crate::compiler::compile_tlobject_impl;
use crate::compiler::compile_initializer;

fn main() -> std::io::Result<()> {
    let tokens = tokenize(std::fs::read_to_string("schema.tl")?);
    let definitions = parse(tokens);
    let mut code = String::new();

    code += "// This is a file generated by cattlc, do not modify unless you know what you are doing.\n\n";
    code += "use core::any::Any;\n";
    code += "use crate::tl_object::TlObject;\n";
    code += "use crate::tl_object::add_reader;\n";
    code += "use crate::bytes_buffer::BytesBuffer;\n\n";
    for definition in &definitions {
        code += &compile_struct(definition);
        code += "\n";
        code += &compile_tlobject_impl(definition);
        code += "\n";
        code += &compile_reader(definition);
        code += "\n\n";
    }

    code += &compile_initializer(&definitions);
    code += "\n";

    std::fs::write("schema.rs", code)?;
    Ok(())
}
