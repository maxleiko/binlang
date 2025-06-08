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
    if let Type::Message(ty) = ty {
        generate_impl_message(hir, ns, ty, false, out)
    }
}

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
    let indent = "  ";
    for field in &ty.fields {
        write!(out, "{indent}"); // indent
        let f_name = match field.associated {
            Some(associated_name) => Cow::Owned(format!(
                "{}.size",
                hir.symbols.get(associated_name).unwrap()
            )),
            None => Cow::Borrowed(hir.symbols.get(field.name).unwrap()),
        };
        match hir.types.get(&field.ty).unwrap() {
            Type::Message(ty) => {
                let f_ty_name = hir.symbols.get(ty.name).unwrap();
                let f_typedef = to_c_name(f_ty_name, true);
                let f_ty_fn_name = to_c_name(f_ty_name, false);
                writeln!(
                    out,
                    "BL_TRY(bl_{ns}__read_{f_ty_fn_name}(b, &value->{f_name}));"
                );
            }
            Type::Bitfield(ty) => {
                writeln!(out, "BL_TRY(bl_slice__read_u8(b, &value->{f_name}));");
            }
            Type::Native(ty) => {
                match ty {
                    NativeType::U8 => {
                        writeln!(out, "BL_TRY(bl_slice__read_u8(b, &value->{f_name}));")
                    }
                    NativeType::U16 => {
                        writeln!(out, "BL_TRY(bl_slice__read_u16(b, &value->{f_name}));")
                    }
                    NativeType::U32 => {
                        writeln!(out, "BL_TRY(bl_slice__read_u32(b, &value->{f_name}));")
                    }
                    NativeType::U64 => {
                        writeln!(out, "BL_TRY(bl_slice__read_u64(b, &value->{f_name}));")
                    }
                    NativeType::I8 => {
                        writeln!(out, "BL_TRY(bl_slice__read_i8(b, &value->{f_name}));")
                    }
                    NativeType::I16 => {
                        writeln!(out, "BL_TRY(bl_slice__read_i16(b, &value->{f_name}));")
                    }
                    NativeType::I32 => {
                        writeln!(out, "BL_TRY(bl_slice__read_i32(b, &value->{f_name}));")
                    }
                    NativeType::I64 => {
                        writeln!(out, "BL_TRY(bl_slice__read_i64(b, &value->{f_name}));")
                    }
                    NativeType::VU32 => {
                        writeln!(out, "BL_TRY(bl_slice__read_vu32(b, &value->{f_name}));")
                    }
                    NativeType::VU64 => {
                        writeln!(out, "BL_TRY(bl_slice__read_vu64(b, &value->{f_name}));")
                    }
                    NativeType::VI32 => {
                        writeln!(out, "BL_TRY(bl_slice__read_vi32(b, &value->{f_name}));")
                    }
                    NativeType::VI64 => {
                        writeln!(out, "BL_TRY(bl_slice__read_vi64(b, &value->{f_name}));")
                    }
                };
            }
            Type::Array(ArrayType::Default(type_id)) => {
                let elem_ty = hir.symbols.get(*type_id).unwrap();
                let elem_ty_name = to_c_name(elem_ty, false);
                writeln!(out, "BL_TRY(bl_slice__read_u32(b, &value->{f_name}.size));");
                writeln!(
                    out,
                    "{indent}array_reserve(&value->{f_name}, value->{f_name}.size);"
                );
                if elem_ty == "u8" {
                    writeln!(
                        out,
                        "{indent}BL_TRY(bl_slice__read_exact(b, value->{f_name}.elems, value->{f_name}.size));"
                    );
                } else {
                    writeln!(
                        out,
                        "{indent}for (uint32_t i = 0; i < value->{f_name}.size; i++) {{"
                    );
                    writeln!(
                        out,
                        "{indent}  BL_TRY(bl_{ns}__read_{elem_ty_name}(b, value->{f_name}.elems + i));"
                    );
                    writeln!(out, "{indent}}}");
                }
            }
            Type::Array(ArrayType::Field {
                elem_type,
                field_name,
                field_type,
            }) => {
                let elem_ty = hir.symbols.get(*elem_type).unwrap();
                let elem_ty_name = to_c_name(elem_ty, false);
                let field_ty = hir.symbols.get(*field_type).unwrap();
                writeln!(
                    out,
                    "array_reserve(&value->{f_name}, value->{f_name}.size);"
                );
                if elem_ty == "u8" {
                    writeln!(
                        out,
                        "{indent}BL_TRY(bl_slice__read_exact(b, value->{f_name}.elems, value->{f_name}.size));"
                    );
                } else {
                    writeln!(
                        out,
                        "{indent}for (uint32_t i = 0; i < value->{f_name}.size; i++) {{"
                    );
                    writeln!(
                        out,
                        "{indent}  BL_TRY(bl_{ns}__read_{elem_ty_name}(b, value->{f_name}.elems + i));"
                    );
                    writeln!(out, "{indent}}}");
                }
            }
        }
    }
    writeln!(out, "  return bl_result_ok;");
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
            writeln!(out, "typedef uint8_t {typedef};");
        }
        _ => (),
    }
}

fn generate_fn_forward_decl<W: Write>(hir: &Hir, ns: &str, ty: &Type, out: &mut W) {
    if let Type::Message(ty) = ty {
        generate_impl_message(hir, ns, ty, true, out);
    }
}

fn generate_type<W: Write>(hir: &Hir, ty: &Type, out: &mut W) {
    match ty {
        Type::Message(ty) => generate_message(hir, ty, out),
        Type::Bitfield(ty) => {}
        Type::Native(ty) => {}
        Type::Array(ty) => {}
    }
}

fn generate_message<W: Write>(hir: &Hir, msg: &MessageType, out: &mut W) {
    writeln!(out, "struct {} {{", hir.symbols.get(msg.name).unwrap());

    for field in &msg.fields {
        if field.associated.is_some() {
            // skip associated fields in struct as we will use the BlArray .size field
            continue;
        }
        let field_ty = hir.types.get(&field.ty).unwrap();
        write!(out, "  ");
        match field_ty {
            Type::Array(ArrayType::Default(elem_type))
            | Type::Array(ArrayType::Field { elem_type, .. }) => {
                write!(
                    out,
                    "BlArray({})",
                    to_c_name(hir.symbols.get(*elem_type).unwrap(), true)
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

/// TODO cache all those strings rather than always re-creating them on the spot
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
