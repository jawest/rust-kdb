use crate::any::KAny;
use crate::error::ConversionError;
use crate::raw::kapi;
use crate::raw::types::*;
use std::convert::TryFrom;
use std::ffi::{CStr, CString, NulError};
use std::fmt;
use std::mem;

pub trait KItem {
    const K_TYPE: KType;
    fn as_k_ptr(&self) -> *const K;

    unsafe fn clone_k_ptr(&self) -> *const K {
        kapi::r1(self.as_k_ptr())
    }

    fn k_type(&self) -> KType {
        unsafe { (*self.as_k_ptr()).t }
    }
}

pub struct InvalidKCastError {
    pub from: KType,
    pub to: KType,
}

macro_rules! impl_katom {
    {$type:ident, AtomType = $atom_type:ident, KType = $k_type:ident, Ctor = $ctor:ident, Accessor: $accessor:ident } => {
        #[repr(transparent)]
        pub struct $type(*const K);

        impl KItem for $type {
            const K_TYPE: KType = $k_type;
            fn as_k_ptr(&self) -> *const K { self.0 }
        }

        impl From<$type> for $atom_type {
            fn from(k_atom: $type) -> $atom_type {
                *k_atom
            }
        }

        impl From<$atom_type> for $type {
            fn from(value: $atom_type) -> $type {
                unsafe { $type(kapi::$ctor(value.into())) }
            }
        }

        impl From<$atom_type> for KAny {
            fn from(value: $atom_type) -> KAny {
                unsafe { KAny((kapi::$ctor(value.into()))) }
            }
        }

        impl From<$type> for KAny {
            fn from(item: $type) -> KAny {
                unsafe { mem::transmute(item) }
            }
        }

        impl TryFrom<KAny> for $type {
            type Error = ConversionError;

            fn try_from(any: KAny) -> Result<Self, Self::Error> {
                let t = any.k_type();
                if t == $k_type {
                    Ok(unsafe { mem::transmute(any) })
                } else {
                    Err(ConversionError::InvalidKCast{ from: t, to: $k_type })
                }
            }
        }

        impl fmt::Debug for $type {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                fmt::Debug::fmt(&**self, f)
            }
        }

        impl Drop for $type {
            fn drop(&mut self) {
                unsafe {
                    kapi::r0(self.0);
                }
            }
        }

        impl std::ops::Deref for $type {
            type Target = $atom_type;
            fn deref(&self) -> &Self::Target {
                unsafe { &(*self.0).union.$accessor }
            }
        }
    }
}

impl_katom! {KByteAtom, AtomType = u8, KType = BYTE_ATOM, Ctor = kg, Accessor: g }
impl_katom! {KCharAtom, AtomType = i8, KType = CHAR_ATOM, Ctor = kc, Accessor: c }
impl_katom! {KShortAtom, AtomType = i16, KType = SHORT_ATOM, Ctor = kh, Accessor: h }
impl_katom! {KIntAtom, AtomType = i32, KType = INT_ATOM, Ctor = ki, Accessor: i }
impl_katom! {KLongAtom, AtomType = i64, KType = LONG_ATOM, Ctor = kj, Accessor: j }
impl_katom! {KRealAtom, AtomType = f32, KType = REAL_ATOM, Ctor = ke, Accessor: e }
impl_katom! {KFloatAtom, AtomType = f64, KType = FLOAT_ATOM, Ctor = kf, Accessor: f }
impl_katom! {KBoolAtom, AtomType = bool, KType = BOOLEAN_ATOM, Ctor = kg, Accessor: bl }
impl_katom! {KSecondAtom, AtomType = KSecond, KType = SECOND_ATOM, Ctor = ki, Accessor: sec }
impl_katom! {KMinuteAtom, AtomType = KMinute, KType = MINUTE_ATOM, Ctor = ki, Accessor: min }
impl_katom! {KMonthAtom, AtomType = KMonth, KType = MONTH_ATOM, Ctor = ki, Accessor: m }
impl_katom! {KTimeAtom, AtomType = KTime, KType = TIME_ATOM, Ctor = ki, Accessor: t }
impl_katom! {KDateAtom, AtomType = KDate, KType = DATE_ATOM, Ctor = ki, Accessor: d }
impl_katom! {KDateTimeAtom, AtomType = KDateTime, KType = DATE_TIME_ATOM, Ctor = kf, Accessor: dt }
impl_katom! {KSymbolAtom, AtomType = KSymbol, KType = SYMBOL_ATOM, Ctor = ks, Accessor: sym }
impl_katom! {KGuidAtom, AtomType = KGuid, KType = GUID_ATOM, Ctor = ku, Accessor: u }
impl_katom! {KTimestampAtom, AtomType = KTimestamp, KType = TIMESTAMP_ATOM, Ctor = kj, Accessor: tst }
impl_katom! {KTimespanAtom, AtomType = KTimespan, KType = TIMESPAN_ATOM, Ctor = kj, Accessor: ts }

