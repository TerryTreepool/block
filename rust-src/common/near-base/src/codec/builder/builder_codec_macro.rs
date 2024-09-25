
use std::collections::{BTreeMap, LinkedList, HashMap};
use std::os::raw::c_void;

use generic_array::{GenericArray, ArrayLength};

use crate::codec::builder_codec::{RawFixedBytes, Serialize, Deserialize, };
use crate::errors::{NearResult, NearError, ErrorCode};

macro_rules! Serialize_Class {
    (bool) => {
        impl RawFixedBytes for bool {
            fn raw_bytes() -> usize {
                std::mem::size_of::<bool>()
            }
        }
        impl Serialize for bool {
            fn raw_capacity(&self) -> usize {
                std::mem::size_of::<bool>()
            }
            fn serialize<'a>(&self,
                             buf: &'a mut [u8]) -> NearResult<&'a mut [u8]> {
                (*self as u8).serialize(buf)
            }
        }
        impl Deserialize for bool {
            fn deserialize<'de>(buf: &'de [u8]) -> NearResult<(Self, &'de [u8])> {
                let (v, buf) = u8::deserialize(buf)?;

                match v {
                    0 => Ok((false, buf)),
                    _ => Ok((true, buf)),
                }
            }
        }
    };
    (generic0, $T: ident, $N: ident) => {
        impl<$N: ArrayLength<$T>> RawFixedBytes for GenericArray<$T, $N> {
            fn raw_bytes() -> usize {
                $N::to_usize() * std::mem::size_of::<$T>()
            }
        }

        impl<$N: ArrayLength<$T>> Serialize for GenericArray<$T, $N> {
            fn raw_capacity(&self) -> usize {
                Self::raw_bytes()
            }
            fn serialize<'a>(&self,
                             buf: &'a mut [u8]) -> NearResult<&'a mut [u8]> {
                let capacity = self.raw_capacity();

                if buf.len() < capacity {
                    return Err(NearError::new(ErrorCode::NEAR_ERROR_OUTOFLIMIT, format!("not enough buffer for serialize genericarray, except={}, got={}", capacity, buf.len())));
                }

                let buf = unsafe {
                    libc::memcpy(buf[..capacity].as_mut_ptr() as *mut c_void,
                                 self.as_slice().as_ptr() as *const c_void,
                                 capacity);
                    &mut buf[capacity..]
                };

                Ok(buf)
            }
        }

        impl<$N: ArrayLength<$T>> Deserialize for GenericArray<$T, $N> {
            fn deserialize<'de>(buf: &'de [u8]) -> NearResult<(Self, &'de [u8])> {
                let capacity = Self::raw_bytes();
                if buf.len() < capacity {
                    return Err(NearError::new(ErrorCode::NEAR_ERROR_OUTOFLIMIT, format!("not enough buffer for deserialize genericarray, except={}, got={}", capacity, buf.len())));
                }

                let v = {
                    let mut val = Self::default();
                    unsafe {
                        libc::memcpy(val.as_mut_slice().as_mut_ptr() as *mut c_void,
                                     buf.as_ptr() as *const c_void,
                                     capacity);
                        // std::ptr::copy(buf.as_ptr(), val.as_byte_slice_mut().as_mut_ptr(), capacity);
                    }
                    val
                };

                Ok((v, &buf[capacity..]))
            }
        }
    };
    ($name: ty) => {
        impl RawFixedBytes for $name {
            fn raw_bytes() -> usize {
                std::mem::size_of::<$name>()
            }
        }
        impl Serialize for $name {
            fn raw_capacity(&self) -> usize {
                std::mem::size_of::<$name>()
            }
            fn serialize<'a>(&self,
                             buf: &'a mut [u8]) -> NearResult<&'a mut [u8]> {
                let capacity = self.raw_capacity();

                if buf.len() < capacity {
                    return Err(NearError::new(ErrorCode::NEAR_ERROR_OUTOFLIMIT, "not enough buffer"));
                }

                let buf = { buf[..capacity].copy_from_slice(&self.to_be_bytes()); &mut buf[capacity..] };

                Ok(buf)
            }
        }
        impl Deserialize for $name {
            fn deserialize<'de>(buf: &'de [u8]) -> NearResult<(Self, &'de [u8])> {
                const CAPACITY: usize = std::mem::size_of::<$name>();
                if buf.len() < CAPACITY {
                    return Err(NearError::new(ErrorCode::NEAR_ERROR_OUTOFLIMIT, "not enough buffer"));
                }

                let v = <$name>::from_be_bytes({
                    let mut v = [0u8; CAPACITY];
                    v.copy_from_slice(&buf[..CAPACITY]);
                    v
                });

                Ok((v, &buf[CAPACITY..]))
            }
        }
    };
    ($c1: ty, $($c2: ty), +) => {
        Serialize_Class!($c1);
        Serialize_Class!($($c2), +);
    }
}

