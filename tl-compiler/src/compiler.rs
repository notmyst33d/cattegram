use convert_case::Case;
use convert_case::Casing;

use crate::parser::analyze_variant_constructors;
use crate::parser::Definition;
use crate::parser::Type;
use crate::parser::TypeDefinition;

pub fn rustify_struct_name(name: &String) -> String {
    name.replace(".", "_").to_case(Case::UpperCamel)
}

pub fn rustify_function_name(name: &String) -> String {
    name.replace(".", "_").to_case(Case::Snake)
}

pub fn rustify_variable_name(name: &String) -> String {
    filtered_name(name.to_lowercase())
}

pub fn filtered_name(name: String) -> String {
    match name.as_str() {
        "static" => "is_static".into(),
        "self" => "is_self".into(),
        "final" => "is_final".into(),
        "loop" => "is_loop".into(),
        "type" => "r#type".into(),
        _ => name,
    }
}

fn compile_type(definitions: &Vec<Definition>, r#type: &TypeDefinition) -> String {
    let t = match r#type.r#type {
        Type::INT | Type::FLAGS => "i32".into(),
        Type::INT128 => "i128".into(),
        Type::LONG => "i64".into(),
        Type::DOUBLE => "f64".into(),
        Type::BYTES | Type::INT256 => "Vec<u8>".into(),
        Type::STRING => "String".into(),
        Type::OBJECT => "Box<SchemaObject>".into(),
        Type::SCHEMA => {
            if r#type.variant {
                format!(
                    "{}Variant",
                    rustify_struct_name(&r#type.schema_type.clone().unwrap())
                )
            } else {
                if let Some(constructor) =
                    find_return_type_constructor(definitions, r#type.schema_type.clone().unwrap())
                {
                    rustify_struct_name(&constructor.predicate)
                } else {
                    rustify_struct_name(&r#type.schema_type.clone().unwrap())
                }
            }
        }
        Type::BOOL | Type::BitBool => "bool".into(),
        Type::VECTOR => {
            let inner = compile_type(definitions, &r#type.inner.as_ref().unwrap());
            format!("Vec<{}>", inner)
        }
        _ => "/* ERROR: Compilation failed */".into(),
    };
    if r#type.flags_name.is_some() && r#type.r#type != Type::BitBool {
        format!("Option<{}>", t)
    } else {
        t
    }
}

pub fn compile_struct(definitions: &Vec<Definition>, definition: &Definition) -> String {
    let mut code = String::new();

    code += "#[allow(dead_code, unused_variables)]\n";
    code += "#[derive(Debug, Clone)]\n";
    code += &format!(
        "pub struct {} {{\n",
        rustify_struct_name(&definition.predicate)
    );

    for param in &definition.params {
        if param.r#type.r#type == Type::FLAGS {
            continue;
        }
        code += &format!(
            "pub {}: {},\n",
            rustify_variable_name(&param.name.clone()),
            compile_type(definitions, &param.r#type)
        );
    }

    code += "}\n";

    code += &format!("impl {} {{\n", rustify_struct_name(&definition.predicate));
    code += &compile_writer(definition);
    code += "}";

    code
}

fn compile_single_write(
    r#type: &TypeDefinition,
    name: &str,
    prefix: &str,
    inner: bool,
    borrow: bool,
) -> String {
    if r#type.flags_name.is_some() && !inner {
        if r#type.r#type == Type::BYTES
            || r#type.r#type == Type::STRING
            || r#type.r#type == Type::SCHEMA
            || r#type.r#type == Type::VECTOR
        {
            return format!(
                "if let Some({0}_inner) = &{1}{0} {{ {2} }}",
                name,
                prefix,
                compile_single_write(r#type, &format!("{}_inner", name), "", true, false)
            );
        } else {
            return format!(
                "if let Some({0}_inner) = {1}{0} {{ {2} }}",
                name,
                prefix,
                compile_single_write(r#type, &format!("{}_inner", name), "", true, true)
            );
        }
    }
    match r#type.r#type {
        Type::LONG => format!("bytes_buffer.write_long({}{})", prefix, name),
        Type::DOUBLE => format!("bytes_buffer.write_double({}{})", prefix, name),
        Type::INT | Type::FLAGS => format!("bytes_buffer.write_int({}{})", prefix, name),
        Type::BOOL => format!("bytes_buffer.write_bool({}{})", prefix, name),
        Type::INT128 => format!("bytes_buffer.write_int128({}{})", prefix, name),
        Type::INT256 => format!("bytes_buffer.write_raw({}{}{})", if borrow { "&" } else { "" }, prefix, name),
        Type::BYTES => format!("bytes_buffer.write_bytes({}{}{})", if borrow { "&" } else { "" }, prefix, name),
        Type::STRING => format!("bytes_buffer.write_string({}{}{})", if borrow { "&" } else { "" }, prefix, name),
        Type::VECTOR => format!("bytes_buffer.write_int(0x1cb5c415);\nbytes_buffer.write_int({1}{0}.len() as i32);\n{1}{0}.iter().for_each(|v| {2})",
            name,
            prefix,
            compile_single_write(&r#type.inner.as_ref().unwrap(), if r#type.inner.as_ref().unwrap().r#type == Type::LONG || r#type.inner.as_ref().unwrap().r#type == Type::INT { "*v" } else { "v" }, "", false, true)
        ),
        Type::OBJECT | Type::SCHEMA => format!("{}{}.write(bytes_buffer)", prefix, name),
        _ => "/* ERROR: Compilation failed */".into(),
    }
}

fn compile_single_read(
    definitions: &Vec<Definition>,
    r#type: &TypeDefinition,
    propagate_error: bool,
    inside_vector: bool,
) -> String {
    let p = if inside_vector { "" } else { "?" };
    let t = match r#type.r#type {
        Type::LONG => "bytes_buffer.read_long()".into(),
        Type::DOUBLE => "bytes_buffer.read_double()".into(),
        Type::INT | Type::FLAGS => "bytes_buffer.read_int()".into(),
        Type::BOOL => "bytes_buffer.read_bool()".into(),
        Type::INT128 => "bytes_buffer.read_int128()".into(),
        Type::INT256 => "bytes_buffer.read_raw(32)".into(),
        Type::BYTES => "bytes_buffer.read_bytes()".into(),
        Type::STRING => "bytes_buffer.read_string()".into(),
        Type::VECTOR => format!(
            r#"{{
    let vector_header = bytes_buffer.read_int()?;
    let length = if vector_header == 0x1cb5c415 {{
        bytes_buffer.read_int()?
    }} else {{
        vector_header
    }};

    (0..length).into_iter().map(|_| {}).collect::<Result<Vec<_>, TlBufferError>>()?
}}"#,
            compile_single_read(definitions, &r#type.inner.as_ref().unwrap(), false, true)
        ),
        Type::OBJECT => "Box::new(read(bytes_buffer)?)".into(),
        Type::SCHEMA => {
            let inner = if r#type.variant {
                format!(
                    "read_{}_variant(bytes_buffer){}",
                    rustify_function_name(&r#type.schema_type.clone().unwrap()),
                    p
                )
            } else {
                if let Some(constructor) =
                    find_return_type_constructor(definitions, r#type.schema_type.clone().unwrap())
                {
                    format!(
                        "read_{}(bytes_buffer){}",
                        rustify_function_name(&constructor.predicate),
                        p
                    )
                } else {
                    format!(
                        "read_{}(bytes_buffer){}",
                        rustify_function_name(&r#type.schema_type.clone().unwrap()),
                        p
                    )
                }
            };
            if r#type.variant || r#type.raw {
                inner
            } else {
                format!("{{ bytes_buffer.read_int()?; {} }}", inner)
            }
        }
        _ => "/* ERROR: Compilation failed */".into(),
    };
    if propagate_error
        && r#type.r#type != Type::VECTOR
        && r#type.r#type != Type::OBJECT
        && r#type.r#type != Type::SCHEMA
    {
        t + "?"
    } else {
        t
    }
}

pub fn compile_writer(definition: &Definition) -> String {
    let mut code = String::new();
    code += "pub fn write(&self, bytes_buffer: &mut TlBuffer) {\n";
    if definition.id != 0 && definition.id != 1 {
        code += &format!("bytes_buffer.write_int({});\n", definition.id);
    }

    code += &definition
        .params
        .iter()
        .filter(|p| p.r#type.r#type == Type::FLAGS)
        .map(|fp| {
            format!(
                "#[allow(unused_mut)]\nlet mut {}: i32 = 0;\n",
                rustify_variable_name(&fp.name.clone())
            ) + &definition
                .params
                .iter()
                .filter(|p| p.r#type.flags_name == Some(fp.name.clone()))
                .map(|p| {
                    format!(
                        "if self.{}{} {{ {} ^= 1 << {} }}\n",
                        rustify_variable_name(&p.name.clone()),
                        if p.r#type.r#type == Type::BitBool {
                            ""
                        } else {
                            ".is_some()"
                        },
                        rustify_variable_name(&p.r#type.flags_name.clone().unwrap()),
                        p.r#type.flags_bit.clone().unwrap()
                    )
                })
                .collect::<String>()
        })
        .collect::<String>();

    code += &definition
        .params
        .iter()
        .map(|param| {
            if param.r#type.r#type == Type::BitBool {
                "".into()
            } else {
                format!(
                    "{};\n",
                    compile_single_write(
                        &param.r#type,
                        &rustify_variable_name(&param.name.clone()),
                        if param.r#type.r#type == Type::FLAGS {
                            ""
                        } else {
                            "self."
                        },
                        false,
                        true,
                    )
                )
            }
        })
        .collect::<String>();

    code += "}\n";
    code
}

pub fn compile_variant_reader(variant_name: String, definitions: &Vec<Definition>) -> String {
    let mut code = String::new();
    code += &format!(
        "pub fn read_{}_variant(data: &mut TlBuffer) -> Result<{}Variant, TlBufferError> {{\n",
        rustify_function_name(&variant_name),
        rustify_struct_name(&variant_name),
    );
    code += "match data.read_int()? {\n";
    code += &definitions
        .iter()
        .map(|d| {
            format!(
                "{} => Ok({}Variant::{}(Box::new(read_{}(data)?))),\n",
                d.id,
                rustify_struct_name(&variant_name),
                rustify_struct_name(&d.predicate),
                rustify_function_name(&d.predicate),
            )
        })
        .collect::<String>();
    code += "i => Err(TlBufferError::Custom(format!(\"no reader for {}\", i))),\n";
    code += "}\n";
    code += "}";
    code
}

pub fn compile_variant_enum(variant_name: String, definitions: &Vec<Definition>) -> String {
    let mut code = String::new();
    code += "#[derive(Debug, Clone)]\n";
    code += &format!(
        "pub enum {}Variant {{\n",
        rustify_struct_name(&variant_name)
    );
    code += &definitions
        .iter()
        .map(|d| {
            format!(
                "{}(Box<{}>),\n",
                rustify_struct_name(&d.predicate),
                rustify_struct_name(&d.predicate),
            )
        })
        .collect::<String>();
    code += "}\n\n";

    code += &format!("impl {}Variant {{\n", rustify_struct_name(&variant_name));
    code += "   pub fn write(&self, data: &mut TlBuffer) {\n";
    code += "       match self {\n";

    code += &definitions
        .iter()
        .map(|d| {
            format!(
                "{}Variant::{}(inner_obj) => inner_obj.write(data),\n",
                rustify_struct_name(&variant_name),
                rustify_struct_name(&d.predicate),
            )
        })
        .collect::<String>();

    code += "       }\n";
    code += "   }\n";
    code += "}\n";

    code
}

pub fn compile_reader(definitions: &Vec<Definition>, definition: &Definition) -> String {
    let mut code = String::new();

    code += "#[allow(dead_code, unused_variables)]\n";
    code += &format!(
        "pub fn read_{}(bytes_buffer: &mut TlBuffer) -> Result<{}, TlBufferError> {{\n",
        rustify_function_name(&definition.predicate),
        rustify_struct_name(&definition.predicate),
    );

    for param in &definition.params {
        if let (Some(flags_name), Some(flags_bit)) =
            (param.r#type.flags_name.clone(), param.r#type.flags_bit)
        {
            if param.r#type.r#type == Type::BitBool {
                code += &format!(
                    "let {} = if {} & (1 << {}) != 0 {{ true }} else {{ false }};\n",
                    rustify_variable_name(&param.name.clone()),
                    rustify_variable_name(&flags_name.clone()),
                    flags_bit
                );
            } else {
                code += &format!(
                    "let {} = if {} & (1 << {}) != 0 {{ ",
                    rustify_variable_name(&param.name.clone()),
                    rustify_variable_name(&flags_name.clone()),
                    flags_bit
                );
                code += "Some(";
                code += &compile_single_read(definitions, &param.r#type, true, false);
                code += ") } else { None };\n";
            }
        } else {
            code += &format!(
                "let {} = {};\n",
                rustify_variable_name(&param.name.clone()),
                compile_single_read(definitions, &param.r#type, true, false)
            );
        }
    }

    code += &format!("Ok({} {{\n", rustify_struct_name(&definition.predicate));
    for param in &definition.params {
        if param.r#type.r#type == Type::FLAGS {
            continue;
        }
        code += &format!("{},\n", rustify_variable_name(&param.name.clone()));
    }
    code += "})\n";
    code += "}\n";
    code
}

pub fn compile_schema_object(definitions: &Vec<Definition>) -> String {
    let mut code = String::new();
    let variants = analyze_variant_constructors(definitions);

    code += "#[derive(Debug, Clone)]\n";
    code += "pub enum SchemaObject {\n";
    code += &definitions
        .iter()
        .map(|d| format!("{}({0}),\n", rustify_struct_name(&d.predicate)))
        .collect::<String>();
    code += &variants
        .iter()
        .map(|(k, _)| format!("{}Variant({0}Variant),\n", rustify_struct_name(k)))
        .collect::<String>();
    code += "Vector(Vec<SchemaObject>),\n";
    code += "DeserializationError(TlBufferError),\n";
    code += "RawVector(Vec<SchemaObject>),\n";
    code += "MsgContainer(Vec<(i64, i32, SchemaObject)>),\n";
    code += "}\n\n";

    code += "impl SchemaObject {\n";
    code += "   pub fn write(&self, data: &mut TlBuffer) {\n";
    code += "       match self {\n";
    code += &definitions
        .iter()
        .map(|d| {
            format!(
                "SchemaObject::{}(inner_obj) => inner_obj.write(data),\n",
                rustify_struct_name(&d.predicate),
            )
        })
        .collect::<String>();
    code += &variants
        .iter()
        .map(|(k, _)| {
            format!(
                "SchemaObject::{}Variant(inner_obj) => inner_obj.write(data),\n",
                rustify_struct_name(k),
            )
        })
        .collect::<String>();
    code += "SchemaObject::Vector(inner_obj) => {
        data.write_int(0x1cb5c415);
        data.write_int(inner_obj.len() as i32);
        inner_obj.iter().for_each(|v| v.write(data));
    },\n";
    code += "SchemaObject::RawVector(inner_obj) => {
        data.write_int(inner_obj.len() as i32);
        inner_obj.iter().for_each(|v| v.write(data));
    },\n";
    code += "SchemaObject::MsgContainer(inner_obj) => {
        data.write_int(1945237724);
        data.write_int(inner_obj.len() as i32);
        inner_obj
            .iter()
            .for_each(|v| {
                data.write_long(v.0);
                data.write_int(v.1);
                let mut tmp = TlBuffer::new(vec![]);
                v.2.write(&mut tmp);
                data.write_int(tmp.len() as i32);
                data.write_raw(tmp.data());
            });
    },\n";
    code += "SchemaObject::DeserializationError(_) => panic!(\"This object is not serializable\")";
    code += "       }\n";
    code += "   }\n";
    code += "}";

    code
}

