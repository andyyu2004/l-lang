use crate::CodegenCtx;
use inkwell::types::*;
use inkwell::AddressSpace;
use itertools::Itertools;
use lcore::ty::*;

// for use with $ctx being a CodegenCtx
#[macro_export]
macro_rules! llty {
    ($ctx:expr, $(ty:tt)+) => {
        llvm_ty!($ctx.llctx, ($ty)+)
    };
}

// for use with a [inkwell::context::Context](inkwell::context::Context)
#[macro_export]
macro_rules! llvm_ty {
    ($ctx:expr, i64) => {
        $ctx.i64_type()
    };
    ($ctx:expr, i32) => {
        $ctx.i32_type()
    };
    ($ctx:expr, i16) => {
        $ctx.i16_type()
    };
    ($ctx:expr, i8) => {
        $ctx.i8_type()
    };
    ($ctx:expr, bool) => {
        $ctx.bool_type()
    };
    ($ctx:expr, void) => {
        $ctx.void_type()
    };
    ($ctx:expr, *$($ty:tt)+) => {
        llvm_ty!($ctx, $($ty)*).ptr_type(inkwell::AddressSpace::Generic)
    };
    ($ctx:expr, fn($($ty:tt),*)) => {
        $ctx.void_type().fn_type(&[$(llvm_ty!($ctx, $ty).into()),*], false)
    };
    // we use the bad idea of using dyn to mean varargs coz it looks cool
    ($ctx:expr, dyn fn($($ty:tt),*)) => {
         $ctx.void_type().fn_type(&[$(llvm_ty!($ctx, $ty).into()),*], true)
    };
    ($ctx:expr, fn($($ty:tt),*) -> $($ret:tt)+) => {
        llvm_ty!($ctx, $($ret)*).fn_type(&[$(llvm_ty!($ctx, $ty).into()),*], false)
    };
    ($ctx:expr, dyn fn($($ty:tt),*) -> $($ret:tt)+) => {
        llvm_ty!($ctx, $($ret)*).fn_type(&[$(llvm_ty!($ctx, $ty).into()),*], true)
    };
    ($ctx:expr, packed {$($ty:tt),*}) => {
        $ctx.struct_type(&[$(llvm_ty!($ctx, $ty).into()),*], true)
    };
    ($ctx:expr, {$($ty:tt),*}) => {
        $ctx.struct_type(&[$(llvm_ty!($ctx, $ty).into()),*], false)
    };
}

impl<'tcx> CodegenCtx<'tcx> {
    pub fn llvm_fn_ty_from_ty(&self, ty: Ty<'tcx>) -> FunctionType<'tcx> {
        let sig = ty.expect_fn_ptr();
        self.llvm_fn_ty(sig)
    }

    // use a separate function for fn types as `FunctionType<'tcx>` is not considered a basic type
    pub fn llvm_fn_ty(&self, sig: FnSig<'tcx>) -> FunctionType<'tcx> {
        self.llvm_ty(sig.ret)
            .fn_type(&sig.params.iter().map(|ty| self.llvm_ty(ty)).collect_vec(), false)
    }

    pub fn llvm_ptr_ty(&self, ty: Ty<'tcx>) -> PointerType<'tcx> {
        self.llvm_ty(ty).into_pointer_type()
    }

    /// converts a L type into a llvm representation
    pub fn llvm_ty(&self, ty: Ty<'tcx>) -> BasicTypeEnum<'tcx> {
        if let Some(&llty) = self.lltypes.borrow().get(ty) {
            return llty;
        }
        let llty = match ty.kind {
            TyKind::Bool => self.types.bool.into(),
            TyKind::Int => self.types.i64.into(),
            TyKind::Discr => self.types.discr.into(),
            TyKind::Float => self.types.float.into(),
            TyKind::Char => todo!(),
            TyKind::Tuple(xs) if xs.is_empty() => self.types.unit.into(),
            TyKind::Array(_ty, _n) => todo!(),
            TyKind::FnPtr(sig) => self.llvm_fn_ty(sig).ptr_type(AddressSpace::Generic).into(),
            TyKind::Closure(sig) => todo!(),
            TyKind::Tuple(tys) => {
                // tuples are represented as anonymous structs
                let lltys = tys.iter().map(|ty| self.llvm_ty(ty)).collect_vec();
                self.llctx.struct_type(&lltys, false).into()
            }
            TyKind::Adt(adt, substs) => {
                let name = format!("{}<{}>", adt.ident.as_str(), substs);
                let opaque_ty = self.llctx.opaque_struct_type(&name);
                // we must insert the opaque type immediately into the map as it may be a
                // recursive type
                self.lltypes.borrow_mut().insert(ty, opaque_ty.into());
                match adt.kind {
                    AdtKind::Struct => {
                        let variant = adt.single_variant();
                        let tys = variant
                            .fields
                            .iter()
                            .map(|f| self.llvm_ty(f.ty(self.tcx, substs)))
                            .collect_vec();
                        opaque_ty.set_body(&tys, false);
                    }
                    AdtKind::Enum => {
                        // it is fine to unwrap here as if the enum has no variants it is not
                        // constructable and this will never be called
                        let largest_variant = adt.variants.iter().max_by(|s, t| {
                            self.variant_size(s, substs).cmp(&self.variant_size(t, substs))
                        });
                        let llvariant =
                            self.variant_ty_to_llvm_ty(largest_variant.unwrap(), substs).into();
                        assert!(adt.variants.len() < 256, "too many variants");
                        opaque_ty.set_body(&[self.types.discr.into(), llvariant], false);
                    }
                }
                return opaque_ty.into();
            }
            // boxes and pointers have the same runtime type
            // however, boxes will have a refcount implicitly stored after the content
            TyKind::Box(ty) | TyKind::Ptr(ty) =>
                self.llvm_ty(ty).ptr_type(AddressSpace::Generic).into(),
            TyKind::Opaque(..) => todo!(),
            TyKind::Param(..) | TyKind::Infer(..) | TyKind::Never | TyKind::Error =>
                unreachable!("{}", ty),
        };
        self.lltypes.borrow_mut().insert(ty, llty);
        llty
    }

    pub fn variant_ty_to_llvm_ty(
        &self,
        variant: &VariantTy,
        substs: SubstsRef<'tcx>,
    ) -> StructType<'tcx> {
        // TODO cache results
        // note we preserve the field declaration order of the struct
        let tys = variant.fields.iter().map(|f| self.llvm_ty(f.ty(self.tcx, substs))).collect_vec();
        self.llctx.struct_type(&tys, false)
    }
}
