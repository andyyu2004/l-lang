#![feature(decl_macro)]
#![feature(crate_visibility_modifier)]
#![feature(box_syntax, box_patterns)]

mod build;
mod traverse;
mod typecheck;

#[macro_use]
extern crate log;

pub use build::*;
pub use traverse::MirVisitor;

use error::{LError, LResult};
use ir::{DefId, FnVisitor, Visitor};
use lcore::mir::Mir;
use lcore::queries::Queries;
use lcore::ty::{Instance, InstanceKind, TyCtx};
use std::collections::BTreeMap;
use std::io::Write;
use typecheck::typecheck;

pub fn provide(queries: &mut Queries) {
    *queries = Queries { mir_of, instance_mir, ..*queries }
}

macro halt_on_error($tcx:expr) {{
    if $tcx.sess.has_errors() {
        return Err(LError::ErrorReported);
    }
}}

fn instance_mir<'tcx>(tcx: TyCtx<'tcx>, instance: Instance<'tcx>) -> LResult<&'tcx Mir<'tcx>> {
    match instance.kind {
        InstanceKind::Item => tcx.mir_of(instance.def_id),
        InstanceKind::Intrinsic => unreachable!("intrinsics don't have mir"),
    }
}

fn mir_of<'tcx>(tcx: TyCtx<'tcx>, def_id: DefId) -> LResult<&'tcx Mir<'tcx>> {
    let node = tcx.defs().get(def_id);
    match node {
        ir::DefNode::Ctor(variant) => Ok(build_variant_ctor(tcx, variant)),
        ir::DefNode::Item(item) => match item.kind {
            ir::ItemKind::Fn(_, _, body) => build_mir(tcx, def_id, body),
            _ => panic!(),
        },
        ir::DefNode::ImplItem(item) => match item.kind {
            ir::ImplItemKind::Fn(_, body) => build_mir(tcx, def_id, body),
        },
        ir::DefNode::ForeignItem(_) => todo!(),
        ir::DefNode::Variant(_) | ir::DefNode::TyParam(_) => panic!(),
    }
}

fn with_lowering_ctx<'tcx, R>(
    tcx: TyCtx<'tcx>,
    def_id: DefId,
    f: impl FnOnce(LoweringCtx<'tcx>) -> R,
) -> LResult<R> {
    let tables = tcx.typeck(def_id)?;
    let lctx = LoweringCtx::new(tcx, tables);
    Ok(f(lctx))
}

pub fn build_mir<'tcx>(
    tcx: TyCtx<'tcx>,
    def_id: DefId,
    body: &'tcx ir::Body<'tcx>,
) -> LResult<&'tcx Mir<'tcx>> {
    with_lowering_ctx(tcx, def_id, |mut lctx| lctx.build_mir(body))
}

pub fn build_tir<'tcx>(tcx: TyCtx<'tcx>) -> LResult<tir::Prog<'tcx>> {
    let prog = tcx.ir;
    let mut items = BTreeMap::new();

    for item in prog.items.values() {
        match item.kind {
            ir::ItemKind::Fn(..) => {
                if let Ok(tir) =
                    with_lowering_ctx(tcx, item.id.def, |mut lctx| lctx.lower_item_tir(item))
                {
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
    let mut mir_dump = MirDump { tcx, writer };
    mir_dump.visit_ir(tcx.ir);
}

struct MirDump<'a, 'tcx> {
    tcx: TyCtx<'tcx>,
    writer: &'a mut dyn Write,
}

impl<'a, 'tcx> FnVisitor<'tcx> for MirDump<'a, 'tcx> {
    fn visit_fn(&mut self, def_id: DefId) {
        let body = self.tcx.defs().body(def_id);
        let _ = with_lowering_ctx(self.tcx, def_id, |mut lctx| {
            let mir = lctx.build_mir(body);
            write!(self.writer, "\n{}", mir).unwrap();
        });
    }
}
