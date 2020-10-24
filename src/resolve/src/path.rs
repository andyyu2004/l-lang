//! this module implements path resolution

use crate::{LateResolver, ResResult, ResolutionError, NS};
use ast::*;
use ir::{ModuleId, Res};

impl<'a, 'r, 'ast> LateResolver<'a, 'r, 'ast> {
    /// `id` belongs to the `Ty` or `Expr`
    crate fn resolve_path(&mut self, path: &'ast Path, ns: NS) {
        let res = match ns {
            NS::Value => self.resolve_val_path(path),
            NS::Type => self.resolve_ty_path(path),
        }
        .unwrap_or_else(|err| {
            err.emit();
            Res::Err
        });
        self.resolve_node(path.id, res)
    }

    fn resolve_val_path(&mut self, path: &'ast Path) -> ResResult<'a, Res<NodeId>> {
        self.resolve_val_path_segments(path, &path.segments)
    }

    fn resolve_module(&mut self, ident: Ident) -> Option<ModuleId> {
        self.resolver.find_module(self.curr_module(), ident)
    }

    fn resolve_val_path_segments(
        &mut self,
        path: &'ast Path,
        segments: &'ast [PathSegment],
    ) -> ResResult<'a, Res<NodeId>> {
        match &segments {
            [segment] => self.resolve_path_segment(path, segment, NS::Value),
            [segment, xs @ ..] => match self.resolve_module(segment.ident).and_then(|module| {
                self.with_module(module, |this| this.resolve_val_path_segments(path, xs).ok())
            }) {
                // if the path is successfully resolved inside the module, return it
                Some(res) => Ok(res),
                // otherwise, try resolve a type relative path
                None => self.resolve_type_relative(path, segment, xs),
            },
            [] => panic!("empty val path"),
        }
    }

    fn resolve_type_relative(
        &mut self,
        path: &'ast Path,
        segment: &'ast PathSegment,
        segments: &'ast [PathSegment],
    ) -> ResResult<'a, Res<NodeId>> {
        Err(self
            .resolver
            .build_error(path.span, ResolutionError::UnresolvedPath(segment.clone(), path.clone())))
    }

    fn resolve_val_path_segment(
        &mut self,
        path: &'ast Path,
        segment: &'ast PathSegment,
    ) -> ResResult<'a, Res<NodeId>> {
        self.resolve_var(segment.ident).ok_or_else(|| {
            self.build_error(
                path.span,
                ResolutionError::UnresolvedPath(segment.clone(), path.clone()),
            )
        })
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
        if let Some(&res) = self.scopes[NS::Type].lookup(&segment.ident) {
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
