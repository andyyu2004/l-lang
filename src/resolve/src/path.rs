use crate::{LateResolver, ResolutionError, NS};
use ast::*;
use ir::{ModuleId, Res};

impl<'a, 'r, 'ast> LateResolver<'a, 'r, 'ast> {
    /// `id` belongs to the `Ty` or `Expr`
    crate fn resolve_path(&mut self, path: &'ast Path, ns: NS) {
        let res = match ns {
            NS::Value => self.resolve_val_path(path),
            NS::Type => self.resolve_ty_path(path),
        };
        self.resolve_node(path.id, res);
    }

    fn resolve_val_path(&mut self, path: &'ast Path) -> Res<NodeId> {
        self.resolve_val_path_segments(path, &path.segments)
    }

    fn resolve_module(&mut self, ident: Ident) -> Option<ModuleId> {
        self.resolver.find_module(self.curr_module(), ident)
    }

    fn resolve_val_path_segments(
        &mut self,
        path: &'ast Path,
        segments: &'ast [PathSegment],
    ) -> Res<NodeId> {
        match &segments {
            [] => panic!("empty val path"),
            [segment] => self.resolve_path_segment(path, segment, NS::Value),
            [segment, xs @ ..] => match self.resolve_module(segment.ident) {
                Some(module) =>
                    self.with_module(module, |this| this.resolve_val_path_segments(path, xs)),
                None =>
                    return self.resolver.emit_error(
                        path.span,
                        ResolutionError::UnresolvedPath(segment.clone(), path.clone()),
                    ),
            },
        }
    }

    fn resolve_val_path_segment(
        &mut self,
        path: &'ast Path,
        segment: &'ast PathSegment,
    ) -> Res<NodeId> {
        self.resolve_var(segment.ident).unwrap_or_else(|| {
            let err = ResolutionError::UnresolvedPath(segment.clone(), path.clone());
            self.resolver.emit_error(path.span, err)
        })
    }

    fn resolve_ty_path(&mut self, path: &'ast Path) -> Res<NodeId> {
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
    ) -> Res<NodeId> {
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
    ) -> Res<NodeId> {
        if let Some(&res) = self.scopes[NS::Type].lookup(&segment.ident) {
            res
        } else if let Some(res) = self.try_resolve_item(segment.ident) {
            res
        } else if let Some(&ty) = self.resolver.primitive_types.get(&segment.ident.symbol) {
            Res::PrimTy(ty)
        } else {
            self.emit_error(path.span, ResolutionError::UnresolvedType(path.clone()))
        }
    }
}
