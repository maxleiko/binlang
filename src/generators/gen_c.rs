use std::collections::{BTreeMap, HashMap, HashSet};
use std::fmt::Write as _;

use crate::{ast::*, error::ParseError, parser::parse};

pub fn generate_c(source: &str) -> Result<String, ParseError> {
    let file = parse(source)?;
    // eprintln!("{file:#?}");
    let mut out = String::with_capacity(8000);

    writeln!(out, "#include \"binlang.h\"\n");

    let mut types: HashSet<&str> = HashSet::new();
    let mut graph: HashMap<&str, HashSet<&str>> = HashMap::new();

    // 1. collect types
    for def in &file.defs {
        match def {
            TopLevel::Message(message) => {
                types.insert(&message.name);
            }
            TopLevel::Bitfield(bitfield) => {
                // types.insert(&bitfield.name);
            }
        }
    }

    // 2. build dependency graph
    for def in &file.defs {
        match def {
            TopLevel::Message(msg) => {
                let mut used = HashSet::new();

                for field in &msg.fields {
                    collect_deps_from_type(&field.ty, &types, &mut used);
                }

                graph.insert(&msg.name, used);
            }
            TopLevel::Bitfield(bf) => {
                writeln!(out, "typedef uint8_t {};", to_c_name(&bf.name, true));
            }
        }
    }

    // 3. topological sort
    let sorted = topo_sort(&graph);
    for struct_name in sorted {
        writeln!(
            out,
            "typedef struct {struct_name} {};",
            to_c_name(struct_name, true)
        );
    }

    out.push('\n');

    generate(&file, &mut out);
    Ok(out)
}

fn generate(file: &File, out: &mut String) {
    for def in &file.defs {
        match def {
            TopLevel::Message(msg) => {
                generate_message(msg, out);
            }
            TopLevel::Bitfield(bitfield) => {
                generate_bitfield(bitfield, out);
            }
        }
    }
}

fn generate_message(msg: &Message, out: &mut String) {
    out.push_str(&format!("struct {} {{\n", &msg.name));

    for field in &msg.fields {
        if let Some(deco) = &field.decorator {
            out.push_str(&format!("  // @{}\n", deco));
        }

        out.push_str("  ");
        generate_type_expr(&field.ty, out);
        out.push(' ');
        out.push_str(&field.name);
        out.push_str(";\n");
    }

    out.push_str("};\n\n");
}

fn generate_bitfield(bitfield: &Bitfield, out: &mut String) {
    out.push_str("// Bitfield: ");
    out.push_str(&bitfield.name);
    out.push('\n');
    for flag in &bitfield.flags {
        out.push_str("#define ");
        out.push_str(&bitfield.name.to_uppercase());
        out.push('_');
        out.push_str(&flag.name.to_uppercase());
        out.push_str(" (1 << ");
        writeln!(out, "{})", flag.offset);
    }
    out.push('\n');
}

fn generate_type_expr(ty: &TypeExpr, out: &mut String) {
    match ty {
        TypeExpr::Ident(ident) => {
            generate_type_ident(ident, out);
        }
        TypeExpr::ArrayNoField(ident) => {
            out.push_str("BlArray(");
            generate_type_ident(ident, out);
            out.push(')');
        }
        TypeExpr::ArrayWithField(ident, _) => {
            generate_type_ident(ident, out);
        }
    }
}

fn generate_type_ident(ty: &TypeIdent, out: &mut String) {
    match ty {
        TypeIdent::Native(native) => match native {
            NativeType::U8 => out.push_str("uint8_t"),
            NativeType::U16 => out.push_str("uint16_t"),
            NativeType::U32 | NativeType::VU32 => out.push_str("uint32_t"),
            NativeType::U64 | NativeType::VU64 => out.push_str("uint64_t"),
            NativeType::I8 => out.push_str("int8_t"),
            NativeType::I16 => out.push_str("int16_t"),
            NativeType::I32 | NativeType::VI32 => out.push_str("int32_t"),
            NativeType::I64 | NativeType::VI64 => out.push_str("int64_t"),
        },
        TypeIdent::Custom(name) => out.push_str(&to_c_name(name, true)),
    }
}

#[allow(dead_code)]
fn to_c_name(input: &str, as_type: bool) -> String {
    let mut out = String::with_capacity(input.len() + 2);
    let mut prev_lowercase = false;

    for ch in input.chars() {
        if ch.is_ascii_uppercase() {
            if prev_lowercase {
                out.push('_');
            }
            out.push(ch.to_ascii_lowercase());
            prev_lowercase = false;
        } else {
            out.push(ch);
            prev_lowercase = ch.is_ascii_lowercase() || ch.is_ascii_digit();
        }
    }

    if as_type {
        out.push_str("_t");
    }

    out
}

fn collect_deps_from_type<'a>(
    ty: &'a TypeExpr,
    types: &HashSet<&'a str>,
    acc: &mut HashSet<&'a str>,
) {
    match ty {
        TypeExpr::Ident(TypeIdent::Custom(name))
        | TypeExpr::ArrayNoField(TypeIdent::Custom(name))
        | TypeExpr::ArrayWithField(TypeIdent::Custom(name), _) => {
            if types.contains(name.as_str()) {
                acc.insert(name);
            }
        }
        _ => (),
    }
}

fn topo_sort<'a>(graph: &HashMap<&'a str, HashSet<&'a str>>) -> Vec<&'a str> {
    let mut visited = HashSet::new();
    let mut temp = HashSet::new();
    let mut out = Vec::new();

    for node in graph.keys() {
        visit(node, graph, &mut visited, &mut temp, &mut out);
    }

    out
}

fn visit<'a>(
    node: &'a str,
    graph: &HashMap<&'a str, HashSet<&'a str>>,
    visited: &mut HashSet<&'a str>,
    temp: &mut HashSet<&'a str>,
    out: &mut Vec<&'a str>,
) {
    if visited.contains(node) {
        return;
    }
    if temp.contains(node) {
        panic!("cyclic dependency involving {}", node);
    }

    temp.insert(node);

    if let Some(neighbors) = graph.get(node) {
        for neighbor in neighbors {
            visit(neighbor, graph, visited, temp, out);
        }
    }

    temp.remove(node);
    visited.insert(node);
    out.push(node);
}
