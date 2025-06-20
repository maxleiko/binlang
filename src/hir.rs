use std::collections::{HashMap, HashSet};

use crate::{
    ast::*,
    symbols::{SymbolId, Symbols},
};

pub struct Hir {
    pub root: SymbolId,
    pub symbols: Symbols,
    pub types: HashMap<SymbolId, Type>,
}

impl Hir {
    pub fn new(file: &File) -> Self {
        let mut symbols = Symbols::new();
        let root = symbols.insert("");
        let natives = NativeTypeSymbols::new(&mut symbols);
        let mut types = HashMap::new();

        types.insert(natives.i8, Type::Native(NativeType::I8));
        types.insert(natives.i16, Type::Native(NativeType::I16));
        types.insert(natives.i32, Type::Native(NativeType::I32));
        types.insert(natives.i64, Type::Native(NativeType::I64));
        types.insert(natives.u8, Type::Native(NativeType::U8));
        types.insert(natives.u16, Type::Native(NativeType::U16));
        types.insert(natives.u32, Type::Native(NativeType::U32));
        types.insert(natives.u64, Type::Native(NativeType::U64));
        types.insert(natives.vi32, Type::Native(NativeType::VI32));
        types.insert(natives.vi64, Type::Native(NativeType::VI64));
        types.insert(natives.vu32, Type::Native(NativeType::VU32));
        types.insert(natives.vu64, Type::Native(NativeType::VU64));

        for def in &file.defs {
            match def {
                TopLevel::Message(message) => {
                    let id = symbols.insert(&message.name);
                    let ty = MessageType::new(id);
                    types.insert(id, Type::Message(ty));
                }
                TopLevel::Bitfield(bitfield) => {
                    let id = symbols.insert(&bitfield.name);
                    let mut ty = BitfieldType::new(id);
                    for flag in &bitfield.flags {
                        let flag_name = symbols.insert(&flag.name);
                        ty.flags.push(Bitflag {
                            name: flag_name,
                            offset: flag.offset,
                        });
                    }
                    types.insert(id, Type::Bitfield(ty));
                }
            }
        }

        // we now have all types defined, let's dive in the fields
        for def in &file.defs {
            if let TopLevel::Message(msg) = def {
                let mut associated_fields = HashMap::new();
                for field in &msg.fields {
                    if let TypeExpr::ArrayWithField(ident, associated_name) = &field.ty {
                        let field_name = symbols.insert(&field.name);
                        let associated_field = msg
                            .fields
                            .iter()
                            .find(|f| &f.name == associated_name)
                            .unwrap_or_else(|| {
                                panic!(
                                    "referenced field '{associated_name}' in array {} is unknown",
                                    field.name
                                )
                            });
                        let associated_field_type = type_expr_to_type_id(
                            &associated_field.ty,
                            &mut symbols,
                            &natives,
                            &mut types,
                            &associated_fields,
                        );
                        associated_fields.insert(
                            associated_name.as_str(),
                            AssociatedField {
                                array_field: field_name,
                                ty: associated_field_type,
                            },
                        );
                    }
                }

                let mut fields = Vec::with_capacity(msg.fields.len());
                for field in &msg.fields {
                    let field_name = symbols.insert(&field.name);
                    let field_type = type_expr_to_type_id(
                        &field.ty,
                        &mut symbols,
                        &natives,
                        &mut types,
                        &associated_fields,
                    );
                    fields.push(Field {
                        name: field_name,
                        ty: field_type,
                        associated: associated_fields.get(&*field.name).map(|a| a.array_field),
                    });
                }
                let msg_name_id = symbols.find(&msg.name).unwrap();
                if let Type::Message(ty) = types.get_mut(&msg_name_id).unwrap() {
                    ty.fields = fields;
                }
            }
        }

        Self {
            root,
            symbols,
            types,
        }
    }
}

impl std::fmt::Debug for Hir {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Hir")
            .field("symbols", &self.symbols)
            .field(
                "types",
                &HirDebugTypes {
                    symbols: &self.symbols,
                    types: &self.types,
                },
            )
            .finish()
    }
}

