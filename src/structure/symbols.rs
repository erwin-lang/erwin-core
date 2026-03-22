use crate::structure::types::Type;

pub(crate) struct Symbol<'a> {
    pub(crate) id: &'a str,
    pub(crate) ty: Type<'a>,
}
