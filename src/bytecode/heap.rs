#[derive(Debug)]
pub enum Obj {
	Str(String),
}

// #[derive(PartialEq, Eq, Clone, Copy, Debug)]
// #[repr(u8)]
// pub enum ObjTy {
// 	Str,
// 	Other,
// }

// #[repr(C)]
// pub struct ObjStr {
// 	ty: ObjTy,
// 	val: String,
// }

// struct ObjRef(*mut ObjTy);
// impl ObjRef {
// 	#[must_use]
// 	pub fn new_string(val: String) -> Self {
// 		Self(unsafe { (&mut ObjStr { ty: ObjTy::Str, val } as *mut ObjStr) as *mut ObjTy })
// 	}
// 	pub fn as_string(&self) -> Option<&mut ObjStr> {
// 		println!("{:?}", **self);
// 		if **self == ObjTy::Str {
// 			Some(unsafe { &mut *(self.0 as *mut ObjStr) })
// 		} else {
// 			None
// 		}
// 	}
// }
// impl core::ops::Deref for ObjRef {
// 	type Target = ObjTy;

// 	fn deref(&self) -> &Self::Target {
// 		unsafe { &*(self.0) }
// 	}
// }

// #[test]
// fn mine() {
// 	let bob = ObjRef::new_string("hello".into());

// 	println!("{:?}", *bob);

// 	assert_eq!(bob.as_string().map(|x| &x.val), Some(&"hello".to_string()))
// }
