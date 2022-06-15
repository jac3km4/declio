use std::marker::PhantomData;

use crate::{Decode, Encode, EncodedSize, Error};

pub struct Via<A, V>(PhantomData<(A, V)>);

impl<A, V> Via<A, V> {
    #[allow(missing_docs)]
    #[inline]
    pub fn encode<'a, Ctx, W>(a: &'a A, ctx: Ctx, writer: &mut W) -> Result<(), Error>
    where
        W: std::io::Write,
        V: Encode<Ctx> + From<&'a A>,
    {
        V::from(a).encode(ctx, writer)
    }

    #[allow(missing_docs)]
    #[inline]
    pub fn decode<R, Ctx>(ctx: Ctx, reader: &mut R) -> Result<A, Error>
    where
        R: std::io::Read,
        V: Decode<Ctx> + Into<A>,
    {
        Ok(V::decode(ctx, reader)?.into())
    }

    #[allow(missing_docs)]
    #[inline]
    pub fn encoded_size<'a, Ctx>(val: &'a A, ctx: Ctx) -> usize
    where
        V: EncodedSize<Ctx> + From<&'a A>,
    {
        V::from(val).encoded_size(ctx)
    }
}