//Extra convenience conversions implemented manually
impl TryFrom<&str> for KSymbolAtom {
    type Error = NulError;
    fn try_from(val: &str) -> Result<Self, Self::Error> {
        let c_str = CString::new(val)?;
        Ok(KSymbolAtom(unsafe { kapi::ks(c_str.as_ptr()) }))
    }
}

impl TryFrom<String> for KSymbolAtom {
    type Error = NulError;
    fn try_from(val: String) -> Result<Self, Self::Error> {
        Self::try_from(&val[..])
    }
}

impl TryFrom<KSymbolAtom> for String {
    type Error = std::str::Utf8Error;
    fn try_from(val: KSymbolAtom) -> Result<Self, Self::Error> {
        let c_str = unsafe { CStr::from_ptr((*val.0).union.s) };
        c_str.to_str().map(str::to_owned)
    }
}

impl TryFrom<&str> for KAny {
    type Error = NulError;
    fn try_from(val: &str) -> Result<Self, Self::Error> {
        let c_str = CString::new(val)?;
        Ok(KAny(unsafe { kapi::ks(c_str.as_ptr()) }))
    }
}

impl TryFrom<KAny> for KSymbol {
    type Error = ConversionError;

    fn try_from(any: KAny) -> Result<Self, Self::Error> {
        let sym = KSymbolAtom::try_from(any)?;
        Ok(KSymbol::from(sym))
    }
}

impl TryFrom<String> for KAny {
    type Error = NulError;
    fn try_from(val: String) -> Result<Self, Self::Error> {
        Self::try_from(&val[..])
    }
}

impl TryFrom<KAny> for String {
    type Error = ConversionError;

    fn try_from(any: KAny) -> Result<Self, Self::Error> {
        let sym = KSymbolAtom::try_from(any)?;
        String::try_from(sym).map_err(ConversionError::from)
    }
}

pub struct KError(pub(crate) *const K);

impl KError {
    pub fn new(s: &str) -> Result<Self, NulError> {
        KSymbolAtom::try_from(s).map(|sym| {
            let k = mem::ManuallyDrop::new(sym).as_k_ptr() as *mut K;
            unsafe {
                (*k).t = ERROR;
            }
            KError(k)
        })
    }
}

impl KItem for KError {
    const K_TYPE: KType = ERROR;
    fn as_k_ptr(&self) -> *const K {
        self.0
    }
}

impl fmt::Debug for KError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let c_str = unsafe { CStr::from_ptr((*self.0).union.s) };
        if let Ok(s) = c_str.to_str() {
            write!(f, "KError({})", s)
        } else {
            write!(f, "KError(Unknown)")
        }
    }
}

impl TryFrom<KAny> for KError {
    type Error = ConversionError;

    fn try_from(any: KAny) -> Result<Self, Self::Error> {
        let t = any.k_type();
        if t == ERROR {
            Ok(unsafe { mem::transmute(any) })
        } else {
            Err(ConversionError::InvalidKCast { from: t, to: ERROR })
        }
    }
}

impl From<KError> for KAny {
    fn from(err: KError) -> KAny {
        unsafe { mem::transmute(err) }
    }
}

impl TryFrom<KError> for String {
    type Error = std::str::Utf8Error;
    fn try_from(val: KError) -> Result<Self, Self::Error> {
        if unsafe { (*val.0).union.s.is_null() } {
            return Ok(String::new());
        }
        let c_str = unsafe { CStr::from_ptr((*val.0).union.s) };
        c_str.to_str().map(str::to_owned)
    }
}

impl From<KError> for crate::error::Error {
    fn from(val: KError) -> Self {
        if unsafe { (*val.0).union.s.is_null() } {
            return crate::error::Error::UnknownQError;
        }

        let c_str = unsafe { CStr::from_ptr((*val.0).union.s) };
        c_str
            .to_str()
            .map(str::to_owned)
            .map(|e| crate::error::Error::QError(e))
            .unwrap_or(crate::error::Error::UnknownQError)
    }
}
