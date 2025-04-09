// Magic
use crate::*;

struct MagicEntry {
    mask: u128,
    magic: u64,
    shift: u8,
    offset: u32,
}

#[rustfmt::skip]
const ROOK_MAGICS: &[MagicEntry; Square::NUM] = &[
    MagicEntry { mask: 0x80402010080402FE, magic: 0x2140000212485100, shift: 50, offset: 0 },
    MagicEntry { mask: 0x100804020100804FC, magic: 0x0020010200004080, shift: 51, offset: 16384 },
    MagicEntry { mask: 0x201008040201008FA, magic: 0x8040020400102240, shift: 51, offset: 24576 },
    MagicEntry { mask: 0x402010080402010F6, magic: 0x00400095A4800040, shift: 51, offset: 32768 },
    MagicEntry { mask: 0x804020100804020EE, magic: 0x08C0020000A08930, shift: 51, offset: 40960 },
    MagicEntry { mask: 0x1008040201008040DE, magic: 0x0080018428884010, shift: 51, offset: 49152 },
    MagicEntry { mask: 0x2010080402010080BE, magic: 0x0440041060000C08, shift: 51, offset: 57344 },
    MagicEntry { mask: 0x40201008040201007E, magic: 0x808000A058021044, shift: 51, offset: 65536 },
    MagicEntry { mask: 0x8040201008040200FE, magic: 0x0440000401040102, shift: 50, offset: 73728 },
    MagicEntry { mask: 0x804020100805FC00, magic: 0x2810400208000502, shift: 51, offset: 90112 },
    MagicEntry { mask: 0x1008040201009F800, magic: 0x0021100002001040, shift: 52, offset: 98304 },
    MagicEntry { mask: 0x2010080402011F400, magic: 0x0010800120020841, shift: 52, offset: 102400 },
    MagicEntry { mask: 0x4020100804021EC00, magic: 0x80022001000C0040, shift: 52, offset: 106496 },
    MagicEntry { mask: 0x8040201008041DC00, magic: 0x0008400048800212, shift: 52, offset: 110592 },
    MagicEntry { mask: 0x10080402010081BC00, magic: 0x2080400100208890, shift: 52, offset: 114688 },
    MagicEntry { mask: 0x201008040201017C00, magic: 0x0040200040000444, shift: 52, offset: 118784 },
    MagicEntry { mask: 0x40201008040200FC00, magic: 0x5003201000100002, shift: 52, offset: 122880 },
    // Square 2i is problematic. Relaxing shift to 50
    // num_trials=10000000 bad_magics=9255755
    MagicEntry { mask: 0x80402010080401FC00, magic: 0x00000A00880A0142, shift: 50, offset: 126976 },
    MagicEntry { mask: 0x804020100BF80200, magic: 0x002000B000208081, shift: 51, offset: 143360 },
    MagicEntry { mask: 0x10080402013F00400, magic: 0x0082882001040700, shift: 52, offset: 151552 },
    MagicEntry { mask: 0x20100804023E80800, magic: 0x0430001001000221, shift: 52, offset: 155648 },
    MagicEntry { mask: 0x40201008043D81000, magic: 0x4800011000808001, shift: 52, offset: 159744 },
    MagicEntry { mask: 0x80402010083B82000, magic: 0x0004002001041112, shift: 52, offset: 163840 },
    MagicEntry { mask: 0x100804020103784000, magic: 0x0021001000340250, shift: 52, offset: 167936 },
    MagicEntry { mask: 0x201008040202F88000, magic: 0x2080002002302238, shift: 52, offset: 172032 },
    MagicEntry { mask: 0x402010080401F90000, magic: 0x01C0001005B0000A, shift: 52, offset: 176128 },
    MagicEntry { mask: 0x804020100803FA0000, magic: 0x2040001000020082, shift: 51, offset: 180224 },
    MagicEntry { mask: 0x80402017F0040200, magic: 0x0314200208000100, shift: 51, offset: 188416 },
    MagicEntry { mask: 0x100804027E0080400, magic: 0x2000081008008500, shift: 52, offset: 196608 },
    MagicEntry { mask: 0x201008047D0100800, magic: 0x0009400850004080, shift: 52, offset: 200704 },
    MagicEntry { mask: 0x402010087B0201000, magic: 0x2020010420008024, shift: 52, offset: 204800 },
    MagicEntry { mask: 0x80402010770402000, magic: 0x3C0012918800C001, shift: 52, offset: 208896 },
    MagicEntry { mask: 0x1008040206F0804000, magic: 0x5808C0223000C010, shift: 52, offset: 212992 },
    MagicEntry { mask: 0x2010080405F1008000, magic: 0x81C0104008000408, shift: 52, offset: 217088 },
    MagicEntry { mask: 0x4020100803F2010000, magic: 0x0820800008040002, shift: 52, offset: 221184 },
    MagicEntry { mask: 0x8040201007F4020000, magic: 0x0040004808000086, shift: 51, offset: 225280 },
    MagicEntry { mask: 0x80402FE008040200, magic: 0x49A00014210C0001, shift: 51, offset: 233472 },
    MagicEntry { mask: 0x100804FC010080400, magic: 0x2000100002160040, shift: 52, offset: 241664 },
    MagicEntry { mask: 0x201008FA020100800, magic: 0x0150000801080022, shift: 52, offset: 245760 },
    MagicEntry { mask: 0x402010F6040201000, magic: 0x1040020011440040, shift: 52, offset: 249856 },
    MagicEntry { mask: 0x804020EE080402000, magic: 0x0101200900040010, shift: 52, offset: 253952 },
    MagicEntry { mask: 0x1008040DE100804000, magic: 0x0201000028040050, shift: 52, offset: 258048 },
    MagicEntry { mask: 0x2010080BE201008000, magic: 0x21400040002C0008, shift: 52, offset: 262144 },
    MagicEntry { mask: 0x40201007E402010000, magic: 0x214001C000840042, shift: 52, offset: 266240 },
    MagicEntry { mask: 0x8040200FE804020000, magic: 0x2140000C04240006, shift: 51, offset: 270336 },
    MagicEntry { mask: 0x805FC01008040200, magic: 0x4011045044001200, shift: 51, offset: 278528 },
    MagicEntry { mask: 0x1009F802010080400, magic: 0x0010020200020300, shift: 52, offset: 286720 },
    MagicEntry { mask: 0x2011F404020100800, magic: 0x9002000420800080, shift: 52, offset: 290816 },
    // Square 6d is problematic. Relaxing shift to 51
    // num_trials=10000000 bad_magics=7524930
    MagicEntry { mask: 0x4021EC08040201000, magic: 0x8040040300463008, shift: 51, offset: 294912 },
    MagicEntry { mask: 0x8041DC10080402000, magic: 0x0140400404008004, shift: 52, offset: 303104 },
    MagicEntry { mask: 0x10081BC20100804000, magic: 0x0900010000200201, shift: 52, offset: 307200 },
    MagicEntry { mask: 0x201017C40201008000, magic: 0x0100010040000402, shift: 52, offset: 311296 },
    MagicEntry { mask: 0x40200FC80402010000, magic: 0x0028400020000201, shift: 52, offset: 315392 },
    // Square 6i is problematic. Relaxing shift to 50
    // num_trials=10000000 bad_magics=9210623
    MagicEntry { mask: 0x80401FD00804020000, magic: 0x4020880088869001, shift: 50, offset: 319488 },
    MagicEntry { mask: 0xBF80201008040200, magic: 0x0000100188111041, shift: 51, offset: 335872 },
    MagicEntry { mask: 0x13F00402010080400, magic: 0x6004100088008183, shift: 52, offset: 344064 },
    MagicEntry { mask: 0x23E80804020100800, magic: 0x4040040080810001, shift: 52, offset: 348160 },
    MagicEntry { mask: 0x43D81008040201000, magic: 0x1000020008018015, shift: 52, offset: 352256 },
    MagicEntry { mask: 0x83B82010080402000, magic: 0x40188803200060A4, shift: 52, offset: 356352 },
    MagicEntry { mask: 0x103784020100804000, magic: 0x830002044100600C, shift: 52, offset: 360448 },
    MagicEntry { mask: 0x202F88040201008000, magic: 0x1300298412806004, shift: 52, offset: 364544 },
    MagicEntry { mask: 0x401F90080402010000, magic: 0x0080121020001042, shift: 52, offset: 368640 },
    MagicEntry { mask: 0x803FA0100804020000, magic: 0x0021202082021004, shift: 51, offset: 372736 },
    MagicEntry { mask: 0x7F0040201008040200, magic: 0x0440008226020007, shift: 51, offset: 380928 },
    MagicEntry { mask: 0x7E0080402010080400, magic: 0x442000C841004480, shift: 52, offset: 389120 },
    MagicEntry { mask: 0x7D0100804020100800, magic: 0x008002080A000042, shift: 52, offset: 393216 },
    MagicEntry { mask: 0x7B0201008040201000, magic: 0x2900050810808084, shift: 52, offset: 397312 },
    MagicEntry { mask: 0x770402010080402000, magic: 0x2300020202041014, shift: 52, offset: 401408 },
    MagicEntry { mask: 0x6F0804020100804000, magic: 0x94400100021C0008, shift: 52, offset: 405504 },
    MagicEntry { mask: 0x5F1008040201008000, magic: 0x0100026040424004, shift: 52, offset: 409600 },
    MagicEntry { mask: 0x3F2010080402010000, magic: 0x02C0002000100005, shift: 52, offset: 413696 },
    MagicEntry { mask: 0x7F4020100804020000, magic: 0x8480004008021082, shift: 51, offset: 417792 },
    MagicEntry { mask: 0xFE008040201008040200, magic: 0x800010000490A080, shift: 50, offset: 425984 },
    MagicEntry { mask: 0xFC010080402010080400, magic: 0x2228800122080101, shift: 51, offset: 442368 },
    MagicEntry { mask: 0xFA020100804020100800, magic: 0x0422200200004080, shift: 51, offset: 450560 },
    MagicEntry { mask: 0xF6040201008040201000, magic: 0x00C2200200009020, shift: 51, offset: 458752 },
    MagicEntry { mask: 0xEE080402010080402000, magic: 0x5581200100000810, shift: 51, offset: 466944 },
    MagicEntry { mask: 0xDE100804020100804000, magic: 0x8082400066801010, shift: 51, offset: 475136 },
    MagicEntry { mask: 0xBE201008040201008000, magic: 0x4300200010900008, shift: 51, offset: 483328 },
    MagicEntry { mask: 0x7E402010080402010000, magic: 0x028040000820310C, shift: 51, offset: 491520 },
    MagicEntry { mask: 0xFE804020100804020000, magic: 0x0C9820000AA08102, shift: 50, offset: 499712 },
];

