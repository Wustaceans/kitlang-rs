use std::fmt;

/// A C function signature extracted from a header.
#[derive(Clone, Debug, PartialEq)]
pub struct CFunction {
    pub name: String,
    pub return_type: CType,
    pub params: Vec<CParam>,
    pub is_variadic: bool,
}

/// A C function parameter.
#[derive(Clone, Debug, PartialEq)]
pub struct CParam {
    pub name: Option<String>,
    pub ty: CType,
}

/// A C struct definition.
#[derive(Clone, Debug, PartialEq)]
pub struct CStruct {
    pub name: String,
    pub fields: Vec<CField>,
}

/// A C union definition.
#[derive(Clone, Debug, PartialEq)]
pub struct CUnion {
    pub name: String,
    pub fields: Vec<CField>,
}

/// A single field in a C struct or union.
#[derive(Clone, Debug, PartialEq)]
pub struct CField {
    pub name: Option<String>,
    pub ty: CType,
}

/// A C enum definition.
#[derive(Clone, Debug, PartialEq)]
pub struct CEnum {
    pub name: String,
    pub variants: Vec<CEnumVariant>,
}

/// A C enum variant (constant).
#[derive(Clone, Debug, PartialEq)]
pub struct CEnumVariant {
    pub name: String,
    pub value: Option<i64>,
}

/// A C typedef (type alias).
#[derive(Clone, Debug, PartialEq)]
pub struct CTypedef {
    pub name: String,
    pub underlying: CType,
}

/// A C macro constant expression.
#[derive(Clone, Debug, PartialEq)]
pub struct CMacroConstant {
    pub name: String,
    pub value: MacroValue,
}

#[derive(Clone, Debug, PartialEq)]
pub enum MacroValue {
    Int(i64),
    Uint(u64),
    Float(f64),
    String(String),
}

/// A C global variable declaration extracted from a header.
#[derive(Clone, Debug, PartialEq)]
pub struct CGlobalVar {
    pub name: String,
    pub ty: CType,
    pub is_const: bool,
}

/// A C pointer qualifier.
#[derive(Clone, Debug, PartialEq)]
pub enum CQualifier {
    Const,
    Volatile,
    Restrict,
}

/// Represents a C type extracted from a header.
/// This mirrors Kit's type system but preserves C naming.
#[derive(Clone, Debug, PartialEq)]
pub enum CType {
    // Primitive types
    Void,
    Char,
    Short,
    Int,
    Long,
    LongLong,
    Float,
    Double,
    LongDouble,
    Bool,
    SignedChar,
    UnsignedChar,
    UnsignedShort,
    UnsignedInt,
    UnsignedLong,
    UnsignedLongLong,

    // Sized integer types (via stdint.h)
    Int8,
    Int16,
    Int32,
    Int64,
    Uint8,
    Uint16,
    Uint32,
    Uint64,

    // Platform-specific
    SizeT,
    SSizeT,
    IntPtr,
    UintPtr,
    PtrDiffT,

    // Named type (struct, enum, typedef name)
    Named(String),

    // Pointer to a type
    Ptr(Box<CType>, Vec<CQualifier>),

    /// Function pointer type
    FunctionPtr {
        return_type: Box<CType>,
        param_types: Vec<CType>,
        is_variadic: bool,
    },

    /// Fixed-size array
    Array {
        element_type: Box<CType>,
        size: Option<usize>,
    },

    /// Unknown type (use as fallback)
    Unknown(String),
}

