use super::codegen::*;
use super::util::{LLVMAsPtrVal, LLVMTypeExt};
use super::CodegenCtx;
use crate::ty::*;
use crate::typeck::TyCtx;
use inkwell::types::*;
use inkwell::values::*;
use inkwell::AddressSpace;
use itertools::Itertools;
use rustc_hash::{FxHashMap, FxHashSet};

impl<'tcx> CodegenCtx<'tcx> {
    pub fn llvm_fn_ty_from_ty(&self, ty: Ty<'tcx>) -> FunctionType<'tcx> {
        let (params, ret) = ty.expect_fn();
        self.llvm_ty(ret).fn_type(&params.iter().map(|ty| self.llvm_ty(ty)).collect_vec(), false)
    }

    pub fn llvm_fn_ty(&self, params: SubstsRef<'tcx>, ret: Ty<'tcx>) -> FunctionType<'tcx> {
        self.llvm_ty(ret).fn_type(&params.iter().map(|ty| self.llvm_ty(ty)).collect_vec(), false)
    }

    /// wraps a `Ty` with refcount info (place the refcount in the second field instead of the first
    /// to allows for easier geps)
    pub fn llvm_boxed_ty(&self, ty: Ty<'tcx>) -> StructType<'tcx> {
        let llty = self.llvm_ty(ty);
        self.llctx.struct_type(&[llty, self.types.int32.into()], true)
    }

    /// converts a L type into a llvm representation
    pub fn llvm_ty(&self, ty: Ty<'tcx>) -> BasicTypeEnum<'tcx> {
        if let Some(&llty) = self.lltypes.borrow().get(ty) {
            return llty;
        }
        let llty = match ty.kind {
            TyKind::Bool => self.types.boolean.into(),
            TyKind::Int => self.types.int.into(),
            TyKind::Float => self.types.float.into(),
            TyKind::Tuple(xs) if xs.is_empty() => self.types.unit.into(),
            TyKind::Char => todo!(),
            TyKind::Array(ty, n) => todo!(),
            TyKind::Fn(params, ret) =>
                self.llvm_fn_ty(params, ret).ptr_type(AddressSpace::Generic).into(),
            TyKind::Tuple(tys) => {
                // tuples are represented as anonymous structs
                let lltys = tys.iter().map(|ty| self.llvm_ty(ty)).collect_vec();
                self.llctx.struct_type(&lltys, false).into()
            }
            TyKind::Adt(adt, substs) => match adt.kind {
                AdtKind::Struct => {
                    let opaque_ty = self.llctx.opaque_struct_type("opaque");
                    // llty.set_body();
                    self.lltypes.borrow_mut().insert(ty, opaque_ty.into());
                    let variant = adt.single_variant();
                    let tys = variant
                        .fields
                        .iter()
                        .map(|f| self.llvm_ty(f.ty(self.tcx, substs)))
                        .collect_vec();
                    opaque_ty.set_body(&tys, false);
                    return opaque_ty.into();
                }
                AdtKind::Enum => {
                    // it is fine to unwrap here as if the enum has no variants it is not
                    // constructable and this will never be called
                    let largest_variant = adt.variants.iter().max_by(|s, t| {
                        self.variant_size(s, substs).cmp(&self.variant_size(t, substs))
                    });
                    let llvariant =
                        self.variant_ty_to_llvm_ty(ty, largest_variant.unwrap(), substs).into();
                    assert!(adt.variants.len() < 256, "too many variants");
                    self.llctx.struct_type(&[self.types.discr.into(), llvariant], false).into()
                }
            },
            TyKind::Ptr(_, ty) => self.llvm_ty(ty).ptr_type(AddressSpace::Generic).into(),
            TyKind::Opaque(..) => todo!(),
            TyKind::Param(..)
            | TyKind::Scheme(..)
            | TyKind::Never
            | TyKind::Error
            | TyKind::Infer(_) => unreachable!(),
        };
        self.lltypes.borrow_mut().insert(ty, llty);
        llty
    }

    pub fn variant_ty_to_llvm_ty(
        &self,
        adt_ty: Ty<'tcx>,
        variant: &VariantTy<'tcx>,
        substs: SubstsRef<'tcx>,
    ) -> StructType<'tcx> {
        // TODO cache results
        // note we preserve the field declaration order of the struct
        let tys = variant.fields.iter().map(|f| self.llvm_ty(f.ty(self.tcx, substs))).collect_vec();
        self.llctx.struct_type(&tys, false)
    }
}
