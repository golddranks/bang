#![allow(dead_code)]
use std::{
    ffi::{CStr, CString, c_char, c_double, c_longlong, c_ulonglong, c_void},
    fmt::Debug,
    marker::PhantomData,
    mem::transmute,
    ops::Not,
    ptr::{NonNull, null_mut},
    sync::atomic::{AtomicPtr, Ordering},
};

pub type Ptr = *mut c_void;
pub type Imp = unsafe extern "C" fn() -> *const c_void;
pub type NSUInteger = c_ulonglong;
pub type NSInteger = c_longlong;
pub type Bool = bool;
pub type CGFloat = c_double;

// SAFETY: OK. each of the extern function signatures are carefully checked and thought of
unsafe extern "C" {
    safe fn NSSetUncaughtExceptionHandler(f: extern "C" fn(NSException::IPtr));
    safe fn class_createInstance(cls: CPtr, extra_bytes: usize) -> OPtr;
    safe fn class_addProtocol(cls: CPtr, protocol: CStrPtr) -> bool;
    safe fn object_getIndexedIvars(obj: OPtr) -> *const c_void;
    safe fn object_getIvar(obj: OPtr, ivar: Ivar) -> OPtr;
    safe fn object_setIvar(obj: OPtr, ivar: Ivar, val: OPtr);
    safe fn ivar_getOffset(v: Ivar) -> usize;
    safe fn class_getInstanceVariable(cls: CPtr, name: CStrPtr) -> Option<Ivar>;
    safe fn class_getInstanceSize(cls: CPtr) -> usize;
    safe fn objc_allocateClassPair(
        class: CPtr,
        name: CStrPtr,
        extraBytes: usize,
    ) -> Option<UnregisteredCls>;
    safe fn objc_registerClassPair(class: UnregisteredCls);
    safe fn objc_getClass(name: CStrPtr) -> Option<CPtr>;
    safe fn sel_registerName(name: CStrPtr) -> Sel;
    // SAFETY: NEEDS CHECK. extremely unsafe to call; requires always transmuting to an appropriate signature
    unsafe fn objc_msgSend();
    // SAFETY: NEEDS CHECK. check whether Imp interface is correct and the types descriptor matches it
    unsafe fn class_addMethod(cls: CPtr, name: Sel, imp: Imp, types: CStrPtr) -> Bool;
    // SAFETY: NEEDS CHECK. check whether the Ivar details are correct and the types descriptor matches them
    unsafe fn class_addIvar(
        cls: UnregisteredCls,
        name: CStrPtr,
        size: usize,
        alignment: u8,
        types: CStrPtr,
    ) -> Bool;
}

pub fn make_subclass(class: CPtr, name: &CStr) -> Option<CPtr> {
    let cls = objc_allocateClassPair(class, CStrPtr::new(name), 0)?;
    Some(cls.register())
}

pub fn sel(name: &CStr) -> Sel {
    sel_registerName(CStrPtr::new(name))
}

pub fn class(name: &CStr) -> Option<CPtr> {
    objc_getClass(CStrPtr::new(name))
}

fn to_imp0<R, Slf>(f: extern "C" fn(Slf, Sel) -> R) -> Imp {
    // SAFETY: OK transmuting to unsafe fn shifts the onus to the caller of that
    unsafe { transmute(f) }
}

fn to_imp1<R, Slf, A0>(f: extern "C" fn(Slf, Sel, A0) -> R) -> Imp {
    // SAFETY: OK transmuting to unsafe fn shifts the onus to the caller of that
    unsafe { transmute(f) }
}

fn to_imp2<R, Slf, A0, A1>(f: extern "C" fn(Slf, Sel, A0, A1) -> R) -> Imp {
    // SAFETY: OK transmuting to unsafe fn shifts the onus to the caller of that
    unsafe { transmute(f) }
}

fn to_imp3<R, Slf, A0, A1, A2>(f: extern "C" fn(Slf, Sel, A0, A1, A2) -> R) -> Imp {
    // SAFETY: OK. transmuting to unsafe fn shifts the onus to the caller of that
    unsafe { transmute(f) }
}