impl fmt::Display for CType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CType::Void => write!(f, "void"),
            CType::Char => write!(f, "char"),
            CType::Short => write!(f, "short"),
            CType::Int => write!(f, "int"),
            CType::Long => write!(f, "long"),
            CType::LongLong => write!(f, "long long"),
            CType::Float => write!(f, "float"),
            CType::Double => write!(f, "double"),
            CType::LongDouble => write!(f, "long double"),
            CType::Bool => write!(f, "_Bool"),
            CType::SignedChar => write!(f, "signed char"),
            CType::UnsignedChar => write!(f, "unsigned char"),
            CType::UnsignedShort => write!(f, "unsigned short"),
            CType::UnsignedInt => write!(f, "unsigned int"),
            CType::UnsignedLong => write!(f, "unsigned long"),
            CType::UnsignedLongLong => write!(f, "unsigned long long"),
            CType::Int8 => write!(f, "int8_t"),
            CType::Int16 => write!(f, "int16_t"),
            CType::Int32 => write!(f, "int32_t"),
            CType::Int64 => write!(f, "int64_t"),
            CType::Uint8 => write!(f, "uint8_t"),
            CType::Uint16 => write!(f, "uint16_t"),
            CType::Uint32 => write!(f, "uint32_t"),
            CType::Uint64 => write!(f, "uint64_t"),
            CType::SizeT => write!(f, "size_t"),
            CType::SSizeT => write!(f, "ssize_t"),
            CType::IntPtr => write!(f, "intptr_t"),
            CType::UintPtr => write!(f, "uintptr_t"),
            CType::PtrDiffT => write!(f, "ptrdiff_t"),
            CType::Named(name) => write!(f, "{name}"),
            CType::Ptr(inner, qualifiers) => {
                for q in qualifiers {
                    write!(f, "{q} ")?;
                }
                write!(f, "{}*", inner)
            }
            CType::FunctionPtr {
                return_type,
                param_types,
                is_variadic,
            } => {
                let params: Vec<String> = param_types.iter().map(ToString::to_string).collect();
                let suffix = if *is_variadic { ", ..." } else { "" };
                write!(f, "{} (*)({}{})", return_type, params.join(", "), suffix)
            }
            CType::Array { element_type, size } => match size {
                Some(s) => write!(f, "{}[{s}]", element_type),
                None => write!(f, "{}[]", element_type),
            },
            CType::Unknown(name) => write!(f, "/* unknown */ {name}"),
        }
    }
}

impl fmt::Display for CQualifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CQualifier::Const => write!(f, "const"),
            CQualifier::Volatile => write!(f, "volatile"),
            CQualifier::Restrict => write!(f, "restrict"),
        }
    }
}

/// The collection of all declarations extracted from a C header.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CDeclarations {
    pub functions: Vec<CFunction>,
    pub structs: Vec<CStruct>,
    pub unions: Vec<CUnion>,
    pub enums: Vec<CEnum>,
    pub typedefs: Vec<CTypedef>,
    pub macro_constants: Vec<CMacroConstant>,
    pub globals: Vec<CGlobalVar>,
}

impl CDeclarations {
    pub fn is_empty(&self) -> bool {
        self.functions.is_empty()
            && self.structs.is_empty()
            && self.unions.is_empty()
            && self.enums.is_empty()
            && self.typedefs.is_empty()
            && self.macro_constants.is_empty()
            && self.globals.is_empty()
    }

    /// Merge another set of declarations into this one.
    pub fn merge(&mut self, other: CDeclarations) {
        self.functions.extend(other.functions);
        self.structs.extend(other.structs);
        self.unions.extend(other.unions);
        self.enums.extend(other.enums);
        self.typedefs.extend(other.typedefs);
        self.macro_constants.extend(other.macro_constants);
        self.globals.extend(other.globals);
    }

    /// Look up a typedef by name.
    pub fn lookup_typedef(&self, name: &str) -> Option<&CType> {
        self.typedefs
            .iter()
            .find(|t| t.name == name)
            .map(|t| &t.underlying)
    }

    /// Look up a struct by name.
    pub fn lookup_struct(&self, name: &str) -> Option<&CStruct> {
        self.structs.iter().find(|s| s.name == name)
    }

    /// Look up functions by name (there can be multiple overloads in C, though unusual).
    pub fn lookup_functions(&self, name: &str) -> Vec<&CFunction> {
        self.functions.iter().filter(|f| f.name == name).collect()
    }
}

impl CType {
    /// Resolve a `Named` type through the typedef chain.
    /// Stops after `max_depth` steps to prevent infinite loops on cyclic typedefs.
    pub fn resolve_typedef<'a>(&'a self, declarations: &'a CDeclarations) -> &'a CType {
        let mut current = self;
        let mut visited = std::collections::HashSet::new();
        loop {
            match current {
                CType::Named(name) => {
                    if !visited.insert(name.as_str()) {
                        return current;
                    }
                    if let Some(underlying) = declarations.lookup_typedef(name) {
                        current = underlying;
                    } else {
                        return current;
                    }
                }
                _ => return current,
            }
        }
    }

