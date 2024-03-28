use crate::parser::Type;
use crate::parser::TypeDefinition;
use crate::parser::Param;
use crate::parser::Definition;

fn normalize(name: &String) -> String {
    name.replace(".", "_")
}

fn compile_type(r#type: &TypeDefinition) -> String {
    match r#type.r#type {
        Type::INT => "u32".into(),
        Type::INT128 => "u128".into(),
        Type::LONG => "u64".into(),
        Type::BYTES => "&'static [u8]".into(),
        Type::STRING => "String".into(),
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
    code += "pub hash: u32,\n";

    for param in &definition.params {
        code += &format!("pub {}: {},\n", param.name, compile_type(&param.r#type));
    }

    code += "}";

    code
}

fn compile_single_read(r#type: &TypeDefinition) -> String {
    match r#type.r#type {
        Type::LONG => "data.read_u64()".into(),
        Type::INT => "data.read_u32()".into(),
        Type::INT128 => "data.read_u128()".into(),
        Type::BYTES => "data.read_bytes()".into(),
        Type::STRING => "data.read_string()".into(),
        Type::VECTOR => format!(r#"{{
    let mut vector_data = vec![];
    let vector_header = data.read_u32();
    let mut length = 0;
    if vector_header == 0x1cb5c415 {{
        length = data.read_u32();
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
    fn hash(&self) -> u32 {{
        self.hash
    }}
}}"#, normalize(&definition.predicate))
}

pub fn compile_reader(definition: &Definition) -> String {
    let mut code = String::new();

    code += &format!("pub fn read_{}(data: &mut BytesBuffer) -> Option<Box<dyn TlObject>> {{\n", normalize(&definition.predicate));
    code += &format!("let mut obj = {}::default();\n", normalize(&definition.predicate));

    code += &format!("obj.hash = {:#08x};\n", definition.id);

    for param in &definition.params {
        code += &format!("obj.{} = {};\n", param.name, compile_single_read(&param.r#type));
    }

    code += "Some(Box::new(obj))\n";
    code += "}";

    code
}
