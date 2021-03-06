//! Reasoning about the maximum encoded size of types.

/// The `SerializedSize` trait is implemented by types that have a predictable
/// maximum size when encoded using `hubpack`.
///
/// `SerializedSize` is implemented for common standard types, and a derive
/// macro is available for your custom types.
pub trait SerializedSize {
    /// Maximum encoded size of `Self`, in bytes.
    const MAX_SIZE: usize;
}

macro_rules! size_derives {
    ($( $t:ty = $n:expr; )*) => {
        $(
            impl SerializedSize for $t {
                const MAX_SIZE: usize = $n;
            }
        )*
    };
}

size_derives! {
    () = 0;

    u8 = 1;
    u16 = 2;
    u32 = 4;
    u64 = 8;
    u128 = 16;

    i8 = 1;
    i16 = 2;
    i32 = 4;
    i64 = 8;
    i128 = 16;

    f32 = 4;
    f64 = 8;

    bool = 1;
    char = 4;
}

const fn const_max(a: usize, b: usize) -> usize {
    if a > b { a } else { b }
}

impl<T: SerializedSize> SerializedSize for Option<T> {
    const MAX_SIZE: usize = 1 + T::MAX_SIZE;
}

impl<T: SerializedSize, E: SerializedSize> SerializedSize for Result<T, E> {
    const MAX_SIZE: usize = 1 + const_max(T::MAX_SIZE, E::MAX_SIZE);
}

impl<T: SerializedSize, const N: usize> SerializedSize for [T; N] {
    const MAX_SIZE: usize = N * T::MAX_SIZE;
}

impl<A: SerializedSize> SerializedSize for (A,) {
    const MAX_SIZE: usize = A::MAX_SIZE;
}

macro_rules! tuple_impl {
    ($a:ident, $($rest:ident),+) => {
        impl<$a: SerializedSize, $($rest: SerializedSize),+> SerializedSize for ($a, $($rest),+) {
            const MAX_SIZE: usize = $a::MAX_SIZE + <($($rest,)*)>::MAX_SIZE;
        }
    };
}

tuple_impl!(A, B);
tuple_impl!(A, B, C);
tuple_impl!(A, B, C, D);
tuple_impl!(A, B, C, D, E);
tuple_impl!(A, B, C, D, E, F);
tuple_impl!(A, B, C, D, E, F, G);
tuple_impl!(A, B, C, D, E, F, G, H);
tuple_impl!(A, B, C, D, E, F, G, H, I);
tuple_impl!(A, B, C, D, E, F, G, H, I, J);
tuple_impl!(A, B, C, D, E, F, G, H, I, J, K);
tuple_impl!(A, B, C, D, E, F, G, H, I, J, K, L);
