use declio::ctx::Endian;
use declio::util::BigEndian;
use declio::{ctx, to_bytes_with_context, Decode, Encode};
use std::fmt::Debug;
use std::io;

#[derive(Debug, PartialEq, Encode, Decode)]
struct UnitStruct;

#[derive(Debug, PartialEq, Encode, Decode)]
struct TupleStruct(u8, BigEndian<u32>);

#[derive(Debug, PartialEq, Encode, Decode)]
struct Struct {
    x: u8,
    y: BigEndian<u32>,
}

#[derive(Debug, PartialEq, Encode, Decode)]
#[declio(id_type = "u8")]
enum Enum {
    #[declio(id = "0")]
    Unit,
    #[declio(id = "1")]
    Tuple(u8, BigEndian<u32>),
    #[declio(id = "2")]
    Struct { x: u8, y: BigEndian<u32> },
}

#[derive(Debug, PartialEq, Encode, Decode)]
struct With {
    #[declio(with = "little_endian")]
    y: u32,
}

#[derive(Debug, PartialEq, Encode, Decode)]
struct WithSeparate {
    #[declio(
        encode_with = "little_endian::encode",
        decode_with = "little_endian::decode"
    )]
    y: u32,
}

#[derive(Debug, PartialEq, Encode, Decode)]
#[declio(ctx_is = "ctx::Endian::Little")]
struct FieldCtx {
    x: u32,
    #[declio(ctx = "(ctx::Len(1), ctx::Endian::Little)")]
    y: Vec<u32>,
}

#[derive(Debug, PartialEq, Encode, Decode)]
struct FieldCtx2 {
    a: FieldCtx
}


#[derive(Debug, PartialEq, Encode, Decode)]
#[declio(ctx = "endian: ctx::Endian")]
struct ContainerCtx {
    #[declio(ctx = "endian")]
    y: u32,
}

#[derive(Debug, PartialEq, Encode, Decode)]
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

#[derive(Debug, PartialEq, Encode, Decode)]
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
}

#[test]
fn tuple_struct() {
    test_bidir(
        TupleStruct(0xab, 0xdeadbeef.into()),
        &[0xab, 0xde, 0xad, 0xbe, 0xef],
    );
}

#[test]
fn struct_encode() {
    test_bidir(
        Struct {
            x: 0xab,
            y: 0xdeadbeef.into(),
        },
        &[0xab, 0xde, 0xad, 0xbe, 0xef],
    );
}

#[test]
fn unit_enum() {
    test_bidir(Enum::Unit, &[0x00]);
}

#[test]
fn tuple_enum() {
    test_bidir(
        Enum::Tuple(0xab, 0xdeadbeef.into()),
        &[0x01, 0xab, 0xde, 0xad, 0xbe, 0xef],
    );
}

#[test]
fn struct_enum() {
    test_bidir(
        Enum::Struct {
            x: 0xab,
            y: 0xdeadbeef.into(),
        },
        &[0x02, 0xab, 0xde, 0xad, 0xbe, 0xef],
    );
}

#[test]
fn with() {
    test_bidir(With { y: 0xdeadbeef }, &[0xef, 0xbe, 0xad, 0xde]);
}

#[test]
fn with_separate() {
    test_bidir(WithSeparate { y: 0xdeadbeef }, &[0xef, 0xbe, 0xad, 0xde]);
}

#[test]
fn field_ctx() {
    let r = to_bytes_with_context(
        FieldCtx {
            x: 1,
            y: vec![0xdeadbeef],
        },
        (1, Endian::Little),
    )
    .unwrap();
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
    test_bidir(
        SkipIf {
            x: 7,
            y: Some(2.into()),
        },
        &[0x07, 0x00, 0x00, 0x00, 0x02],
    );
}
