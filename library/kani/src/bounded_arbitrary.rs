// Copyright Kani Contributors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! This module introduces the `Arbitrary` trait as well as implementation for
//! primitive types and other std containers.

use std::ops::Deref;

pub trait BoundedArbitrary {
    fn bounded_any<const N: usize>() -> Self;
}

pub fn bounded_any<T: BoundedArbitrary, const N: usize>() -> T {
    T::bounded_any::<N>()
}

#[derive(PartialEq, Clone, Debug)]
pub struct BoundedAny<T, const N: usize>(T);

impl<T, const N: usize> BoundedAny<T, N> {
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T, const N: usize> std::ops::Deref for BoundedAny<T, N> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T, const N: usize> AsRef<T> for BoundedAny<T, N>
where
    <BoundedAny<T, N> as std::ops::Deref>::Target: AsRef<T>,
{
    fn as_ref(&self) -> &T {
        self.deref()
    }
}

impl<T, const N: usize> kani::Arbitrary for BoundedAny<T, N>
where
    T: BoundedArbitrary,
{
    fn any() -> Self {
        BoundedAny(T::bounded_any::<N>())
    }
}

impl<T: kani::Arbitrary> BoundedArbitrary for Vec<T> {
    fn bounded_any<const N: usize>() -> Self {
        let real_length = kani::any_where(|&size| size <= N);
        let boxed_array: Box<[T; N]> = Box::new(kani::any());

        let mut vec = <[T]>::into_vec(boxed_array);

        // SAFETY: real length is less then or equal to N
        unsafe {
            vec.set_len(real_length);
        }

        vec
    }
}

impl BoundedArbitrary for String {
    fn bounded_any<const N: usize>() -> Self {
        let bytes: [u8; N] = kani::any();

        let mut string = String::new();
        bytes.utf8_chunks().for_each(|chunk| string.push_str(chunk.valid()));
        string
    }
}

impl BoundedArbitrary for std::ffi::OsString {
    fn bounded_any<const N: usize>() -> Self {
        let bounded_string = String::bounded_any::<N>();
        bounded_string.into()
    }
}

impl<K, V> BoundedArbitrary
    for std::collections::HashMap<K, V, std::hash::BuildHasherDefault<std::hash::DefaultHasher>>
where
    K: kani::Arbitrary + std::cmp::Eq + std::hash::Hash,
    V: kani::Arbitrary,
{
    fn bounded_any<const N: usize>() -> Self {
        let mut hash_map = std::collections::HashMap::default();
        for _ in 0..N {
            hash_map.insert(K::any(), V::any());
        }
        hash_map
    }
}

impl<T> BoundedArbitrary for Option<T>
where
    T: BoundedArbitrary,
{
    fn bounded_any<const N: usize>() -> Self {
        let opt: Option<()> = kani::any();
        opt.map(|_| T::bounded_any::<N>())
    }
}

impl<T, E> BoundedArbitrary for Result<T, E>
where
    T: BoundedArbitrary,
    E: BoundedArbitrary,
{
    fn bounded_any<const N: usize>() -> Self {
        let opt: Result<(), ()> = kani::any();
        opt.map(|_| T::bounded_any::<N>()).map_err(|_| E::bounded_any::<N>())
    }
}
