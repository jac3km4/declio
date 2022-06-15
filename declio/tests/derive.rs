use declio::ctx::Endian;
use declio::util::{BigEndian, Bytes, PrefixVec};
use declio::{ctx, to_bytes_with_context, Decode, Encode, EncodedSize};
use declio_derive::EncodedSize;
use std::fmt::Debug;
use std::io;

#[derive(Debug, PartialEq, Encode, Decode, EncodedSize)]
struct UnitStruct;

#[derive(Debug, PartialEq, Encode, Decode, EncodedSize)]
struct TupleStruct(u8, BigEndian<u32>);

#[derive(Debug, PartialEq, Encode, Decode, EncodedSize)]
struct Struct {
    x: u8,
    y: BigEndian<u32>,
}

#[derive(Debug, PartialEq, Encode, Decode, EncodedSize)]
#[declio(id_type = "u8")]
enum Enum {
    #[declio(id = "0")]
    Unit,
    #[declio(id = "1")]
    Tuple(u8, BigEndian<u32>),
    #[declio(id = "2")]
    Struct { x: u8, y: BigEndian<u32> },
}

#[derive(Debug, PartialEq, Encode, Decode, EncodedSize)]
struct With {
    #[declio(with = "little_endian")]
    y: u32,
}

#[derive(Debug, PartialEq, Encode, Decode, EncodedSize)]
#[declio(ctx_is = "Endian::Little")]
struct Via {
    #[declio(via = "PrefixVec<u8, u32>")]
    x: Vec<u32>,
    #[declio(via = "Bytes<u16>")]
    y: Vec<u8>,
}

#[derive(Debug, PartialEq, Encode, Decode)]
struct WithSeparate {
    #[declio(
        encode_with = "little_endian::encode",
        decode_with = "little_endian::decode"
    )]
    y: u32,
}

#[derive(Debug, PartialEq, Encode, Decode, EncodedSize)]
#[declio(ctx_is = "ctx::Endian::Little")]
struct FieldCtx {
    x: u32,
    #[declio(ctx = "(ctx::Len(1), ctx::Endian::Little)")]
    y: Vec<u32>,
}

#[derive(Debug, PartialEq, Encode, Decode, EncodedSize)]
struct FieldCtx2 {
    a: FieldCtx,
}

#[derive(Debug, PartialEq, Encode, Decode, EncodedSize)]
#[declio(ctx = "endian: ctx::Endian")]
struct ContainerCtx {
    #[declio(ctx = "endian")]
    y: u32,
}

#[derive(Debug, PartialEq, Encode, Decode, EncodedSize)]
#[declio(id_type = "u16", id_ctx = "ctx::Endian::Little")]
enum IdCtx {
    #[declio(id = "1")]
    Bar,
}

#[derive(Debug, PartialEq, Encode, Decode)]
#[declio(ctx = "id: u8", id_expr = "id")]
enum IdExpr {
    #[declio(id = "1")]
    Bar,
    #[declio(id = "2")]
    Baz,
}

#[derive(Debug, PartialEq, Encode, Decode, EncodedSize)]
struct SkipIf {
    x: u8,
    #[declio(skip_if = "*x == 8")]
    y: Option<BigEndian<u32>>,
}

mod little_endian {
    use super::*;

    pub fn encode<W, C>(x: &u32, _: C, writer: &mut W) -> Result<(), declio::Error>
    where
        W: io::Write,
    {
        x.encode(ctx::Endian::Little, writer)
    }

    pub fn decode<R, C>(_: C, reader: &mut R) -> Result<u32, declio::Error>
    where
        R: io::Read,
    {
        u32::decode(ctx::Endian::Little, reader)
    }

    pub fn encoded_size<Ctx>(_b: &u32, _ctx: Ctx) -> usize {
        4
    }
}

fn test_encode<T, Ctx>(input: T, expected: &[u8], ctx: Ctx)
where
    T: Encode<Ctx>,
{
    let output = declio::to_bytes_with_context(&input, ctx).unwrap();
    assert_eq!(output, expected);
}

fn test_decode<T, Ctx>(input: &[u8], expected: &T, ctx: Ctx)
where
    T: Decode<Ctx> + Debug + PartialEq,
{
    let output: T = declio::from_bytes_with_context(input, ctx).unwrap();
    assert_eq!(output, *expected);
}

