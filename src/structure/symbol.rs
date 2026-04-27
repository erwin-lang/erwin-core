use crate::structure::{ast::Visibility, types::Type};

pub(crate) struct ScopeSymbol<'a> {
    pub(crate) visibility: &'a Visibility,
    pub(crate) ty: Type<'a>,
}
