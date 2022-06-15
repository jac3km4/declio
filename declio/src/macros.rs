/// Defines a type that encodes and decodes as a constant byte string.
///
/// When decoding, the bytes read will be compared against the given string,
/// and an error will be returned if there is a mismatch.
///
/// # Example
///
/// ```
/// use declio::util::magic_bytes;
///
/// // creates a `pub struct Foo;` and a `struct Bar;`:
/// magic_bytes! {
///     #[derive(Debug)]
///     pub Foo(b"FOO");
///
///     #[derive(Debug)]
///     Bar(b"BAR");
/// }
///
/// let bytes: Vec<u8> = declio::to_bytes(&Foo).unwrap();
/// assert_eq!(bytes, b"FOO");
///
/// assert!(declio::from_bytes::<Foo>(&bytes).is_ok());
/// assert!(declio::from_bytes::<Bar>(&bytes).is_err());
/// ```
#[macro_export]
macro_rules! magic_bytes {
    ($($(#[$attr:meta])* $vis:vis $name:ident($bytes:expr);)*) => {$(
        $(#[$attr])*
        $vis struct $name;

        impl <C>$crate::Encode<C> for $name {
            #[inline]
            fn encode<W>(&self, _ctx: C, writer: &mut W) -> Result<(), $crate::Error>
            where
                W: std::io::Write,
            {
                ($bytes).encode((), writer)
            }
        }

        impl <C> $crate::Decode<C> for $name {
            #[inline]
            fn decode<R>(_ctx: C, reader: &mut R) -> Result<Self, $crate::Error>
            where
                R: std::io::Read,
            {
                let bytes: [u8; ($bytes).len()] = $crate::Decode::decode((), reader)?;
                if &bytes != $bytes {
                    return Err($crate::Error::new(format!(
                        "magic bytes mismatch: expected {:x?}, got {:x?}",
                        $bytes, bytes,
                    )));
                }
                Ok(Self)
            }
        }

        impl<'a, Ctx> $crate::EncodedSize<Ctx> for $name {
            #[inline]
            fn encoded_size(&self, _ctx: Ctx) -> usize {
                $bytes.len()
            }
        }
    )*}
}
