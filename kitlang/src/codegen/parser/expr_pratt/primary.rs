// Primary expression parsers. Included into `mod.rs` via `include!`.
// This file is in the same module scope as `mod.rs`, so imports are
// inherited and all `ExprParser` private methods are accessible.

impl<'a> ExprParser<'a> {
    /// Iteratively apply postfix operators (call, index, field access) to
    /// a base expression. Zero stack frames added per iteration. The
    /// chain is bounded by the source's syntactic length, but the parser
    /// is iterative, so the *call stack* depth is constant.
    pub(crate) fn parse_postfix_chain(
        &mut self,
        mut base: Expr,
    ) -> Result<Expr, ExprParseError> {
        loop {
            let kind = self.peek().kind.clone();
            if postfix(&kind).is_none() {
                break;
            }
            base = match kind {
                Tok::Dot => self.parse_field_access(base)?,
                Tok::LBracket => self.parse_index(base)?,
                Tok::LParen => self.parse_call(base)?,
                _ => unreachable!("postfix returned Some for {kind:?}"),
            };
        }
        Ok(base)
    }

    /// Parse a primary expression: literals, identifiers, parenthesized
    /// expressions, function calls, array literals, struct inits, and
    /// the if-expression. Postfix operations (`.field`, `[i]`, `(args)`)
    /// are handled in the outer Pratt loop, *not* here, so this function
    /// only needs to produce the base expression.
    pub(crate) fn parse_primary(&mut self) -> Result<Expr, ExprParseError> {
        let tok = self.peek().kind.clone();
        // `span` is read for documentation; the parser doesn't currently
        // attach it to AST nodes. Future PRs will use it.
        let _span: Span = self.peek().span.clone();

        match tok {
            Tok::IntLit(n) => {
                self.advance();
                Ok(Expr::Literal {
                    value: Literal::Int(n),
                    ty: TypeId::default(),
                })
            }
            Tok::FloatLit(f) => {
                self.advance();
                Ok(Expr::Literal {
                    value: Literal::Float(f),
                    ty: TypeId::default(),
                })
            }
            Tok::CharLit(c) => {
                self.advance();
                Ok(Expr::Literal {
                    value: Literal::Char(c),
                    ty: TypeId::default(),
                })
            }
            Tok::StringLit(s) => {
                self.advance();
                Ok(Expr::Literal {
                    value: Literal::String(s),
                    ty: TypeId::default(),
                })
            }
            Tok::KwTrue => {
                self.advance();
                Ok(Expr::Literal {
                    value: Literal::Bool(true),
                    ty: TypeId::default(),
                })
            }
            Tok::KwFalse => {
                self.advance();
                Ok(Expr::Literal {
                    value: Literal::Bool(false),
                    ty: TypeId::default(),
                })
            }
            Tok::KwNull => {
                self.advance();
                Ok(Expr::Literal {
                    value: Literal::Null,
                    ty: TypeId::default(),
                })
            }
            Tok::KwThis | Tok::KwSelf => {
                let name = match tok {
                    Tok::KwThis => "this",
                    _ => "Self",
                };
                self.advance();
                Ok(Expr::Identifier {
                    name: name.to_string(),
                    ty: TypeId::default(),
                })
            }
            Tok::Ident(name) => {
                self.advance();
                Ok(Expr::Identifier {
                    name,
                    ty: TypeId::default(),
                })
            }
            Tok::LParen => {
                self.advance(); // consume `(`
                let first = self.parse_expr()?;
                if self.peek().kind == Tok::Comma {
                    return Err(ExprParseError::Custom(
                        "tuple literals are not yet supported by the Pratt parser".into(),
                    ));
                }
                self.expect(&Tok::RParen)?;
                Ok(first)
            }
            Tok::LBracket => self.parse_array_literal(),
            Tok::KwStruct => self.parse_struct_init(),
            Tok::KwIf => self.parse_if_expr(),
            Tok::KwEmpty => {
                self.advance();
                Ok(Expr::Identifier {
                    name: "empty".to_string(),
                    ty: TypeId::default(),
                })
            }
            _ => {
                if self.at_eof() {
                    Err(ExprParseError::UnexpectedEof {
                        expected: &[
                            "integer literal",
                            "float literal",
                            "string literal",
                            "char literal",
                            "identifier",
                            "`(`",
                            "`[`",
                            "`if`",
                            "`null`",
                            "`true`",
                            "`false`",
                        ],
                    })
                } else {
                    Err(ExprParseError::UnexpectedToken {
                        found: tok,
                        expected: &[
                            "integer literal",
                            "float literal",
                            "string literal",
                            "char literal",
                            "identifier",
                            "`(`",
                            "`[`",
                            "`if`",
                            "`null`",
                            "`true`",
                            "`false`",
                        ],
                    })
                }
            }
        }
    }

