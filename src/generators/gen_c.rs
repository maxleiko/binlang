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

    let ns = to_c_name(filename, false);
    for ty in sorted {
        generate_forward_decl(hir, &ns, ty, &mut buf);
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

    for ty in sorted {
        generate_fn_forward_decl(hir, &ns, ty, &mut buf);
    }

    writeln!(buf);
    writeln!(buf, "#endif // BINLANG_{filename}_H_");

    let inner = buf.into_inner()?;

    Ok(())
}

fn generate_impl_file(filename: &str, outdir: &Path, hir: &Hir, sorted: &[&Type]) -> Result<()> {
    let mut file = File::create(outdir.join(format!("{filename}.c")))?;
    let mut buf = BufWriter::new(file);

    writeln!(buf, "#include \"{filename}.h\"");
    writeln!(buf);

    let ns = to_c_name(filename, false);
    for ty in sorted {
        generate_impl_type(hir, &ns, ty, &mut buf);
    }

    Ok(())
}

fn generate_impl_type<W: Write>(hir: &Hir, ns: &str, ty: &Type, out: &mut W) {
    match ty {
        Type::Message(ty) => generate_impl_message(hir, ns, ty, false, out),
        Type::Bitfield(ty) => generate_impl_bitfield(hir, ns, ty, false, out),
        Type::Native(_ty) => (),
        Type::Array(_ty) => (),
    }
}

/*
bl_result_t bl_greycat_abi__headers(bl_slice_t *b, headers_t *value) {
  BL_TRY(bl_slice__read_u16(b, &value->major));
  BL_TRY(bl_slice__read_u16(b, &value->magic));
  BL_TRY(bl_slice__read_u16(b, &value->version));
  BL_TRY(bl_slice__read_u64(b, &value->crc));
  return gc_result_ok;
}
*/

fn generate_impl_message<W: Write>(
    hir: &Hir,
    ns: &str,
    ty: &MessageType,
    forward_decl: bool,
    out: &mut W,
) {
    let name = hir.symbols.get(ty.name).unwrap();
    let typedef = to_c_name(name, true);
    let fn_name = to_c_name(name, false);
    write!(
        out,
        "bl_result_t bl_{ns}__read_{fn_name}(bl_slice_t *b, {typedef} *value)"
    );
    if forward_decl {
        writeln!(out, ";");
        return;
    }
    writeln!(out, " {{");
    for field in &ty.fields {
        write!(out, "  BL_TRY(bl_");
        let field_ty = hir.types.get(&field.ty).unwrap();
        match field_ty {
            Type::Message(ty) => {
                let f_name = hir.symbols.get(field.name).unwrap();
                let f_ty_name = hir.symbols.get(ty.name).unwrap();
                let f_typedef = to_c_name(f_ty_name, true);
                let f_ty_fn_name = to_c_name(f_ty_name, false);
                write!(out, "{ns}__read_{f_ty_fn_name}(b, &value->{f_name})");
            }
            Type::Bitfield(ty) => {
                // TODO
                write!(
                    out,
                    "TODO/* bitfield: {} */(b, NULL)",
                    hir.symbols.get(ty.name).unwrap()
                );
            }
            Type::Native(ty) => {
                let f_name = hir.symbols.get(field.name).unwrap();
                match ty {
                    NativeType::U8 => write!(out, "slice__read_u8(b, &value->{f_name})"),
                    NativeType::U16 => write!(out, "slice__read_u16(b, &value->{f_name})"),
                    NativeType::U32 => write!(out, "slice__read_u32(b, &value->{f_name})"),
                    NativeType::U64 => write!(out, "slice__read_u64(b, &value->{f_name})"),
                    NativeType::I8 => write!(out, "slice__read_i8(b, &value->{f_name})"),
                    NativeType::I16 => write!(out, "slice__read_i16(b, &value->{f_name})"),
                    NativeType::I32 => write!(out, "slice__read_i32(b, &value->{f_name})"),
                    NativeType::I64 => write!(out, "slice__read_i64(b, &value->{f_name})"),
                    NativeType::VU32 => write!(out, "slice__read_vu32(b, &value->{f_name})"),
                    NativeType::VU64 => write!(out, "slice__read_vu64(b, &value->{f_name})"),
                    NativeType::VI32 => write!(out, "slice__read_vi32(b, &value->{f_name})"),
                    NativeType::VI64 => write!(out, "slice__read_vi64(b, &value->{f_name})"),
                };
            }
            Type::Array(symbol_id) => {
                // TODO
                write!(
                    out,
                    "TODO/* array: {} */(b, NULL)",
                    hir.symbols.get(*symbol_id).unwrap()
                );
            }
        }
        writeln!(out, ");");
    }
    writeln!(out, "  return bl_result_ok;");
    writeln!(out, "}}");
}

fn generate_impl_bitfield<W: Write>(
    hir: &Hir,
    ns: &str,
    ty: &BitfieldType,
    forward_decl: bool,
    out: &mut W,
) {
    let name = hir.symbols.get(ty.name).unwrap();
    let typedef = to_c_name(name, true);
    let fn_name = to_c_name(name, false);
    write!(
        out,
        "bl_result_t bl_{ns}__read_{fn_name}(bl_unused bl_slice_t *b, bl_unused {typedef} *value)"
    );
    if forward_decl {
        writeln!(out, ";");
        return;
    }
    writeln!(out, " {{");
    // TODO
    writeln!(out, "  return bl_result_err;");
    writeln!(out, "}}");
}

fn generate_forward_decl<W: Write>(hir: &Hir, ns: &str, ty: &Type, out: &mut W) {
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

fn generate_fn_forward_decl<W: Write>(hir: &Hir, ns: &str, ty: &Type, out: &mut W) {
    match ty {
        Type::Message(ty) => {
            generate_impl_message(hir, ns, ty, true, out);
        }
        Type::Bitfield(ty) => {
            generate_impl_bitfield(hir, ns, ty, true, out);
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
