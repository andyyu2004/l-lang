#![feature(box_syntax, box_patterns)]
#![feature(crate_visibility_modifier)]
#![feature(decl_macro)]

mod build;

#[macro_use]
extern crate log;

pub use build::{build_fn, MirCtx};

use ast::Ident;
use error::{LError, LResult};
use ir::{DefId, FnVisitor, ItemVisitor};
use lcore::TyCtx;
use std::collections::BTreeMap;
use std::io::Write;
use typeck::{InheritedCtx, TcxCollectExt};

macro halt_on_error($tcx:expr) {{
    if $tcx.sess.has_errors() {
        return Err(LError::ErrorReported);
    }
}}

pub fn with_mir_ctx<'tcx, R>(
    tcx: TyCtx<'tcx>,
    def_id: DefId,
    sig: &ir::FnSig<'tcx>,
    generics: &ir::Generics<'tcx>,
    body: &'tcx ir::Body<'tcx>,
    f: impl for<'a> FnOnce(MirCtx<'a, 'tcx>) -> R,
) -> LResult<R> {
    InheritedCtx::build(tcx, def_id).enter(|inherited| {
        let fcx = inherited.check_fn_item(def_id, sig, generics, body);
        // don't bother continuing if typeck failed
        // note that the failure to typeck could also come from resolution errors
        halt_on_error!(tcx);
        let tables = fcx.resolve_inference_variables(body);
        let lctx = MirCtx::new(&inherited, tables);
        Ok(f(lctx))
    })
}

pub fn build_tir<'tcx>(tcx: TyCtx<'tcx>) -> LResult<tir::Prog<'tcx>> {
    tcx.collect_item_types();
    let prog = tcx.ir;
    let mut items = BTreeMap::new();

    for item in prog.items.values() {
        match item.kind {
            ir::ItemKind::Fn(sig, generics, body) => {
                if let Ok(tir) = with_mir_ctx(tcx, item.id.def, sig, generics, body, |mut lctx| {
                    lctx.lower_item_tir(item)
                }) {
                    items.insert(item.id, tir);
                }
            }
            ir::ItemKind::Struct(..) => {}
            // note that no tir is generated for enum constructors
            // the constructor code is generated at mir level only
            ir::ItemKind::Enum(..) => {}
            ir::ItemKind::Impl { .. } => unimplemented!(),
        }
    }
    halt_on_error!(tcx);
    Ok(tir::Prog { items })
}

pub fn dump_mir<'tcx>(tcx: TyCtx<'tcx>, writer: &mut dyn Write) {
    typeck::collect_item_types(tcx);
    MirDump { writer, tcx }.visit_ir(tcx.ir);
}

struct MirDump<'a, 'tcx> {
    tcx: TyCtx<'tcx>,
    writer: &'a mut dyn Write,
}

impl<'a, 'tcx> FnVisitor<'tcx> for MirDump<'a, 'tcx> {
    fn visit_fn(
        &mut self,
        def_id: DefId,
        _ident: Ident,
        sig: &'tcx ir::FnSig<'tcx>,
        generics: &'tcx ir::Generics<'tcx>,
        body: &'tcx ir::Body<'tcx>,
    ) {
        let _ = with_mir_ctx(self.tcx, def_id, sig, generics, body, |mut lctx| {
            let mir = lctx.build_mir(body);
            write!(self.writer, "\n{}", mir).unwrap();
        });
    }
}