macro_rules! Serialize_Variable_Class {
    (string, $name: ty) => {

        impl Serialize for $name {
            fn raw_capacity(&self) -> usize {
                self.as_bytes().raw_capacity()
            }

            fn serialize<'a>(&self,
                             buf: &'a mut [u8]) -> NearResult<&'a mut [u8]> {
                self.as_bytes().serialize(buf)
            }
        }
    };
    (option, $x: ident) => {

        impl<$x: Serialize> Serialize for Option<$x> {
            fn raw_capacity(&self) -> usize {
                if let Some(t) = self {
                    t.raw_capacity() + 1
                } else {
                    1
                }
            }

            fn serialize<'a>(&self,
                             buf: &'a mut [u8]) -> NearResult<&'a mut [u8]> {
                // serialize number
                if let Some(t) = self {
                    let buf = true.serialize(buf)?;
                    let buf = t.serialize(buf)?;
                    Ok(buf)
                } else {
                    false.serialize(buf)
                }
            }
        }

    };
    ($name: ty, $x: ident) => {

        impl<$x: Serialize> Serialize for $name {
            fn raw_capacity(&self) -> usize {
                let mut len = (self.len() as u16).raw_capacity();
                self.iter().for_each(| c | len += c.raw_capacity() );
                len
            }

            fn serialize<'a>(&self,
                             buf: &'a mut [u8]) -> NearResult<&'a mut [u8]> {
                // serialize number
                let buf = (self.len() as u16).serialize(buf)?;
                let mut offset = 0usize;

                for i in self {
                    offset = buf.len() - i.serialize(&mut buf[offset..])?.len();
                }

                Ok(&mut buf[offset..])
            }
        }
    };
    ($name: ty, $x: ident, $y: ident) => {

        impl<$x: Serialize, $y: Serialize> Serialize for $name {
            fn raw_capacity(&self) -> usize {
                let mut len = (self.len() as u16).raw_capacity();
                self.iter().for_each(|(k, v)| len += k.raw_capacity() + v.raw_capacity());
                len
            }

            fn serialize<'a>(&self,
                             buf: &'a mut [u8]) -> NearResult<&'a mut [u8]> {
                let buf = (self.len() as u16).serialize(buf)?;
                let mut offset = 0usize;

                for (k, v) in self {
                    offset = buf.len() - k.serialize(&mut buf[offset..])?.len();
                    offset = buf.len() - v.serialize(&mut buf[offset..])?.len();
                }
                Ok(&mut buf[offset..])
            }
        }
    };
}

