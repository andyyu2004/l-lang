#![feature(decl_macro)]
#![feature(crate_visibility_modifier)]
#![feature(box_syntax, box_patterns)]

mod build;
mod traverse;
mod typecheck;

#[macro_use]
extern crate log;

pub use build::*;
pub use traverse::Visitor;

use ast::Ident;
use error::{LError, LResult};
use ir::{DefId, FnVisitor, ItemVisitor};
use lcore::mir::Mir;
use lcore::ty::{Instance, InstanceKind, TyCtx};
use std::collections::BTreeMap;
use std::io::Write;
use typecheck::typecheck;
use typeck::InheritedCtx;

macro halt_on_error($tcx:expr) {{
    if $tcx.sess.has_errors() {
        return Err(LError::ErrorReported);
    }
}}

pub trait TyCtxMirExt<'tcx> {
    fn mir_of_def(self, def_id: DefId) -> LResult<&'tcx Mir<'tcx>>;
    fn mir_of_instance(self, instance: Instance<'tcx>) -> LResult<&'tcx Mir<'tcx>>;
}

impl<'tcx> TyCtxMirExt<'tcx> for TyCtx<'tcx> {
    fn mir_of_def(self, def_id: DefId) -> LResult<&'tcx Mir<'tcx>> {
        let node = self.defs().get(def_id);
        match node {
            ir::DefNode::Ctor(variant) => Ok(build_variant_ctor(self, variant)),
            ir::DefNode::Item(item) => match item.kind {
                ir::ItemKind::Fn(sig, generics, body) =>
                    build_mir(self, def_id, sig, generics, body),
                _ => panic!(),
            },
            ir::DefNode::ImplItem(item) => match item.kind {
                ir::ImplItemKind::Fn(sig, body) =>
                    build_mir(self, def_id, sig, item.generics, body),
            },
            ir::DefNode::ForeignItem(_) => todo!(),
            ir::DefNode::Variant(_) | ir::DefNode::TyParam(_) => panic!(),
        }
    }

    fn mir_of_instance(self, instance: Instance<'tcx>) -> LResult<&'tcx Mir<'tcx>> {
        match instance.kind {
            InstanceKind::Item => self.mir_of_def(instance.def_id),
            InstanceKind::Intrinsic => unreachable!("intrinsics don't have mir"),
        }
    }
}

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
        // note that the failure to typeck could also come from earlier resolution errors
        halt_on_error!(tcx);
        let tables = fcx.resolve_inference_variables(body);
        let lctx = MirCtx::new(&inherited, tables);
        Ok(f(lctx))
    })
}

pub fn build_mir<'tcx>(
    tcx: TyCtx<'tcx>,
    def_id: DefId,
    sig: &ir::FnSig<'tcx>,
    generics: &ir::Generics<'tcx>,
    body: &'tcx ir::Body<'tcx>,
) -> LResult<&'tcx Mir<'tcx>> {
    with_mir_ctx(tcx, def_id, sig, generics, body, |mut lctx| lctx.build_mir(body))
}

pub fn build_tir<'tcx>(tcx: TyCtx<'tcx>) -> LResult<tir::Prog<'tcx>> {
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
            ir::ItemKind::Extern(_) => todo!(),
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
