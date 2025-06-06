#[derive(Debug)]
pub struct File {
    pub defs: Vec<TopLevel>,
}

#[derive(Debug)]
pub enum TopLevel {
    Message(Message),
    Bitfield(Bitfield),
}

#[derive(Debug)]
pub struct Message {
    pub name: String,
    pub fields: Vec<Field>,
}

#[derive(Debug)]
pub struct Bitfield {
    pub name: String,
    pub flags: Vec<BitFlag>,
}

#[derive(Debug)]
pub struct BitFlag {
    pub name: String,
    pub offset: u8,
}

#[derive(Debug)]
pub struct Field {
    pub decorator: Option<String>,
    pub name: String,
    pub ty: TypeExpr,
}

#[derive(Debug)]
pub enum TypeExpr {
    Ident(TypeIdent),
    ArrayNoField(TypeIdent),
    ArrayWithField(TypeIdent, String), // u8[size]
}

#[derive(Debug)]
pub enum TypeIdent {
    Native(NativeType),
    Custom(String),
}

#[derive(Debug)]
pub enum NativeType {
    U8,
    U16,
    U32,
    U64,
    I8,
    I16,
    I32,
    I64,
    VU32,
    VU64,
    VI32,
    VI64,
}
