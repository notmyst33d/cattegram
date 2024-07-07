use std::{
    env::var,
    fs::{read_to_string, write},
};

use catte_tl_compiler::{compiler::compile_schema, parser::parse, tokenizer::tokenize};

fn main() {
    println!("cargo:rerun-if-changed=../mtproto.tl");
    println!("cargo:rerun-if-changed=../api.tl");

    let mut schema = vec![];
    schema.extend(parse(tokenize(read_to_string("../mtproto.tl").unwrap())));
    schema.extend(parse(tokenize(read_to_string("../api.tl").unwrap())));

    write(
        format!("{}/generated_schema.rs", var("OUT_DIR").unwrap()),
        compile_schema(&schema),
    )
    .unwrap();
}
