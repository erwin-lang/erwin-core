use std::collections::HashMap;

use crate::structure::{ast::Visibility, types::Type};

#[derive(Debug)]
pub(crate) struct ModuleTable<'a> {
    pub(crate) registry: HashMap<&'a str, Entry<'a>>,
    pub(crate) symbols: HashMap<&'a str, Symbol<'a>>,
    pub(crate) containers: HashMap<&'a str, Container<'a>>,
}

#[derive(Debug)]
pub(crate) struct Entry<'a> {
    pub(crate) visibility: &'a Visibility,
    pub(crate) symbols: HashMap<&'a str, Symbol<'a>>,
}

#[derive(Debug)]
pub(crate) struct Container<'a> {
    pub(crate) visibility: &'a Visibility,
    pub(crate) registry: HashMap<&'a str, Entry<'a>>,
}

#[derive(Clone, Debug)]
pub(crate) struct Symbol<'a> {
    pub(crate) ty: Type<'a>,
    pub(crate) visibility: &'a Visibility,
    pub(crate) is_static_member: bool,
    pub(crate) is_mutable: bool,
}
