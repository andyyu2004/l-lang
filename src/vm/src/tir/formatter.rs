use crate::{tir, util};
use itertools::Itertools;
use std::fmt;
use std::fmt::Write;

/// amount to indent
const INDENT: usize = 4;

/// pretty prints `tir`
crate struct Formatter<'a, W> {
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

macro_rules! indent {
    ($s:expr, $($x:expr),*) => {{
        for _ in 0..$s.current_indent {
            write!($s.writer, "{}", $s.indent)?;
        }
        write!($s.writer, $($x),*)
    }};
}

macro_rules! indent_each {
    ($s:expr, $($x:expr),*) => {{
        // we need to indent every line that is written, not just the first
        let s = format!($($x),*);
        let lines = s.split_inclusive("\n").collect_vec();
        for line in &lines {
            indent!($s, "{}", line)?;
        }
        Ok(())
    }};
}

macro_rules! indent_each_ln {
    ($s:expr, $($x:expr),*) => {{
        indent_each!($s, $($x),*)?;
        write!($s.writer, "\n")
    }};
}

macro_rules! indentln {
    ($s:expr, $($x:expr),*) => {{
        indent!($s, $($x),*)?;
        indent!($s, "\n")
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
        match item.kind {
            tir::ItemKind::Fn(sig, generics, body) => {
                let (params, inputs) = (body.params, sig.inputs);
                let params = params.iter().zip(inputs).map(|(p, t)| {
                    // we don't need to print the given ty as the pattern already has a ty
                    // however, we do check that they are in fact the same type
                    assert_eq!(p.pat.ty, t);
                    format!("{}", p)
                });
                indentln!(
                    self,
                    "{}fn #{:?}({}) -> {} {}\n",
                    item.vis.node,
                    item.id.def,
                    util::join2(params, ", "),
                    sig.output,
                    body
                )
            }
        }
    }

    pub fn fmt_expr(&mut self, expr: &tir::Expr) -> fmt::Result {
        match expr.kind {
            tir::ExprKind::Lit(c) => indent!(self, "{}", c),
            tir::ExprKind::Bin(op, l, r) => indent!(self, "({} {} {})", op, l, r),
            tir::ExprKind::Unary(op, expr) => indent!(self, "({} {})", op, expr),
            tir::ExprKind::Block(block) => self.fmt_block(block),
            tir::ExprKind::VarRef(id) => indent!(self, "${:?}", id.local),
            tir::ExprKind::ItemRef(def_id) => indent!(self, "#{:?}", def_id),
            tir::ExprKind::Tuple(xs) => indent!(self, "({})", util::join2(xs.iter(), ",")),
            tir::ExprKind::Match(expr, arms) => self.fmt_match(expr, arms),
            tir::ExprKind::Lambda(b) =>
                indent!(self, "(λ({}) {})", util::join2(b.params.iter(), ","), b.expr),
            tir::ExprKind::Call(f, args) => self.fmt_call(f, args),
        }?;
        write!(self.writer, ":{}", expr.ty)
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
            for stmt in block.stmts {
                indent_each_ln!(this, "{};", stmt)?;
            }
            if let Some(expr) = block.expr {
                indent_each!(this, "{}", expr)?;
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
                indent_each_ln!(this, "{},", arm)?;
            }
            Ok(())
        })?;
        indent!(self, "}}")
    }
}
