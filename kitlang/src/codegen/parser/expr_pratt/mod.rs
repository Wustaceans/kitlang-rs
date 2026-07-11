//! Hand-written Pratt parser for Kit expressions.
//!
//! This module takes over expression parsing from the pest-based grammar.
//!
//! Pest still handles the program, declaration, statement, and type-annotation grammars. For every
//! `Pair<'_, Rule>` whose rule is an expression (i.e. the 13 precedence levels `expr -> assign ->
//! logical_or -> ... -> primary`), the parser hands off to [`ExprParser::parse_expr`].
//!
//! Pratt Parsing & Operator Precedence
//! -----------------------------------
//! The pest grammar used 13 mutually recursive functions (one per precedence level, which
//! overflowed the default 1 MB stack on Windows.
//!
//! The Pratt parser uses a single function with a binding-power loop, dramatically reducing
//! recursive depth compared to the old 13-level grammar chain.
//!
//! Parenthesized sub-expressions still recurse through `parse_primary` and `parse_expr`, so stack
//! depth is `O(parenthetical nesting)`. Operator precedence is defined in
//! [`binding_power::infix`], [`binding_power::postfix`], [`binding_power::prefix`].
//!
//! Each infix operator has a (lbp, rbp) pair: `lbp < rbp` = right-associative (assignment),
//! `lbp == rbp` = left-associative (most binary ops).
//!
//! Errors & Source Spans
//! ---------------------
//! All parse errors are values of [`ExprParseError`] (in `diagnostics.rs`).
//!
//! The parser never prints, never allocates strings for error messages, and never holds
//! source-file identity. Conversion to `Compilation::ParseError(String)` happens at
//! `PestExpr::parse` in `parser/mod.rs`.
//!
//! The token stream carries byte ranges; the parser uses them internally but does not currently
//! attach them to AST nodes.

use crate::codegen::ast::{Expr, Literal};
use crate::codegen::type_ast::FieldInit;
use crate::codegen::types::{Type, TypeId};
use crate::lexer::{LexicalError, Span, SpannedTok, Tok, tokenize};

use super::binding_power::{
    infix, is_range_op, postfix, prefix, tok_to_assign_op, tok_to_binary_op, tok_to_unary_op,
};
use super::diagnostics::{ExprParseError, expected_name};

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
    /// Callers must check `pos()` against the token length to ensure
    /// no trailing tokens were left unparsed.
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

        // Infix operators (binary ops, range)
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
        // Right-associativity requires the RHS to be parsed at the same
        // binding power as the operator's lbp, so `a = b = c` becomes
        // `a = (b = c)`. We use `lbp` (not `rbp` from the table) since
        // assignment needs `rbp <= lbp` for the RHS to see the operator.
        loop {
            let kind = self.peek().kind.clone();
            let Some(op) = tok_to_assign_op(&kind) else {
                break;
            };
            let Some((lbp, _rbp)) = infix(&kind) else {
                break;
            };
            if lbp < min_bp {
                break;
            }
            self.advance();
            let rhs = self.parse_pratt(lbp)?;
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
    /// return an `UnexpectedToken` (or `UnexpectedEof` if at end) error
    /// with `expected`'s name.
    fn expect(&mut self, expected: &Tok) -> Result<(), ExprParseError> {
        if &self.peek().kind == expected {
            self.advance();
            Ok(())
        } else if self.at_eof() {
            Err(ExprParseError::UnexpectedEof {
                expected: expected_name(expected),
            })
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

/// Extract a string callee name from an expression for name-mangling
/// and symbol-table lookup. Returns `None` for indirect calls that
/// must be resolved by the callee's inferred type.
pub(crate) fn callee_name(expr: &Expr) -> Option<String> {
    match expr {
        Expr::Identifier { name, .. } => Some(name.clone()),
        Expr::FieldAccess {
            expr: base,
            field_name,
            ..
        } => Some(format!("{}.{}", callee_name(base)?, field_name)),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Module surface: parse an expression from source text.
// ---------------------------------------------------------------------------

/// Parse a Kit expression from source text. This is the public entry point used by the
/// pest-to-Pratt bridge (`PestExpr::parse`).
///
/// The `text` should be the source text of the expression as a `Pair::as_str()` slice.
/// Tokenization, parsing, and conversion to an `Expr` all happen here.
///
/// # Errors
///
/// Errors are returned for:
/// - Unrecognized characters in the source
/// - Integer literals that overflow `i64`
/// - Trailing tokens after the expression
/// - Any parse error from the Pratt loop
pub(crate) fn parse_kit_expr(text: &str) -> Result<Expr, ExprParseError> {
    let tokens = tokenize(text).map_err(|e| match e {
        LexicalError::UnexpectedCharacter { offset } => {
            ExprParseError::Custom(format!("unexpected character at byte offset {}", offset))
        }
        LexicalError::IntegerOverflow { text, .. } => {
            ExprParseError::Custom(format!("integer literal `{text}` is out of range for i64"))
        }
    })?;

    let mut parser = ExprParser::new(&tokens);
    let expr = parser.parse_expr()?;

    // Reject leftover tokens (e.g. from a stray character that was dropped
    // or from genuinely malformed input like `a b`).
    if parser.pos() < tokens.len() {
        return Err(ExprParseError::UnexpectedToken {
            found: tokens[parser.pos()].kind.clone(),
            expected: &["end of expression"],
        });
    }

    Ok(expr)
}

// Primary expression parsers (include'd into this module scope).
include!("primary.rs");

// Tests (include'd into this module scope, test builds only).
#[cfg(test)]
include!("tests.rs");
