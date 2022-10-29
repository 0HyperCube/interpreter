use core::alloc::Layout;
use std::alloc::{alloc, dealloc};

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[repr(u8)]
pub enum ObjTy {
	Str,
	Other,
}
impl ObjTy {
	pub fn free(boxed: Box<Self>) {
		match &*boxed {
			ObjTy::Str => unsafe { dealloc(Box::into_raw(boxed) as *mut u8, Layout::new::<Obj<String>>()) },
			ObjTy::Other => unreachable!(),
		}
	}
	pub fn of<T: 'static>() -> Self {
		let id = core::any::TypeId::of::<T>();
		if id == core::any::TypeId::of::<String>() {
			Self::Str
		} else {
			Self::Other
		}
	}
}

#[repr(C)]
pub struct Obj<T> {
	ty: ObjTy,
	val: T,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct ObjRef(*mut ObjTy);

impl ObjRef {
	#[must_use]
	#[inline]
	pub fn new<T: 'static>(val: T) -> (Self, Box<ObjTy>) {
		let ptr = unsafe { alloc(Layout::new::<Obj<T>>()) as *mut Obj<T> };
		let ty = ObjTy::of::<T>();
		unsafe { ptr.write(Obj { ty, val }) };
		(Self(unsafe { ptr as *mut ObjTy }), unsafe { Box::from_raw(ptr as *mut ObjTy) })
	}

	#[inline]
	pub fn as_ref_unchecked<T>(&self) -> &T {
		&unsafe { &*(self.0 as *const Obj<T>) }.val
	}

	#[inline]
	pub fn as_mut_unchecked<T>(&mut self) -> &mut T {
		&mut unsafe { &mut *(self.0 as *mut Obj<T>) }.val
	}

	#[inline]
	pub fn as_mut<T: 'static>(&mut self) -> Option<&mut T> {
		(self.object_ty() == ObjTy::of::<T>()).then(|| self.as_mut_unchecked())
	}

	#[inline]
	pub fn as_ref<T: 'static>(&self) -> Option<&T> {
		(self.object_ty() == ObjTy::of::<T>()).then(|| self.as_ref_unchecked())
	}

	#[must_use]
	#[inline]
	pub fn object_ty(&self) -> ObjTy {
		unsafe { *(self.0) }
	}
}

impl core::fmt::Debug for ObjRef {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		f.write_str(match self.object_ty() {
			ObjTy::Str => self.as_ref_unchecked::<String>(),
			ObjTy::Other => todo!(),
		})
	}
}

#[test]
fn mine() {
	{
		let s = "hello".to_string();
	}

	let (mut refer, owned) = {
		let mut real_str = "hello".to_string();
		ObjRef::new(real_str)
	};

	assert_eq!(refer.as_ref::<String>(), Some(&"hello".to_string()))
}
