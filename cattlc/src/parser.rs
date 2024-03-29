#[derive(Debug, PartialEq)]
pub enum Type {
    INT,
    INT128,
    INT256,
    LONG,
    DOUBLE,
    BYTES,
    VECTOR,
    STRING,
    OBJECT,
    SCHEMA,
}

#[derive(Debug)]
pub struct TypeDefinition {
    pub r#type: Type,
    pub inner: Option<Box<TypeDefinition>>,
    pub schema_type: Option<String>,
}

#[derive(Debug)]
pub struct Param {
    pub r#type: TypeDefinition,
    pub name: String,
}

#[derive(Debug, Default)]
pub struct Definition {
    pub id: i32,
    pub predicate: String,
    pub params: Vec<Param>
}

pub fn parse(source: Vec<String>) -> Vec<Definition> {
    let mut definition = Definition::default();
    let mut definitions = vec![];
    let mut i = 0;

    let resolve_type = |t| {
        match t {
            "int" => Type::INT,
            "int128" => Type::INT128,
            "long" => Type::LONG,
            "bytes" => Type::BYTES,
            "Vector" | "vector" => Type::VECTOR,
            _ => Type::SCHEMA,
        }
    };

    while i < source.len() {
        let token = &source[i];
        match token.as_str() {
            ";" => {
                definitions.push(definition);
                definition = Definition::default();
            },
            ":" => {
                let name = &source[i - 1];
                let r#type = &source[i + 1];
                i += 1;

                let mut typedef = TypeDefinition {
                    r#type: resolve_type(r#type.as_str()),
                    inner: None,
                    schema_type: None,
                };

                if typedef.r#type == Type::SCHEMA {
                    typedef.schema_type = Some(r#type.clone());
                }

                if typedef.r#type == Type::VECTOR {
                    typedef.inner = Some(Box::new(TypeDefinition {
                        r#type: resolve_type(&source[i + 2]),
                        inner: None,
                        schema_type: None,
                    }));
                    if let Some(ref mut inner) = typedef.inner {
                        if inner.r#type == Type::SCHEMA {
                            inner.schema_type = Some(source[i + 2].clone());
                        }
                    }
                    i += 3;
                }

                let param = Param {
                    name: name.to_string(),
                    r#type: typedef,
                };

                definition.params.push(param);
            },
            "#" => {
                definition.predicate = source[i - 1].clone();
                definition.id = u32::from_str_radix(&source[i + 1], 16).unwrap() as i32;
                i += 1;
            },
            &_ => {},
        }
        i += 1;
    }

    definitions
}