macro_rules! Deserialize_Variable_Class {
    ($name: ty; $x: ident => $handler: expr) => {
        impl<$x: Deserialize> Deserialize for $name {
            fn deserialize<'de>(buf: &'de [u8]) -> NearResult<(Self, &'de [u8])> {
                let (count, buf) = u16::deserialize(buf)?;

                let mut class = <$name>::default();
                let remain_len = buf.len();
                let mut offset = 0usize;

                for _ in 0..count {
                    let (v, next) = $x::deserialize(&buf[offset..])?;
                    $handler(&mut class, v);
                    offset = remain_len - next.len();
                }

                Ok((class, &buf[offset..]))
            }
        }
    };
    (option, $x: ident) => {
        impl<$x: Deserialize> Deserialize for Option<$x> {
            fn deserialize<'de>(buf: &'de [u8]) -> NearResult<(Self, &'de [u8])> {
                let (exist, buf) = bool::deserialize(buf)?;

                if exist {
                    let (class, buf) = <$x>::deserialize(buf)?;
                    Ok((Some(class), buf))
                } else {
                    Ok((None, buf))
                }
            }
        }
    };
    (string, $x: ty) => {
        impl Deserialize for String {
            fn deserialize<'de>(buf: &'de [u8]) -> NearResult<(Self, &'de [u8])> {
                let (v, buf) = Vec::<u8>::deserialize(buf)?;

                match String::from_utf8(v) {
                    Ok(str) => Ok((str, buf)),
                    Err(e) => {
                        Err(NearError::new(ErrorCode::NEAR_ERROR_SYSTERM,
                                           format!("failed convert string with error {}", e.to_string())))
                    }
                }
            }
        }
    };
    (hash $name: ty; $x: ident; $y: ident => $handler: expr) => {
        impl<$x: Eq + std::hash::Hash + Deserialize, $y: Deserialize> Deserialize for $name {
            fn deserialize<'de>(buf: &'de [u8]) -> NearResult<(Self, &'de [u8])> {
                let (count, buf) = u16::deserialize(buf)?;

                let mut class = <$name>::default();
                let mut offset = 0usize;
                let remain_len = buf.len();

                for _ in 0..count {
                    let (k, next) = $x::deserialize(&buf[offset..])?;
                    offset = remain_len - next.len();
                    let (v, next) = $y::deserialize(&buf[offset..])?;
                    offset = remain_len - next.len();
                    $handler(&mut class, k, v);
                }

                Ok((class, &buf[offset..]))
            }
        }
    };
    (map $name: ty; $x: ident; $y: ident => $handler: expr) => {
        impl<$x: Ord + Deserialize, $y: Deserialize> Deserialize for $name {
            fn deserialize<'de>(buf: &'de [u8]) -> NearResult<(Self, &'de [u8])> {
                let (count, buf) = u16::deserialize(buf)?;

                let mut class = <$name>::default();
                let mut offset = 0usize;
                let remain_len = buf.len();

                for _ in 0..count {
                    let (k, next) = $x::deserialize(&buf[offset..])?;
                    offset = remain_len - next.len();
                    let (v, next) = $y::deserialize(&buf[offset..])?;
                    offset = remain_len - next.len();
                    $handler(&mut class, k, v);
                }

                Ok((class, &buf[offset..]))
            }
        }
    };
}

macro_rules! Serialize_Tuple {
    () => {
        impl RawFixedBytes for () {
            fn raw_bytes() -> usize {
                0
            }
        }
        impl Serialize for () {
            fn raw_capacity(&self) -> usize {
                0
            }
            fn serialize<'a>(&self,
                             buf: &'a mut [u8]) -> NearResult<&'a mut [u8]> {
                Ok(buf)
            }
        }
        impl Deserialize for () {
            fn deserialize<'de>(buf: &'de [u8]) -> NearResult<(Self, &'de [u8])> {
                Ok(((), buf))
            }
        }

    };
    ($last:ident $($name:ident)*) => {
        impl<$($name,)* $last> Serialize for ($($name,)* $last,)
        where $($name: Serialize + Deserialize,)*
              $last: Serialize + Deserialize {

            #[allow(non_snake_case)]
            fn raw_capacity(&self) -> usize {

                let ($($name,)* $last,) = self;
                [$($name.raw_capacity(),)* $last.raw_capacity()].iter().sum()

            }

            #[allow(non_snake_case)]
            fn serialize<'a>(&self,
                             buf: &'a mut [u8]) -> NearResult<&'a mut [u8]> {

                let ($($name,)* $last,) = self;

                let dst = buf;
                $(let dst = $name.serialize(dst)?;)*
                let dst = $last.serialize(dst)?;

                Ok(dst)
            }
        }

        #[allow(non_snake_case)]
        impl<$($name,)* $last> Deserialize for ($($name,)* $last,)
        where $($name: Serialize + Deserialize,)*
              $last: Serialize + Deserialize {
            fn deserialize<'de>(buf: &'de [u8]) -> NearResult<(Self, &'de [u8])> {

                let dst = buf;

                $(let ($name, dst) = $name::deserialize(dst)?;)*

                let (l, dst) = $last::deserialize(dst)?;

                let r = ($($name,)* l,);

                Ok((r, dst))
                // $(let $name = values.pop_front().unwrap_or(Nil);)*
                // let $last = FromLuaMulti::from_lua_multi(values, lua)?;
                // Ok(($(FromLua::from_lua($name, lua)?,)* $last,))
            }
        }
    }
}

