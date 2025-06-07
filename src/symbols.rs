use string_interner::{StringInterner, backend::BucketBackend, symbol::SymbolU32};

#[derive(Debug, Hash, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct SymbolId(SymbolU32);

pub struct Symbols {
    interner: StringInterner<BucketBackend>,
}

impl Symbols {
    pub fn new() -> Self {
        Self {
            interner: StringInterner::<BucketBackend>::new(),
        }
    }

    #[allow(unused)]
    pub fn len(&self) -> usize {
        self.interner.len()
    }

    pub fn insert(&mut self, symbol: impl AsRef<str>) -> SymbolId {
        SymbolId(self.interner.get_or_intern(symbol))
    }

    pub fn get(&self, id: SymbolId) -> Option<&str> {
        self.interner.resolve(id.0)
    }

    pub fn find(&self, symbol: impl AsRef<str>) -> Option<SymbolId> {
        self.interner.get(symbol).map(SymbolId)
    }
}

impl Default for Symbols {
    #[inline(always)]
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for Symbols {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list()
            .entries(self.interner.iter().map(|(_, s)| s))
            .finish()
    }
}

#[test]
fn dedup() {
    let a = "foo";
    let b = "foo";
    let mut symbols = Symbols::new();
    let a_id = symbols.insert(a);
    let b_id = symbols.insert(b);
    assert_eq!(a_id, b_id);
    assert_eq!(symbols.len(), 1);
}

#[test]
fn find() {
    let a = "foo";
    let mut symbols = Symbols::new();
    let a_id = symbols.insert(a);
    assert_eq!(symbols.find(a), Some(a_id));
}

#[test]
fn get() {
    let a = "foo";
    let mut symbols = Symbols::new();
    let a_id = symbols.insert(a);
    assert_eq!(symbols.get(a_id), Some(a));
}

#[test]
fn dedup_stress() {
    let mut symbols = Symbols::new();
    let id = symbols.insert("hello world");
    (0..10_000 - 1).for_each(|_| {
        symbols.insert("hello world");
    });
    assert_eq!(symbols.len(), 1);
    assert_eq!(symbols.find("hello world"), Some(id));
}
