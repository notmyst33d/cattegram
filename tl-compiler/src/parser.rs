use std::collections::HashMap;

#[derive(Debug, Clone, Default, PartialEq)]
pub enum Type {
    #[default]
    UNDEFINED,
    INT,
    INT128,
    INT256,
    BOOL,
    BitBool,
    LONG,
    BYTES,
    DOUBLE,
    VECTOR,
    STRING,
    FLAGS,
    OBJECT,
    SCHEMA,
}

#[derive(Debug, Clone, Default)]
pub struct TypeDefinition {
    pub r#type: Type,
    pub inner: Option<Box<TypeDefinition>>,
    pub schema_type: Option<String>,
    pub flags_name: Option<String>,
    pub flags_bit: Option<u8>,
    pub variant: bool,
    pub raw: bool,
}

#[derive(Debug, Clone, Default)]
pub struct Param {
    pub r#type: TypeDefinition,
    pub name: String,
}

#[derive(Debug, Clone, Default)]
pub struct Definition {
    pub id: i32,
    pub predicate: String,
    pub params: Vec<Param>,
    pub return_type: TypeDefinition,
    pub function: bool,
}

pub fn parse(source: Vec<String>) -> Vec<Definition> {
    let mut definition = Definition::default();
    let mut definitions = vec![];
    let mut i = 0;
    let mut functions: bool = false;

    let resolve_type = |t: String| match t.as_str() {
        "int" => Type::INT,
        "int128" => Type::INT128,
        "int256" => Type::INT256,
        "long" => Type::LONG,
        "bytes" => Type::BYTES,
        "double" => Type::DOUBLE,
        "string" => Type::STRING,
        "bool" | "Bool" => Type::BOOL,
        "true" => Type::BitBool,
        "#" => Type::FLAGS,
        "Vector" | "vector" => Type::VECTOR,
        "Object" | "X" | "!X" => Type::OBJECT,
        _ => Type::SCHEMA,
    };

    while i < source.len() {
        let token = &source[i];
        match token.as_str() {
            "---functions---" => functions = true,
            ";" => {
                definition.function = functions;
                definitions.push(definition);
                definition = Definition::default();
            }
            "=" => {
                let r#type = source[i + 1].clone();
                i += 1;

                let def_type = resolve_type(r#type.clone());
                match def_type {
                    Type::VECTOR => {
                        definition.return_type.inner = Some(Box::new(TypeDefinition {
                            r#type: resolve_type(source[i + 2].clone()),
                            inner: None,
                            schema_type: None,
                            flags_name: None,
                            flags_bit: None,
                            variant: false,
                            raw: false,
                        }));
                        i += 3;
                    }
                    Type::SCHEMA => {
                        definition.return_type.schema_type = Some(r#type);
                    }
                    _ => {}
                }
                definition.return_type.r#type = def_type;
            }
            ":" => {
                let name = source[i - 1].clone();
                let mut r#type = source[i + 1].clone();
                i += 1;

                let mut flags_name = None;
                let mut flags_bit = None;
                let split = r#type.split("?").collect::<Vec<&str>>();
                if split.len() != 1 {
                    let flags_data = split[0].split(".").collect::<Vec<&str>>();
                    flags_name = Some(flags_data[0].to_string());
                    flags_bit = Some(flags_data[1].to_string().parse::<u8>().unwrap());
                    r#type = split[1].to_string();
                }

                let mut typedef = TypeDefinition {
                    r#type: resolve_type(r#type.clone()),
                    inner: None,
                    schema_type: None,
                    flags_name: flags_name.clone(),
                    flags_bit,
                    variant: false,
                    raw: false,
                };

                if typedef.r#type == Type::SCHEMA {
                    typedef.schema_type = Some(r#type.clone());
                }

                if typedef.r#type == Type::VECTOR {
                    let mut inner_type = source[i + 2].clone();
                    let mut inner_raw = false;
                    if inner_type.starts_with("%") {
                        inner_type = inner_type[1..].to_string();
                        inner_raw = true;
                    }
                    typedef.inner = Some(Box::new(TypeDefinition {
                        r#type: resolve_type(inner_type.clone()),
                        inner: None,
                        schema_type: None,
                        flags_name: None,
                        flags_bit: None,
                        variant: false,
                        raw: inner_raw,
                    }));
                    if let Some(ref mut inner) = typedef.inner {
                        if inner.r#type == Type::SCHEMA {
                            inner.schema_type = Some(inner_type);
                        }
                    }
                    i += 3;
                }

                let param = Param {
                    name: name.to_string(),
                    r#type: typedef,
                };

                definition.params.push(param);
            }
            "#" => {
                definition.predicate = source[i - 1].clone();
                definition.id = u32::from_str_radix(&source[i + 1], 16).unwrap() as i32;
                i += 1;
            }
            "{" => {
                i += 5;
            }
            &_ => {}
        }
        i += 1;
    }

    let variant_constructors = analyze_variant_constructors(&definitions);
    definitions.iter_mut().for_each(|d| {
        d.params.iter_mut().for_each(|p| {
            p.r#type.schema_type.clone().map(|s| {
                if variant_constructors.contains_key(&s.clone()) {
                    p.r#type.variant = true;
                }
            });
            p.r#type.inner.as_mut().map(|t| {
                t.schema_type.as_mut().map(|s| {
                    if variant_constructors.contains_key(&s.clone()) {
                        t.variant = true;
                    }
                });
            });
        })
    });

    definitions
}

pub fn analyze_variant_constructors(
    definitions: &Vec<Definition>,
) -> HashMap<String, Vec<Definition>> {
    let mut counter: HashMap<String, u32> = HashMap::new();
    let mut variants: HashMap<String, Vec<Definition>> = HashMap::new();
    definitions.iter().for_each(|d| {
        if let Some(schema_type) = d.return_type.schema_type.clone() {
            if d.function {
                return;
            }
            if counter.contains_key(&schema_type) {
                let value = *counter.get(&schema_type).unwrap();
                counter.insert(schema_type.clone(), value + 1);
            } else {
                counter.insert(schema_type, 1);
            }
        }
    });
    counter.iter().for_each(|(k, v)| {
        if *v > 1 {
            variants.insert(k.clone(), vec![]);
            definitions.iter().for_each(|d| {
                if d.function {
                    return;
                }
                if Some(k.clone()) == d.return_type.schema_type {
                    variants.get_mut(k).unwrap().push(d.clone());
                }
            });
        }
    });
    variants
}
