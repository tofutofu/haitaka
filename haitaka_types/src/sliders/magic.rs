// Magic
use crate::*;

struct MagicEntry {
    mask: u128,  // relevant blockers mask for slider on square
    magic: u64,  // multiplier for hash:  hash = (mask & occ) * multiplier
    shift: u8,   // bit shift for index: index = hash >> shift
    offset: u32, // offset in final table: table[offset + index] = moves
}

#[rustfmt::skip]
const ROOK_MAGICS: &[MagicEntry; Square::NUM] = &[
    MagicEntry { mask: 0x80402010080402FE, magic: 0x424000040948C100, shift: 50, offset: 0 },
    MagicEntry { mask: 0x100804020100804FC, magic: 0x8020008200005480, shift: 51, offset: 16384 },
    MagicEntry { mask: 0x201008040201008FA, magic: 0x01800208C1060080, shift: 51, offset: 24576 },
    MagicEntry { mask: 0x402010080402010F6, magic: 0x0040020000401440, shift: 51, offset: 32768 },
    MagicEntry { mask: 0x804020100804020EE, magic: 0x1040008A80010820, shift: 51, offset: 40960 },
    MagicEntry { mask: 0x1008040201008040DE, magic: 0x00C00142C0000510, shift: 51, offset: 49152 },
    MagicEntry { mask: 0x2010080402010080BE, magic: 0x0080004254042008, shift: 51, offset: 57344 },
    MagicEntry { mask: 0x40201008040201007E, magic: 0x4140002001044144, shift: 51, offset: 65536 },
    MagicEntry { mask: 0x8040201008040200FE, magic: 0x254000040800813A, shift: 50, offset: 73728 },
    MagicEntry { mask: 0x804020100805FC00, magic: 0x0411A0000808A100, shift: 51, offset: 90112 },
    MagicEntry { mask: 0x1008040201009F800, magic: 0x8124100081000480, shift: 52, offset: 98304 },
    MagicEntry { mask: 0x2010080402011F400, magic: 0x0020400210044080, shift: 52, offset: 102400 },
    MagicEntry { mask: 0x4020100804021EC00, magic: 0x0211400100468002, shift: 52, offset: 106496 },
    MagicEntry { mask: 0x8040201008041DC00, magic: 0x1231200100040430, shift: 52, offset: 110592 },
    MagicEntry { mask: 0x10080402010081BC00, magic: 0x0802A00040040010, shift: 52, offset: 114688 },
    MagicEntry { mask: 0x201008040201017C00, magic: 0x00402000400084A4, shift: 52, offset: 118784 },
    MagicEntry { mask: 0x40201008040200FC00, magic: 0x2000201000080004, shift: 52, offset: 122880 },
    MagicEntry { mask: 0x80402010080401FC00, magic: 0x8000440102002002, shift: 50, offset: 126976 },
    MagicEntry { mask: 0x804020100BF80200, magic: 0x4408089000820003, shift: 51, offset: 143360 },
    MagicEntry { mask: 0x10080402013F00400, magic: 0x6020004801004080, shift: 52, offset: 151552 },
    MagicEntry { mask: 0x20100804023E80800, magic: 0x010108C001020021, shift: 52, offset: 155648 },
    MagicEntry { mask: 0x40201008043D81000, magic: 0x0080262000430840, shift: 52, offset: 159744 },
    MagicEntry { mask: 0x80402010083B82000, magic: 0x0022042000280220, shift: 52, offset: 163840 },
    MagicEntry { mask: 0x100804020103784000, magic: 0x4022001000140028, shift: 52, offset: 167936 },
    MagicEntry { mask: 0x201008040202F88000, magic: 0x1080012000C14888, shift: 52, offset: 172032 },
    MagicEntry { mask: 0x402010080401F90000, magic: 0x0B40021000100052, shift: 52, offset: 176128 },
    MagicEntry { mask: 0x804020100803FA0000, magic: 0x044002B000060082, shift: 51, offset: 180224 },
    MagicEntry { mask: 0x80402017F0040200, magic: 0x80004D0864001180, shift: 51, offset: 188416 },
    MagicEntry { mask: 0x100804027E0080400, magic: 0x2320040004002080, shift: 52, offset: 196608 },
    MagicEntry { mask: 0x201008047D0100800, magic: 0x004D041810010002, shift: 52, offset: 200704 },
    MagicEntry { mask: 0x402010087B0201000, magic: 0x0824000088008001, shift: 52, offset: 204800 },
    MagicEntry { mask: 0x80402010770402000, magic: 0xA140020008001410, shift: 52, offset: 208896 },
    MagicEntry { mask: 0x1008040206F0804000, magic: 0x0080004050004030, shift: 52, offset: 212992 },
    MagicEntry { mask: 0x2010080405F1008000, magic: 0x008001C090002008, shift: 52, offset: 217088 },
    MagicEntry { mask: 0x4020100803F2010000, magic: 0x0080088010010204, shift: 52, offset: 221184 },
    MagicEntry { mask: 0x8040201007F4020000, magic: 0x20400088480000E2, shift: 51, offset: 225280 },
    MagicEntry { mask: 0x80402FE008040200, magic: 0x0000100410180041, shift: 51, offset: 233472 },
    MagicEntry { mask: 0x100804FC010080400, magic: 0x0890801808100046, shift: 52, offset: 241664 },
    MagicEntry { mask: 0x201008FA020100800, magic: 0x001000C108240040, shift: 52, offset: 245760 },
    MagicEntry { mask: 0x402010F6040201000, magic: 0x0C04000888040040, shift: 52, offset: 249856 },
    MagicEntry { mask: 0x804020EE080402000, magic: 0x0009420008040010, shift: 52, offset: 253952 },
    MagicEntry { mask: 0x1008040DE100804000, magic: 0x4240094040040010, shift: 52, offset: 258048 },
    MagicEntry { mask: 0x2010080BE201008000, magic: 0x0080210020080028, shift: 52, offset: 262144 },
    MagicEntry { mask: 0x40201007E402010000, magic: 0x0540004001040002, shift: 52, offset: 266240 },
    MagicEntry { mask: 0x8040200FE804020000, magic: 0x4840000802240002, shift: 51, offset: 270336 },
    MagicEntry { mask: 0x805FC01008040200, magic: 0x0808486800040A00, shift: 51, offset: 278528 },
    MagicEntry { mask: 0x1009F802010080400, magic: 0x0090000809004100, shift: 52, offset: 286720 },
    MagicEntry { mask: 0x2011F404020100800, magic: 0x1002004400800280, shift: 52, offset: 290816 },
    MagicEntry { mask: 0x4021EC08040201000, magic: 0x0800008080020408, shift: 51, offset: 294912 },
    MagicEntry { mask: 0x8041DC10080402000, magic: 0x0160082008028004, shift: 52, offset: 303104 },
    MagicEntry { mask: 0x10081BC20100804000, magic: 0x00020E6200204001, shift: 51, offset: 307200 },
    MagicEntry { mask: 0x201017C40201008000, magic: 0x0080008000100201, shift: 52, offset: 315392 },
    MagicEntry { mask: 0x40200FC80402010000, magic: 0x9402400020000201, shift: 52, offset: 319488 },
    MagicEntry { mask: 0x80401FD00804020000, magic: 0x002000020412401A, shift: 50, offset: 323584 },
    MagicEntry { mask: 0xBF80201008040200, magic: 0x0000081204000611, shift: 51, offset: 339968 },
    MagicEntry { mask: 0x13F00402010080400, magic: 0x001000102C002041, shift: 52, offset: 348160 },
    MagicEntry { mask: 0x23E80804020100800, magic: 0x0008000400040021, shift: 52, offset: 352256 },
    MagicEntry { mask: 0x43D81008040201000, magic: 0x00100A0811002804, shift: 52, offset: 356352 },
    MagicEntry { mask: 0x83B82010080402000, magic: 0x0308010040800402, shift: 52, offset: 360448 },
    MagicEntry { mask: 0x103784020100804000, magic: 0x01CD000160400201, shift: 52, offset: 364544 },
    MagicEntry { mask: 0x202F88040201008000, magic: 0x0100002040404004, shift: 52, offset: 368640 },
    MagicEntry { mask: 0x401F90080402010000, magic: 0x84A0404080801688, shift: 52, offset: 372736 },
    MagicEntry { mask: 0x803FA0100804020000, magic: 0x00410004101A0A04, shift: 51, offset: 376832 },
    MagicEntry { mask: 0x7F0040201008040200, magic: 0x0040000488408300, shift: 51, offset: 385024 },
    MagicEntry { mask: 0x7E0080402010080400, magic: 0x0040042800050005, shift: 52, offset: 393216 },
    MagicEntry { mask: 0x7D0100804020100800, magic: 0xA10004010A102101, shift: 52, offset: 397312 },
    MagicEntry { mask: 0x7B0201008040201000, magic: 0x08400110010000A0, shift: 52, offset: 401408 },
    MagicEntry { mask: 0x770402010080402000, magic: 0x20C0008100040810, shift: 52, offset: 405504 },
    MagicEntry { mask: 0x6F0804020100804000, magic: 0x0040010000600087, shift: 52, offset: 409600 },
    MagicEntry { mask: 0x5F1008040201008000, magic: 0x0300108280284004, shift: 52, offset: 413696 },
    MagicEntry { mask: 0x3F2010080402010000, magic: 0x2100010010208214, shift: 52, offset: 417792 },
    MagicEntry { mask: 0x7F4020100804020000, magic: 0x0440000428004902, shift: 51, offset: 421888 },
    MagicEntry { mask: 0xFE008040201008040200, magic: 0x0005A0002A118100, shift: 50, offset: 430080 },
    MagicEntry { mask: 0xFC010080402010080400, magic: 0x2000100002165040, shift: 51, offset: 446464 },
    MagicEntry { mask: 0xFA020100804020100800, magic: 0x21A0400200902080, shift: 51, offset: 454656 },
    MagicEntry { mask: 0xF6040201008040201000, magic: 0x1000100280001220, shift: 51, offset: 462848 },
    MagicEntry { mask: 0xEE080402010080402000, magic: 0x0020A00080001420, shift: 51, offset: 471040 },
    MagicEntry { mask: 0xDE100804020100804000, magic: 0x1600200140100410, shift: 51, offset: 479232 },
    MagicEntry { mask: 0xBE201008040201008000, magic: 0x0000400160042048, shift: 51, offset: 487424 },
    MagicEntry { mask: 0x7E402010080402010000, magic: 0x0500400048421004, shift: 51, offset: 495616 },
    MagicEntry { mask: 0xFE804020100804020000, magic: 0x0008A0000141030A, shift: 50, offset: 503808 },
];
pub const ROOK_TABLE_SIZE: usize = 520192;

