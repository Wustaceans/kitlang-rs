//! Defer statement expansion pass.
//!
//! Transforms `Defer` statements into inline cleanup code at scope exits.
//! Deferred statements execute in LIFO order when a scope exits via fall-through, `return`, `break`,
//! or `continue`.
//!
//! This pass runs after type inference and before C codegen, so the codegen layer
//! never sees `Defer` nodes.

use super::ast::{Block, Function, MatchStmt, Program, Stmt, StmtKind};

/// Expand all `Defer` statements in a program.
pub fn expand_defers(program: &mut Program) {
    for func in &mut program.functions {
        expand_function(func);
    }
}

fn expand_function(func: &mut Function) {
    // At the function top level, `return` runs all pending defers.
    // `break`/`continue` are only valid inside loops and unwind to the loop boundary.
    expand_block(&mut func.body, &[], &[]);
}

/// Expand defers within a block, rebuilding its statement list.
///
/// Two inherited lists control which defers run at scope exits:
/// - `inherited_run`: defers that run on `return`, accumulated from all enclosing scopes.
/// - `inherited_break`: defers that run on `break`/`continue`, only from scopes between
///   the current point and the innermost enclosing loop (empty outside any loop).
fn expand_block(block: &mut Block, inherited_run: &[Stmt], inherited_break: &[Stmt]) {
    let old = std::mem::take(&mut block.stmts);
    let mut pending: Vec<Stmt> = Vec::new();
    let mut new_stmts: Vec<Stmt> = Vec::with_capacity(old.len());

    for stmt in old {
        expand_stmt(
            stmt,
            &mut pending,
            inherited_run,
            inherited_break,
            &mut new_stmts,
        );
    }

    // Fall-through: emit remaining pending defers in LIFO order.
    for d in pending.into_iter().rev() {
        new_stmts.push(d);
    }

    block.stmts = new_stmts;
}

/// Expand a single statement, appending results to `output`.
fn expand_stmt(
    stmt: Stmt,
    pending: &mut Vec<Stmt>,
    inherited_run: &[Stmt],
    inherited_break: &[Stmt],
    output: &mut Vec<Stmt>,
) {
    match stmt.kind {
        StmtKind::Defer { body } => {
            // Recursively expand the body so nested defers are relocated to their
            // correct execution point instead of surviving into codegen.
            let mut body_pending: Vec<Stmt> = Vec::new();
            let mut body_output: Vec<Stmt> = Vec::new();
            expand_stmt(
                *body,
                &mut body_pending,
                inherited_run,
                inherited_break,
                &mut body_output,
            );
            // Drain body's own defers (LIFO) into its output.
            for d in body_pending.into_iter().rev() {
                body_output.push(d);
            }
            if body_output.is_empty() {
                return;
            }
            let expanded = if body_output.len() == 1 {
                body_output.into_iter().next().unwrap()
            } else {
                Stmt::new(StmtKind::Block(Block { stmts: body_output }), stmt.span)
            };
            pending.push(expanded);
        }

        StmtKind::Block(block) => {
            // A bare block is a scope boundary. Inherit this scope's pending defers so they run
            // when the block exits (like if/match branches).
            let mut block_run = inherited_run.to_vec();
            block_run.extend_from_slice(pending);

            let mut block_break = inherited_break.to_vec();
            block_break.extend_from_slice(pending);

            let mut new_block = block;
            expand_block(&mut new_block, &block_run, &block_break);

            output.push(Stmt {
                kind: StmtKind::Block(new_block),
                span: stmt.span,
            });
        }

        StmtKind::Return(_) => {
            // Run local pending defers (LIFO), then all inherited defers (LIFO), then return.
            output.extend(pending.drain(..).rev());
            output.extend(inherited_run.iter().rev().cloned());
            output.push(stmt);
        }

        StmtKind::Break | StmtKind::Continue => {
            // Only run defers from scopes between here and the loop boundary.
            output.extend(pending.drain(..).rev());
            output.extend(inherited_break.iter().rev().cloned());
            output.push(stmt);
        }

        StmtKind::If {
            cond,
            then_branch,
            else_branch,
        } => {
            // Branches inherit this scope's pending defers for both run and break paths.
            // Outer scopes first, so that reversing at the exit point yields LIFO order.
            let mut branch_run = inherited_run.to_vec();
            branch_run.extend_from_slice(pending);
            let mut branch_break = inherited_break.to_vec();
            branch_break.extend_from_slice(pending);

            let mut new_then = then_branch;
            let mut new_else = else_branch;
            expand_block(&mut new_then, &branch_run, &branch_break);
            if let Some(e) = new_else.as_mut() {
                expand_block(e, &branch_run, &branch_break);
            }
            output.push(Stmt {
                kind: StmtKind::If {
                    cond,
                    then_branch: new_then,
                    else_branch: new_else,
                },
                span: stmt.span,
            });
        }

        StmtKind::While { cond, body } => {
            // Loop body: `return` still runs outer defers, but `break`/`continue` only run
            // the loop body's own defers (not outer ones). Include this scope's pending
            // defers in the loop body's inherited_run so return inside the loop works.
            let mut new_body = body;
            let mut loop_inherited_run = inherited_run.to_vec();
            loop_inherited_run.extend_from_slice(pending);
            expand_block(&mut new_body, &loop_inherited_run, &[]);
            output.push(Stmt {
                kind: StmtKind::While {
                    cond,
                    body: new_body,
                },
                span: stmt.span,
            });
        }

        StmtKind::For { var, iter, body } => {
            let mut new_body = body;
            let mut loop_inherited_run = inherited_run.to_vec();
            loop_inherited_run.extend_from_slice(pending);
            expand_block(&mut new_body, &loop_inherited_run, &[]);
            output.push(Stmt {
                kind: StmtKind::For {
                    var,
                    iter,
                    body: new_body,
                },
                span: stmt.span,
            });
        }

        StmtKind::Match(m) => {
            // Match arms inherit pending defers for both run and break paths.
            // Outer scopes first, so that reversing at the exit point yields LIFO order.
            let mut arm_run = inherited_run.to_vec();
            arm_run.extend_from_slice(pending);
            let mut arm_break = inherited_break.to_vec();
            arm_break.extend_from_slice(pending);

            let mut new_arms = m.arms;
            for arm in &mut new_arms {
                expand_block(&mut arm.body, &arm_run, &arm_break);
            }
            output.push(Stmt {
                kind: StmtKind::Match(MatchStmt {
                    arms: new_arms,
                    ..m
                }),
                span: stmt.span,
            });
        }

        _ => {
            output.push(stmt);
        }
    }
}
