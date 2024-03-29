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
        Type::BYTES => "&'static [u8]".into(),
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

    code += "#[derive(Debug, Default)]\n";
    code += &format!("pub struct {} {{\n", normalize(&definition.predicate));
    code += "pub hash: i32,\n";

    for param in &definition.params {
        code += &format!("pub {}: {},\n", param.name, compile_type(&param.r#type));
    }

    code += "}";

    code
}

fn compile_single_read(r#type: &TypeDefinition) -> String {
    match r#type.r#type {
        Type::LONG => "data.read_long()?".into(),
        Type::INT => "data.read_int()?".into(),
        Type::INT128 => "data.read_int128()?".into(),
        Type::BYTES => "data.read_bytes()?".into(),
        Type::VECTOR => format!(r#"{{
    let mut vector_data = vec![];
    let vector_header = data.read_int()?;
    let mut length = 0;
    if vector_header == 0x1cb5c415 {{
        length = data.read_int()?;
    }} else {{
        length = vector_header;
    }}

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
        self.hash
    }}
}}"#, normalize(&definition.predicate))
}

pub fn compile_reader(definition: &Definition) -> String {
    let mut code = String::new();

    code += &format!("pub fn read_raw_{}(data: &mut BytesBuffer) -> Option<{0}> {{\n", normalize(&definition.predicate));
    code += &format!("let mut obj = {}::default();\n", normalize(&definition.predicate));

    code += &format!("obj.hash = {};\n", definition.id);

    for param in &definition.params {
        code += &format!("obj.{} = {};\n", param.name, compile_single_read(&param.r#type));
    }

    code += "Some(obj)\n";
    code += "}\n";

    code += &format!("pub fn read_{}(data: &mut BytesBuffer) -> Option<Box<dyn Any>> {{\n", normalize(&definition.predicate));
    code += &format!("Some(Box::new(read_raw_{}(data)?))\n", normalize(&definition.predicate));
    code += "}";
    code
}

pub fn compile_initializer(definitions: &Vec<Definition>) -> String {
    let mut code = String::new();

    code += "pub fn init() {\n";
    for definition in definitions {
        code += &format!("add_reader({}, read_{});\n", definition.id, normalize(&definition.predicate));
    }
    code += "}";

    code
}