struct HirDebugTypes<'a> {
    symbols: &'a Symbols,
    types: &'a HashMap<SymbolId, Type>,
}

impl std::fmt::Debug for HirDebugTypes<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_map()
            .entries(self.types.iter().map(|(name, ty)| {
                let name = self.symbols.get(*name).unwrap();
                let ty = HirDebugType {
                    symbols: self.symbols,
                    ty,
                };
                (name, ty)
            }))
            .finish()
    }
}

pub(crate) struct HirDebugType<'a> {
    symbols: &'a Symbols,
    ty: &'a Type,
}

impl std::fmt::Debug for HirDebugType<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.ty {
            Type::Message(ty) => HirDebugMessageType {
                symbols: self.symbols,
                msg: ty,
            }
            .fmt(f),
            Type::Bitfield(ty) => HirDebugBitfieldType {
                symbols: self.symbols,
                bf: ty,
            }
            .fmt(f),
            Type::Native(ty) => ty.fmt(f),
            Type::Array(ArrayType::Default(id)) => {
                write!(f, "{}[]", self.symbols.get(*id).unwrap())
            }
            Type::Array(ArrayType::Field {
                elem_type,
                field_name,
                field_type,
            }) => {
                let elem_type = self.symbols.get(*elem_type).unwrap();
                let field_name = self.symbols.get(*field_name).unwrap();
                let field_type = self.symbols.get(*field_type).unwrap();
                write!(f, "{elem_type}[{field_name}: {field_type}]")
            }
        }
    }
}

struct HirDebugMessageType<'a> {
    symbols: &'a Symbols,
    msg: &'a MessageType,
}

impl std::fmt::Debug for HirDebugMessageType<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_map()
            .entries(self.msg.fields.iter().map(|f| {
                (
                    self.symbols.get(f.name).unwrap(),
                    self.symbols.get(f.ty).unwrap(),
                )
            }))
            .finish()
    }
}

struct HirDebugBitfieldType<'a> {
    symbols: &'a Symbols,
    bf: &'a BitfieldType,
}

impl std::fmt::Debug for HirDebugBitfieldType<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_map()
            .entries(
                self.bf
                    .flags
                    .iter()
                    .map(|f| (self.symbols.get(f.name).unwrap(), f.offset)),
            )
            .finish()
    }
}

pub struct NativeTypeSymbols {
    pub i8: SymbolId,
    pub i16: SymbolId,
    pub i32: SymbolId,
    pub i64: SymbolId,
    pub u8: SymbolId,
    pub u16: SymbolId,
    pub u32: SymbolId,
    pub u64: SymbolId,
    pub vi32: SymbolId,
    pub vi64: SymbolId,
    pub vu32: SymbolId,
    pub vu64: SymbolId,
    pub f32: SymbolId,
    pub f64: SymbolId,
}

impl NativeTypeSymbols {
    fn new(symbols: &mut Symbols) -> Self {
        Self {
            i8: symbols.insert("i8"),
            i16: symbols.insert("i16"),
            i32: symbols.insert("i32"),
            i64: symbols.insert("i64"),
            u8: symbols.insert("u8"),
            u16: symbols.insert("u16"),
            u32: symbols.insert("u32"),
            u64: symbols.insert("u64"),
            vi32: symbols.insert("vi32"),
            vi64: symbols.insert("vi64"),
            vu32: symbols.insert("vu32"),
            vu64: symbols.insert("vu64"),
            f32: symbols.insert("f32"),
            f64: symbols.insert("f64"),
        }
    }

    pub fn type_id(&self, ty: NativeType) -> SymbolId {
        match ty {
            NativeType::U8 => self.u8,
            NativeType::U16 => self.u16,
            NativeType::U32 => self.u32,
            NativeType::U64 => self.u64,
            NativeType::I8 => self.i8,
            NativeType::I16 => self.i16,
            NativeType::I32 => self.i32,
            NativeType::I64 => self.i64,
            NativeType::VU32 => self.vu32,
            NativeType::VU64 => self.vu64,
            NativeType::VI32 => self.vi32,
            NativeType::VI64 => self.vi64,
            NativeType::F32 => self.f32,
            NativeType::F64 => self.f64,
        }
    }
}