#[derive(Debug)]
pub struct NamedStaticPtr {
    name: &'static CStr,
    ptr: AtomicPtr<c_void>,
}

impl NamedStaticPtr {
    pub const fn new(name: &'static CStr) -> Self {
        Self {
            name,
            ptr: AtomicPtr::new(null_mut()),
        }
    }
    pub fn init(&self, obj: OPtr) {
        if self
            .ptr
            .compare_exchange(
                null_mut(),
                obj.0.as_ptr(),
                Ordering::Relaxed,
                Ordering::Relaxed,
            )
            .is_err()
        {
            panic!("No re-setting!");
        }
    }

    pub fn obj(&self) -> OPtr {
        let ptr = self.ptr.load(Ordering::Relaxed);
        debug_assert!(ptr.is_null().not(), "{:?} is uninitialized!", self.name);
        OPtr::new(ptr)
    }
}

#[derive(Debug)]
#[repr(transparent)]
pub struct CStrPtr<'a>(*const c_char, PhantomData<&'a c_char>);

impl<'a> CStrPtr<'a> {
    pub fn new(cstr: &CStr) -> Self {
        Self(cstr.as_ptr(), PhantomData)
    }

    pub fn to_cstr(&self) -> &'a CStr {
        unsafe { CStr::from_ptr(self.0) }
    }
}

#[derive(Debug)]
#[repr(transparent)]
pub struct AllocObj<T>(OPtr, PhantomData<T>);

impl<T> AllocObj<T> {
    pub(super) fn obj(&self) -> OPtr {
        self.0
    }
}

#[derive(Debug, Copy, Clone)]
#[repr(transparent)]
pub struct TypedIvar<T>(Ivar, PhantomData<T>);

#[derive(Debug, Copy, Clone)]
#[repr(transparent)]
pub struct Ivar(NonNull<c_void>);

impl Ivar {
    pub fn offset(self) -> usize {
        ivar_getOffset(self)
    }
}

impl<T: TypedPtr> TypedIvar<T> {
    pub unsafe fn new(ivar: Ivar) -> Self {
        TypedIvar(ivar, PhantomData)
    }

    pub fn offset(self) -> usize {
        self.0.offset()
    }

    pub fn set(self, obj: OPtr, value: T) {
        object_setIvar(obj, self.0, value.obj())
    }

    pub fn get(self, obj: OPtr) -> T {
        unsafe { T::new(object_getIvar(obj, self.0)) }
    }
}

#[derive(Debug, Copy, Clone)]
#[repr(transparent)]
pub struct OPtr(NonNull<c_void>);

impl OPtr {
    pub const fn new(ptr: Ptr) -> Self {
        OPtr(NonNull::new(ptr).expect("CALLER BUG: must be called with non-null pointer"))
    }

    pub unsafe fn get_index_ivars<T>(&mut self) -> &mut T {
        let ptr = object_getIndexedIvars(*self) as *mut T;
        unsafe { &mut *ptr }
    }
}

#[derive(Debug)]
#[repr(transparent)]
pub struct UnregisteredCls(OPtr);

impl UnregisteredCls {
    fn cls(&self) -> CPtr {
        CPtr(self.0)
    }

    pub fn register(self) -> CPtr {
        let cls = self.cls();
        objc_registerClassPair(self);
        cls
    }

    // types: https://developer.apple.com/library/archive/documentation/Cocoa/Conceptual/ObjCRuntimeGuide/Articles/ocrtTypeEncodings.html
    pub unsafe fn add_ivar<T>(&self, name: &CStr, types: &CStr) -> bool {
        unsafe {
            class_addIvar(
                UnregisteredCls(self.0),
                CStrPtr::new(name),
                size_of::<T>(),
                align_of::<T>() as u8,
                CStrPtr::new(types),
            )
        }
    }
}

#[derive(Debug, Copy, Clone)]
#[repr(transparent)]
pub struct CPtr(OPtr);

impl CPtr {
    pub fn obj(self) -> OPtr {
        self.0
    }

    pub fn instance_size(self) -> usize {
        class_getInstanceSize(self)
    }

