use ast::Ast;
use lc_ast as ast;

pub struct MacroExpander {}

impl MacroExpander {
    pub fn new() -> Self {
        Self {}
    }

    pub fn expand(self, ast: Ast) -> Ast {
        ast
    }
}

impl Default for MacroExpander {
    fn default() -> Self {
        Self::new()
    }
}

impl ast::Visitor<'_> for MacroExpander {
}
