use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet, VecDeque};
use std::fmt::Write as _;

use topological_sort::TopologicalSort;

use crate::hir::*;
use crate::symbols::SymbolId;
use crate::{ast::*, error::ParseError, parser::parse};

pub fn generate_c(source: &str) -> Result<String, ParseError> {
    let file = parse(source)?;
    // log::debug!("{file:#?}");
    let hir = Hir::new(&file);
    log::debug!("{hir:#?}");
    let mut out = String::with_capacity(8000);

    writeln!(out, "#include \"binlang.h\"\n");

    let mut ts2 = TopologicalSort::<SymbolId>::new();
    for (id, ty) in &hir.types {
        match ty {
            Type::Message(ty) => {
                log::info!("message: {}", hir.symbols.get(ty.name).unwrap());
                ts2.add_dependency(ty.name, hir.root);
                for field in &ty.fields {
                    let field_ty = hir.types.get(&field.ty).unwrap();
                    if field_ty.is_message() {
                        ts2.add_dependency(field.ty, ty.name);
                    }
                }
            }
            Type::Bitfield(ty) => {
                log::info!("bitfield: {}", hir.symbols.get(ty.name).unwrap());
                ts2.add_dependency(ty.name, hir.root);
            }
            Type::Native(ty) => {
                log::info!("native: {ty:?}");
            }
            Type::Array(gen_id) => {
                log::info!("array: {}", hir.symbols.get(*gen_id).unwrap());
            }
        }
    }

    let mut sorted = Vec::new();

    loop {
        let mut v = ts2.pop_all();
        if v.is_empty() {
            break;
        }
        v.sort();
        for id in v.iter().rev() {
            if *id != hir.root {
                let ty = hir.types.get(id).unwrap();
                sorted.push(ty);
            }
        }
    }

    for ty in &sorted {
        generate_forward_decl(&hir, ty, &mut out);
    }

    out.push('\n');

    for ty in &sorted {
        if let Type::Bitfield(bitfield) = ty {
            generate_bitfield(&hir, bitfield, &mut out);
        }
    }

    for ty in &sorted {
        generate_type(&hir, ty, &mut out);
    }

    Ok(out)
}

fn generate_forward_decl(hir: &Hir, ty: &Type, out: &mut String) {
    match ty {
        Type::Message(ty) => {
            let struct_name = hir.symbols.get(ty.name).unwrap();
            let typedef = to_c_name(struct_name, true);
            writeln!(out, "typedef struct {struct_name} {typedef};");
        }
        Type::Bitfield(ty) => {
            let struct_name = hir.symbols.get(ty.name).unwrap();
            let typedef = to_c_name(struct_name, true);
            writeln!(out, "typedef int8_t {typedef};");
        }
        Type::Native(native_type) => todo!(),
        Type::Array(symbol_id) => todo!(),
    }
}

fn generate_type(hir: &Hir, ty: &Type, out: &mut String) {
    match ty {
        Type::Message(ty) => generate_message(hir, ty, out),
        Type::Bitfield(ty) => {}
        Type::Native(ty) => {}
        Type::Array(type_id) => {}
    }
}

fn generate_message(hir: &Hir, msg: &MessageType, out: &mut String) {
    out.push_str(&format!(
        "struct {} {{\n",
        hir.symbols.get(msg.name).unwrap()
    ));

    for field in &msg.fields {
        let field_ty = hir.types.get(&field.ty).unwrap();
        out.push_str("  ");
        match field_ty {
            Type::Array(generic_type) => {
                out.push_str("BlArray(");
                out.push_str(&to_c_name(hir.symbols.get(*generic_type).unwrap(), true));
                out.push(')');
            }
            _ => {
                out.push_str(&to_c_name(hir.symbols.get(field.ty).unwrap(), true));
            }
        }
        out.push(' ');
        out.push_str(hir.symbols.get(field.name).unwrap());
        out.push_str(";\n");
    }

    out.push_str("};\n\n");
}

fn generate_bitfield(hir: &Hir, bitfield: &BitfieldType, out: &mut String) {
    let name = hir.symbols.get(bitfield.name).unwrap();
    out.push_str("/// Bitfield: ");
    out.push_str(name);
    out.push('\n');
    for flag in &bitfield.flags {
        out.push_str("#define ");
        out.push_str(&to_c_name(name, false).to_uppercase());
        out.push('_');
        out.push_str(&to_c_name(hir.symbols.get(flag.name).unwrap(), false).to_uppercase());
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
fn to_c_name(input: &str, as_type: bool) -> Cow<'_, str> {
    match input {
        "u8" => return Cow::Borrowed("uint8_t"),
        "u16" => return Cow::Borrowed("uint16_t"),
        "u32" | "vu32" => return Cow::Borrowed("uint32_t"),
        "u64" | "vu64" => return Cow::Borrowed("uint64_t"),
        "i8" => return Cow::Borrowed("int8_t"),
        "i16" => return Cow::Borrowed("int16_t"),
        "i32" | "vi32" => return Cow::Borrowed("int32_t"),
        "i64" | "vi64" => return Cow::Borrowed("int64_t"),
        _ => (),
    }

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

    Cow::Owned(out)
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