    pub fn ivar(self, name: &CStr) -> Option<Ivar> {
        class_getInstanceVariable(self, CStrPtr::new(name))
    }

    pub fn add_protocol(self, protocol: &CStr) -> bool {
        class_addProtocol(self, CStrPtr::new(protocol))
    }

    // SAFETY: NEEDS CHECK. whether Imp interface is correct and the types descriptor matches it
    pub unsafe fn add_method0<R, Slf>(
        self,
        sel: Sel,
        fn_ptr: extern "C" fn(Slf, Sel) -> R,
        types: &CStr,
    ) -> bool {
        // SAFETY: OK. the onus is on the caller
        unsafe { class_addMethod(self, sel, to_imp0(fn_ptr), CStrPtr::new(types)) }
    }

    // SAFETY: NEEDS CHECK. whether Imp interface is correct and the types descriptor matches it
    pub unsafe fn add_method1<R, Slf, A0>(
        self,
        sel: Sel,
        fn_ptr: extern "C" fn(Slf, Sel, A0) -> R,
        types: &CStr,
    ) -> bool {
        // SAFETY: OK. the onus is on the caller
        unsafe { class_addMethod(self, sel, to_imp1(fn_ptr), CStrPtr::new(types)) }
    }

    // SAFETY: NEEDS CHECK. whether Imp interface is correct and the types descriptor matches it
    pub unsafe fn add_method2<R, Slf, A0, A1>(
        self,
        sel: Sel,
        fn_ptr: extern "C" fn(Slf, Sel, A0, A1) -> R,
        types: &CStr,
    ) -> bool {
        // SAFETY: OK. the onus is on the caller
        unsafe { class_addMethod(self, sel, to_imp2(fn_ptr), CStrPtr::new(types)) }
    }

    // SAFETY: NEEDS CHECK. whether Imp interface is correct and the types descriptor matches it
    pub unsafe fn add_method3<R, Slf, A0, A1, A2>(
        self,
        sel: Sel,
        fn_ptr: extern "C" fn(Slf, Sel, A0, A1, A2) -> R,
        types: &CStr,
    ) -> bool {
        // SAFETY: OK. the onus is on the caller
        unsafe { class_addMethod(self, sel, to_imp3(fn_ptr), CStrPtr::new(types)) }
    }

    // SAFETY: the caller must ensure that the type T is layout-compatible with the allocation,
    // i.e. that the self Cls object is a subclass of T's Cls
    pub unsafe fn alloc<T>(self) -> AllocObj<T> {
        unsafe { msg0::<AllocObj<T>>(self.obj(), sel::alloc.sel()) }
    }

    pub unsafe fn alloc_indexed<T>(self, extra_bytes: usize) -> AllocObj<T> {
        AllocObj(class_createInstance(self, extra_bytes), PhantomData)
    }
}

#[derive(Debug, Copy, Clone)]
#[repr(transparent)]
pub struct Sel(OPtr);

#[derive(Debug)]
#[repr(transparent)]
pub struct StaticClsPtr(NamedStaticPtr);

impl StaticClsPtr {
    pub const fn new(name: &'static CStr) -> Self {
        Self(NamedStaticPtr::new(name))
    }

    pub fn cls(&self) -> CPtr {
        CPtr(self.obj())
    }

    pub fn obj(&self) -> OPtr {
        self.0.obj()
    }

    pub fn init(&self) {
        let Some(cls) = class(self.0.name) else {
            panic!("CALLER BUG: unknown class name: {:?}", self.0.name);
        };
        self.0.init(cls.obj());
    }

    pub fn init_with(&self, cls: CPtr) {
        self.0.init(cls.obj());
    }
}

#[repr(transparent)]
pub struct StaticSelPtr(NamedStaticPtr);

impl StaticSelPtr {
    pub const fn new(name: &'static CStr) -> Self {
        Self(NamedStaticPtr::new(name))
    }

    pub fn sel(&self) -> Sel {
        Sel(self.0.obj())
    }

    pub fn init(&self) {
        let colon_name = cstr_replace(self.0.name, b'_', b':');
        let sel = sel(&colon_name);

        self.0.init(sel.0);
    }