fn test_bidir<T>(val: T, bytes: &[u8])
where
    T: Encode<()> + Decode<()> + Debug + PartialEq,
{
    test_bidir_ctx(val, bytes, ());
}

fn test_bidir_ctx<T, Ctx>(val: T, bytes: &[u8], ctx: Ctx)
where
    T: Encode<Ctx> + Decode<Ctx> + Debug + PartialEq,
    Ctx: Copy,
{
    test_encode(&val, bytes, ctx);
    test_decode(bytes, &val, ctx);
}

#[test]
fn unit_struct() {
    test_bidir(UnitStruct, &[]);

    assert_eq!(UnitStruct.encoded_size(()), 0)
}

#[test]
fn tuple_struct() {
    let val = TupleStruct(0xab, 0xdeadbeef.into());
    assert_eq!(val.encoded_size(()), 5);
    test_bidir(val, &[0xab, 0xde, 0xad, 0xbe, 0xef]);
}

#[test]
fn struct_encode() {
    let val = Struct {
        x: 0xab,
        y: 0xdeadbeef.into(),
    };
    assert_eq!(val.encoded_size(()), 5);
    test_bidir(val, &[0xab, 0xde, 0xad, 0xbe, 0xef]);
}

#[test]
fn unit_enum() {
    test_bidir(Enum::Unit, &[0x00]);
    assert_eq!(Enum::Unit.encoded_size(()), 1);
}

#[test]
fn tuple_enum() {
    let val = Enum::Tuple(0xab, 0xdeadbeef.into());
    assert_eq!(val.encoded_size(()), 6);
    test_bidir(val, &[0x01, 0xab, 0xde, 0xad, 0xbe, 0xef]);
}

#[test]
fn struct_enum() {
    let val = Enum::Struct {
        x: 0xab,
        y: 0xdeadbeef.into(),
    };
    assert_eq!(val.encoded_size(()), 6);
    test_bidir(val, &[0x02, 0xab, 0xde, 0xad, 0xbe, 0xef]);
}

#[test]
fn with() {
    let val = With { y: 0xdeadbeef };
    assert_eq!(val.encoded_size(()), 4);
    test_bidir(val, &[0xef, 0xbe, 0xad, 0xde]);
}

#[test]
fn with_separate() {
    test_bidir(WithSeparate { y: 0xdeadbeef }, &[0xef, 0xbe, 0xad, 0xde]);
}

#[test]
fn field_ctx() {
    let val = FieldCtx {
        x: 1,
        y: vec![0xdeadbeef],
    };
    let ctx = (1, Endian::Little);
    assert_eq!(val.encoded_size((1, Endian::Little)), 8);
    let r = to_bytes_with_context(val, ctx).unwrap();

    assert_eq!(r, vec![0x1, 0x0, 0x0, 0x0, 0xef, 0xbe, 0xad, 0xde])
}

#[test]
fn container_ctx() {
    test_bidir_ctx(
        ContainerCtx { y: 0xdeadbeef },
        &[0xef, 0xbe, 0xad, 0xde],
        ctx::Endian::Little,
    );
}

#[test]
fn id_ctx() {
    test_bidir(IdCtx::Bar, &[0x01, 0x00]);
}

#[test]
fn id_expr() {
    test_bidir_ctx(IdExpr::Baz, &[], 2u8);
}

#[test]
fn skip_if() {
    test_bidir(SkipIf { x: 8, y: None }, &[0x08]);
    assert_eq!(SkipIf { x: 8, y: None }.encoded_size(()), 1);

    let some = SkipIf {
        x: 7,
        y: Some(2.into()),
    };
    assert_eq!(some.encoded_size(()), 5);
    test_bidir(some, &[0x07, 0x00, 0x00, 0x00, 0x02]);
}

#[test]
fn via() {
    let val = Via {
        x: vec![1],
        y: vec![2],
    };
    assert_eq!(val.encoded_size(()), 8);
    let r = to_bytes_with_context(val, ()).unwrap();

    assert_eq!(r, vec![0x1, 0x1, 0x0, 0x0, 0x0, 0x1, 0x0, 0x2])
}
