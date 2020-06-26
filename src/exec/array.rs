use crate::exec::Type;
use crate::gc::Trace;

#[derive(Debug)]
pub struct Array {
    len: usize,
    ty: Type,
    vec: Vec<u8>,
}

impl Trace for Array {
}

impl Array {
    pub fn new(len: usize, ty: Type) -> Self {
        let vec = vec![0u8; len * ty.size()];
        Self { len, ty, vec }
    }

    pub fn get<T>(&self, index: isize) -> T {
        assert!(index < self.len as isize);
        let ptr = self.vec.as_ptr() as *const T;
        unsafe { std::ptr::read(ptr.offset(index)) }
    }

    pub fn set<T>(&mut self, index: isize, value: T) {
        assert!(index < self.len as isize);
        let ptr = self.vec.as_mut_ptr() as *mut T;
        unsafe { std::ptr::write(ptr.offset(index), value) }
    }
}