pub enum Type {
    Message(MessageType),
    Bitfield(BitfieldType),
    Native(NativeType),
    Array(ArrayType),
}

pub enum ArrayType {
    Default(SymbolId),
    Field {
        elem_type: SymbolId,
        field_name: SymbolId,
        field_type: SymbolId,
    },
}

#[allow(unused)]
impl Type {
    pub fn is_message(&self) -> bool {
        matches!(self, Self::Message(_))
    }

    pub fn is_bitfield(&self) -> bool {
        matches!(self, Self::Bitfield(_))
    }

    pub fn is_array(&self) -> bool {
        matches!(self, Self::Array(_))
    }

    pub fn is_native(&self) -> bool {
        matches!(self, Self::Native(_))
    }

    pub fn to_debug<'a>(&'a self, symbols: &'a Symbols) -> HirDebugType<'a> {
        HirDebugType { symbols, ty: self }
    }
}

pub struct MessageType {
    pub name: SymbolId,
    pub fields: Vec<Field>,
}

pub struct Field {
    pub name: SymbolId,
    pub ty: SymbolId,
    pub associated: Option<SymbolId>,
}

impl MessageType {
    fn new(name: SymbolId) -> Self {
        Self {
            name,
            fields: Default::default(),
        }
    }
}

pub struct BitfieldType {
    pub name: SymbolId,
    pub flags: Vec<Bitflag>,
}

pub struct Bitflag {
    pub name: SymbolId,
    pub offset: u8,
}

impl BitfieldType {
    fn new(name: SymbolId) -> Self {
        Self {
            name,
            flags: Default::default(),
        }
    }
}

struct AssociatedField {
    array_field: SymbolId,
    ty: SymbolId,
}

fn type_expr_to_type_id(
    ty: &TypeExpr,
    symbols: &mut Symbols,
    natives: &NativeTypeSymbols,
    types: &mut HashMap<SymbolId, Type>,
    associated_fields: &HashMap<&str, AssociatedField>,
) -> SymbolId {
    match ty {
        TypeExpr::Ident(ty) => match ty {
            TypeIdent::Native(native_type) => natives.type_id(*native_type),
            TypeIdent::Custom(name) => symbols
                .find(name)
                .unwrap_or_else(|| panic!("use of undefined type '{name}'")),
        },
        TypeExpr::ArrayNoField(ty) => match ty {
            TypeIdent::Native(native_type) => {
                let generic_id = natives.type_id(*native_type);
                let name = symbols.get(generic_id).unwrap();
                let array_id = symbols.insert(format!("{name}[]"));
                types.insert(array_id, Type::Array(ArrayType::Default(generic_id)));
                array_id
            }
            TypeIdent::Custom(name) => {
                let generic_id = symbols
                    .find(name)
                    .unwrap_or_else(|| panic!("use of undefined type '{name}'"));
                let array_id = symbols.insert(format!("{name}[]"));
                types.insert(array_id, Type::Array(ArrayType::Default(generic_id)));
                array_id
            }
        },
        TypeExpr::ArrayWithField(ty, field) => {
            let AssociatedField {
                ty: associated_type,
                ..
            } = associated_fields.get(field.as_str()).unwrap();
            match ty {
                TypeIdent::Native(native_type) => {
                    let elem_type = natives.type_id(*native_type);
                    let name = symbols.get(elem_type).unwrap();
                    let array_id = symbols.insert(format!("{name}[{field}]"));
                    types.insert(
                        array_id,
                        Type::Array(ArrayType::Field {
                            elem_type,
                            field_name: symbols.insert(field),
                            field_type: *associated_type,
                        }),
                    );
                    array_id
                }
                TypeIdent::Custom(name) => {
                    let elem_type = symbols
                        .find(name)
                        .unwrap_or_else(|| panic!("use of undefined type '{name}'"));
                    let array_id = symbols.insert(format!("{name}[{field}]"));
                    types.insert(
                        array_id,
                        Type::Array(ArrayType::Field {
                            elem_type,
                            field_name: symbols.insert(field),
                            field_type: *associated_type,
                        }),
                    );
                    array_id
                }
            }
        }
    }
}