#[rustfmt::skip]
const BISHOP_MAGICS: &[MagicEntry; Square::NUM] = &[
    MagicEntry { mask: 0x401004010040100400, magic: 0x0210081880840081, shift: 57, offset: 0 },
    MagicEntry { mask: 0x2008020080200800, magic: 0x4802811100A84906, shift: 58, offset: 128 },
    MagicEntry { mask: 0x0010040100401400, magic: 0x0008144044020400, shift: 58, offset: 192 },
    MagicEntry { mask: 0x0000080200882800, magic: 0x420A010002841813, shift: 58, offset: 256 },
    MagicEntry { mask: 0x0000000411105000, magic: 0x02010082011A0018, shift: 58, offset: 320 },
    MagicEntry { mask: 0x000000202220A000, magic: 0x8004412084000634, shift: 58, offset: 384 },
    MagicEntry { mask: 0x0000404040414000, magic: 0x0092812101904202, shift: 58, offset: 448 },
    MagicEntry { mask: 0x0080808080808000, magic: 0x02090011208008C0, shift: 58, offset: 512 },
    MagicEntry { mask: 0x10101010101010000, magic: 0x4028222014401018, shift: 57, offset: 576 },
    MagicEntry { mask: 0x200802008020080000, magic: 0x610090002040188B, shift: 58, offset: 704 },
    MagicEntry { mask: 0x401004010040100000, magic: 0x0090085280402005, shift: 58, offset: 768 },
    MagicEntry { mask: 0x2008020080280000, magic: 0x0402020090221009, shift: 58, offset: 832 },
    MagicEntry { mask: 0x0010040110500000, magic: 0x2004060040820102, shift: 58, offset: 896 },
    MagicEntry { mask: 0x0000082220A00000, magic: 0x8082028104009000, shift: 58, offset: 960 },
    MagicEntry { mask: 0x0000404441400000, magic: 0x148084C082001014, shift: 58, offset: 1024 },
    MagicEntry { mask: 0x0080808082800000, magic: 0x0001214020800A13, shift: 58, offset: 1088 },
    MagicEntry { mask: 0x10101010101000000, magic: 0x0240401810804120, shift: 58, offset: 1152 },
    MagicEntry { mask: 0x20202020202000000, magic: 0x4182020810244020, shift: 58, offset: 1216 },
    MagicEntry { mask: 0x100401004010000400, magic: 0x00C4022A0A014209, shift: 58, offset: 1280 },
    MagicEntry { mask: 0x200802008020000800, magic: 0x0010249100301828, shift: 58, offset: 1344 },
    MagicEntry { mask: 0x401004010050001400, magic: 0x8122021100420908, shift: 56, offset: 1408 },
    MagicEntry { mask: 0x20080220A0002800, magic: 0x0404000220108102, shift: 56, offset: 1664 },
    MagicEntry { mask: 0x0010444140005000, magic: 0x08210041800A1010, shift: 56, offset: 1920 },
    MagicEntry { mask: 0x008088828000A000, magic: 0x0000800040046850, shift: 56, offset: 2176 },
    MagicEntry { mask: 0x10101010500014000, magic: 0x40004C0022104060, shift: 56, offset: 2432 },
    MagicEntry { mask: 0x20202020200008000, magic: 0x022040C088402220, shift: 58, offset: 2688 },
    MagicEntry { mask: 0x40404040400010000, magic: 0x0201408001200802, shift: 58, offset: 2752 },
    MagicEntry { mask: 0x80200802000080800, magic: 0x0800840002001088, shift: 58, offset: 2816 },
    MagicEntry { mask: 0x100401004000101000, magic: 0x2822009402041090, shift: 58, offset: 2880 },
    MagicEntry { mask: 0x20080200A000282000, magic: 0x0082002000304002, shift: 56, offset: 2944 },
    MagicEntry { mask: 0x401004414000504400, magic: 0x4109002001800411, shift: 54, offset: 3200 },
    MagicEntry { mask: 0x2088828000A08800, magic: 0x3002010025040082, shift: 54, offset: 4224 },
    MagicEntry { mask: 0x10111050001411000, magic: 0x0440700801200040, shift: 54, offset: 5248 },
    MagicEntry { mask: 0x202020A0002802000, magic: 0x0801840400402002, shift: 56, offset: 6272 },
    MagicEntry { mask: 0x40404040001004000, magic: 0x8084201000200810, shift: 58, offset: 6528 },
    MagicEntry { mask: 0x80808080002008000, magic: 0x960020200004A510, shift: 58, offset: 6592 },
    MagicEntry { mask: 0x40100400010101000, magic: 0x0800428800684122, shift: 58, offset: 6656 },
    MagicEntry { mask: 0x80200800020202000, magic: 0x00404881011180C0, shift: 58, offset: 6720 },
    MagicEntry { mask: 0x100401400050404000, magic: 0x0019002048008004, shift: 56, offset: 6784 },
    MagicEntry { mask: 0x2008828000A0888000, magic: 0x0091610008002004, shift: 54, offset: 7040 },
    MagicEntry { mask: 0x411105000141110400, magic: 0x004A004002000008, shift: 51, offset: 8064 },
    MagicEntry { mask: 0x2220A000282200800, magic: 0x0010002810020042, shift: 54, offset: 16256 },
    MagicEntry { mask: 0x40414000500401000, magic: 0x0801002009040820, shift: 56, offset: 17280 },
    MagicEntry { mask: 0x80808000200802000, magic: 0x0C04000890820102, shift: 58, offset: 17536 },
    MagicEntry { mask: 0x101010000401004000, magic: 0x004100062400080A, shift: 58, offset: 17600 },
    MagicEntry { mask: 0x20080002020202000, magic: 0x0804218042040100, shift: 58, offset: 17664 },
    MagicEntry { mask: 0x40100004040404000, magic: 0x440A142100454880, shift: 58, offset: 17728 },
    MagicEntry { mask: 0x8028000A080808000, magic: 0x2020802011002002, shift: 56, offset: 17792 },
    MagicEntry { mask: 0x110500014111010000, magic: 0x0500401080011104, shift: 54, offset: 18048 },
    MagicEntry { mask: 0x220A00028222080000, magic: 0x20C0100020050904, shift: 54, offset: 19072 },
    MagicEntry { mask: 0x441400050440100400, magic: 0x8800810002030138, shift: 54, offset: 20096 },
    MagicEntry { mask: 0x828000A0080200800, magic: 0x0800330008090006, shift: 56, offset: 21120 },
    MagicEntry { mask: 0x101000040100401000, magic: 0x8044004104084408, shift: 58, offset: 21376 },
    MagicEntry { mask: 0x202000080200802000, magic: 0x0600808002210005, shift: 58, offset: 21440 },
    MagicEntry { mask: 0x10000404040404000, magic: 0x4412001043028008, shift: 58, offset: 21504 },
    MagicEntry { mask: 0x20000808080808000, magic: 0x2A20390040802012, shift: 58, offset: 21568 },
    MagicEntry { mask: 0x50001410101010000, magic: 0x9080104002288004, shift: 56, offset: 21632 },
    MagicEntry { mask: 0xA0002822202000000, magic: 0x8402084044100200, shift: 56, offset: 21888 },
    MagicEntry { mask: 0x140005044410000000, magic: 0x1C00001401040A00, shift: 56, offset: 22144 },
    MagicEntry { mask: 0x28000A088020080000, magic: 0x02001A0008380100, shift: 56, offset: 22400 },
    MagicEntry { mask: 0x500014010040100400, magic: 0x8500888100901040, shift: 56, offset: 22656 },
    MagicEntry { mask: 0x200008020080200800, magic: 0xC018140010100201, shift: 58, offset: 22912 },
    MagicEntry { mask: 0x400010040100401000, magic: 0x01080100100800A0, shift: 58, offset: 22976 },
    MagicEntry { mask: 0x0080808080808000, magic: 0x9840218340110088, shift: 58, offset: 23040 },
    MagicEntry { mask: 0x0101010101010000, magic: 0x2001240860104080, shift: 58, offset: 23104 },
    MagicEntry { mask: 0x0282020202000000, magic: 0x00088018A0400210, shift: 58, offset: 23168 },
    MagicEntry { mask: 0x0504440400000000, magic: 0x0088000414201008, shift: 58, offset: 23232 },
    MagicEntry { mask: 0x0A08882000000000, magic: 0x1000224804A90408, shift: 58, offset: 23296 },
    MagicEntry { mask: 0x1411004010000000, magic: 0x06A0B80A0300C821, shift: 58, offset: 23360 },
    MagicEntry { mask: 0x2802008020080000, magic: 0x5000210081086104, shift: 58, offset: 23424 },
    MagicEntry { mask: 0x1004010040100400, magic: 0x3008080101180145, shift: 58, offset: 23488 },
    MagicEntry { mask: 0x2008020080200800, magic: 0x118A020012C040A3, shift: 58, offset: 23552 },
    MagicEntry { mask: 0x10101010101010000, magic: 0x8522802810204044, shift: 57, offset: 23616 },
    MagicEntry { mask: 0x20202020202000000, magic: 0x020040C011822048, shift: 58, offset: 23744 },
    MagicEntry { mask: 0x50404040400000000, magic: 0xC401040060100401, shift: 58, offset: 23808 },
    MagicEntry { mask: 0xA0888080000000000, magic: 0x2084014410840208, shift: 58, offset: 23872 },
    MagicEntry { mask: 0x141110400000000000, magic: 0x0C000C2808012043, shift: 58, offset: 23936 },
    MagicEntry { mask: 0x282200802000000000, magic: 0x1220003804012082, shift: 58, offset: 24000 },
    MagicEntry { mask: 0x500401004010000000, magic: 0x2B000000A1016404, shift: 58, offset: 24064 },
    MagicEntry { mask: 0x200802008020080000, magic: 0x0104022840410250, shift: 58, offset: 24128 },
    MagicEntry { mask: 0x401004010040100400, magic: 0x1008040208486428, shift: 57, offset: 24192 },
];

pub const BISHOP_TABLE_SIZE: usize = 24320;
pub const SLIDING_MOVES_TABLE_SIZE: usize = 544512;

// Fold the 17 bits of the high quadword (the last 17 squares) into the low quadword
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
// reason bit 63 in their low quadword is deliberately never used. This seems to be
// a relic from a past in which an array of two ulongs, instead of a __m128si,
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
