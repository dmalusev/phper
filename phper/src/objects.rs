//! Apis relate to [crate::sys::zend_object].

use crate::{
    alloc::{EAllocatable, EBox},
    classes::ClassEntry,
    errors::NotRefCountedTypeError,
    functions::{call_internal, ZendFunction},
    sys::*,
    values::Val,
};
use std::{
    any::Any,
    convert::TryInto,
    marker::PhantomData,
    mem::{size_of, ManuallyDrop},
    ptr::null_mut,
};

/// Used to represent objects generated by classes not registered by this
/// framework, or objects that do not have or do not want to process state.
pub type StatelessObject = Object<()>;

/// Wrapper of [crate::sys::zend_object].
#[repr(transparent)]
pub struct Object<T: 'static> {
    inner: zend_object,
    _p: PhantomData<T>,
}

impl<T: 'static> Object<T> {
    /// Another way to new object like [crate::classes::ClassEntry::new_object].
    pub fn new(class_entry: &ClassEntry<T>, arguments: &mut [Val]) -> crate::Result<EBox<Self>> {
        class_entry.new_object(arguments)
    }

    pub fn new_by_class_name(
        class_name: impl AsRef<str>, arguments: &mut [Val],
    ) -> crate::Result<EBox<Self>> {
        let class_entry = ClassEntry::from_globals(class_name)?;
        Self::new(class_entry, arguments)
    }

    /// # Safety
    pub unsafe fn from_mut_ptr<'a>(ptr: *mut zend_object) -> &'a mut Self {
        (ptr as *mut Self).as_mut().expect("ptr should not be null")
    }

    pub fn as_ptr(&self) -> *const zend_object {
        &self.inner
    }

    pub fn as_mut_ptr(&mut self) -> *mut zend_object {
        &mut self.inner
    }

    pub fn as_state(&self) -> &T {
        let eo = ExtendObject::fetch(&self.inner);
        eo.state.downcast_ref().unwrap()
    }

    pub fn as_mut_state(&mut self) -> &mut T {
        let eo = ExtendObject::fetch_mut(&mut self.inner);
        eo.state.downcast_mut().unwrap()
    }

    pub fn get_class(&self) -> &ClassEntry<T> {
        ClassEntry::from_ptr(self.inner.ce)
    }

    pub fn get_property(&mut self, name: impl AsRef<str>) -> &Val {
        self.get_mut_property(name)
    }

    pub fn duplicate_property(
        &mut self, name: impl AsRef<str>,
    ) -> Result<EBox<Val>, NotRefCountedTypeError> {
        self.get_mut_property(name).duplicate()
    }

    fn get_mut_property(&mut self, name: impl AsRef<str>) -> &mut Val {
        let name = name.as_ref();

        let prop = unsafe {
            #[cfg(phper_major_version = "8")]
            {
                zend_read_property(
                    self.inner.ce,
                    &self.inner as *const _ as *mut _,
                    name.as_ptr().cast(),
                    name.len().try_into().unwrap(),
                    true.into(),
                    null_mut(),
                )
            }
            #[cfg(phper_major_version = "7")]
            {
                let mut zv = std::mem::zeroed::<zval>();
                phper_zval_obj(&mut zv, self.as_ptr() as *mut _);
                zend_read_property(
                    self.inner.ce,
                    &mut zv,
                    name.as_ptr().cast(),
                    name.len().try_into().unwrap(),
                    true.into(),
                    null_mut(),
                )
            }
        };

        unsafe { Val::from_mut_ptr(prop) }
    }

    pub fn set_property(&mut self, name: impl AsRef<str>, val: Val) {
        let name = name.as_ref();
        let val = EBox::new(val);
        unsafe {
            #[cfg(phper_major_version = "8")]
            {
                zend_update_property(
                    self.inner.ce,
                    &mut self.inner,
                    name.as_ptr().cast(),
                    name.len().try_into().unwrap(),
                    EBox::into_raw(val).cast(),
                )
            }
            #[cfg(phper_major_version = "7")]
            {
                let mut zv = std::mem::zeroed::<zval>();
                phper_zval_obj(&mut zv, self.as_mut_ptr());
                zend_update_property(
                    self.inner.ce,
                    &mut zv,
                    name.as_ptr().cast(),
                    name.len().try_into().unwrap(),
                    EBox::into_raw(val).cast(),
                )
            }
        }
    }

    pub fn clone_obj(&self) -> EBox<Self> {
        unsafe {
            let new_obj = {
                #[cfg(phper_major_version = "8")]
                {
                    zend_objects_clone_obj(self.as_ptr() as *mut _).cast()
                }
                #[cfg(phper_major_version = "7")]
                {
                    let mut zv = std::mem::zeroed::<zval>();
                    phper_zval_obj(&mut zv, self.as_ptr() as *mut _);
                    zend_objects_clone_obj(&mut zv).cast()
                }
            };

            EBox::from_raw(new_obj)
        }
    }

    /// Only add refcount.
    ///
    /// TODO Make a reference type to wrap self.
    pub fn duplicate(&mut self) -> EBox<Self> {
        unsafe {
            self.inner.gc.refcount += 1;
            EBox::from_raw(self.as_mut_ptr().cast())
        }
    }

    /// Call the object method by name.
    ///
    /// # Examples
    ///
    /// ```
    /// use phper::{alloc::EBox, classes::StatelessClassEntry, values::Val};
    ///
    /// fn example() -> phper::Result<EBox<Val>> {
    ///     let mut memcached = StatelessClassEntry::from_globals("Memcached")?.new_object(&mut [])?;
    ///     memcached.call("addServer", &mut [Val::new("127.0.0.1"), Val::new(11211)])?;
    ///     let r = memcached.call("get", &mut [Val::new("hello")])?;
    ///     Ok(r)
    /// }
    /// ```
    pub fn call(
        &mut self, method_name: &str, arguments: impl AsMut<[Val]>,
    ) -> crate::Result<EBox<Val>> {
        let mut method = Val::new(method_name);

        unsafe {
            let mut val = Val::undef();
            phper_zval_obj(val.as_mut_ptr(), self.as_mut_ptr());
            call_internal(&mut method, Some(self), arguments)
        }
    }

    /// Return bool represents whether the constructor exists.
    pub(crate) fn call_construct(&mut self, arguments: impl AsMut<[Val]>) -> crate::Result<bool> {
        unsafe {
            match (*self.inner.handlers).get_constructor {
                Some(get_constructor) => {
                    let f = get_constructor(self.as_mut_ptr());
                    if f.is_null() {
                        Ok(false)
                    } else {
                        let zend_fn = ZendFunction::from_mut_ptr(f);
                        zend_fn.call(Some(self), arguments)?;
                        Ok(true)
                    }
                }
                None => Ok(false),
            }
        }
    }
}

