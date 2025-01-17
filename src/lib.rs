//! So, you have a nice `async fn` and you want to store a future it returns in
//! a struct. There's no need for boxing or dynamic dispatch: you statically
//! know the type. You just need to `#[name_it]`.
//!
//! ```rust
//! # use name_it::name_it;
//! # use futures::executor::block_on;
//! # async fn do_something_very_async() {}
//! #[name_it(Test)]
//! async fn add(x: i32, y: i32) -> i32 {
//!     do_something_very_async().await;
//!     x + y
//! }
//!
//! # fn main() {
//! let foo: Test = add(2, 3);
//! assert_eq!(block_on(foo), 5);
//! # }
//! ```
#![doc = include_str!("../readme-parts/main.md")]
#![no_std]
// lint me harder
#![forbid(non_ascii_idents)]
#![deny(
    future_incompatible,
    keyword_idents,
    elided_lifetimes_in_paths,
    meta_variable_misuse,
    noop_method_call,
    pointer_structural_match,
    unused_lifetimes,
    unused_qualifications,
    unsafe_op_in_unsafe_fn,
    clippy::undocumented_unsafe_blocks,
    clippy::wildcard_dependencies,
    clippy::debug_assert_with_mut_call,
    clippy::empty_line_after_outer_attr,
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::redundant_field_names,
    clippy::rest_pat_in_fully_bound_structs,
    clippy::unneeded_field_pattern,
    clippy::useless_let_if_seq,
    clippy::default_union_representation
)]
#![warn(clippy::pedantic)]
// not that hard:
#![allow(
    // ideally all the functions must be optimized to nothing, which requires always inlining
    clippy::inline_always,
    // we don't actually export functions, so it's not needed
    clippy::must_use_candidate,
)]

use core::{
    future::Future,
    marker::PhantomPinned,
    mem::{ManuallyDrop, MaybeUninit},
    pin::Pin,
    task::{Context, Poll},
};
use core::marker::PhantomData;
/// A way to name the return type of an async function. See [crate docs](crate)
/// for more info.
pub use name_it_macros::name_it;
pub use name_it_macros::async_trait;

// Manual formatting looks better here
#[rustfmt::skip]
#[doc(hidden)]
pub mod markers;

// SAFETY: can only be implemented on functions returning `Self::Fut`
#[doc(hidden)]
pub unsafe trait FutParams {
    type Fut: Future<Output = Self::Output>;
    type Output;
}

#[doc(hidden)]
pub use elain as _elain;

#[doc(hidden)]
// This function is never called, it's only a placeholder
#[allow(clippy::panic)]
pub fn any<T>(_: &str) -> T {
    panic!()
}

