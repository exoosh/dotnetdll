use super::metadata::index::Token;
use dotnetdll_macros::instructions;

instructions! {
    prefixes {
        #[target(Call, Callvirt)]
        Constrained(Token) = 0x16,
        #[target(Castclass, Unbox, Ldelem*, Stelem*, Ldfld, Stfld, Callvirt, Ldvirtftn)]
        Nocheck(u8) = 0x19,
        #[target(Ldelema)]
        Readonly = 0x1E,
        #[target(Call, Calli, Callvirt)]
        Tail = 0x14,
        #[target(Ldind*, Stind*, Ldfld, Stfld, Ldobj, Stobj, Initblk, Cpblk)]
        #[compose(Volatile)]
        Unaligned(u8) = 0x12,
        #[target(Ldind*, Stind*, Ldfld, Stfld, Ldobj, Stobj, Initblk, Cpblk, Ldsfld, Stsfld)]
        #[compose(Unaligned)]
        Volatile = 0x13
    }

    // base instructions
    Add = 0x58,
    AddOvf = 0xD6,
    AddOvfUn = 0xD7,
    And = 0x5F,
    #[extended] Arglist = 0x00,
    Beq(i32) = 0x3B,
    BeqS(i8) = 0x2E,
    Bge(i32) = 0x3C,
    BgeS(i8) = 0x2F,
    BgeUn(i32) = 0x41,
    BgeUnS(i8) = 0x34,
    Bgt(i32) = 0x3D,
    BgtS(i8) = 0x30,
    BgtUn(i32) = 0x42,
    BgtUnS(i8) = 0x35,
    Ble(i32) = 0x3E,
    BleS(i8) = 0x31,
    BleUn(i32) = 0x43,
    BleUnS(i8) = 0x36,
    Blt(i32) = 0x3F,
    BltS(i8) = 0x32,
    BltUn(i32) = 0x44,
    BltUnS(i8) = 0x37,
    BneUn(i32) = 0x40,
    BneUnS(i8) = 0x33,
    Br(i32) = 0x38,
    BrS(i8) = 0x2B,
    Break = 0x01,
    Brfalse(i32) = 0x39,
    BrfalseS(i8) = 0x2C,
    Brtrue(i32) = 0x3A,
    BrtrueS(i8) = 0x2D,
    Call(Token) = 0x28,
    Calli(Token) = 0x29,
    #[extended] Ceq = 0x01,
    #[extended] Cgt = 0x02,
    #[extended] CgtUn = 0x03,
    Ckfinite = 0xC3,
    #[extended] Clt = 0x04,
    #[extended] CltUn = 0x05,
    ConvI1 = 0x67,
    ConvI2 = 0x68,
    ConvI4 = 0x69,
    ConvI8 = 0x6A,
    ConvR4 = 0x6B,
    ConvR8 = 0x6C,
    ConvU1 = 0xD2,
    ConvU2 = 0xD1,
    ConvU4 = 0x6D,
    ConvU8 = 0x6E,
    ConvI = 0xD3,
    ConvU = 0xE0,
    ConvRUn = 0x76,
    ConvOvfI1 = 0xB3,
    ConvOvfI2 = 0xB5,
    ConvOvfI4 = 0xB7,
    ConvOvfI8 = 0xB9,
    ConvOvfU1 = 0xB4,
    ConvOvfU2 = 0xB6,
    ConvOvfU4 = 0xB8,
    ConvOvfU8 = 0xBA,
    ConvOvfI = 0xD4,
    ConvOvfU = 0xD5,
    ConvOvfI1Un = 0x82,
    ConvOvfI2Un = 0x83,
    ConvOvfI4Un = 0x84,
    ConvOvfI8Un = 0x85,
    ConvOvfU1Un = 0x86,
    ConvOvfU2Un = 0x87,
    ConvOvfU4Un = 0x88,
    ConvOvfU8Un = 0x89,
    ConvOvfIUn = 0x8A,
    ConvOvfUUn = 0x8B,
    #[extended] Cpblk = 0x17,
    Div = 0x5B,
    DivUn = 0x5C,
    Dup = 0x25,
    #[extended] Endfilter = 0x11,
    Endfinally = 0xDC,
    #[extended] Initblk = 0x18,
    Jmp(Token) = 0x27,
    #[extended] Ldarg(u16) = 0x09,
    LdargS(u8) = 0x0E,
    Ldarg0 = 0x02,
    Ldarg1 = 0x03,
    Ldarg2 = 0x04,
    Ldarg3 = 0x05,
    #[extended] Ldarga(u16) = 0x0A,
    LdargaS(u8) = 0x0F,
    LdcI4(i32) = 0x20,
    LdcI8(i64) = 0x21,
    LdcR4(f32) = 0x22,
    LdcR8(f64) = 0x23,
    LdcI40 = 0x16,
    LdcI41 = 0x17,
    LdcI42 = 0x18,
    LdcI43 = 0x19,
    LdcI44 = 0x1A,
    LdcI45 = 0x1B,
    LdcI46 = 0x1C,
    LdcI47 = 0x1D,
    LdcI48 = 0x1E,
    LdcI4M1 = 0x15,
    LdcI4S(i8) = 0x1F,
    #[extended] Ldftn(Token) = 0x06,
    LdindI1 = 0x46,
    LdindI2 = 0x48,
    LdindI4 = 0x4A,
    LdindI8 = 0x4C,
    LdindU1 = 0x47,
    LdindU2 = 0x49,
    LdindU4 = 0x4B,
    LdindR4 = 0x4E,
    LdindR8 = 0x4F,
    LdindI = 0x4D,
    LdindRef = 0x50,
    #[extended] Ldloc(u16) = 0x0C,
    LdlocS(u8) = 0x11,
    Ldloc0 = 0x06,
    Ldloc1 = 0x07,
    Ldloc2 = 0x08,
    Ldloc3 = 0x09,
    #[extended] Ldloca(u16) = 0x0D,
    LdlocaS(u8) = 0x12,
    Ldnull = 0x14,
    Leave(i32) = 0xDD,
    LeaveS(i8) = 0xDE,
    #[extended] Localloc = 0x0F,
    Mul = 0x5A,
    MulOvf = 0xD8,
    MulOvfUn = 0xD9,
    Neg = 0x65,
    Nop = 0x00,
    Not = 0x66,
    Or = 0x60,
    Pop = 0x26,
    Rem = 0x5D,
    RemUn = 0x5E,
    Ret = 0x2A,
    Shl = 0x62,
    Shr = 0x63,
    ShrUn = 0x64,
    #[extended] Starg(u16) = 0x0B,
    StargS(u8) = 0x10,
    StindI1 = 0x52,
    StindI2 = 0x53,
    StindI4 = 0x54,
    StindI8 = 0x55,
    StindR4 = 0x56,
    StindR8 = 0x57,
    StindI = 0xDF,
    StindRef = 0x51,
    #[extended] Stloc(u16) = 0x0E,
    StlocS(u8) = 0x13,
    Stloc0 = 0x0A,
    Stloc1 = 0x0B,
    Stloc2 = 0x0C,
    Stloc3 = 0x0D,
    Sub = 0x59,
    SubOvf = 0xDA,
    SubOvfUn = 0xDB,
    Switch(Vec<i32>) = 0x45,
    Xor = 0x61,

    // object model instructions
    Box(Token) = 0x8C,
    Callvirt(Token) = 0x6F,
    Castclass(Token) = 0x74,
    Cpobj(Token) = 0x70,
    #[extended] Initobj(Token) = 0x15,
    Isinst(Token) = 0x75,
    Ldelem(Token) = 0xA3,
    LdelemI1 = 0x90,
    LdelemI2 = 0x92,
    LdelemI4 = 0x94,
    LdelemI8 = 0x96,
    LdelemU1 = 0x91,
    LdelemU2 = 0x93,
    LdelemU4 = 0x95,
    LdelemR4 = 0x98,
    LdelemR8 = 0x99,
    LdelemI = 0x97,
    LdelemRef = 0x9A,
    Ldelema(Token) = 0x8F,
    Ldfld(Token) = 0x7B,
    Ldflda(Token) = 0x7C,
    Ldlen = 0x8E,
    Ldobj(Token) = 0x71,
    Ldsfld(Token) = 0x7E,
    Ldsflda(Token) = 0x7F,
    Ldstr(Token) = 0x72,
    Ldtoken(Token) = 0xD0,
    #[extended] Ldvirtftn(Token) = 0x07,
    Mkrefany(Token) = 0xC6,
    Newarr(Token) = 0x8D,
    Newobj(Token) = 0x73,
    #[extended] Refanytype = 0x1D,
    Refanyval(Token) = 0xC2,
    #[extended] Rethrow = 0x1A,
    #[extended] Sizeof(Token) = 0x1C,
    Stelem(Token) = 0xA4,
    StelemI1 = 0x9C,
    StelemI2 = 0x9D,
    StelemI4 = 0x9E,
    StelemI8 = 0x9F,
    StelemR4 = 0xA0,
    StelemR8 = 0xA1,
    StelemI = 0x9B,
    StelemRef = 0xA2,
    Stfld(Token) = 0x7D,
    Stobj(Token) = 0x81,
    Stsfld(Token) = 0x80,
    Throw = 0x7A,
    Unbox(Token) = 0x79,
    UnboxAny(Token) = 0xA5
}
