use ospl_common::ast::{module::VPathError, repr::Type};

#[derive(Debug)]
pub enum CompilerError {
    AtLeastOneScopeRequired,
    NotFoundInScope {
        id: String
    },
    ObjectHasNoProperties,
    MismatchedType {
        expected: Type,
        got: Type
    },
    TypeNotCallable,
    GenericFIXME,
    VPath(VPathError),
    WrongArgCount {
        expected: usize,
        got: usize
    },
    DeclaringPropertiesIsIllegal {
        property: String
    },
    InstantiationOfClassWithUnresolvedDeclares,
}

impl From<VPathError> for CompilerError {
    fn from(value: VPathError) -> Self {
        return Self::VPath(value)
    }
}

pub struct Eval {
    pub index: usize,
    pub ty: Type,
}

pub type Res<T> = Result<T, CompilerError>;