macro_rules! Serialize_Variable_Class_Impl {
    (Option<T>) => {
        Serialize_Variable_Class!(option, T);
        Deserialize_Variable_Class!(option, T);
    };
    ([T]) => {
        Serialize_Variable_Class!([T], T);
    };
    (&str) => {
        Serialize_Variable_Class!(string, &str);
    };
    (String) => {
        Serialize_Variable_Class!(string, String);
        Deserialize_Variable_Class!(string, String);
    };
    (Vec<T>) => {
        Serialize_Variable_Class!(Vec<T>, T);
        Deserialize_Variable_Class!(Vec<T>; T => |a: &mut Vec<T>, b| { a.push(b); });
    };
    (LinkedList<T>) => {
        Serialize_Variable_Class!(LinkedList<T>, T);
        Deserialize_Variable_Class!(LinkedList<T>; T => |a: &mut LinkedList<T>, b| { a.push_back(b); });
    };
    (HashMap<K, V>) => {
        Serialize_Variable_Class!(HashMap<K, V>, K, V);
        Deserialize_Variable_Class!(hash HashMap<K, V>; K; V => |a: &mut HashMap<K, V>, k, v| { a.insert(k, v); });
    };
    (BTreeMap<K, V>) => {
        Serialize_Variable_Class!(BTreeMap<K, V>, K, V);
        Deserialize_Variable_Class!(map BTreeMap<K, V>; K; V => |a: &mut BTreeMap<K, V>, k, v| { a.insert(k, v); });
    };
}

Serialize_Class!(bool);
Serialize_Class!(generic0, u8, N);
Serialize_Class!(generic0, i8, N);
Serialize_Class!(generic0, u16, N);
Serialize_Class!(generic0, i16, N);
Serialize_Class!(generic0, u32, N);
Serialize_Class!(generic0, i32, N);
Serialize_Class!(generic0, u64, N);
Serialize_Class!(generic0, i64, N);
Serialize_Class!(generic0, u128, N);
Serialize_Class!(generic0, i128, N);
Serialize_Class!(usize, u8, u16, u32, u64, u128);
Serialize_Class!(isize, i8, i16, i32, i64, i128);
Serialize_Class!(f32, f64);

Serialize_Variable_Class_Impl!(Option<T>);
Serialize_Variable_Class_Impl!(String);
Serialize_Variable_Class_Impl!(&str);
Serialize_Variable_Class_Impl!([T]);
Serialize_Variable_Class_Impl!(Vec<T>);
Serialize_Variable_Class_Impl!(LinkedList<T>);
Serialize_Variable_Class_Impl!(HashMap<K, V>);
Serialize_Variable_Class_Impl!(BTreeMap<K, V>);

Serialize_Tuple!();
Serialize_Tuple!(A);
Serialize_Tuple!(A B);
Serialize_Tuple!(A B C);
Serialize_Tuple!(A B C D);
Serialize_Tuple!(A B C D E);
Serialize_Tuple!(A B C D E F);
Serialize_Tuple!(A B C D E F G);
Serialize_Tuple!(A B C D E F G H);
Serialize_Tuple!(A B C D E F G H I);
Serialize_Tuple!(A B C D E F G H I J);
Serialize_Tuple!(A B C D E F G H I J K);
Serialize_Tuple!(A B C D E F G H I J K L);
Serialize_Tuple!(A B C D E F G H I J K L M);

#[derive(Clone, Default)]
pub struct Empty;

impl Serialize for Empty {
    fn raw_capacity(&self) -> usize {
        0
    }

