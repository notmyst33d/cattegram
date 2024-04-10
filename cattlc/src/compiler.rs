use crate::parser::Type;
use crate::parser::TypeDefinition;
use crate::parser::Definition;

fn normalize(name: &String) -> String {
    name.replace(".", "_")
}

fn compile_type(r#type: &TypeDefinition) -> String {
    match r#type.r#type {
        Type::INT => "i32".into(),
        Type::INT128 => "i128".into(),
        Type::LONG => "i64".into(),
        Type::BYTES | Type::INT256 => "&'static [u8]".into(),
        Type::OBJECT => "Box<dyn TlObject>".into(),
        Type::SCHEMA => normalize(r#type.schema_type.as_ref().unwrap()),
        Type::VECTOR => {
            let inner = compile_type(&r#type.inner.as_ref().unwrap());
            format!("Vec<{}>", inner)
        },
        _ => "/* ERROR: Compilation failed */".into(),
    }
}

pub fn compile_struct(definition: &Definition) -> String {
    let mut code = String::new();

    code += "#[allow(non_camel_case_types)]\n";
    code += "#[derive(Debug, Default)]\n";
    code += &format!("pub struct {} {{\n", normalize(&definition.predicate));

    for param in &definition.params {
        code += &format!("pub {}: {},\n", param.name, compile_type(&param.r#type));
    }

    code += "}";

    code
}

fn compile_single_write(r#type: &TypeDefinition, name: &str) -> String {
    match r#type.r#type {
        Type::LONG => format!("data.write_long({})", name),
        Type::INT => format!("data.write_int({})", name),
        Type::INT128 => format!("data.write_int128({})", name),
        Type::INT256 => format!("data.write_raw({})", name),
        Type::BYTES => format!("data.write_bytes({})", name),
        Type::VECTOR => format!(r#"{{
    data.write_int(0x1cb5c415);
    data.write_int({0}.len() as i32);
    for element in &{0} {{
        {1};
    }}
}}"#, name, compile_single_write(&r#type.inner.as_ref().unwrap(), "*element")),
        Type::SCHEMA => format!("{}.write(data)", name),
        _ => "/* ERROR: Compilation failed */".into(),
    }
}

fn compile_single_read(r#type: &TypeDefinition) -> String {
    match r#type.r#type {
        Type::LONG => "data.read_long()?".into(),
        Type::INT => "data.read_int()?".into(),
        Type::INT128 => "data.read_int128()?".into(),
        Type::INT256 => "data.read_raw(32)?".into(),
        Type::BYTES => "data.read_bytes()?".into(),
        Type::VECTOR => format!(r#"{{
    let mut vector_data = vec![];
    let vector_header = data.read_int()?;
    let length = if vector_header == 0x1cb5c415 {{
        data.read_int()?
    }} else {{
        vector_header
    }};

    for _ in 0..length {{
        let value = {};
        vector_data.push(value);
    }}

    vector_data
}}"#, compile_single_read(&r#type.inner.as_ref().unwrap())),
        Type::SCHEMA => format!("read_{}(data)", r#type.schema_type.as_ref().unwrap()),
        _ => "/* ERROR: Compilation failed */".into(),
    }
}

pub fn compile_tlobject_impl(definition: &Definition) -> String {
    format!(r#"impl TlObject for {} {{
    fn hash(&self) -> i32 {{
        {}
    }}
    fn write(&self, data: &mut BytesBuffer) {{
        {}
    }}
}}"#, normalize(&definition.predicate), definition.id, compile_writer(definition))
}

pub fn compile_writer(definition: &Definition) -> String {
    let mut code = String::new();
    code += &format!("data.write_int({});\n", definition.id);
    for param in &definition.params {
        code += &format!("{};\n", compile_single_write(&param.r#type, &format!("self.{}", param.name)));
    }
    code
}

pub fn compile_reader(definition: &Definition) -> String {
    let mut code = String::new();

    code += "#[allow(non_snake_case)]\n";
    code += &format!("pub fn read_{}(data: &mut BytesBuffer) -> Result<{0}, TlError> {{\n", normalize(&definition.predicate));
    code += &format!("let mut obj = {}::default();\n", normalize(&definition.predicate));

    for param in &definition.params {
        code += &format!("obj.{} = {};\n", param.name, compile_single_read(&param.r#type));
    }

    code += "Ok(obj)\n";
    code += "}\n";
    code
}

pub fn compile_extend_reader(definitions: &Vec<Definition>) -> String {
    let mut code = String::new();

    code += "pub fn extend_reader(reader: &mut TlReader) {\n";
    for definition in definitions {
        code += &format!("reader.add_reader({}, |data| read_{}(data).map(|o| Box::new(o) as Box<dyn TlObject>));\n", definition.id, normalize(&definition.predicate));
    }
    code += "}";

    code
}