impl Object<()> {
    pub fn new_by_std_class() -> EBox<Self> {
        Self::new_by_class_name("stdclass", &mut []).unwrap()
    }
}

impl<T> EAllocatable for Object<T> {
    unsafe fn free(ptr: *mut Self) {
        (*ptr).inner.gc.refcount -= 1;
        if (*ptr).inner.gc.refcount == 0 {
            zend_objects_store_del(ptr.cast());
        }
    }
}

impl<T> Drop for Object<T> {
    fn drop(&mut self) {
        unreachable!("Allocation on the stack is not allowed")
    }
}

pub(crate) type ManuallyDropState = ManuallyDrop<Box<dyn Any>>;

/// The Object contains `zend_object` and the user defined state data.
#[repr(C)]
pub(crate) struct ExtendObject {
    state: ManuallyDropState,
    object: zend_object,
}

impl ExtendObject {
    pub(crate) const fn offset() -> usize {
        size_of::<ManuallyDropState>()
    }

    pub(crate) fn fetch(object: &zend_object) -> &Self {
        unsafe {
            (((object as *const _ as usize) - ExtendObject::offset()) as *const Self)
                .as_ref()
                .unwrap()
        }
    }

    pub(crate) fn fetch_mut(object: &mut zend_object) -> &mut Self {
        unsafe {
            (((object as *mut _ as usize) - ExtendObject::offset()) as *mut Self)
                .as_mut()
                .unwrap()
        }
    }

    pub(crate) fn fetch_ptr(object: *mut zend_object) -> *mut Self {
        (object as usize - ExtendObject::offset()) as *mut Self
    }

    pub(crate) unsafe fn drop_state(this: *mut Self) {
        let state = &mut (*this).state;
        ManuallyDrop::drop(state);
    }

    pub(crate) unsafe fn as_mut_state<'a>(this: *mut Self) -> &'a mut ManuallyDropState {
        &mut (*this).state
    }

    pub(crate) unsafe fn as_mut_object<'a>(this: *mut Self) -> &'a mut zend_object {
        &mut (*this).object
    }
}
