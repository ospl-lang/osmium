#[derive(Debug, Clone, Eq, Hash)]
pub enum Type {
    Nul, Undefined,

    /// A type-erased type.
    /// 
    /// It has an unknown runtime type. It cannot be operated upon. Types
    /// can only be promoted to unknown, but the original type cannot be
    /// recovered.
    Unknown,

    /// [`Self::Unknown`]'s non-erased counterpart.
    /// 
    /// It has a runtime type, but it could be any type, and OSPL doesn't
    /// care what type that is.
    Any,

    /// A list of types
    Union(Vec<Type>),

    /// A type whose ID matters in checking.
    Nominal(usize, Box<Self>),

    Int, Address, Float, Char, Str, Bool, List(Box<Self>),
    Scope(super::Scope<Self>),
    Function(Box<super::FunctionType<Self>>),

    ForeignLibrary,
    ForeignFunction(Vec<Self>, Box<Self>),
}

impl PartialEq for Type {
    fn eq(&self, t2: &Type) -> bool {
        return match (self, t2) {
            // special rule: Unknown and Any match everything
            (Type::Unknown, _) | (_, Type::Unknown) => true,
            (Type::Any,     _) | (_, Type::Any) => true,
            (Type::Union(u), has) => u.contains(has),
            (has, Type::Union(u)) => u.contains(has),  // maybe remove this arm later?

            (Type::Nominal(id1, _), Type::Nominal(id2, _)) => id1 == id2,

            // special rule: Undefined only matches itself
            // but is legal to compare to all other objects
            (Type::Undefined, Type::Undefined) => true,
            (Type::Undefined, _) => false,
            (_, Type::Undefined) => false,

            // normal structural equality
            (Type::Nul, Type::Nul) => true,
            (Type::Int, Type::Int) => true,
            (Type::Address, Type::Address) => true,
            (Type::Float, Type::Float) => true,
            (Type::Str, Type::Str) => true,
            (Type::Char, Type::Char) => true,
            (Type::Bool, Type::Bool) => true,

            (Type::List(a), Type::List(b)) => a == b,
            (Type::Scope(a), Type::Scope(b)) => a == b,
            (Type::Function(a), Type::Function(b)) => a == b,

            (Type::ForeignLibrary, Type::ForeignLibrary) => true,

            (Type::ForeignFunction(args1, ret1), Type::ForeignFunction(args2, ret2)) => {
                args1 == args2 && ret1 == ret2
            }

            _ => false,
        };
    }
}

impl Type {
    pub fn is_indexable(&self) -> bool {
        return matches!(self, Self::Str | Self::List(_))
    }

    pub fn is_sliceable(&self) -> bool {
        return matches!(self, Self::Str | Self::List(_))
    }

    pub fn to_primitive_type_id(&self) -> usize {
        match self {
            Self::Nul => 0,
            Self::Int => 1,
            Self::Address => 2,
            Self::Float => 3,
            Self::Str => 4,
            Self::Undefined => 5,
            Self::Char => 6,

            t => panic!("cannot call .to_primitive_type_id() on a non-primitive type {t:?}"),
        }
    }

    pub fn from_primitive_type_id(p: usize) -> Self {
        match p {
            0 => Self::Nul,
            1 => Self::Int,
            2 => Self::Address,
            3 => Self::Float,
            4 => Self::Str,
            5 => Self::Undefined,
            6 => Self::Char,

            t => panic!("unknown primitive type ID: {t}")
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum UType {
    Resolved(Type),
    Nominal(Box<UType>),
    Typeof(String),
    Property(Box<Self>, String),
    Returnof(Box<Self>),
    Function(Box<super::FunctionType<Self>>),
    List(Box<Self>),
    Scope(super::Scope<Self>),
    InferScope,
}
