pub trait SerializedSize {
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

impl<A: SerializedSize, B: SerializedSize> SerializedSize for (A, B) {
    const MAX_SIZE: usize = const_max(A::MAX_SIZE, B::MAX_SIZE);
}

impl<A: SerializedSize, B: SerializedSize, C: SerializedSize> SerializedSize for (A, B, C) {
    const MAX_SIZE: usize = const_max(A::MAX_SIZE, <(B, C)>::MAX_SIZE);
}


