/// Defines a type that encodes and decodes as a constant byte string.
///
/// When decoding, the bytes read will be compared against the given string,
/// and an error will be returned if there is a mismatch.
///
/// # Example
///
/// ```
/// use declio::util::magic_bytes;
/// use declio::ctx::Endian;
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
/// let bytes: Vec<u8> = declio::to_bytes(&Foo, Endian::Big).unwrap();
/// assert_eq!(bytes, b"FOO");
///
/// assert!(declio::from_bytes::<Foo>(&bytes, Endian::Big).is_ok());
/// assert!(declio::from_bytes::<Bar>(&bytes, Endian::Big).is_err());
/// ```
#[macro_export]
macro_rules! magic_bytes {
    ($($(#[$attr:meta])* $vis:vis $name:ident($bytes:expr);)*) => {$(
        $(#[$attr])*
        $vis struct $name;

        impl $crate::Encode<()> for $name {
            fn encode<W>(&self, _ctx: (), endian: Endian, writer: &mut W) -> Result<(), $crate::Error>
            where
                W: std::io::Write,
            {
                ($bytes).encode((), endian, writer)
            }
        }

        impl $crate::Decode<()> for $name {
            fn decode<R>(_ctx: (), endian: Endian, reader: &mut R) -> Result<Self, $crate::Error>
            where
                R: std::io::Read,
            {
                let bytes: [u8; ($bytes).len()] = $crate::Decode::decode((), endian, reader)?;
                if &bytes != $bytes {
                    return Err($crate::Error::new(format!(
                        "magic bytes mismatch: expected {:x?}, got {:x?}",
                        $bytes, bytes,
                    )));
                }
                Ok(Self)
            }
        }
    )*}
}