    fn serialize<'a>(&self,
                     buf: &'a mut [u8]) -> NearResult<&'a mut [u8]> {
        Ok(buf)
    }
}

impl Deserialize for Empty {
    fn deserialize<'de>(buf: &'de [u8]) -> NearResult<(Self, &'de [u8])> {
        Ok((Empty, buf))
    }
}

impl std::fmt::Display for Empty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Null")
    }
}

impl std::fmt::Debug for Empty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        (self as &dyn std::fmt::Display).fmt(f)
    }
}

/*
impl<N: ArrayLength<u8>> Serialize for GenericArray<u8, N> {
    fn raw_capacity(&self) -> usize {
        self.len()
    }

    fn serialize<'a>(&self,
                     buf: &'a mut [u8],
                     builder: &mut BuilderCounter) -> NearResult<&'a mut [u8]> {
        let (cur, capacity) = self.serialize_head(buf, builder)?;

        let cur = { cur[..capacity].copy_from_slice(&self.as_slice()); &mut cur[capacity..] };

        Ok(cur)
    }
}

impl<N: ArrayLength<u8>> Deserialize for GenericArray<u8, N> {
    fn deserialize<'de>(buf: &'de [u8],
                        builder: &mut BuilderCounter) -> NearResult<(Self, &'de [u8])> {
        let (end, capacity) = {
            if buf.len() < SERIALIZE_HEADER_SIZE {
                return Err(NearError::new(ErrorCode::NEAR_ERROR_OUTOFLIMIT, "not enough buffer"));
            }

            if builder.next() != buf[0] {
                return Err(NearError::new(ErrorCode::NEAR_ERROR_INVALIDFORMAT,
                                            format!("packet target({}!={}) invalid", buf[0], builder.curr())));
            }

            (&buf[SERIALIZE_HEADER_SIZE..], buf[1] as usize)
        };

        if end.len() < capacity {
            return Err(NearError::new(ErrorCode::NEAR_ERROR_OUTOFLIMIT, "not enough buffer"));
        }

        let v = {
            let mut r = vec![0u8; capacity];
            r.copy_from_slice(&end[..capacity]);
            GenericArray::<u8, N>::clone_from_slice(r.as_slice())
        };

        Ok((v, &end[capacity..]))
    }

}
*/

#[cfg(test)]
#[allow(unused)]
mod test_serialize{
    use std::collections::{BTreeMap, LinkedList};
    use std::u8;

    use generic_array::GenericArray;
    use generic_array::typenum::U32;
    use libc::c_void;

    use crate::codec::builder::{Serialize, Deserialize};

    #[test]
    fn test_tuple() {
        let a = (5u8, 7u8, 9u8, 11u8);

        let len = a.raw_capacity();
        println!("len:{len}");
        let mut buf = vec![0u8; len];
        let dst = a.serialize(&mut buf).unwrap();

        println!("{:?}", buf);

        let (b, buf) = 
            <(u8, u8, u8, u8)>::deserialize(&mut buf).unwrap();

        println!("{:?}", b);
    }

    #[test]
    fn t3() {
        let mut buf = vec![0u8; 2000];

        {
            let mut data = Vec::new();
            for i in 0..1500 {
                data.push(i as u8);
            }

            println!("{}", data.raw_capacity());
            data.serialize(&mut buf).unwrap();
        }

        {
            let (data, _) = Vec::<u8>::deserialize(&buf).unwrap();

            println!("{}", data.len());
        }
    }

