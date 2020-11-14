//! this module implements path resolution

use crate::*;
use ast::*;
use ir::{PartialRes, Res};

impl<'a, 'r, 'ast> LateResolver<'a, 'r, 'ast> {
    /// `id` belongs to the `Ty` or `Expr`
    crate fn resolve_path(&mut self, path: &'ast Path, ns: NS) {
        let partial_res = match ns {
            NS::Value => self.resolve_val_path(path),
            NS::Type => self.resolve_ty_path(path).map(PartialRes::resolved),
        }
        .unwrap_or_else(|err| {
            err.emit();
            PartialRes::resolved(Res::Err)
        });
        self.partially_resolve_node(path.id, partial_res)
    }

    fn resolve_val_path(&mut self, path: &'ast Path) -> ResResult<'a, PartialRes> {
        self.resolve_val_path_segments(path, &path.segments)
    }

    crate fn resolve_module(&mut self, ident: Ident) -> Option<ModuleId> {
        self.resolver.find_module(self.curr_module(), ident)
    }

    fn resolve_val_path_segments(
        &mut self,
        path: &'ast Path,
        segments: &'ast [PathSegment],
    ) -> ResResult<'a, PartialRes> {
        match &segments {
            [segment] =>
                self.resolve_path_segment(path, segment, NS::Value).map(PartialRes::resolved),
            [segment, remaining @ ..] =>
                match self.resolve_module(segment.ident).and_then(|module_id| {
                    self.with_module_id(module_id, |this| {
                        this.resolve_val_path_segments(path, remaining).ok()
                    })
                }) {
                    // if the path is successfully resolved inside the module, return it
                    Some(res) => Ok(res),
                    // otherwise, try resolve a type relative path
                    None => self.resolve_type_relative(path, remaining, segment),
                },
            [] => panic!("empty val path"),
        }
    }

    fn resolve_type_relative(
        &mut self,
        path: &'ast Path,
        remaining: &'ast [PathSegment],
        segment: &'ast PathSegment,
    ) -> ResResult<'a, PartialRes> {
        let base_res = self.resolve_ty_path_segment(path, segment).map_err(|_| {
            self.resolver.build_error(
                path.span,
                ResolutionError::UnresolvedPath(segment.clone(), path.clone()),
            )
        })?;
        Ok(PartialRes::new(base_res, remaining.len()))
    }

    fn resolve_val_path_segment(
        &mut self,
        path: &'ast Path,
        segment: &'ast PathSegment,
    ) -> ResResult<'a, Res<NodeId>> {
        let res = self.resolve_ident(segment.ident).ok_or_else(|| {
            self.build_error(
                path.span,
                ResolutionError::UnresolvedPath(segment.clone(), path.clone()),
            )
        })?;

        if let Res::Def(_, def_kind) = res {
            match def_kind {
                DefKind::Mod =>
                    return Err(
                        self.build_error(path.span, ResolutionError::InvalidValuePath(def_kind))
                    ),
                DefKind::TyParam(..) | DefKind::Extern | DefKind::Use | DefKind::Impl => panic!(),
                DefKind::Ctor(..)
                | DefKind::Fn
                | DefKind::TypeAlias
                | DefKind::AssocFn
                | DefKind::Enum
                | DefKind::Struct => {}
            }
        };

        Ok(res)
    }

    fn resolve_ty_path(&mut self, path: &'ast Path) -> ResResult<Res<NodeId>> {
        match path.segments.as_slice() {
            [] => panic!("empty ty path"),
            [segment] => self.resolve_path_segment(path, segment, NS::Type),
            [_xs @ .., _segment] => todo!(),
        }
    }

    fn resolve_path_segment(
        &mut self,
        path: &'ast Path,
        segment: &'ast PathSegment,
        ns: NS,
    ) -> ResResult<'a, Res<NodeId>> {
        self.visit_path_segment(segment);
        match ns {
            NS::Value => self.resolve_val_path_segment(path, segment),
            NS::Type => self.resolve_ty_path_segment(path, segment),
        }
    }

    fn resolve_ty_path_segment(
        &mut self,
        path: &'ast Path,
        segment: &'ast PathSegment,
    ) -> ResResult<'a, Res<NodeId>> {
        if let Some(&res) = self.scopes[NS::Type].lookup(segment.ident) {
            Ok(res)
        } else if let Some(res) = self.try_resolve_item(segment.ident) {
            Ok(res)
        } else if let Some(&ty) = self.resolver.primitive_types.get(&segment.ident.symbol) {
            Ok(Res::PrimTy(ty))
        } else {
            Err(self.build_error(path.span, ResolutionError::UnresolvedType(path.clone())))
        }
    }
}
