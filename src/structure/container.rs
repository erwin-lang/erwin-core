use crate::structure::ast::Visibility;

pub(crate) struct Container<'a> {
    visibility: &'a Visibility,
    type_symbols: Vec<&'a str>,
}