#[doc(hidden)]
#[macro_export]
macro_rules! _produce_any {
    ($f:ident $($xs:pat),*$(,)?) => {
        $f(
            $($crate::any(stringify!($xs))),*
        )
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! _name_it_inner {
    ($v:vis type $name:ident = $func:ident($($underscores:tt)*) -> $ret:ty$(;)?) => {
        #[repr(C)]
        $v struct $name
        where
            $crate::_elain::Align<{$crate::align_of_fut(&($func as fn($($underscores)*) -> _))}>: $crate::_elain::Alignment,
        {
            bytes: [::core::mem::MaybeUninit<u8>; $crate::size_of_fut(&($func as fn($($underscores)*) -> _))],
            _alignment: $crate::_elain::Align<{$crate::align_of_fut(&($func as fn($($underscores)*) -> _))}>,
            // FIXME: invariant is probably too strict
            _markers: $crate::markers!($crate::_produce_any!($func $($underscores)*)),
        }

        impl $name {
            #[doc(hidden)]
            $v unsafe fn new(bytes: [::core::mem::MaybeUninit<u8>; $crate::size_of_fut(&($func as fn($($underscores)*) -> _))]) -> Self {
                Self {
                    bytes,
                    _alignment: $crate::_elain::Align::NEW,
                    _markers: $crate::markers::Markers::new(),
                }
            }
        }

        impl ::core::future::Future for $name {
            type Output = $ret;

            fn poll(self: ::core::pin::Pin<&mut Self>, cx: &mut ::core::task::Context<'_>) -> ::core::task::Poll<$ret> {
                // SAFETY:
                // 1. `::poll()` is safe since we're not lying about the type
                // 2. `transmute()` is safe since the representation is the same
                unsafe {
                    $crate::poll(
                        ::core::mem::transmute::<
                            _, ::core::pin::Pin<&mut [::core::mem::MaybeUninit<u8>; $crate::size_of_fut(&($func as fn($($underscores)*) -> _))]>
                        >(self),
                        cx, $func as fn($($underscores)*) -> _
                    )
                }
            }
        }

        impl ::core::ops::Drop for $name {
            fn drop(&mut self) {
                // SAFETY: this is the only `::dispose()` call and we're not lying about the type
                unsafe {
                    $crate::dispose(&mut self.bytes, ($func as fn($($underscores)*) -> _));
                }
            }
        }
    };
}

/// Wrapper type for named futures.
///
/// Type of your future will be something like
/// ```rust,ignore
/// type YourName<'fut> = Named</* implementation detail */>;
/// ```
#[repr(transparent)]
pub struct Named<T> {
    // Oh, we read this field, just not as you expected, poor rustc
    #[allow(dead_code)]
    inner: T,
    _pinned: PhantomPinned,
}

impl<T> Named<T> {
    #[doc(hidden)]
    pub fn new(inner: T) -> Self {
        Self {
            inner,
            _pinned: PhantomPinned,
        }
    }
}

impl<T> Future for Named<T>
where
    T: Future,
{
    type Output = T::Output;

    #[inline]
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // SAFETY: the representation is the same
        unsafe { core::mem::transmute::<_, Pin<&mut T>>(self) }.poll(cx)
    }
}

#[repr(C)]
union Transmute<From, To> {
    from: ManuallyDrop<From>,
    to: ManuallyDrop<To>,
}

#[inline]
#[doc(hidden)]
pub unsafe fn transmute_generic<From, To>(val: From) -> To {
    ManuallyDrop::into_inner(
        // SAFETY: caller-guaranteed
        unsafe {
            Transmute::<From, To> {
                from: ManuallyDrop::new(val),
            }
            .to
        },
    )
}

#[inline]
#[doc(hidden)]
pub unsafe fn poll<F: FutParams, const N: usize>(
    this: Pin<&mut [MaybeUninit<u8>; N]>,
    cx: &mut Context<'_>,
    _: F,
) -> Poll<F::Output> {
    // SAFETY: `transmute_generic()` is safe because caller promised us that's the
    // type inside
    let fut = unsafe { transmute_generic::<Pin<&mut _>, Pin<&mut F::Fut>>(this) };
    fut.poll(cx)
}

#[inline]
#[doc(hidden)]
pub unsafe fn dispose<F: FutParams, const N: usize>(this: &mut [MaybeUninit<u8>; N], _: F) {
    // SAFETY: caller promised us that's the type inside
    let fut = unsafe { transmute_generic::<&mut _, &mut MaybeUninit<F::Fut>>(this) };
    // SAFETY: we're only calling this one time, in our `Drop`, and never use this
    // after
    unsafe { fut.assume_init_drop() };
}

#[doc(hidden)]
pub const fn size_of_fut<F: FutParams>(_: &F) -> usize {
    core::mem::size_of::<F::Fut>()
}

#[doc(hidden)]
pub const fn align_of_fut<F: FutParams>(_: &F) -> usize {
    core::mem::align_of::<F::Fut>()
}

macro_rules! impl_fut_params {
    ($($t:ident $($ts:ident)*)?) => {
        // SAFETY: we're implementing this for a function returning `Fut`
        unsafe impl<$($t, $($ts,)*)? R, Fut> FutParams for fn($($t, $($ts,)*)?) -> Fut
        where
            Fut: Future<Output = R>
        {
            type Fut = Fut;
            type Output = R;
        }

        $(impl_fut_params!($($ts)*);)?
    };
}

impl_fut_params!(
    T00 T01 T02 T03 T04 T05 T06 T07 T08 T09 T10 T11 T12 T13 T14 T15
    T16 T17 T18 T19 T20 T21 T22 T23 T24 T25 T26 T27 T28 T29 T30 T31
);


