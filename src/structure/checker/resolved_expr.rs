use crate::structure::checker::{
    literal::Literal,
    module_table::{Container, ScopeSymbol, Type, TypeSymbol},
};

#[derive(Debug, Clone)]
pub(crate) enum ResolvedExpr<'a> {
    Type(&'a Type<'a>), // Example: (Str, Int32); std::math::Var; (only used in explicit typing like vars and funcs)
    ScopeSymbol(&'a ScopeSymbol<'a>), // Example: foo; std::math::PI; std::math::fib(); (used everywhere)
    TypeSymbol(&'a TypeSymbol<'a>),   // Example: Tuple; Array; Int32; (Used everywhere)
    Container(&'a Container<'a>), // Example: GenericInt; GenericFloat; (only used in method blocks)
    Literal(Literal<'a>),
}
