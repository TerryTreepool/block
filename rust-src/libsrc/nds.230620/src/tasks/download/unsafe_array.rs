
use core::ptr::NonNull;
use std::{marker::{PhantomData, }, alloc::{Layout, alloc, dealloc}, sync::Arc};

use near_base::{NearResult, NearError, ErrorCode};
use near_core::near_error;

struct NearUnique<T: ?Sized> {
    ptr: *mut T,
    _marker: PhantomData<T>,
}

impl<T: ?Sized> NearUnique<T> {
    pub fn new(ptr: *mut T) -> Option<Self> {
        if ptr.is_null() {
            None
        } else {
            Some(Self {
                ptr: ptr as _,
                _marker: Default::default(),
            })
        }
    }

    pub unsafe fn new_unchecked(ptr: *mut T) -> Self {
        Self {
            ptr: ptr as _,
            _marker: Default::default(),
        }
    }

    #[inline]
    pub const fn as_ptr(self) -> *const T {
        self.ptr
    }

    #[inline]
    pub const fn as_mut(self) -> *mut T {
        self.ptr
    }

}

unsafe impl<T: ?Sized> Send for NearUnique<T> { }

unsafe impl<T: ?Sized> Sync for NearUnique<T> { }

impl<T: ?Sized> Copy for NearUnique<T> { }

impl<T: ?Sized> Clone for NearUnique<T> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

struct UnsafeArray<T> {
    ptr: NearUnique<T>,
    cap: usize,
}
    
impl<T> UnsafeArray<T> {

    pub fn with_capacity(count: usize) -> NearResult<Self> {

        let ptr = unsafe {
            let layout = Layout::array::<*const T>(count).map_err(| e | {
                near_error!(ErrorCode::NEAR_ERROR_OUTOFMEMORY, format!("Cloud not alloc [{}*{}] memory.", count, std::mem::size_of::<*const T>()))
            })?;

            alloc(layout)
        };


        unsafe {
            Ok(Self {
                ptr: NearUnique::new_unchecked(ptr as _),
                cap: count,
            })
        }
    }

    pub fn swap(&self, index: usize, item: *const T) -> Option<*const T> {
        if index < self.cap {

            let align_size = std::mem::size_of::<*const T>() * index;

            unsafe {
                let r = self.ptr.as_ptr().add(align_size);

                std::ptr::copy(item, self.ptr.as_mut().add(align_size), 1);

                Some(r)
            }
        } else {
            None
        }
    }

    pub fn index_of(&self, index: usize) -> Option<*const T> {
        if index < self.cap {
            let align_size = std::mem::size_of::<*const T>() * index;

            unsafe {
                Some(self.ptr.as_ptr().add(align_size))
            }
        } else {
            None
        }
    }
}

#[test]
fn test_unsafearray() {

    #[derive(Debug)]
    struct AB {
        a: u8,
        b: u16,
        c: u32,
    };

    let data = {
        let mut data = vec![];

        for i in 0u8..10 {
            data.push(
                Arc::new(AB{
                    a: i,
                    b: i as u16 * 10,
                    c: i as u32 * 100,
                })
            )
        }

        data
    };

    let data2 = {
        let mut data = vec![];

        for i in 11u8..15 {
            data.push(
                Arc::new(AB{
                    a: i,
                    b: i as u16 * 10,
                    c: i as u32 * 100,
                })
            )
        }

        data

    };

    let array = UnsafeArray::<AB>::with_capacity(10).unwrap();

    for (i, ptr) in data.iter().enumerate() {
        array.swap(i, Arc::as_ptr(ptr));
        // array.swap(i, ptr.clone())
    }

    println!("/////////1");
    for i in 0..data.len() {
        if let Some(a) = array.index_of(i) {
            let a = unsafe { &*a };
            println!("{:?}", a);
        }
    }

    println!("/////////2");
    for (i, ptr) in data2.iter().enumerate() {
        array.swap(i+3, Arc::as_ptr(ptr));
    }
    for i in 0..data.len() {
        if let Some(a) = array.index_of(i) {
            let a = unsafe { &*a };
            println!("{:?}", a);
        }
    }

    println!("/////////3");
    for (i, ptr) in data2.iter().enumerate() {
        array.swap(i+3, Arc::as_ptr(ptr));
    }
    for i in 0..data.len() {
        if let Some(a) = array.index_of(i) {
            let a = unsafe { &*a };
            println!("{:?}", a);
        }
    }

    if let Some(a0) = data2.get(1) {
        unsafe {
            let a = Arc::as_ptr(a0) as *mut AB;
            (*a).a = 199;
        }
    }

    println!("/////////4");
    for i in 0..data.len() {
        if let Some(a) = array.index_of(i) {
            // let a = unsafe { &*a };
            let a = { a };
            println!("{:?}", a);
        }
    }




    // println!("{:?}", data);
    println!("{:?}", data2);


    drop(array);
}

// #[test]
// fn test_pointer() {
//     #[derive(Debug)]
//     struct AB {
//         a: u8,
//         b: u16,
//     }

//     let mut orig_array = vec![];
//     let mut fixed_array = Vec<*const AB>;

//     for i in 0..4 {
//         orig_array.push(Arc::new(AB{
//             a: i,
//             b: i as u16 * 10,
//         }))
//     }

//     println!("{:?}", orig_array);
// }