pub const ROOK_TABLE_SIZE: usize = 516096;
// more than 5x the length of a Rook table in chess, so more than 10x memory foot print

#[rustfmt::skip]
const BISHOP_MAGICS: &[MagicEntry; Square::NUM] = &[
    MagicEntry { mask: 0x401004010040100400, magic: 0x1010020A00020501, shift: 57, offset: 0 },
    MagicEntry { mask: 0x2008020080200800, magic: 0x0010004224200081, shift: 58, offset: 128 },
    MagicEntry { mask: 0x0010040100401400, magic: 0x002052A020140400, shift: 58, offset: 192 },
    MagicEntry { mask: 0x0000080200882800, magic: 0x0000830091080028, shift: 58, offset: 256 },
    MagicEntry { mask: 0x0000000411105000, magic: 0x0202050090001120, shift: 58, offset: 320 },
    MagicEntry { mask: 0x000000202220A000, magic: 0x0044804101226081, shift: 58, offset: 384 },
    MagicEntry { mask: 0x0000404040414000, magic: 0xC101420080A01100, shift: 58, offset: 448 },
    MagicEntry { mask: 0x0080808080808000, magic: 0x6000240980110041, shift: 58, offset: 512 },
    MagicEntry { mask: 0x10101010101010000, magic: 0x4000600218504004, shift: 57, offset: 576 },
    MagicEntry { mask: 0x200802008020080000, magic: 0x1A0300804C801002, shift: 58, offset: 704 },
    MagicEntry { mask: 0x401004010040100000, magic: 0x0042014100282017, shift: 58, offset: 768 },
    MagicEntry { mask: 0x2008020080280000, magic: 0x4000822100240402, shift: 58, offset: 832 },
    MagicEntry { mask: 0x0010040110500000, magic: 0x1040008104104809, shift: 58, offset: 896 },
    MagicEntry { mask: 0x0000082220A00000, magic: 0x0440824082100010, shift: 58, offset: 960 },
    MagicEntry { mask: 0x0000404441400000, magic: 0x1818120118201000, shift: 58, offset: 1024 },
    MagicEntry { mask: 0x0080808082800000, magic: 0xC610411080082100, shift: 58, offset: 1088 },
    MagicEntry { mask: 0x10101010101000000, magic: 0x101000409080086C, shift: 58, offset: 1152 },
    MagicEntry { mask: 0x20202020202000000, magic: 0x0800001820409402, shift: 58, offset: 1216 },
    MagicEntry { mask: 0x100401004010000400, magic: 0x208102020C218090, shift: 58, offset: 1280 },
    MagicEntry { mask: 0x200802008020000800, magic: 0x8548002025001085, shift: 58, offset: 1344 },
    MagicEntry { mask: 0x401004010050001400, magic: 0x8008050420055011, shift: 56, offset: 1408 },
    MagicEntry { mask: 0x20080220A0002800, magic: 0x0001000080328284, shift: 56, offset: 1664 },
    MagicEntry { mask: 0x0010444140005000, magic: 0x0000800040050800, shift: 56, offset: 1920 },
    MagicEntry { mask: 0x008088828000A000, magic: 0x4202110640080620, shift: 56, offset: 2176 },
    MagicEntry { mask: 0x10101010500014000, magic: 0x00A3080040200410, shift: 56, offset: 2432 },
    MagicEntry { mask: 0x20202020200008000, magic: 0x0208800010086240, shift: 58, offset: 2688 },
    MagicEntry { mask: 0x40404040400010000, magic: 0x0902900401203088, shift: 58, offset: 2752 },
    MagicEntry { mask: 0x80200802000080800, magic: 0x4408860304009410, shift: 58, offset: 2816 },
    MagicEntry { mask: 0x100401004000101000, magic: 0x8028880083021008, shift: 58, offset: 2880 },
    MagicEntry { mask: 0x20080200A000282000, magic: 0x0244010090208508, shift: 56, offset: 2944 },
    MagicEntry { mask: 0x401004414000504400, magic: 0x202200080040010A, shift: 54, offset: 3200 },
    MagicEntry { mask: 0x2088828000A08800, magic: 0x008C008001020104, shift: 54, offset: 4224 },
    MagicEntry { mask: 0x10111050001411000, magic: 0x600010200003800C, shift: 54, offset: 5248 },
    MagicEntry { mask: 0x202020A0002802000, magic: 0xA00020200A10408C, shift: 56, offset: 6272 },
    MagicEntry { mask: 0x40404040001004000, magic: 0x484000800104181A, shift: 58, offset: 6528 },
    MagicEntry { mask: 0x80808080002008000, magic: 0x081020221C910408, shift: 58, offset: 6592 },
    MagicEntry { mask: 0x40100400010101000, magic: 0x0800608502820020, shift: 58, offset: 6656 },
    MagicEntry { mask: 0x80200800020202000, magic: 0x0801014A80000820, shift: 58, offset: 6720 },
    MagicEntry { mask: 0x100401400050404000, magic: 0x0408801100002220, shift: 56, offset: 6784 },
    MagicEntry { mask: 0x2008828000A0888000, magic: 0xC1A1004008042001, shift: 54, offset: 7040 },
    // Square 5e is problematic. Relaxing shift to 51
    // num_trials=10000000 bad_magics=6520935
    MagicEntry { mask: 0x411105000141110400, magic: 0x0050102000208004, shift: 51, offset: 8064 },
    MagicEntry { mask: 0x2220A000282200800, magic: 0x001000C402100801, shift: 54, offset: 16256 },
    MagicEntry { mask: 0x40414000500401000, magic: 0x1060800920101401, shift: 56, offset: 17280 },
    MagicEntry { mask: 0x80808000200802000, magic: 0x08011420AA041821, shift: 58, offset: 17536 },
    MagicEntry { mask: 0x101010000401004000, magic: 0x0301200300020102, shift: 58, offset: 17600 },
    MagicEntry { mask: 0x20080002020202000, magic: 0xD1040180420003C6, shift: 58, offset: 17664 },
    MagicEntry { mask: 0x40100004040404000, magic: 0x181140802B104180, shift: 58, offset: 17728 },
    MagicEntry { mask: 0x8028000A080808000, magic: 0x1100810040080004, shift: 56, offset: 17792 },
    MagicEntry { mask: 0x110500014111010000, magic: 0x0220060810800010, shift: 54, offset: 18048 },
    MagicEntry { mask: 0x220A00028222080000, magic: 0x010090A808400001, shift: 54, offset: 19072 },
    MagicEntry { mask: 0x441400050440100400, magic: 0x8040100820010014, shift: 54, offset: 20096 },
    MagicEntry { mask: 0x828000A0080200800, magic: 0x208080100208000C, shift: 56, offset: 21120 },
    MagicEntry { mask: 0x101000040100401000, magic: 0x0021010021029008, shift: 58, offset: 21376 },
    MagicEntry { mask: 0x202000080200802000, magic: 0x7420210020024245, shift: 58, offset: 21440 },
    MagicEntry { mask: 0x10000404040404000, magic: 0x0242402900820001, shift: 58, offset: 21504 },
    MagicEntry { mask: 0x20000808080808000, magic: 0x9011204100508840, shift: 58, offset: 21568 },
    MagicEntry { mask: 0x50001410101010000, magic: 0x9200804001200800, shift: 56, offset: 21632 },
    MagicEntry { mask: 0xA0002822202000000, magic: 0xC802004A20080402, shift: 56, offset: 21888 },
    MagicEntry { mask: 0x140005044410000000, magic: 0x8400101402A04C02, shift: 56, offset: 22144 },
    MagicEntry { mask: 0x28000A088020080000, magic: 0x0100040210100888, shift: 56, offset: 22400 },
    MagicEntry { mask: 0x500014010040100400, magic: 0x0824440020101800, shift: 56, offset: 22656 },
    MagicEntry { mask: 0x200008020080200800, magic: 0x3101002940400220, shift: 58, offset: 22912 },
    MagicEntry { mask: 0x400010040100401000, magic: 0x0422020040020043, shift: 58, offset: 22976 },
    MagicEntry { mask: 0x0080808080808000, magic: 0x000840801349A100, shift: 58, offset: 23040 },
    MagicEntry { mask: 0x0101010101010000, magic: 0x0500082010800440, shift: 58, offset: 23104 },
    MagicEntry { mask: 0x0282020202000000, magic: 0x4081082022540940, shift: 58, offset: 23168 },
    MagicEntry { mask: 0x0504440400000000, magic: 0x0022000220068150, shift: 58, offset: 23232 },
    MagicEntry { mask: 0x0A08882000000000, magic: 0x0201100211101908, shift: 58, offset: 23296 },
    MagicEntry { mask: 0x1411004010000000, magic: 0x40A0000140800814, shift: 58, offset: 23360 },
    MagicEntry { mask: 0x2802008020080000, magic: 0x1402992028801884, shift: 58, offset: 23424 },
    MagicEntry { mask: 0x1004010040100400, magic: 0x080110810810A002, shift: 58, offset: 23488 },
    MagicEntry { mask: 0x2008020080200800, magic: 0x0000824022200404, shift: 58, offset: 23552 },
    MagicEntry { mask: 0x10101010101010000, magic: 0xA000C82010038044, shift: 57, offset: 23616 },
    MagicEntry { mask: 0x20202020202000000, magic: 0xA411008C410A1020, shift: 58, offset: 23744 },
    MagicEntry { mask: 0x50404040400000000, magic: 0x4480800208100460, shift: 58, offset: 23808 },
    MagicEntry { mask: 0xA0888080000000000, magic: 0x040008000010040D, shift: 58, offset: 23872 },
    MagicEntry { mask: 0x141110400000000000, magic: 0x8D000000045A0141, shift: 58, offset: 23936 },
    MagicEntry { mask: 0x282200802000000000, magic: 0x0822020021002002, shift: 58, offset: 24000 },
    MagicEntry { mask: 0x500401004010000000, magic: 0x0081080280102220, shift: 58, offset: 24064 },
    MagicEntry { mask: 0x200802008020080000, magic: 0x0022008402400888, shift: 58, offset: 24128 },
    MagicEntry { mask: 0x401004010040100400, magic: 0x0101010260031002, shift: 57, offset: 24192 },
];

