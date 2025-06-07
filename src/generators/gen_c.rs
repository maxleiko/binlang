use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};
use std::fmt::Write as _;

use topological_sort::TopologicalSort;

use crate::hir::*;
use crate::symbols::SymbolId;
use crate::{ast::*, error::ParseError, parser::parse};

pub fn generate_c(source: &str) -> Result<String, ParseError> {
    let file = parse(source)?;
    // eprintln!("{file:#?}");
    let hir = Hir::new(&file);
    eprintln!("{hir:#?}");
    let mut out = String::with_capacity(8000);

    writeln!(out, "#include \"binlang.h\"\n");

    let mut ts2 = TopologicalSort::<SymbolId>::new();
    for ty in hir.iter_messages() {
        eprintln!("message: {} ({:?})", hir.symbols.get(ty.name).unwrap(), ty.name);
        for field in &ty.fields {
            let field_ty = hir.types.get(&field.ty).unwrap();
            if field_ty.is_message() {
                eprintln!(
                    "  dep: {} -> {}",
                    hir.symbols.get(field.ty).unwrap(),
                    hir.symbols.get(ty.name).unwrap()
                );
                ts2.add_dependency(field.ty, ty.name);
            }
        }
    }
    eprintln!("====");
    loop {
        let mut v = ts2.pop_all();
        if v.is_empty() {
            break;
        }
        v.sort();
        for id in v {
            let struct_name = hir.symbols.get(id).unwrap();
            let typedef_name = to_c_name(struct_name, true);
            eprintln!("{struct_name} {typedef_name}");
            writeln!(out, "typedef struct {struct_name} {typedef_name};");
        }
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
            acc.insert(name);
            // if types.contains(name.as_str()) {
            // }
        }
        _ => (),
    }
}

fn depth_first_search_visit<'a>(
    graph: &HashMap<&'a str, HashSet<&'a str>>,
    node: &'a str,
    edges: &HashSet<&'a str>,
    ordered: &mut VecDeque<&'a str>,
    permanents: &mut HashSet<&'a str>,
    temporaries: &mut HashSet<&'a str>,
) {
    if permanents.contains(node) {
        return;
    }
    if temporaries.contains(node) {
        panic!("cyclic dependency found for {node}");
    }

    temporaries.insert(node);

    for edge in edges {
        if let Some(m_edges) = graph.get(edge) {
            depth_first_search_visit(graph, edge, m_edges, ordered, permanents, temporaries);
        }
    }

    permanents.insert(node);
    ordered.push_front(node)
}

fn depth_first_search_sort<'a>(graph: &HashMap<&'a str, HashSet<&'a str>>) -> Vec<&'a str> {
    let mut ordered = VecDeque::new();
    let mut permanents = HashSet::new();
    let mut temporaries = HashSet::new();

    for (node, edges) in graph {
        if permanents.contains(*node) {
            return ordered.into();
        }
        depth_first_search_visit(
            graph,
            node,
            edges,
            &mut ordered,
            &mut permanents,
            &mut temporaries,
        );
    }

    ordered.into()
}
