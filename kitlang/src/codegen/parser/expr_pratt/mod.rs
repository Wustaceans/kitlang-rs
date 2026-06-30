//! Hand-written Pratt parser for Kit expressions.
//!
//! This module takes over expression parsing from the pest-based grammar.
//! Pest still handles the program, declaration, statement, and type-annotation
//! grammars. For every `Pair<'_, Rule>` whose rule is an expression (i.e.,
//! the 13 precedence levels `expr → assign → logical_or → ... → primary`),
//! the parser hands off to [`ExprParser::parse_expr`].
//!
//! Pratt Parsing & Operator Precedence
//! -----------------------------------
//! The pest grammar used 13 mutually recursive functions (one per precedence
//! level), which overflowed the default 1 MB stack on Windows. The Pratt
//! parser uses a single function with a binding-power loop, bounding stack
//! depth to `O(precedence levels) = O(13)` regardless of expression length.
//! Operator precedence is defined in [`binding_power::infix`],
//! [`binding_power::postfix`], [`binding_power::prefix`]. Each infix operator
//! has a (lbp, rbp) pair: `lbp < rbp` = right-associative (assignment),
//! `lbp == rbp` = left-associative (most binary ops).
//!
//! Errors & Source Spans
//! ---------------------
//! All parse errors are values of [`ExprParseError`] (in `diagnostics.rs`).
//! The parser never prints, never allocates strings for error messages,
//! and never holds source-file identity. Conversion to
//! `CompilationError::ParseError(String)` happens at `PestExpr::parse`
//! in `parser/mod.rs`. The token stream carries byte ranges; the parser
//! uses them internally but does not currently attach them to AST nodes.

use crate::codegen::ast::{Expr, Literal};
use crate::codegen::type_ast::FieldInit;
use crate::codegen::types::{Type, TypeId};
use crate::lexer::{Span, SpannedTok, Tok, tokenize};

use super::binding_power::{
    infix, is_range_op, postfix, prefix, tok_to_assign_op, tok_to_binary_op, tok_to_unary_op,
};
use super::diagnostics::{ExprParseError, expected_name};

// ---------------------------------------------------------------------------
// Parser
// ---------------------------------------------------------------------------

/// A Pratt parser for Kit expressions.
///
/// One instance is built per expression being parsed. The parser is
/// single-use: create a new one for each `parse_expr` call.
pub(crate) struct ExprParser<'a> {
    tokens: &'a [SpannedTok],
    pos: usize,
}

impl<'a> ExprParser<'a> {
    /// Build a parser over a token slice. The slice must be the result of
    /// [`tokenize`] applied to the expression's source text.
    pub(crate) fn new(tokens: &'a [SpannedTok]) -> Self {
        Self { tokens, pos: 0 }
    }

    /// Entry point. Parses one complete expression and returns it.
    /// The expression may be followed by trailing tokens, which are left
    /// in the stream (the caller can `pos()`-check or re-parse).
    pub(crate) fn parse_expr(&mut self) -> Result<Expr, ExprParseError> {
        self.parse_pratt(0)
    }

    /// The current position in the token stream. Useful for tests and
    /// for callers that want to know how many tokens were consumed.
    pub(crate) fn pos(&self) -> usize {
        self.pos
    }

    // --- Token stream helpers (private) ---

    /// Peek the current token without consuming it. Returns a synthetic
    /// "EOF" token when the stream is exhausted.
    fn peek(&self) -> &SpannedTok {
        // We never return None from peek; EOF is represented by a synthetic
        // SpannedTok at span 0..0. This lets the Pratt loop compare with
        // `==` against `Tok::...` cleanly.
        static EOF: SpannedTok = SpannedTok {
            kind: Tok::Semi, // any token that has no infix/postfix/prefix bp
            span: 0..0,
        };
        self.tokens.get(self.pos).unwrap_or(&EOF)
    }

    /// Peek the next token (one past the current). Same EOF behavior.
    fn peek_next(&self) -> &SpannedTok {
        static EOF: SpannedTok = SpannedTok {
            kind: Tok::Semi,
            span: 0..0,
        };
        self.tokens.get(self.pos + 1).unwrap_or(&EOF)
    }

    /// Consume and return the current token.
    fn advance(&mut self) -> &SpannedTok {
        let tok = &self.tokens[self.pos];
        self.pos += 1;
        tok
    }

    /// True if the parser is at or past the end of the token stream.
    fn at_eof(&self) -> bool {
        self.pos >= self.tokens.len()
    }

    // --- Pratt core ---

