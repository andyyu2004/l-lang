use crate as tir;
use itertools::Itertools;
use std::fmt;
use std::fmt::Write;

/// amount to indent
const INDENT: usize = 4;

/// pretty prints `tir`
pub struct Formatter<'a, W> {
    /// number of indents
    current_indent: usize,
    /// character to indent with
    indent: &'a str,
    writer: W,
}

impl<'a, W> Formatter<'a, W> {
    pub fn new(writer: W) -> Self {
        Self { current_indent: 0, indent: " ", writer }
    }
}

macro_rules! indent_internal {
    ($s:expr, $($x:expr),*) => {{
        for _ in 0..$s.current_indent {
            write!($s.writer, "{}", $s.indent)?;
        }
        write!($s.writer, $($x),*)
    }};
}

macro_rules! indent {
    ($s:expr, $($x:expr),*) => {{
        // we need to indent every line that is written, not just the first
        let s = format!($($x),*);
        let lines = s.split_inclusive("\n").collect_vec();
        for line in &lines {
            indent_internal!($s, "{}", line)?;
        }
        Ok(())
    }};
}

macro_rules! indentln {
    ($s:expr, $($x:expr),*) => {{
        indent!($s, $($x),*)?;
        write!($s.writer, "\n")
    }};
}

impl<'a, W> Formatter<'a, W>
where
    W: Write,
{
    pub fn fmt(&mut self, prog: &tir::Prog) -> fmt::Result {
        for item in prog.items.values() {
            self.fmt_item(item)?;
        }
        Ok(())
    }

    pub fn fmt_item(&mut self, item: &tir::Item) -> fmt::Result {
        match &item.kind {
            tir::ItemKind::Fn(ty, _generics, body) => {
                let params = body.params.iter().map(|p| format!("{}", p.pat));
                indentln!(self, "fn {} :: {}", item.ident, ty)?;
                indentln!(
                    self,
                    "{}fn {}<>({}) {}\n",
                    item.vis.node,
                    item.ident,
                    // generics,
                    util::join2(params, ", "),
                    body
                )
            }
        }
    }

    pub fn fmt_stmt(&mut self, stmt: &tir::Stmt) -> fmt::Result {
        match &stmt.kind {
            tir::StmtKind::Let(l) => indent!(self, "{}", l),
            tir::StmtKind::Expr(expr) => indent!(self, "{}", expr),
        }
    }

    pub fn fmt_expr(&mut self, expr: &tir::Expr) -> fmt::Result {
        match &expr.kind {
            tir::ExprKind::Box(expr) => indent!(self, "(box {})", expr),
            tir::ExprKind::Loop(block) => indent!(self, "loop {}", block),
            tir::ExprKind::Const(c) => indent!(self, "{}", c),
            tir::ExprKind::Bin(op, l, r) => indent!(self, "({} {} {})", op, l, r),
            tir::ExprKind::Unary(op, expr) => indent!(self, "({}{})", op, expr),
            tir::ExprKind::Block(block) => self.fmt_block(block),
            tir::ExprKind::VarRef(_id) => indent!(self, "{}", expr.span.to_string()),
            tir::ExprKind::Field(base, field_idx) => indent!(self, "{}->{:?}", base, field_idx),
            tir::ExprKind::Tuple(xs) => indent!(self, "({})", util::join2(xs.iter(), ",")),
            tir::ExprKind::Ref(expr) => indent!(self, "(&{})", expr),
            tir::ExprKind::Deref(expr) => indent!(self, "(*{})", expr),
            tir::ExprKind::Ret(expr) => match expr {
                Some(expr) => indent!(self, "return {}", expr),
                None => indent!(self, "return"),
            },
            tir::ExprKind::Match(expr, arms) => self.fmt_match(expr, arms),
            tir::ExprKind::Closure { upvars: _, body } =>
                indent!(self, "(λ({}) {})", util::join2(body.params.iter(), ","), body),
            tir::ExprKind::Call(f, args) => self.fmt_call(f, args),
            tir::ExprKind::Assign(l, r) => indent!(self, "({} = {})", l, r),
            tir::ExprKind::ItemRef(_def_id, substs) =>
                indent!(self, "{}<{}>", expr.span.to_string(), substs),
            tir::ExprKind::Adt { adt, fields, .. } => {
                indentln!(self, "{} {{", adt.ident)?;
                self.with_indent(4, |fmt| {
                    for field in fields {
                        fmt.fmt_field(field)?;
                    }
                    Ok(())
                })?;
                indent!(self, "}}")
            }
            tir::ExprKind::Break => indent!(self, "break"),
            tir::ExprKind::Continue => indent!(self, "continue"),
        }?;
        write!(self.writer, ":{}", expr.ty)
    }

    pub fn fmt_field(&mut self, field: &tir::Field) -> fmt::Result {
        indentln!(self, "{}: {},", field.ident, field.expr)
    }

    fn fmt_call(&mut self, f: &tir::Expr, args: &[tir::Expr]) -> fmt::Result {
        match args.len() {
            0 => indent!(self, "({})", f),
            _ => indent!(self, "({} {})", f, util::join2(args.iter(), " ")),
        }
    }

    fn fmt_block(&mut self, block: &tir::Block) -> fmt::Result {
        indentln!(self, "{{")?;
        self.with_indent(INDENT, |this| {
            for stmt in &block.stmts {
                indentln!(this, "{};", stmt)?;
            }
            if let Some(expr) = &block.expr {
                indent!(this, "{}", expr)?;
            }
            Ok(())
        })?;
        indent!(self, "\n}}")
    }

    fn with_indent<R>(&mut self, indent: usize, f: impl FnOnce(&mut Self) -> R) -> R {
        self.current_indent += indent;
        let ret = f(self);
        self.current_indent -= indent;
        ret
    }

    fn fmt_match(&mut self, expr: &tir::Expr, arms: &[tir::Arm]) -> fmt::Result {
        indentln!(self, "match {} {{", expr)?;
        self.with_indent(INDENT, |this| {
            for arm in arms.iter() {
                indentln!(this, "{},", arm)?;
            }
            Ok(())
        })?;
        indent!(self, "}}")
    }
}
