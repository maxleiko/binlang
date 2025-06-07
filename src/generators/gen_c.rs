use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet, VecDeque};
use std::fmt::Write as _;
use std::fs::File;
use std::io::Write;
use std::io::{BufReader, BufWriter, LineWriter};
use std::path::Path;

use anyhow::Result;
use topological_sort::TopologicalSort;

use crate::hir::*;
use crate::symbols::SymbolId;
use crate::{ast::*, error::ParseError, parser::parse};

pub fn generate_c(filename: &str, source: &str, outdir: &Path) -> Result<()> {
    let file = parse(source)?;
    let hir = Hir::new(&file);
    let sorted = topological_sort(&hir);
    log::debug!("{hir:#?}");

    generate_header_file(filename, outdir, &hir, &sorted)?;
    generate_impl_file(filename, outdir, &hir, &sorted)?;

    Ok(())
}

fn generate_header_file(filename: &str, outdir: &Path, hir: &Hir, sorted: &[&Type]) -> Result<()> {
    let mut file = File::create(outdir.join(format!("{filename}.h")))?;
    let mut buf = BufWriter::new(file);

    writeln!(buf, "#ifndef BINLANG_{filename}_H_");
    writeln!(buf, "#define BINLANG_{filename}_H_");
    writeln!(buf);
    writeln!(buf, "#include \"binlang.h\"");
    writeln!(buf);

    for ty in sorted {
        generate_forward_decl(hir, ty, &mut buf);
    }

    writeln!(buf);

    for ty in sorted {
        if let Type::Bitfield(bitfield) = ty {
            generate_bitfield(hir, bitfield, &mut buf);
        }
    }

    for ty in sorted {
        generate_type(hir, ty, &mut buf);
    }

    writeln!(buf, "#endif // BINLANG_{filename}_H_");

    let inner = buf.into_inner()?;

    Ok(())
}

fn generate_impl_file(filename: &str, outdir: &Path, hir: &Hir, sorted: &[&Type]) -> Result<()> {
    let mut file = File::create(outdir.join(format!("{filename}.c")))?;
    let mut buf = BufWriter::new(file);

    writeln!(buf, "#include \"{filename}.h\"");
    writeln!(buf);

    Ok(())
}

fn generate_forward_decl<W: Write>(hir: &Hir, ty: &Type, out: &mut W) {
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
        _ => (),
    }
}

fn generate_type<W: Write>(hir: &Hir, ty: &Type, out: &mut W) {
    match ty {
        Type::Message(ty) => generate_message(hir, ty, out),
        Type::Bitfield(ty) => {}
        Type::Native(ty) => {}
        Type::Array(type_id) => {}
    }
}

fn generate_message<W: Write>(hir: &Hir, msg: &MessageType, out: &mut W) {
    writeln!(out, "struct {} {{", hir.symbols.get(msg.name).unwrap());

    for field in &msg.fields {
        let field_ty = hir.types.get(&field.ty).unwrap();
        write!(out, "  ");
        match field_ty {
            Type::Array(generic_type) => {
                write!(
                    out,
                    "BlArray({})",
                    to_c_name(hir.symbols.get(*generic_type).unwrap(), true)
                );
            }
            _ => {
                write!(
                    out,
                    "{}",
                    to_c_name(hir.symbols.get(field.ty).unwrap(), true)
                );
            }
        }
        writeln!(out, " {};", hir.symbols.get(field.name).unwrap());
    }

    writeln!(out, "}};\n");
}

fn generate_bitfield<W: Write>(hir: &Hir, bitfield: &BitfieldType, out: &mut W) {
    let name = hir.symbols.get(bitfield.name).unwrap();
    writeln!(out, "/// Bitfield: {name}");

    let upper_name = to_c_name(name, false).to_uppercase();
    for flag in &bitfield.flags {
        let upper_flag_name = to_c_name(hir.symbols.get(flag.name).unwrap(), false).to_uppercase();
        writeln!(
            out,
            "#define {upper_name}_{upper_flag_name} (1 << {})",
            flag.offset
        );
    }

    writeln!(out);
}

fn generate_type_expr<W: Write>(ty: &TypeExpr, out: &mut W) {
    match ty {
        TypeExpr::Ident(ident) => {
            generate_type_ident(ident, out);
        }
        TypeExpr::ArrayNoField(ident) => {
            write!(out, "BlArray(");
            generate_type_ident(ident, out);
            write!(out, ")");
        }
        TypeExpr::ArrayWithField(ident, _) => {
            generate_type_ident(ident, out);
        }
    }
}

fn generate_type_ident<W: Write>(
    ty: &TypeIdent,
    out: &mut W,
) -> std::result::Result<(), std::io::Error> {
    match ty {
        TypeIdent::Native(native) => match native {
            NativeType::U8 => write!(out, "uint8_t"),
            NativeType::U16 => write!(out, "uint16_t"),
            NativeType::U32 | NativeType::VU32 => write!(out, "uint32_t"),
            NativeType::U64 | NativeType::VU64 => write!(out, "uint64_t"),
            NativeType::I8 => write!(out, "int8_t"),
            NativeType::I16 => write!(out, "int16_t"),
            NativeType::I32 | NativeType::VI32 => write!(out, "int32_t"),
            NativeType::I64 | NativeType::VI64 => write!(out, "int64_t"),
        },
        TypeIdent::Custom(name) => write!(out, "{}", to_c_name(name, true)),
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

fn topological_sort(hir: &Hir) -> Vec<&Type> {
    let mut ts2 = TopologicalSort::<SymbolId>::new();
    for (id, ty) in &hir.types {
        match ty {
            Type::Message(ty) => {
                log::debug!("message: {}", hir.symbols.get(ty.name).unwrap());
                ts2.add_dependency(ty.name, hir.root);
                for field in &ty.fields {
                    let field_ty = hir.types.get(&field.ty).unwrap();
                    if field_ty.is_message() {
                        ts2.add_dependency(field.ty, ty.name);
                    }
                }
            }
            Type::Bitfield(ty) => {
                log::debug!("bitfield: {}", hir.symbols.get(ty.name).unwrap());
                ts2.add_dependency(ty.name, hir.root);
            }
            _ => (),
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

    sorted
}