    /// Parse an expression with the given minimum binding power.
    fn parse_pratt(&mut self, min_bp: u8) -> Result<Expr, ExprParseError> {
        // Parse leading prefix operators (e.g. `-a`, `!!x`, `&arr[i]`).
        // Prefix binds tighter than infix but looser than postfix,
        // so `&arr[i]` = `&(arr[i])` (postfix on `arr` first).
        let mut lhs = if let Some(pfx_bp) = prefix(&self.peek().kind) {
            let op = tok_to_unary_op(&self.peek().kind).unwrap();
            self.advance();
            let rhs = self.parse_pratt(pfx_bp)?;
            Expr::UnaryOp {
                op,
                expr: Box::new(rhs),
                ty: TypeId::default(),
            }
        } else {
            self.parse_primary()?
        };

        // Postfix chain: field access, index, call.
        lhs = self.parse_postfix_chain(lhs)?;

        // Infix operators (binary ops, range).
        loop {
            let kind = self.peek().kind.clone();
            let Some((lbp, rbp)) = infix(&kind) else {
                break;
            };
            if lbp < min_bp {
                break;
            }
            if tok_to_assign_op(&kind).is_some() {
                break;
            }
            self.advance();
            if is_range_op(&kind) {
                let rhs = self.parse_pratt(rbp)?;
                lhs = Expr::RangeLiteral {
                    start: Box::new(lhs),
                    end: Box::new(rhs),
                };
                continue;
            }
            let op = tok_to_binary_op(&kind).ok_or_else(|| {
                ExprParseError::Custom(format!("internal: no binary op for {kind:?}"))
            })?;
            let rhs = self.parse_pratt(rbp)?;
            lhs = Expr::BinaryOp {
                op,
                left: Box::new(lhs),
                right: Box::new(rhs),
                ty: TypeId::default(),
            };
        }

        // Assignment (right-associative, lowest precedence).
        loop {
            let kind = self.peek().kind.clone();
            let Some(op) = tok_to_assign_op(&kind) else {
                break;
            };
            // Assignment has the lowest precedence (lbp=0, rbp=1 in the
            // infix table, but we don't use that table here). Right-
            // associativity: a = b = c means a = (b = c). Recurse with
            // min_bp=0 so the rhs sees *all* operators including another
            // assignment.
            if 0 < min_bp {
                break;
            }
            self.advance();
            let rhs = self.parse_pratt(0)?;
            lhs = Expr::Assign {
                op,
                left: Box::new(lhs),
                right: Box::new(rhs),
                ty: TypeId::default(),
            };
        }

        Ok(lhs)
    }

    // --- Helpers ---

    /// Consume the current token if it matches `expected`, otherwise
    /// return an `UnexpectedToken` error with `expected`'s name.
    fn expect(&mut self, expected: &Tok) -> Result<(), ExprParseError> {
        if &self.peek().kind == expected {
            self.advance();
            Ok(())
        } else {
            Err(ExprParseError::UnexpectedToken {
                found: self.peek().kind.clone(),
                expected: expected_name(expected),
            })
        }
    }

    /// Parse a comma-separated list of `T` terminated by `closer`.
    /// Allows zero or more elements (an empty list is valid for fn
    /// calls with no args, empty array literals, etc.).
    fn parse_comma_list<T, F>(&mut self, closer: Tok, mut f: F) -> Result<Vec<T>, ExprParseError>
    where
        F: FnMut(&mut Self) -> Result<T, ExprParseError>,
    {
        let mut out = Vec::new();
        // Empty list case.
        if self.peek().kind == closer {
            self.advance();
            return Ok(out);
        }
        out.push(f(self)?);
        while self.peek().kind == Tok::Comma {
            self.advance();
            // Trailing comma is allowed (parses to empty trailing element).
            if self.peek().kind == closer {
                break;
            }
            out.push(f(self)?);
        }
        self.expect(&closer)?;
        Ok(out)
    }
}

/// Extract a callee name from a function-call's leading expression.
///
/// For `Expr::Identifier { name, .. }` this is just `name`.
/// For `Expr::FieldAccess { expr, field_name, .. }` chains like
/// `pkg.math.add`, this concatenates the path with `.`.
/// For other expressions (e.g. a parenthesized call), we fall back to
/// `Display`-formatting the expression, which the transpiler can still
/// route through name resolution.
fn expr_to_callee_name(expr: &Expr) -> String {
    match expr {
        Expr::Identifier { name, .. } => name.clone(),
        Expr::FieldAccess {
            expr: base,
            field_name,
            ..
        } => {
            let base_name = expr_to_callee_name(base);
            format!("{base_name}.{field_name}")
        }
        other => format!("{other:?}"),
    }
}

// ---------------------------------------------------------------------------
// Module surface: parse an expression from source text.
// ---------------------------------------------------------------------------

/// Parse a Kit expression from source text. This is the public entry
/// point used by the pest-to-Pratt bridge (`PestExpr::parse`).
///
/// The `text` should be the source text of the expression as a
/// `Pair::as_str()` slice. Tokenization, parsing, and conversion to an
/// `Expr` all happen here.
pub(crate) fn parse_kit_expr(text: &str) -> Result<Expr, ExprParseError> {
    let tokens = tokenize(text);
    let mut parser = ExprParser::new(&tokens);
    parser.parse_expr()
}

// ---------------------------------------------------------------------------
// Primary expression parsers (include'd into this module scope).
// ---------------------------------------------------------------------------
include!("primary.rs");

// ---------------------------------------------------------------------------
// Tests (include'd into this module scope, test builds only).
// ---------------------------------------------------------------------------
#[cfg(test)]
include!("tests.rs");
