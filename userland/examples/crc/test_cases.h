// CRC test cases
//
// Output values computed here, with "reverse data bytes" set:
//
//   http://www.zorc.breitbandkatze.de/crc.html

CASE(CRC_CCIT16, 0xffff1541, "ABCDEFG")
CASE(CRC_CCIT16, 0xffffB34B, "ABCD")
CASE(CRC_CCIT16, 0xffff1C2D, "0123456")
CASE(CRC_CCIT16, 0xffffD5A8, "0123")
CASE(CRC_CCIT16, 0xffffC21F, "01234567")
CASE(CRC_CCIT16, 0xffff35B3, "012345678")
CASE(CRC_CCIT16, 0xffff57C4, "01234567A")
CASE(CRC_CCIT16, 0xffffE06E, "01234567ABCDE")
CASE(CRC_CCIT16, 0xffffEC86, "0000000000000")

CASE(CRC_CCIT8023, 0x3D29F670, "ABCDEFG")     // unit says c2d6098f
CASE(CRC_CCIT8023, 0xBEB96665, "0123")        // unit says 4146999a

CASE(CRC_CASTAGNOLI, 0xA66AEE34, "ABCDEFG")   // unit says 599511cb
CASE(CRC_CASTAGNOLI, 0x9D469C60, "0123")      // unit says 62b9639f

// For the following cases, no callback happens:
CASE(CRC_CCIT16, 0xffff7B2E, "00000000000000")    // 14 bytes
CASE(CRC_CCIT16, 0xffffDFCA, "01234567ABCDEF")    // 14 bytes
CASE(CRC_CCIT16, 0xffff2DFE, "01234567ABCDEFG")   // 15 bytes
CASE(CRC_CCIT16, 0xffff39BC, "01234567ABCDEFGH")  // 16 bytes
CASE(CRC_CCIT16, 0xffffB881, "01234567ABCDEFGHI") // 17 bytes
