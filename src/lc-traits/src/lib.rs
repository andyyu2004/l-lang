// mod interner;

// use lc_core::ty::{Ty, TyCtx, TyKind};
// use logic_ir::*;

// #[derive(Copy, Clone)]
// pub struct LInterner<'tcx> {
//     tcx: TyCtx<'tcx>,
// }

// impl<'tcx> LInterner<'tcx> {
//     fn tys_to_terms(self, ty: &[Ty<'tcx>]) -> Terms<Self> {
//         Terms::intern(self, ty.iter().map(|ty| self.ty_to_term(ty)))
//     }

//     // this is probably a really bad idea, just experimenting
//     fn ty_to_term(self, ty: Ty<'tcx>) -> Term<Self> {
//         let mk_atom = |atom| Atom::new(Sym::intern(atom));
//         let mk_term_atom = |atom| Term::intern(self, TermData::Atom(mk_atom(atom)));
//         let mk_structure = |f, terms| Term::intern(self, TermData::Structure(mk_atom(f), terms));

//         match ty.kind {
//             TyKind::Box(inner) => mk_structure("box", self.tys_to_terms(&[inner])),
//             TyKind::Tuple(tys) => mk_structure("tuple", self.tys_to_terms(tys)),
//             TyKind::FnPtr(fnptr) => todo!(),
//             TyKind::Array(_, _) => todo!(),
//             TyKind::Infer(_) => todo!(),
//             TyKind::Ptr(_) => todo!(),
//             TyKind::Param(_) => todo!(),
//             TyKind::Opaque(_, _) => todo!(),
//             TyKind::Adt(adt, substs) => todo!(),
//             TyKind::Bool => mk_term_atom("bool"),
//             TyKind::Discr => mk_term_atom("discr"),
//             TyKind::Char => mk_term_atom("char"),
//             TyKind::Float => mk_term_atom("float"),
//             TyKind::Int => mk_term_atom("int"),
//             TyKind::Error => mk_term_atom("err"),
//             TyKind::Never => mk_term_atom("never"),
//         }
//     }
// }

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use lc_driver::Driver;

//     #[test]
//     fn ty_to_term() {
//         Driver::from_src("")
//             .with_tcx(|tcx| {
//                 let ty = tcx.mk_ty(TyKind::Int);
//                 let ty = tcx.mk_tup_iter(vec![ty, ty, ty].into_iter());
//                 dbg!(ty);
//                 let interner = LInterner { tcx };
//                 let term = interner.ty_to_term(ty);
//                 dbg!(term);
//             })
//             .unwrap();
//     }
// }
