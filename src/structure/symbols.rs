use std::collections::HashMap;

use crate::structure::{ast::Visibility, types::Type};

pub(crate) struct ModuleTable<'a> {
    pub(crate) registry: HashMap<&'a str, StaticEntry<'a>>,
    pub(crate) symbols: Scope<'a>,
}

pub(crate) struct StaticEntry<'a> {
    pub(crate) visibility: &'a Visibility,
    pub(crate) members: HashMap<&'a str, ScopedSymbol<'a>>,
}

pub(crate) struct Scope<'a> {
    pub(crate) symbols: HashMap<&'a str, ScopedSymbol<'a>>,
}

#[derive(Clone)]
pub(crate) struct ScopedSymbol<'a> {
    pub(crate) ty: Type<'a>,
    pub(crate) visibility: &'a Visibility,
    pub(crate) is_static_member: bool,
}