    pub fn init_setter(&self) {
        let mut buf = b"set".to_vec();
        buf.extend_from_slice(self.0.name.to_bytes());
        buf[3] = buf[3].to_ascii_uppercase();
        buf.push(b':');
        buf.push(b'\0');
        let setter_name = CString::from_vec_with_nul(buf).expect("UNREACHABLE");
        let sel = sel(&setter_name);

        self.0.init(sel.0);
    }
}

fn cstr_replace(cstr: &CStr, needle: u8, with: u8) -> CString {
    let mut buf = cstr.to_bytes_with_nul().to_owned();
    for c in &mut buf {
        if *c == needle {
            *c = with;
        }
    }
    CString::from_vec_with_nul(buf).expect("UNREACHABLE: originally from to_bytes_with_nul")
}

pub unsafe fn msg0<R>(receiver: OPtr, selector: Sel) -> R {
    unsafe {
        let fn_ptr = transmute::<
            unsafe extern "C" fn(),
            unsafe extern "C" fn(receiver: OPtr, selector: Sel) -> R,
        >(objc_msgSend as unsafe extern "C" fn());
        fn_ptr(receiver, selector)
    }
}

pub unsafe fn msg1<R, A0>(receiver: OPtr, selector: Sel, arg0: A0) -> R {
    unsafe {
        let fn_ptr = transmute::<
            unsafe extern "C" fn(),
            unsafe extern "C" fn(receiver: OPtr, selector: Sel, arg0: A0) -> R,
        >(objc_msgSend as unsafe extern "C" fn());
        fn_ptr(receiver, selector, arg0)
    }
}

pub unsafe fn msg2<R, A0, A1>(receiver: OPtr, selector: Sel, arg0: A0, arg1: A1) -> R {
    unsafe {
        let fn_ptr = transmute::<
            unsafe extern "C" fn(),
            unsafe extern "C" fn(receiver: OPtr, selector: Sel, arg0: A0, arg1: A1) -> R,
        >(objc_msgSend as unsafe extern "C" fn());
        fn_ptr(receiver, selector, arg0, arg1)
    }
}

pub unsafe fn msg3<R, A0, A1, A2>(
    receiver: OPtr,
    selector: Sel,
    arg0: A0,
    arg1: A1,
    arg2: A2,
) -> R {
    unsafe {
        let fn_ptr = transmute::<
            unsafe extern "C" fn(),
            unsafe extern "C" fn(receiver: OPtr, selector: Sel, arg0: A0, arg1: A1, arg2: A2) -> R,
        >(objc_msgSend as unsafe extern "C" fn());
        fn_ptr(receiver, selector, arg0, arg1, arg2)
    }
}

pub unsafe fn msg4<R, A0, A1, A2, A3>(
    receiver: OPtr,
    selector: Sel,
    arg0: A0,
    arg1: A1,
    arg2: A2,
    arg3: A3,
) -> R {
    unsafe {
        let fn_ptr = transmute::<
            unsafe extern "C" fn(),
            unsafe extern "C" fn(
                receiver: OPtr,
                selector: Sel,
                arg0: A0,
                arg1: A1,
                arg2: A2,
                arg3: A3,
            ) -> R,
        >(objc_msgSend as unsafe extern "C" fn());
        fn_ptr(receiver, selector, arg0, arg1, arg2, arg3)
    }
}

pub fn make_class(name: &CStr) -> Option<CPtr> {
    let cls = objc_allocateClassPair(NSObject::cls(), CStrPtr::new(name), 0)?;
    Some(cls.register())
}

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct TypedObj<T>(OPtr, PhantomData<T>);

impl<T> crate::objc::TypedPtr for TypedObj<T> {
    unsafe fn new(obj: OPtr) -> Self {
        Self(obj, PhantomData)
    }
    fn obj(&self) -> OPtr {
        self.0
    }
}

impl<T> TypedObj<T> {
    pub fn get_inner(&mut self) -> &mut T {
        unsafe { &mut *self.0.get_index_ivars() }
    }

