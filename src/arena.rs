use std::{cell::RefCell, marker::PhantomData};

pub(crate) struct Arena<'a> {
    pub(crate) storage: RefCell<Vec<Box<dyn Erased + 'a>>>,
    _marker: PhantomData<&'a ()>,
}

impl<'a> Arena<'a> {
    pub(crate) fn new() -> Self {
        Self {
            storage: RefCell::new(Vec::new()),
            _marker: PhantomData,
        }
    }

    pub(crate) fn alloc<T: 'a>(&self, val: T) -> &'a T {
        let boxed = Box::new(val);
        let ptr: *const T = &*boxed;

        self.storage.borrow_mut().push(boxed);

        unsafe { &*ptr }
    }
}

trait Erased {}
impl<T> Erased for T {}