pub const BISHOP_TABLE_SIZE: usize = 24320;
// also about 5x as long as the table in chess

pub const SLIDING_MOVE_TABLE_SIZE: usize = 540416;

// Fold the 17 bits of the hi quadword (the last 17 squares) into the lo quadword
// in such a way that this never overwrites any bits that are already set to 1.
// In general this method would of course overwrite bits in the lower u64, but
// if the inputs are restricted to Rook rays or Bishop rays, then a simple,
// deterministic right shift by 63 is always safe! (Other simple shifts would
// cause problems for Rook rays, essentially causing some bits to awlays be lost!)
//
// I saw this trick in the Apery source code.  It enables sticking to a 64-bit hash
// multiplier, so we can use the exact same hash function as used for Western Chess.
//
// Note that Apery does use a 64-bit shift. This works for them because their
// board layout -- how squares are mapped to bits -- differs from ours. For some
// reason bit 63 in their lo quadword is deliberately never used. This seems to be
// an awkard relic from a past in which an array of two ulongs, instead of a __m128si,
// were needed to represent the board. (YaneuraOu also uses that funky layout.)
#[inline(always)]
const fn merge(x: u128) -> u64 {
    ((x >> 63) | x) as u64
}

const fn get_magic_index(
    magics: &[MagicEntry; Square::NUM],
    square: Square,
    blockers: BitBoard,
) -> usize {
    let magic = &magics[square as usize];
    let blockers = merge(blockers.0 & magic.mask);
    let hash = blockers.wrapping_mul(magic.magic);
    magic.offset as usize + (hash >> magic.shift) as usize
}

pub const fn get_rook_moves_index(square: Square, blockers: BitBoard) -> usize {
    get_magic_index(ROOK_MAGICS, square, blockers)
}

pub const fn get_bishop_moves_index(square: Square, blockers: BitBoard) -> usize {
    get_magic_index(BISHOP_MAGICS, square, blockers)
}