    /// Parse a `.field` access postfix.
    fn parse_field_access(&mut self, base: Expr) -> Result<Expr, ExprParseError> {
        self.advance(); // consume `.`
        let field_tok = self.peek().kind.clone();
        match field_tok {
            Tok::Ident(name) => {
                self.advance();
                Ok(Expr::FieldAccess {
                    expr: Box::new(base),
                    field_name: name,
                    ty: TypeId::default(),
                })
            }
            _ => Err(ExprParseError::UnexpectedToken {
                found: field_tok,
                expected: &["identifier"],
            }),
        }
    }

    /// Parse a `[index]` postfix.
    fn parse_index(&mut self, base: Expr) -> Result<Expr, ExprParseError> {
        self.advance(); // consume `[`
        let index = self.parse_expr()?;
        self.expect(&Tok::RBracket)?;
        Ok(Expr::Index {
            expr: Box::new(base),
            index: Box::new(index),
            ty: TypeId::default(),
        })
    }

    /// Parse a function call postfix: `(arg1, arg2, ...)`.
    /// The callee is any expression; no rejection of indirect calls.
    fn parse_call(&mut self, callee: Expr) -> Result<Expr, ExprParseError> {
        self.advance(); // consume `(`
        let args = self.parse_comma_list(Tok::RParen, |p| p.parse_expr())?;
        Ok(Expr::Call {
            callee: Box::new(callee),
            args,
            ty: TypeId::default(),
        })
    }

    /// Parse an array literal: `[expr, expr, ...]`.
    fn parse_array_literal(&mut self) -> Result<Expr, ExprParseError> {
        self.advance(); // consume `[`
        let elements = self.parse_comma_list(Tok::RBracket, |p| p.parse_expr())?;
        Ok(Expr::ArrayLiteral {
            elements,
            ty: TypeId::default(),
        })
    }

    /// Parse a struct init: `struct Name { field: expr, ... }`.
    fn parse_struct_init(&mut self) -> Result<Expr, ExprParseError> {
        self.advance(); // consume `struct`
        let type_tok = self.peek().kind.clone();
        let type_name = match type_tok {
            Tok::Ident(name) => {
                self.advance();
                name
            }
            _ => {
                return Err(ExprParseError::UnexpectedToken {
                    found: type_tok,
                    expected: &["type name"],
                });
            }
        };
        self.expect(&Tok::LBrace)?;
        let fields = self.parse_comma_list(Tok::RBrace, |p| {
            let name = match &p.peek().kind {
                Tok::Ident(n) => {
                    let n = n.clone();
                    p.advance();
                    n
                }
                _ => {
                    return Err(ExprParseError::UnexpectedToken {
                        found: p.peek().kind.clone(),
                        expected: &["field name"],
                    });
                }
            };
            p.expect(&Tok::Colon)?;
            let value = p.parse_expr()?;
            Ok(FieldInit { name, value })
        })?;
        Ok(Expr::StructInit {
            ty: TypeId::default(),
            struct_type: Some(Type::from_kit(&type_name)),
            fields,
        })
    }

    /// Parse an if-expression: `if cond then a else b`.
    fn parse_if_expr(&mut self) -> Result<Expr, ExprParseError> {
        self.advance(); // consume `if`
        let cond = self.parse_expr()?;
        self.expect(&Tok::KwThen)?;
        let then_branch = self.parse_expr()?;
        self.expect(&Tok::KwElse)?;
        let else_branch = self.parse_expr()?;
        Ok(Expr::If {
            cond: Box::new(cond),
            then_branch: Box::new(then_branch),
            else_branch: Box::new(else_branch),
            ty: TypeId::default(),
        })
    }
}