pub fn compile_schema(definitions: &Vec<Definition>) -> String {
    let mut code = "// This is a file generated by catte-tl-compiler, do not modify unless you know what you are doing.\n\n".to_string();
    code += "use catte_tl_buffer::{TlBuffer, TlBufferError};\n";
    code += definitions
        .iter()
        .map(|d| {
            [
                compile_struct(definitions, d),
                if d.id != 1 {
                    compile_reader(definitions, d)
                } else {
                    "".to_string()
                },
            ]
            .join("\n\n")
        })
        .collect::<Vec<String>>()
        .join("\n")
        .as_ref();

    code += "\n";

    let variants = analyze_variant_constructors(&definitions);
    code += &variants
        .iter()
        .map(|(k, v)| compile_variant_enum(k.clone(), v))
        .collect::<Vec<String>>()
        .join("\n\n");

    code += "\n\n";

    code += &variants
        .iter()
        .map(|(k, v)| compile_variant_reader(k.clone(), v))
        .collect::<Vec<String>>()
        .join("\n\n");

    code += "\n\n";
    code += &compile_schema_object(&definitions);
    code += "\n\n";

    code += &format!(
        r#"pub fn read(data: &mut TlBuffer) -> Result<SchemaObject, TlBufferError> {{
    match data.read_int()? {{
{}
        1945237724 => {{
            Ok(SchemaObject::MsgContainer(
                (0..data.read_int()?)
                    .into_iter()
                    .map(|_| -> Result<(i64, i32, SchemaObject), TlBufferError> {{
                        let msg_id = data.read_long()?;
                        let seq_no = data.read_int()?;
                        let _length = data.read_int()?;
                        let obj = match read(data) {{
                            Ok(result) => result,
                            Err(e) => SchemaObject::RpcResult(RpcResult {{
                                req_msg_id: msg_id,
                                result: Box::new(SchemaObject::RpcError(RpcError {{
                                    error_code: 0,
                                    error_message: format!("deserialization error: {{}}", e),
                                }})),
                            }}),
                        }};
                        Ok((msg_id, seq_no, obj))
                    }})
                    .collect::<Result<Vec<_>, _>>()?
            ))
        }},
        i => Err(TlBufferError::Custom(format!("no reader for {{}}", i))),
    }}
}}"#,
        definitions
            .iter()
            .map(|d| if d.id != 0 && d.id != 1 {
                format!(
                    "{} => Ok(SchemaObject::{}(read_{}(data)?)),\n",
                    d.id,
                    rustify_struct_name(&d.predicate),
                    rustify_function_name(&d.predicate)
                )
            } else {
                "".into()
            })
            .collect::<String>()
    );

    code
}

pub fn is_variant(definitions: &Vec<Definition>, schema_type: String) -> bool {
    analyze_variant_constructors(definitions)
        .iter()
        .any(|(k, _)| k.clone() == schema_type.clone())
}

pub fn find_return_type_constructor(
    definitions: &Vec<Definition>,
    schema_type: String,
) -> Option<Definition> {
    definitions
        .iter()
        .filter(|d| d.return_type.schema_type == Some(schema_type.clone()))
        .next()
        .cloned()
}
