//! escape analysis of variables
//! (to decide whether they can safely live on the stack or if we must allocate on the heap)

// https://segment.com/blog/allocation-efficiency-in-high-performance-go-services/

use super::inference::FnCtx;
use crate::ir::{self, DefId, Visitor};
use rustc_hash::{FxHashMap, FxHashSet};
use std::marker::PhantomData;

impl<'a, 'tcx> FnCtx<'a, 'tcx> {
    /// constructs a new typeck table with all inference variables replaced by their actual types
    pub fn analyze_escape(&self, body: &'tcx ir::Body<'tcx>) {
        let mut analyzer = BodyReturnAnalyzer::default();
        analyzer.visit_body(body);
        dbg!(analyzer);
    }
}

#[derive(Debug)]
pub struct UpvarId {
    /// `Id` of the variable that this `Upvar` captures
    id: ir::Id,
    /// `Id` of the body of the lambda that closes over the `Upvar`
    lambda: ir::Id,
}

#[derive(Default, Debug)]
pub struct EscapeVisitor<'ir> {
    _marker: PhantomData<&'ir ()>,
    /// set of `DefId`s of lambdas that are returned from some function
    /// this is the only possible way for a variable to escape
    /// for simplicity, we assume if a lambda is returned, all upvars it captures are also escaping
    escaped: FxHashSet<DefId>,
    upvars: FxHashMap<UpvarId, ()>,
}

#[derive(Default, Debug)]
struct BodyReturnAnalyzer<'ir> {
    body_ids: Vec<ir::Id>,
    /// potential returns for each body id
    potential_returns: FxHashMap<ir::Id, Vec<ir::Id>>,
    /// PatId -> BodyId
    body_owners: FxHashMap<ir::Id, ir::Id>,
    /// map of `Id` -> `Expr`, we need this to analyze expressions such as `f(x)` as we need to
    /// know the body of `f`
    expr_map: FxHashMap<ir::Id, &'ir ir::Expr<'ir>>,
    _pd: PhantomData<&'ir ()>,
}

impl<'ir> BodyReturnAnalyzer<'ir> {
    fn with_body<R>(&mut self, body: &'ir ir::Body<'ir>, f: impl FnOnce(&mut Self) -> R) -> R {
        self.body_ids.push(body.id());
        let ret = f(self);
        self.body_ids.pop();
        ret
    }

    fn curr_body_id(&self) -> ir::Id {
        *self.body_ids.last().unwrap()
    }

    fn record_escape(&mut self, expr_id: ir::Id) {
        let body_id = self.curr_body_id();
        let vec = self.potential_returns.entry(body_id).or_insert_with(Default::default);
        vec.push(expr_id);
    }

    /// ideally, this function will only recurse where escape is possible (i.e. where something is returned)
    fn analyze_expr(&mut self, expr: &'ir ir::Expr<'ir>) {
        match expr.kind {
            ir::ExprKind::Lit(_) => {}
            ir::ExprKind::Ret(expr) =>
                if let Some(expr) = expr {
                    self.analyze_expr(expr)
                },
            ir::ExprKind::Bin(_, _, _) => {}
            ir::ExprKind::Unary(_, _) => {}
            ir::ExprKind::Block(b) => ir::walk_block(self, b),
            ir::ExprKind::Path(p) => self.analyze_path(p),
            ir::ExprKind::Tuple(xs) => xs.iter().for_each(|x| self.analyze_expr(x)),
            ir::ExprKind::Closure(_, body) => {
                self.record_escape(expr.id);
                self.visit_body(body);
            }
            ir::ExprKind::Assign(_, _) => {}
            ir::ExprKind::Call(f, _) => self.analyze_call(f),
            ir::ExprKind::Match(_, arms, _) =>
                arms.iter().for_each(|arm| self.analyze_expr(arm.body)),
            ir::ExprKind::Struct(_, fields) =>
                fields.iter().for_each(|f| self.analyze_expr(f.expr)),
            ir::ExprKind::Field(_, _) => todo!(),
        }
    }

    // pretty god awful code :)
    // the idea is to only look at the body of the function that is being called,
    // as that is what is returned
    fn analyze_call(&mut self, f: &'ir ir::Expr<'ir>) {
        match f.kind {
            ir::ExprKind::Path(path) => match path.res {
                ir::Res::PrimTy(_) => unreachable!(),
                ir::Res::Def(_, _) => {}
                ir::Res::Local(id) => match self.find_expr(id).kind {
                    ir::ExprKind::Closure(_, body) => self.analyze_expr(body.expr),
                    _ => unreachable!(),
                },
            },
            ir::ExprKind::Closure(_, body) => self.analyze_expr(body.expr),
            _ => unreachable!(),
        }
    }

    fn analyze_path(&mut self, path: &'ir ir::Path<'ir>) {
        match path.res {
            ir::Res::PrimTy(_) => unreachable!(),
            // items cannot close over upvars
            ir::Res::Def(_, _) => {}
            ir::Res::Local(id) => self.record_escape(id),
        }
    }

    fn analyze_stmt(&mut self, stmt: &'ir ir::Stmt<'ir>) {
        match stmt.kind {
            // expressions statements can't result in a escape so we ignore them
            ir::StmtKind::Expr(_) => {}
            ir::StmtKind::Semi(_) => {}
            ir::StmtKind::Let(l) => self.analyze_let(l),
        }
    }

    fn analyze_let(&mut self, l: &'ir ir::Let<'ir>) {
        // assignment makes this difficult
        // what value does a local variable have at the point of return?
        // seems like a cfg is a good idea
        todo!()
    }

    fn find_expr(&mut self, id: ir::Id) -> &'ir ir::Expr<'ir> {
        self.expr_map[&id]
    }

    fn record_expr(&mut self, pat_id: ir::Id, expr: &'ir ir::Expr<'ir>) {
        self.expr_map.insert(pat_id, expr);
    }
}

impl<'ir> ir::Visitor<'ir> for BodyReturnAnalyzer<'ir> {
    fn visit_body(&mut self, body: &'ir ir::Body<'ir>) {
        self.with_body(body, |this| ir::walk_body(this, body));
    }

    fn visit_stmt(&mut self, stmt: &'ir ir::Stmt<'ir>) {
        self.analyze_stmt(stmt)
    }

    fn visit_expr(&mut self, expr: &'ir ir::Expr<'ir>) {
        self.analyze_expr(expr)
    }
}
