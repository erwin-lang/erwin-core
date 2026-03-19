use crate::{
    error::Error,
    parser::Parser,
    syntax::{
        ast::{Statement, VarKind, Visibility},
        token::TokenKind,
    },
};

impl<'a> Parser<'a> {
    pub(super) fn parse_statement(&mut self) -> Result<Statement<'a>, Error> {
        match self.peek(0)?.kind {
            TokenKind::Pub => self.parse_visibility(),
            TokenKind::Import => self.parse_import(),
            TokenKind::Var => self.parse_var_def(VarKind::Var, Visibility::Priv),
            TokenKind::Const => self.parse_var_def(VarKind::Const, Visibility::Priv),
            TokenKind::Node => self.parse_node_def(Visibility::Priv),
            TokenKind::Func => self.parse_func_def(Visibility::Priv),
            TokenKind::For => self.parse_for(),
            TokenKind::While => self.parse_while(),
            TokenKind::If => self.parse_if(),
            TokenKind::Obj => self.parse_obj_def(Visibility::Priv),
            _ => self.parse_statement_expr(),
        }
    }

    pub(super) fn parse_import(&mut self) -> Result<Statement<'a>, Error> {
        self.advance()?;

        let stmt = Statement::Import {
            path: self.parse_path()?,
        };

        if !matches!(self.peek(0)?.kind, TokenKind::Semicolon) {
            return self.error("Expected ';'");
        }

        self.advance()?;
        Ok(stmt)
    }

    pub(super) fn parse_var_def(
        &mut self,
        kind: VarKind,
        visibility: Visibility,
    ) -> Result<Statement<'a>, Error> {
        self.advance()?;

        let identifier = if let TokenKind::Identifier(name) = self.peek(0)?.kind {
            name
        } else {
            return self.error("Expected variable name");
        };
        self.advance()?;

        let ty = if matches!(self.peek(0)?.kind, TokenKind::Colon) {
            self.advance()?;
            Some(self.parse_type()?)
        } else {
            None
        };

        let value = if matches!(self.peek(0)?.kind, TokenKind::Assign) {
            self.advance()?;
            self.parse_expr()?
        } else {
            return self.error("Expected assignment");
        };

        let stmt = Statement::VarDeclare {
            visibility,
            kind,
            identifier,
            ty,
            value,
        };

        if !matches!(self.peek(0)?.kind, TokenKind::Semicolon) {
            return self.error("Expected ';'");
        }

        self.advance()?;
        Ok(stmt)
    }

    pub(super) fn parse_node_def(
        &mut self,
        visibility: Visibility,
    ) -> Result<Statement<'a>, Error> {
        self.advance()?;

        let identifier = if let TokenKind::Identifier(name) = self.peek(0)?.kind {
            self.advance()?;
            name
        } else {
            return self.error("Expected node name");
        };

        let ty = if matches!(self.peek(0)?.kind, TokenKind::Colon) {
            self.advance()?;
            Some(self.parse_type()?)
        } else {
            None
        };

        let value = if matches!(self.peek(0)?.kind, TokenKind::Assign) {
            self.advance()?;
            self.parse_expr()?
        } else {
            return self.error("Expected expression");
        };

        let mut body = Vec::new();

        match self.peek(0)?.kind {
            TokenKind::Semicolon => self.advance()?,
            TokenKind::LBrace => {
                body = self.parse_block()?;
            }
            _ => return self.error("Expected ';' or '{'"),
        }

        Ok(Statement::NodeDeclare {
            visibility,
            identifier,
            ty,
            value,
            body,
        })
    }

    pub(super) fn parse_func_def(
        &mut self,
        visibility: Visibility,
    ) -> Result<Statement<'a>, Error> {
        self.advance()?;

        let id = if let TokenKind::Identifier(name) = self.peek(0)?.kind {
            self.advance()?;
            name
        } else {
            return self.error("Expected function name");
        };

        if !matches!(self.peek(0)?.kind, TokenKind::LParen) {
            return self.error("Expected '('");
        }

        self.advance()?;
        let params = self.parse_comma_separated(|parser| parser.parse_param())?;

        if !matches!(self.peek(0)?.kind, TokenKind::RParen) {
            return self.error("Expected ')'");
        }

        self.advance()?;

        if !matches!(self.peek(0)?.kind, TokenKind::RArrow) {
            return self.error("Expected '->'");
        }

        self.advance()?;

        let ty = self.parse_type()?;
        let body = self.parse_block()?;

        Ok(Statement::Func {
            visibility,
            identifier: id,
            params,
            return_ty: ty,
            body,
        })
    }

    pub(super) fn parse_for(&mut self) -> Result<Statement<'a>, Error> {}

    pub(super) fn parse_while(&mut self) -> Result<Statement<'a>, Error> {}

    pub(super) fn parse_if(&mut self) -> Result<Statement<'a>, Error> {}

    pub(super) fn parse_obj_def(&mut self, visibility: Visibility) -> Result<Statement<'a>, Error> {
    }

    pub(super) fn parse_statement_expr(&mut self) -> Result<Statement<'a>, Error> {}

    pub(super) fn parse_visibility(&mut self) -> Result<Statement<'a>, Error> {
        self.advance()?;
        match self.peek(0)?.kind {
            TokenKind::Var => self.parse_var_def(VarKind::Var, Visibility::Pub),
            TokenKind::Const => self.parse_var_def(VarKind::Const, Visibility::Pub),
            TokenKind::Node => self.parse_node_def(Visibility::Pub),
            TokenKind::Func => self.parse_func_def(Visibility::Pub),
            TokenKind::Obj => self.parse_obj_def(Visibility::Pub),
            TokenKind::Pub => self.error("Repeated visibility modifier"),
            _ => self.error("Invalid visibility modifier"),
        }
    }
}
