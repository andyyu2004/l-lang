use crate::ir;

#[derive(Debug)]
crate enum Res<Id = ir::Id> {
    PrimTy(ir::PrimTy),
    Local(Id),
}

/// namespaces for types and values
crate enum NS {
    Type,
    Value,
}

/// a `T` for each namespace
crate struct PerNS<T> {
    pub value: T,
    pub ty: T,
}

impl<T> std::ops::Index<NS> for PerNS<T> {
    type Output = T;
    fn index(&self, ns: NS) -> &Self::Output {
        match ns {
            NS::Value => &self.value,
            NS::Type => &self.ty,
        }
    }
}

impl<T> std::ops::IndexMut<NS> for PerNS<T> {
    fn index_mut(&mut self, ns: NS) -> &mut Self::Output {
        match ns {
            NS::Value => &mut self.value,
            NS::Type => &mut self.ty,
        }
    }
}