    #[test]
    fn t2() {

        let mut a: GenericArray<u32, U32> = Default::default();
        {
            let a_len = a.raw_capacity();
            let mut a_array = Vec::new();
            for c in 0..a_len {
                a_array.push(c as u8);
            }
            // a.clone_from_slice(a_array.as_slice());
            unsafe {
                libc::memcpy(a.as_mut_slice().as_mut_ptr() as *mut c_void,
                             a_array.as_slice().as_ptr() as *const c_void,
                             a_len);
            }   
        }

        let mut buffer = [0u8; 128];
        let _ = a.serialize(&mut buffer).unwrap();

        let (b, _) = GenericArray::<u32, U32>::deserialize(&buffer).unwrap();
        println!("{:?}", a.as_slice());
        println!("---");
        println!("{:?}", b.as_slice());

        // let a2 = a.copy_within(src, dest)
        // let a2 = a.byt
        // let a1 = a.as_byte_slice_mut().as_mut_ptr();

        macro_rules! tt1 {
            ($n1: expr, $h: expr) => {
                $h($n1)
            };
            ($n1: expr) => {
                tt1!($n1, |p| p )
            };
        }

        let s = String::from("123");
        let s = tt1!(s);
        println!("tt1: {}", s);

        let _r =
        match String::from_utf8(vec![1,234]) {
            Ok(e) => e,
            Err(e) => {
                e.to_string()
            }
        };


        // println!("{}", (pingjie!(1, 2, 3)));

        let x = u128::MAX as usize;
        let _y = u128::MAX;
        let _s1 = std::mem::size_of::<usize>();
        let _s2 = std::mem::size_of_val(&x);
        let _s3 = std::mem::size_of::<u128>();

        let mut b = [0u8; 1024];
        let _ = 0u8.serialize(&mut b);

        let _ = true.serialize(&mut b);

        let (v2, _) = Vec::<u8>::deserialize(&b).unwrap();
        println!("{:?}", v2);

    }

    #[test]
    fn t1() {
        {
            let mut b3 = [0u8; 1024];

            let a1 = u128::MAX as usize;
            a1.serialize(&mut b3).unwrap();

            let (a11, _) = usize::deserialize(&b3).unwrap();
            assert_eq!(a1, a11);

            let mut b3 = [0u8; 1024];

            let v1 = vec!["abc", "defdfad", "efdxxasf", "er234123"];
            println!("{}", v1.raw_capacity());
            let _build_ptr = v1.serialize(&mut b3).unwrap();
            let _build_ptr_len = _build_ptr.len();
            println!("{}", 1024 - _build_ptr_len);

            let (wv, _end_ptr) = Vec::<String>::deserialize(&b3).unwrap();
            println!("wv={:?}", wv);
        }

        {
            println!("BTreeMap==================================================");
            let mut b = [0u8; 1024];

            let mut v1: BTreeMap<String, u16> = std::collections::BTreeMap::new();
            v1.insert("aaaa".to_string(), 1000u16);
            v1.insert("abbb".to_string(), 1001u16);
            v1.insert("abcc".to_string(), 1010u16);
            v1.insert("abcd".to_string(), 1100u16);
            v1.insert("bbcd".to_string(), 1101u16);
            let _build_ptr = v1.serialize(&mut b);
            println!("wb={:?}", b);

            let (wv, _end_ptr) = BTreeMap::<String, u16>::deserialize(&b).unwrap();
            println!("wv={:?}", wv);
        }

        {
            println!("LinkedList==================================================");
            let mut b = [0u8; 1024];

            let mut v1 = std::collections::LinkedList::new();
            v1.push_back("aaaa");
            v1.push_back("abbb");
            v1.push_back("abcc");
            v1.push_back("abcd");
            v1.push_back("bbcd");
            println!("capacity: {}", v1.raw_capacity());
            let _build_ptr = v1.serialize(&mut b).unwrap();
            let _build_ptr_len = 1024 - _build_ptr.len();
            println!("wc_len: {}", _build_ptr_len);

            let (wv, _end_ptr) = LinkedList::<String>::deserialize(&b).unwrap();
            println!("wv={:?}", wv);
        }

        {
            println!("HashMap==================================================");

            let mut v1 = std::collections::HashMap::new();
            v1.insert("aaaa".to_string(), 1000u16);
            v1.insert("abbb".to_string(), 1001u16);
            v1.insert("abcc".to_string(), 1010u16);
            v1.insert("abcd".to_string(), 1100u16);
            v1.insert("bbcd".to_string(), 1101u16);
            let mut b = vec![0u8; v1.raw_capacity()];

            let _build_ptr = v1.serialize(&mut b);
            println!("wb={:?}", b);

            let (wv, _end_ptr) = std::collections::HashMap::<String, u16>::deserialize(&b).unwrap();
            println!("wv={:?}", wv);
        }

    }
}