    /// Check if this type is an integer type.
    pub fn is_integer(&self) -> bool {
        matches!(
            self,
            CType::Char
                | CType::Short
                | CType::Int
                | CType::Long
                | CType::LongLong
                | CType::SignedChar
                | CType::UnsignedChar
                | CType::UnsignedShort
                | CType::UnsignedInt
                | CType::UnsignedLong
                | CType::UnsignedLongLong
                | CType::Int8
                | CType::Int16
                | CType::Int32
                | CType::Int64
                | CType::Uint8
                | CType::Uint16
                | CType::Uint32
                | CType::Uint64
                | CType::SizeT
                | CType::SSizeT
                | CType::IntPtr
                | CType::UintPtr
                | CType::PtrDiffT
                | CType::Bool
        )
    }

    /// Check if this type is a float type.
    pub fn is_floating(&self) -> bool {
        matches!(self, CType::Float | CType::Double | CType::LongDouble)
    }

    /// Get the Kit type name string for this C type.
    /// This is used when registering the type in Kit's type system.
    pub fn to_kit_name(&self) -> String {
        match self {
            CType::Void => "Void".to_string(),
            CType::Char => "Char".to_string(),
            CType::Short => "Int16".to_string(),
            CType::Int => "Int".to_string(),
            CType::Long => "Int64".to_string(),
            CType::LongLong => "Int64".to_string(),
            CType::Float => "Float".to_string(),
            CType::Double => "Float64".to_string(),
            CType::LongDouble => "Float64".to_string(),
            CType::Bool => "Bool".to_string(),
            CType::SignedChar => "Int8".to_string(),
            CType::UnsignedChar => "Uint8".to_string(),
            CType::UnsignedShort => "Uint16".to_string(),
            CType::UnsignedInt => "Uint32".to_string(),
            CType::UnsignedLong => "Uint64".to_string(),
            CType::UnsignedLongLong => "Uint64".to_string(),
            CType::Int8 => "Int8".to_string(),
            CType::Int16 => "Int16".to_string(),
            CType::Int32 => "Int32".to_string(),
            CType::Int64 => "Int64".to_string(),
            CType::Uint8 => "Uint8".to_string(),
            CType::Uint16 => "Uint16".to_string(),
            CType::Uint32 => "Uint32".to_string(),
            CType::Uint64 => "Uint64".to_string(),
            CType::SizeT => "Size".to_string(),
            CType::SSizeT => "Int64".to_string(),
            CType::IntPtr => "Int64".to_string(),
            CType::UintPtr => "Uint64".to_string(),
            CType::PtrDiffT => "Int64".to_string(),
            CType::Named(name) => name.clone(),
            CType::Ptr(inner, _) => format!("Ptr({})", inner.to_kit_name()),
            CType::FunctionPtr { .. } => "/* function pointer */ Void".to_string(),
            CType::Array { element_type, .. } => format!("{}[]", element_type.to_kit_name()),
            CType::Unknown(name) => name.clone(),
        }
    }
}

/// Map a string type name from the C parser to our internal CType.
pub fn c_type_from_name(name: &str) -> Option<CType> {
    match name {
        "void" => Some(CType::Void),
        "char" => Some(CType::Char),
        "short" | "short int" => Some(CType::Short),
        "int" => Some(CType::Int),
        "long" | "long int" => Some(CType::Long),
        "long long" | "long long int" => Some(CType::LongLong),
        "float" => Some(CType::Float),
        "double" => Some(CType::Double),
        "long double" => Some(CType::LongDouble),
        "_Bool" | "bool" => Some(CType::Bool),
        "signed char" => Some(CType::SignedChar),
        "unsigned char" => Some(CType::UnsignedChar),
        "unsigned short" | "unsigned short int" => Some(CType::UnsignedShort),
        "unsigned" | "unsigned int" => Some(CType::UnsignedInt),
        "unsigned long" | "unsigned long int" => Some(CType::UnsignedLong),
        "unsigned long long" | "unsigned long long int" => Some(CType::UnsignedLongLong),
        "int8_t" => Some(CType::Int8),
        "int16_t" => Some(CType::Int16),
        "int32_t" => Some(CType::Int32),
        "int64_t" => Some(CType::Int64),
        "uint8_t" => Some(CType::Uint8),
        "uint16_t" => Some(CType::Uint16),
        "uint32_t" => Some(CType::Uint32),
        "uint64_t" => Some(CType::Uint64),
        "size_t" => Some(CType::SizeT),
        "ssize_t" => Some(CType::SSizeT),
        "intptr_t" => Some(CType::IntPtr),
        "uintptr_t" => Some(CType::UintPtr),
        "ptrdiff_t" => Some(CType::PtrDiffT),
        _ => None,
    }
}
