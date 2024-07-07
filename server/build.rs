use std::{
    env::var,
    fs::{read_to_string, write},
};

use catte_tl_compiler::{
    compiler::{rustify_function_name, rustify_struct_name},
    parser::{parse, Definition},
    tokenizer::tokenize,
};

fn generate_invoke_code(definitions: &Vec<Definition>) -> String {
    format!(
        r#"use std::sync::Arc;
use tokio::sync::Mutex;
use std::error::Error;
use catte_tl_schema::SchemaObject;
use crate::Session;

pub struct Message<T> {{
    pub msg_id: i64,
    pub seq_no: i32,
    pub obj: T,
}}

pub async fn invoke(
    session: Arc<Mutex<Session>>,
    request: (i64, i32, SchemaObject),
) -> Result<SchemaObject, Box<dyn Error + Send + Sync>> {{
    match request.2 {{
        {}
        _ => Err("no rpc function".into())
    }}
}}"#,
        definitions
            .iter()
            .filter(|d| d.function)
            .map(|d| format!(
                "SchemaObject::{}(obj) => Ok(rpc_{}(session, Message {{ msg_id: request.0, seq_no: request.1, obj }}).await?),\n",
                rustify_struct_name(&d.predicate),
                rustify_function_name(&d.predicate),
            ))
            .collect::<String>()
    )
}

fn main() {
    println!("cargo:rerun-if-changed=../mtproto.tl");
    println!("cargo:rerun-if-changed=../api.tl");

    let mut schema = vec![];
    schema.extend(parse(tokenize(read_to_string("../mtproto.tl").unwrap())));
    schema.extend(parse(tokenize(read_to_string("../api.tl").unwrap())));

    write(
        format!("{}/generated_invoke.rs", var("OUT_DIR").unwrap()),
        generate_invoke_code(&schema),
    )
    .unwrap();
}
