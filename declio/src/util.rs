//! Utilities that aren't part of the "core" of declio, but may be useful in reducing boilerplate.

use std::borrow::Cow;
use std::fmt::Debug;
use std::marker::PhantomData;

use crate::ctx::{Endian, Len};
use crate::{Decode, Encode, EncodedSize, Error};

#[doc(inline)]
pub use crate::magic_bytes;

macro_rules! endian_wrappers {
    ($($(#[$attr:meta])* $name:ident: $endian:expr,)*) => {$(
        $(#[$attr])*
        #[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name<T>(pub T);

        impl<T, C> Encode<C> for $name<T>
        where
            T: Encode<Endian>,
        {
            fn encode<W>(&self, _ctx: C, writer: &mut W) -> Result<(), Error>
            where
                W: std::io::Write,
            {
                self.0.encode($endian, writer)
            }
        }

        impl<T, C> Decode<C> for $name<T>
        where
            T: Decode<Endian>,
        {
            fn decode<R>(_ctx: C, reader: &mut R) -> Result<Self, Error>
            where
                R: std::io::Read,
            {
                T::decode($endian, reader).map(Self)
            }
        }

        impl<T, Ctx> EncodedSize<Ctx> for $name<T>
        where
            T: EncodedSize<Ctx>
        {
            #[inline]
            fn encoded_size(&self, ctx: Ctx) -> usize {
                self.0.encoded_size(ctx)
            }
        }

        impl<T> From<T> for $name<T> {
            fn from(value: T) -> Self {
                Self(value)
            }
        }

        impl<T: Copy> From<&T> for $name<T> {
            fn from(value: &T) -> Self {
                Self(*value)
            }
        }

            // fn from(value: &T) -> Self {
                // Self(*value)
            // }

        /* NB: Orphan rules prohibit something like this:
        impl<T> From<$name<T>> for T {
            fn from(wrapper: $name<T>) -> Self {
                wrapper.0
            }
        }
        */

        impl<T> $name<T> {
            /// Unwraps and returns the inner `T` value.
            pub fn into_inner(self) -> T {
                self.0
            }
        }
    )*}
}

endian_wrappers! {
    /// Little-endian wrapper type for primitives.
    ///
    /// Encodes and decodes the inner type in little-endian format.
    ///
    /// # Example
    ///
    /// ```
    /// use declio::Encode;
    /// use declio::util::LittleEndian;
    ///
    /// type Uint = LittleEndian<u32>;
    ///
    /// let x: Uint = 0xdeadbeef.into();
    ///
    /// let mut bytes = Vec::new();
    /// x.encode((), &mut bytes).unwrap();
    ///
    /// assert_eq!(bytes, &[0xef, 0xbe, 0xad, 0xde]);
    /// ```
    LittleEndian: Endian::Little,

    /// Big-endian wrapper type for primitives.
    ///
    /// Encodes and decodes the inner type in big-endian format.
    ///
    /// # Example
    ///
    /// ```
    /// use declio::Encode;
    /// use declio::util::BigEndian;
    ///
    /// type Uint = BigEndian<u32>;
    ///
    /// let x: Uint = 0xdeadbeef.into();
    ///
    /// let mut bytes = Vec::new();
    /// x.encode((), &mut bytes).unwrap();
    ///
    /// assert_eq!(bytes, &[0xde, 0xad, 0xbe, 0xef]);
    /// ```
    BigEndian: Endian::Big,
}

/// Helper module alternative to [`Utf8`], for use in derive macros.
///
/// # Examples
///
/// ```
/// use declio::{Encode, Decode};
/// use declio::ctx::{Endian, Len};
/// use declio::util::utf8;
/// use std::convert::TryInto;
///
/// #[derive(Debug, PartialEq, Encode, Decode)]
/// pub struct Text {
///     #[declio(ctx = "Endian::Big")]
///     len: u32,
///
///     // Note here, we are using `with = "utf8"` instead of a `Utf8` wrapper type.
///     #[declio(with = "utf8", ctx = "Len((*len).try_into()?)")]
///     value: String,
/// }
///
/// let value = String::from("Hello World");
/// let text = Text {
///     len: value.len().try_into().unwrap(),
///     value,
/// };
///
/// let mut bytes = Vec::new();
/// text.encode((), &mut bytes).unwrap();
/// assert_eq!(bytes, b"\x00\x00\x00\x0bHello World");
///
/// let mut decoder = bytes.as_slice();
/// let decoded = Text::decode((), &mut decoder).unwrap();
/// assert_eq!(decoded, text);
///
/// ```
pub mod utf8 {
    use crate::{Decode, Encode, Error};

    #[allow(missing_docs)]
    pub fn encode<S, Ctx, W>(string: &S, ctx: Ctx, writer: &mut W) -> Result<(), Error>
    where
        S: AsRef<str>,
        [u8]: Encode<Ctx>,
        W: std::io::Write,
    {
        string.as_ref().as_bytes().encode(ctx, writer)
    }

    #[allow(missing_docs)]
    pub fn decode<Ctx, R>(ctx: Ctx, reader: &mut R) -> Result<String, Error>
    where
        Vec<u8>: Decode<Ctx>,
        R: std::io::Read,
    {
        let bytes: Vec<u8> = Decode::decode(ctx, reader)?;
        let string = String::from_utf8(bytes)?;
        Ok(string)
    }

    #[allow(missing_docs)]
    #[inline]
    pub fn encoded_size<S, Ctx>(str: S, _ctx: Ctx) -> usize
    where
        S: AsRef<str>,
    {
        str.as_ref().len()
    }
}

/// UTF-8 wrapper type for strings.
///
/// Encodes and decodes strings as a UTF-8 byte string. Like other sequence types, decoding
/// requires a [`Len`] context value, to specify how many bytes should be read.
///
/// # Examples
///
/// ```
/// use declio::{Encode, Decode};
/// use declio::ctx::{Endian, Len};
/// use declio::util::Utf8;
/// use std::convert::TryInto;
///
/// #[derive(Debug, PartialEq, Encode, Decode)]
/// pub struct Text {
///     #[declio(ctx = "Endian::Big")]
///     len: u32,
///     #[declio(ctx = "Len((*len).try_into()?)")]
///     value: Utf8,
/// }
///
/// let value = String::from("Hello World");
/// let text = Text {
///     len: value.len().try_into().unwrap(),
///     value: Utf8(value),
/// };
///
/// let mut bytes = Vec::new();
/// text.encode((), &mut bytes).unwrap();
/// assert_eq!(bytes, b"\x00\x00\x00\x0bHello World");
///
/// let mut decoder = bytes.as_slice();
/// let decoded = Text::decode((), &mut decoder).unwrap();
/// assert_eq!(decoded, text);
///
/// ```
#[derive(Default, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Utf8(pub String);

impl Encode<Len> for Utf8 {
    fn encode<W>(&self, ctx: Len, writer: &mut W) -> Result<(), Error>
    where
        W: std::io::Write,
    {
        utf8::encode(&self.0, ctx, writer)
    }
}

impl Encode<()> for Utf8 {
    fn encode<W>(&self, _ctx: (), writer: &mut W) -> Result<(), Error>
    where
        W: std::io::Write,
    {
        utf8::encode(&self.0, ((),), writer)
    }
}

impl Decode<Len> for Utf8 {
    fn decode<R>(ctx: Len, reader: &mut R) -> Result<Self, Error>
    where
        R: std::io::Read,
    {
        utf8::decode(ctx, reader).map(Self)
    }
}

impl<Ctx> EncodedSize<Ctx> for Utf8 {
    #[inline]
    fn encoded_size(&self, _ctx: Ctx) -> usize {
        self.0.len()
    }
}

impl From<String> for Utf8 {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<&str> for Utf8 {
    fn from(value: &str) -> Self {
        value.to_string().into()
    }
}

impl From<Utf8> for String {
    fn from(wrapper: Utf8) -> Self {
        wrapper.0
    }
}

/// Helper module alternative to [`ZeroOne`], for use in derive macros.
///
/// # Examples
///
/// ```
/// use declio::{Encode, Decode};
/// use declio::util::zero_one;
///
/// #[derive(Debug, PartialEq, Encode, Decode)]
/// struct MyBoolean {
///     // Note here, we are using `with = "zero_one"` instead of a `ZeroOne` wrapper type.
///     #[declio(with = "zero_one")]
///     value: bool,
/// }
///
/// let value = MyBoolean { value: true };
///
/// let mut bytes = Vec::new();
/// value.encode((), &mut bytes).unwrap();
/// assert_eq!(bytes, b"\x01");
///
/// let mut decoder = bytes.as_slice();
/// let decoded = MyBoolean::decode((), &mut decoder).unwrap();
/// assert_eq!(decoded, value);
/// ```
pub mod zero_one {
    use crate::{Decode, Encode, Error};

    #[allow(missing_docs)]
    pub fn encode<W, C>(b: &bool, _ctx: C, writer: &mut W) -> Result<(), Error>
    where
        W: std::io::Write,
    {
        let byte: u8 = match b {
            false => 0,
            true => 1,
        };
        byte.encode((), writer)
    }

    #[allow(missing_docs)]
    pub fn decode<R, C>(_ctx: C, reader: &mut R) -> Result<bool, Error>
    where
        R: std::io::Read,
    {
        let byte: u8 = Decode::decode((), reader)?;
        match byte {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(Error::new(format!(
                "invalid byte value for boolean: expected 0 or 1, got {:?}",
                byte
            ))),
        }
    }

    #[allow(missing_docs)]
    #[inline]
    pub fn encoded_size<Ctx>(_b: &bool, _ctx: Ctx) -> usize {
        1
    }
}

/// Zero-one wrapper type for booleans.
///
/// Encodes and decodes booleans as a single byte, either a zero `0` for `false`, or a one `1` for
/// true.
///
/// # Examples
///
/// ```
/// use declio::{Encode, Decode};
/// use declio::util::ZeroOne;
///
/// let value = ZeroOne(true);
///
/// let mut bytes = Vec::new();
/// value.encode((), &mut bytes).unwrap();
/// assert_eq!(bytes, b"\x01");
///
/// let mut decoder = bytes.as_slice();
/// let decoded = ZeroOne::decode((), &mut decoder).unwrap();
/// assert_eq!(decoded, value);
/// ```
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ZeroOne(pub bool);

impl<C> Encode<C> for ZeroOne {
    fn encode<W>(&self, _ctx: C, writer: &mut W) -> Result<(), Error>
    where
        W: std::io::Write,
    {
        zero_one::encode(&self.0, (), writer)
    }
}

impl<C> Decode<C> for ZeroOne {
    fn decode<R>(_ctx: C, reader: &mut R) -> Result<Self, Error>
    where
        R: std::io::Read,
    {
        zero_one::decode((), reader).map(Self)
    }
}

impl<Ctx> EncodedSize<Ctx> for ZeroOne {
    #[inline]
    fn encoded_size(&self, ctx: Ctx) -> usize {
        zero_one::encoded_size(&self.0, ctx)
    }
}

impl From<bool> for ZeroOne {
    fn from(value: bool) -> Self {
        Self(value)
    }
}

impl From<ZeroOne> for bool {
    fn from(wrapper: ZeroOne) -> Self {
        wrapper.0
    }
}

#[derive(Debug, Default)]
pub struct NoPrefix;

impl<Ctx> EncodedSize<Ctx> for NoPrefix {
    #[inline]
    fn encoded_size(&self, _ctx: Ctx) -> usize {
        0
    }
}

#[derive(Debug, Clone)]
pub struct Bytes<'a, P = NoPrefix>(Cow<'a, [u8]>, PhantomData<P>);

impl<'a, P> Bytes<'a, P> {
    #[inline]
    pub const fn new(vec: Vec<u8>) -> Self {
        Self(Cow::Owned(vec), PhantomData)
    }

    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    #[inline]
    pub fn into_vec(self) -> Vec<u8> {
        self.0.into_owned()
    }
}

impl<'a, S, P> From<&'a S> for Bytes<'a, P>
where
    S: AsRef<[u8]>,
{
    #[inline]
    fn from(val: &'a S) -> Self {
        Self(Cow::Borrowed(val.as_ref()), PhantomData)
    }
}

impl<'a, P> From<Bytes<'a, P>> for Vec<u8> {
    #[inline]
    fn from(val: Bytes<'a, P>) -> Self {
        val.into_vec()
    }
}

impl<'a, C> Encode<C> for Bytes<'a> {
    #[inline]
    fn encode<W>(&self, _ctx: C, writer: &mut W) -> Result<(), Error>
    where
        W: std::io::Write,
    {
        writer.write_all(&self.0)?;
        Ok(())
    }
}

impl<'a> Decode<Len> for Bytes<'a> {
    #[inline]
    fn decode<R>(len: Len, reader: &mut R) -> Result<Self, Error>
    where
        R: std::io::Read,
    {
        let mut buf = vec![0; len.0];
        reader.read_exact(&mut buf)?;
        Ok(Self::new(buf))
    }
}

impl<'a, P, C> Encode<C> for Bytes<'a, P>
where
    P: Encode<C> + TryFrom<usize>,
    P::Error: std::error::Error,
{
    fn encode<W>(&self, ctx: C, writer: &mut W) -> Result<(), Error>
    where
        W: std::io::Write,
    {
        let size: P = self.0.len().try_into().map_err(Error::new)?;
        size.encode(ctx, writer)?;
        writer.write_all(&self.0)?;
        Ok(())
    }
}

impl<'a, P, C> Decode<C> for Bytes<'a, P>
where
    P: Decode<C> + TryInto<usize>,
    P::Error: std::error::Error,
{
    fn decode<R>(ctx: C, reader: &mut R) -> Result<Self, Error>
    where
        R: std::io::Read,
    {
        let size = P::decode(ctx, reader)?.try_into().map_err(Error::new)?;
        let mut buf = vec![0; size];
        reader.read_exact(&mut buf)?;
        Ok(Self::new(buf))
    }
}

impl<'a, P, Ctx> EncodedSize<Ctx> for Bytes<'a, P>
where
    P: EncodedSize<Ctx> + Default,
{
    #[inline]
    fn encoded_size(&self, ctx: Ctx) -> usize {
        P::default_encoded_size(ctx) + self.0.len()
    }
}

#[derive(Clone)]
pub struct PrefixVec<'a, P, A>(Cow<'a, [A]>, PhantomData<P>)
where
    [A]: ToOwned;

impl<'a, P, A> PrefixVec<'a, P, A>
where
    A: Clone,
{
    #[inline]
    pub const fn new(vec: Vec<A>) -> Self {
        Self(Cow::Owned(vec), PhantomData)
    }

    #[inline]
    pub fn into_vec(self) -> Vec<A> {
        self.0.into_owned()
    }
}

impl<'a, S, P, A> From<&'a S> for PrefixVec<'a, P, A>
where
    S: AsRef<[A]>,
    A: Clone,
{
    #[inline]
    fn from(val: &'a S) -> Self {
        Self(Cow::Borrowed(val.as_ref()), PhantomData)
    }
}

impl<'a, P, A> From<PrefixVec<'a, P, A>> for Vec<A>
where
    A: Clone,
{
    #[inline]
    fn from(val: PrefixVec<'a, P, A>) -> Self {
        val.into_vec()
    }
}

impl<'a, Ctx, P, A> Encode<Ctx> for PrefixVec<'a, P, A>
where
    P: Encode<Ctx> + TryFrom<usize>,
    P::Error: std::error::Error,
    A: Encode<Ctx> + Clone,
    Ctx: Clone,
{
    fn encode<W>(&self, ctx: Ctx, writer: &mut W) -> Result<(), Error>
    where
        W: std::io::Write,
    {
        let size: P = self.0.len().try_into().map_err(Error::new)?;
        size.encode(ctx.clone(), writer)?;
        self.0.as_ref().encode((ctx,), writer)?;
        Ok(())
    }
}

impl<'a, Ctx, P, A> Decode<Ctx> for PrefixVec<'a, P, A>
where
    P: Decode<Ctx> + TryInto<usize>,
    P::Error: std::error::Error,
    A: Decode<Ctx> + Clone,
    Ctx: Clone,
{
    fn decode<R>(ctx: Ctx, reader: &mut R) -> Result<Self, Error>
    where
        R: std::io::Read,
    {
        let size = P::decode(ctx.clone(), reader)?
            .try_into()
            .map_err(Error::new)?;
        let buf = Decode::decode((Len(size), ctx), reader)?;
        Ok(Self::new(buf))
    }
}

impl<'a, Ctx, P, A> EncodedSize<Ctx> for PrefixVec<'a, P, A>
where
    P: EncodedSize<Ctx> + Default,
    A: EncodedSize<Ctx> + Clone,
    Ctx: Clone,
{
    fn encoded_size(&self, ctx: Ctx) -> usize {
        let vec_size: usize = self.0.iter().map(|el| el.encoded_size(ctx.clone())).sum();
        P::default_encoded_size(ctx) + vec_size
    }
}
