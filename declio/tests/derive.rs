use declio::ctx::Endian;
use declio::{Decode, Encode};
use std::fmt::Debug;

#[derive(Debug, PartialEq, Encode, Decode)]
struct UnitStruct;

#[derive(Debug, PartialEq, Encode, Decode)]
struct TupleStruct(u8, u32);

#[derive(Debug, PartialEq, Encode, Decode)]
struct Struct {
    x: u8,
    y: u32,
}

#[derive(Debug, PartialEq, Encode, Decode)]
#[declio(id_type = "u8")]
enum Enum {
    #[declio(id = "0")]
    Unit,
    #[declio(id = "1")]
    Tuple(u8, u32),
    #[declio(id = "2")]
    Struct { x: u8, y: u32 },
}

#[derive(Debug, PartialEq, Encode, Decode)]
struct FieldCtx {
    y: u32,
}

#[derive(Debug, PartialEq, Encode, Decode)]
struct ContainerCtx {
    y: u32,
}

#[derive(Debug, PartialEq, Encode, Decode)]
#[declio(id_type = "u16")]
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
    y: Option<u32>,
}

fn test_encode<T, Ctx>(input: T, expected: &[u8], ctx: Ctx, endian: Endian)
where
    T: Encode<Ctx>,
{
    let output = declio::to_bytes_with_context(&input, ctx, endian).unwrap();
    assert_eq!(output, expected);
}

fn test_decode<T, Ctx>(input: &[u8], expected: &T, ctx: Ctx, endian: Endian)
where
    T: Decode<Ctx> + Debug + PartialEq,
{
    let output: T = declio::from_bytes_with_context(input, ctx, endian).unwrap();
    assert_eq!(output, *expected);
}

fn test_bidir<T>(val: T, bytes: &[u8], endian: Endian)
where
    T: Encode + Decode + Debug + PartialEq,
{
    test_bidir_ctx(val, bytes, (), endian);
}

fn test_bidir_ctx<T, Ctx>(val: T, bytes: &[u8], ctx: Ctx, endian: Endian)
where
    T: Encode<Ctx> + Decode<Ctx> + Debug + PartialEq,
    Ctx: Copy,
{
    test_encode(&val, bytes, ctx, endian);
    test_decode(bytes, &val, ctx, endian);
}

#[test]
fn unit_struct() {
    test_bidir(UnitStruct, &[], Endian::Big);
}

#[test]
fn tuple_struct() {
    test_bidir(
        TupleStruct(0xab, 0xdeadbeef),
        &[0xab, 0xde, 0xad, 0xbe, 0xef],
        Endian::Big,
    );
}

#[test]
fn struct_encode() {
    test_bidir(
        Struct {
            x: 0xab,
            y: 0xdeadbeef,
        },
        &[0xab, 0xde, 0xad, 0xbe, 0xef],
        Endian::Big,
    );
}

#[test]
fn unit_enum() {
    test_bidir(Enum::Unit, &[0x00], Endian::Big);
}

#[test]
fn tuple_enum() {
    test_bidir(
        Enum::Tuple(0xab, 0xdeadbeef),
        &[0x01, 0xab, 0xde, 0xad, 0xbe, 0xef],
        Endian::Big,
    );
}

#[test]
fn struct_enum() {
    test_bidir(
        Enum::Struct {
            x: 0xab,
            y: 0xdeadbeef,
        },
        &[0x02, 0xab, 0xde, 0xad, 0xbe, 0xef],
        Endian::Big,
    );
}

#[test]
fn field_ctx() {
    test_bidir(
        FieldCtx { y: 0xdeadbeef },
        &[0xef, 0xbe, 0xad, 0xde],
        Endian::Little,
    );
}

#[test]
fn id_ctx() {
    test_bidir(IdCtx::Bar, &[0x01, 0x00], Endian::Little);
}

#[test]
fn id_expr() {
    test_bidir_ctx(IdExpr::Baz, &[], 2u8, Endian::Big);
}

#[test]
fn skip_if() {
    test_bidir(SkipIf { x: 8, y: None }, &[0x08], Endian::Big);
    test_bidir(
        SkipIf { x: 7, y: Some(2) },
        &[0x07, 0x00, 0x00, 0x00, 0x02],
        Endian::Big,
    );
}
