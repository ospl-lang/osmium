use ospl_common::nicities::fmt_type;

use crate::CE;

pub fn simplify(e: &CE) -> String {
    match &e.error {
        crate::CEData::Bug => "A bug in OSPL was triggered".to_string(),
        crate::CEData::InternalError(_) => "An internal error occured".to_string(),
        crate::CEData::InvalidAssignOp { op } => {
            return format!("Invalid assign operation {op:?}")
        },  // mismatched types, expected String, found ()
        crate::CEData::MismatchedTypes { expected, got } => {
            let exp: String = match expected {
                crate::TypeExpectation::AnyList => "list of any type".to_string(),
                crate::TypeExpectation::AnyScope => "scope of any type".to_string(),
                crate::TypeExpectation::Exact(e) => fmt_type(e.clone()),
                crate::TypeExpectation::Indexable => "indexable types (list, str)".to_string(),
                crate::TypeExpectation::Slicable => "indexable types (list, str)".to_string(),
            };

            let got = fmt_type(got.clone());

            return format!("\
Type mismatch
### expected
{exp}

### got
{got}
            ")
        },
        crate::CEData::UnionDoesntHaveType { union, doesnt_have } => {
            let u = fmt_type(union.clone());
            let got = fmt_type(doesnt_have.clone());
            return format!("\
Union mismatch
### expected one of
{u}

### got
{got}
                ")
        },
        crate::CEData::SafeUnionUsingNonNominal { union, non_nominal } => {
            let u = fmt_type(union.clone());
            let got = fmt_type(non_nominal.clone());
            return format!("\
Safe unions must use nominal types
### union
{u}

### attempted non-nominal cast
{got}
                ")
        },
        crate::CEData::NotFoundInScope { needed, scope } => {
            let got = fmt_type(ospl_common::ast::Type::Scope(scope.clone()));
            return format!("
needed: {needed}

### attempted access on this scope
{got}
            ")
        },
        oth => format!("{oth:?}")  // temp
    }
}