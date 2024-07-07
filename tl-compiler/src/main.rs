use std::fs::{read_to_string, write};

use catte_tl_compiler::{compiler::compile_schema, parser::parse, tokenizer::tokenize};

fn main() {
    let mut schema = vec![];
    schema.extend(parse(tokenize(read_to_string("../raw.tl").unwrap())));
    schema.extend(parse(tokenize(read_to_string("../base.tl").unwrap())));
    write("generated_schema.rs", compile_schema(&schema)).unwrap();
}