    fn init(alloc_obj: AllocObj<Self>) -> Self {
        unsafe { msg0::<Self>(alloc_obj.obj(), sel::init.sel()) }
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct TypedCls<T, P>(CPtr, PhantomData<(T, P)>);

impl<T, S> TypedCls<T, S> {
    pub fn make_subclass(cls: CPtr, name: &CStr) -> Option<Self> {
        let cls = make_subclass(cls, name)?;
        Some(Self(cls, PhantomData))
    }
}

impl<T, P: TypedPtr> TypedCls<T, P> {
    pub fn cls(&self) -> CPtr {
        self.0
    }

    pub fn make_class(name: &CStr) -> Option<Self> {
        let cls = make_class(name)?;
        Some(Self(cls, PhantomData))
    }

    pub fn alloc_init_upcasted(self, inner: T) -> P {
        let alloc_obj = unsafe { self.0.alloc_indexed::<TypedObj<T>>(size_of::<T>()) };
        let mut obj = TypedObj::init(alloc_obj);
        let new_inner = obj.get_inner();
        *new_inner = inner;
        unsafe { P::new(obj.0) }
    }

    pub fn alloc_upcasted(self, inner: T) -> AllocObj<P> {
        let mut alloc_obj = unsafe { self.0.alloc_indexed::<TypedObj<T>>(size_of::<T>()) };
        let obj_inner = unsafe { alloc_obj.0.get_index_ivars() };
        *obj_inner = inner;
        AllocObj(alloc_obj.0, PhantomData)
    }
}

pub trait TypedPtr: Sized {
    unsafe fn new(obj: OPtr) -> Self;
    fn obj(&self) -> OPtr;
}

pub trait InstancePtr: TypedPtr {
    fn cls() -> CPtr;
    fn alloc() -> AllocObj<Self> {
        unsafe { Self::cls().alloc::<Self>() }
    }
}

/// # Safety
/// Implementors must ensure that the type
pub unsafe trait Protocol {
    fn new(obj: OPtr) -> Self;
}

macro_rules! c_stringify {
    ($str:expr) => {
        const {
            match std::ffi::CStr::from_bytes_with_nul(concat!(stringify!($str), "\0").as_bytes()) {
                Ok(cstr) => cstr,
                Err(_) => unreachable!(),
            }
        }
    };
}

macro_rules! objc_sel {
    ( $sel:ident ) => {
        #[allow(nonstandard_style)]
        pub static $sel: crate::objc::crimes::StaticSelPtr =
            crate::objc::crimes::StaticSelPtr::new(crate::objc::crimes::c_stringify!($sel));
    };
}

macro_rules! objc_prop_sel {
    ( $prop:ident ) => {
        #[allow(nonstandard_style)]
        pub mod $prop {
            pub static GETTER: crate::objc::crimes::StaticSelPtr =
                crate::objc::crimes::StaticSelPtr::new(crate::objc::crimes::c_stringify!($prop));
            pub static SETTER: crate::objc::crimes::StaticSelPtr =
                crate::objc::crimes::StaticSelPtr::new(crate::objc::crimes::c_stringify!($prop));
        }
    };
}

macro_rules! objc_prop_sel_init {
    ( $prop:ident ) => {
        sel::$prop::GETTER.init();
        sel::$prop::SETTER.init_setter();
    };
}

macro_rules! objc_prop_impl {
    ( $prop:ident, $prop_type:ty, $getter:ident ) => {
        pub fn $getter(&self) -> $prop_type {
            unsafe { msg0::<$prop_type>(self.0, sel::$prop::GETTER.sel()) }
        }
    };
    ( $prop:ident, $prop_type:ty, $getter:ident, $setter:ident ) => {
        objc_prop_impl!($prop, $prop_type, $getter);
        pub fn $setter(&self, arg: $prop_type) {
            unsafe { msg1::<(), $prop_type>(self.0, sel::$prop::SETTER.sel(), arg) };
        }
    };
}

macro_rules! objc_class {
    ( $type:ident) => {
        objc_class!($type, $type, (Debug, Clone, Copy));
    };
    ( $type:ident, $cls:ident, $derives:tt ) => {
        #[allow(nonstandard_style)]
        pub mod $type {
            use crate::objc::crimes::{
                CPtr, InstancePtr, OPtr, StaticClsPtr, TypedPtr, c_stringify, AllocObj
            };
            #[derive $derives ]
            #[repr(transparent)]
            pub struct IPtr(pub(super) OPtr);

            impl TypedPtr for IPtr {
                unsafe fn new(obj: OPtr) -> Self {
                    Self(obj)
                }
                fn obj(&self) -> OPtr {
                    self.0
                }
            }

            impl InstancePtr for IPtr {
                fn cls() -> CPtr {
                    CLS.cls()
                }
            }

            pub fn cls() -> CPtr {
                CLS.cls()
            }

            pub fn obj() -> OPtr {
                CLS.cls().obj()
            }

            pub fn init() {
                CLS.init();
            }

            pub fn alloc() -> AllocObj<IPtr> {
                IPtr::alloc()
            }

            pub static CLS: StaticClsPtr = StaticClsPtr::new(c_stringify!($cls));
        }
    };
}

macro_rules! objc_protocol {
    ( $type:ident ) => {
        #[allow(nonstandard_style)]
        pub mod $type {
            use crate::objc::crimes::{OPtr, TypedPtr};
            #[derive(Debug, Clone, Copy)]
            #[repr(transparent)]
            pub struct PPtr(pub(super) OPtr);

            impl TypedPtr for PPtr {
                unsafe fn new(obj: OPtr) -> Self {
                    Self(obj)
                }
                fn obj(&self) -> OPtr {
                    self.0
                }
            }
        }
    };
}

macro_rules! derive_BitOr {
    ($type:ty) => {
        impl std::ops::BitOr for $type {
            type Output = Self;

            fn bitor(self, rhs: Self) -> Self::Output {
                Self(self.0 | rhs.0)
            }
        }
    };
}

pub(crate) use c_stringify;
pub(crate) use derive_BitOr;
pub(crate) use objc_class;
pub(crate) use objc_prop_impl;
pub(crate) use objc_prop_sel;
pub(crate) use objc_prop_sel_init;
pub(crate) use objc_protocol;
pub(crate) use objc_sel;

objc_class!(NSString, NSString, (Clone, Copy));
objc_class!(NSObject);
objc_class!(NSException);

pub mod sel {
    // NSObject
    objc_sel!(alloc);
    objc_sel!(init);

    // NSException
    objc_prop_sel!(name);
    objc_prop_sel!(reason);

    // NSString
    objc_sel!(UTF8String);
    objc_sel!(stringWithUTF8String_);
}
pub fn init_objc_core() {
    NSObject::init();
    NSException::init();
    NSString::init();

    // NSObject
    sel::alloc.init();
    sel::init.init();

    // NSException
    objc_prop_sel_init!(name);
    objc_prop_sel_init!(reason);

    // NSString
    sel::stringWithUTF8String_.init();
    sel::UTF8String.init();

    NSSetUncaughtExceptionHandler(handle_exception);
}

impl NSException::IPtr {
    objc_prop_impl!(name, NSString::IPtr, name, set_name);
    objc_prop_impl!(reason, NSString::IPtr, reason, set_reason);
}

extern "C" fn handle_exception(e: NSException::IPtr) {
    eprintln!("{:?}", e.name().as_cstr());
}

impl NSString::IPtr {
    pub fn new(s: &CStr) -> NSString::IPtr {
        // SAFETY: OK.
        unsafe {
            msg1::<NSString::IPtr, CStrPtr>(
                NSString::obj(),
                sel::stringWithUTF8String_.sel(),
                CStrPtr::new(s),
            )
        }
    }

    pub fn as_cstr(&self) -> &CStr {
        // SAFETY: OK. the CStrPtr lifetime is constrained by the output &CStr, which is constrained by &self
        unsafe { msg0::<CStrPtr>(self.0, sel::UTF8String.sel()) }.to_cstr()
    }
}

impl Debug for NSString::IPtr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.as_cstr().fmt(f)
    }
}

#[test]
fn test_ns_string() {
    init_objc_core();
    let s = NSString::IPtr::new(c"huhheiやー");
    assert_eq!(s.as_cstr(), c"huhheiやー");
}
