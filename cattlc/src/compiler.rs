use crate::parser::Type;
use crate::parser::Definition;

pub fn compile_reader(definition: &Definition) -> String {
    let mut code = String::new();

    code += &format!("int read_{}(cattl_object *obj, cattl_reader *reader) {{\n", definition.predicate.replace(".", "_"));

    for param in &definition.params {
        match param.r#type.r#type {
            Type::INT128 => {
                code += &format!("cattl_put(obj, \"{}\", cattl_read_int128(reader));\n", param.name);
            },
            _ => {
                code += &format!("// WARN: Compilation failed for param \"{}\" with type \"{:?}\" (schema type: {:?})\n", param.name, param.r#type.r#type, param.r#type.schema_type);
                code += "// Dump:\n";

                let data = format!("{:#?}", param);
                code += &data.split("\n").into_iter().map(|v| format!("// {}", v)).collect::<Vec<String>>().join("\n");
                code += "\n";
            },
        }
    }

    code += "return 0;\n";
    code += "}";

    code
}
