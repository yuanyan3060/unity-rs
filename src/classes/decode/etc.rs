#![allow(unused)]
#![allow(non_upper_case_globals)]

static WriteOrderTable: [u8; 16] = [0, 4, 8, 12, 1, 5, 9, 13, 2, 6, 10, 14, 3, 7, 11, 15];
static WriteOrderTableRev: [u8; 16] = [15, 11, 7, 3, 14, 10, 6, 2, 13, 9, 5, 1, 12, 8, 4, 0];
static Etc1ModifierTable: [[u8; 2]; 8] = [[2, 8], [5, 17], [9, 29], [13, 42], [18, 60], [24, 80], [33, 106], [47, 183]];
static Etc2aModifierTable: [[[u8; 2]; 8]; 2] = [
    [[0, 8], [0, 17], [0, 29], [0, 42], [0, 60], [0, 80], [0, 106], [0, 183]],
    [[2, 8], [5, 17], [9, 29], [13, 42], [18, 60], [24, 80], [33, 10], [47, 183]],
];
static Etc1SubblockTable: [[u8; 16]; 2] = [[0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1], [0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1]];
static Etc2DistanceTable: [u8; 8] = [3, 6, 11, 16, 23, 32, 41, 64];
static Etc2AlphaModTable: [[i8; 8]; 16] = [
    [-3, -6, -9, -15, 2, 5, 8, 14],
    [-3, -7, -10, -13, 2, 6, 9, 12],
    [-2, -5, -8, -13, 1, 4, 7, 12],
    [-2, -4, -6, -13, 1, 3, 5, 12],
    [-3, -6, -8, -12, 2, 5, 7, 11],
    [-3, -7, -9, -11, 2, 6, 8, 10],
    [-4, -7, -8, -11, 3, 6, 7, 10],
    [-3, -5, -8, -11, 2, 4, 7, 10],
    [-2, -6, -8, -10, 1, 5, 7, 9],
    [-2, -5, -8, -10, 1, 4, 7, 9],
    [-2, -4, -8, -10, 1, 3, 7, 9],
    [-2, -5, -7, -10, 1, 4, 6, 9],
    [-3, -4, -7, -10, 2, 3, 6, 9],
    [-1, -2, -3, -10, 0, 1, 2, 9],
    [-4, -6, -8, -9, 3, 5, 7, 8],
    [-3, -5, -7, -9, 2, 4, 6, 8],
];

#[inline(always)]
pub fn color(r: u8, g: u8, b: u8, a: u8) -> [u8; 4] {
    [r, g, b, a]
}

#[inline(always)]
pub fn clamp(n: i32) -> u8 {
    if n < 0 {
        return 0;
    } else if n > 255 {
        return 255;
    } else {
        return n as u8;
    }
}

#[inline(always)]
pub fn applicate_color(c: [u8; 3], m: i32) -> [u8; 4] {
    color(clamp(c[0] as i32 + m), clamp(c[1] as i32 + m), clamp(c[2] as i32 + m), 255)
}

#[inline(always)]
pub fn applicate_color_alpha(c: [u8; 3], m: i32, transparent: u8) -> [u8; 4] {
    let a = {
        if transparent > 0 {
            255
        } else {
            0
        }
    };
    color(clamp(c[0] as i32 + m), clamp(c[1] as i32 + m), clamp(c[2] as i32 + m), a)
}

#[inline(always)]
pub fn applicate_color_raw(c: [u8; 3]) -> [u8; 4] {
    color(c[0], c[1], c[2], 255)
}

#[inline(always)]
pub fn copy_block_buffer(bx: usize, by: usize, w: usize, h: usize, bw: usize, bh: usize, buffer: &[[u8; 4]], image: &mut [u8]) {
    unsafe {
        let image = image.as_mut_ptr() as *mut [u8; 4];
        let x = bw * bx;
        let xl = {
            if bw * (bx + 1) > w {
                w - bw * bx
            } else {
                bw
            }
        };
        let mut buffer_offset = 0;
        let buffer_end = bw * bh;
        for y in by * bh..h {
            if buffer_offset >= buffer_end {
                break;
            }
            for i in 0..xl {
                let y = h - 1 - y;
                let offset = y * w + x + i;
                *image.add(offset) = *buffer.get_unchecked(buffer_offset + i);
            }
            buffer_offset += bw;
        }
    }
}

pub fn decode_etc1_block(data: &[u8], outbuf: &mut [[u8; 4]]) {
    let code = [data[3] >> 5, data[3] >> 2 & 0x07];
    let table = Etc1SubblockTable[(data[3] & 0x01) as usize];
    let mut c: [[u8; 3]; 2] = [[0; 3]; 2];
    if (data[3] & 2) != 0 {
        // diff bit == 1
        c[0][0] = data[0] & 0xf8;
        c[0][1] = data[1] & 0xf8;
        c[0][2] = data[2] & 0xf8;
        c[1][0] = (c[0][0] as i32 + (data[0] << 3 & 0x18) as i32 - (data[0] << 3 & 0x20) as i32) as u8;
        c[1][1] = (c[0][1] as i32 + (data[1] << 3 & 0x18) as i32 - (data[1] << 3 & 0x20) as i32) as u8;
        c[1][2] = (c[0][2] as i32 + (data[2] << 3 & 0x18) as i32 - (data[2] << 3 & 0x20) as i32) as u8;
        c[0][0] |= c[0][0] >> 5;
        c[0][1] |= c[0][1] >> 5;
        c[0][2] |= c[0][2] >> 5;
        c[1][0] |= c[1][0] >> 5;
        c[1][1] |= c[1][1] >> 5;
        c[1][2] |= c[1][2] >> 5;
    } else {
        c[0][0] = (data[0] & 0xf0) | data[0] >> 4;
        c[1][0] = (data[0] & 0x0f) | data[0] << 4;
        c[0][1] = (data[1] & 0xf0) | data[1] >> 4;
        c[1][1] = (data[1] & 0x0f) | data[1] << 4;
        c[0][2] = (data[2] & 0xf0) | data[2] >> 4;
        c[1][2] = (data[2] & 0x0f) | data[2] << 4;
    }
    let mut j: u16 = (data[6] as u16) << 8 | data[7] as u16; // less significant pixel index bits
    let mut k: u16 = (data[4] as u16) << 8 | data[5] as u16; // more significant pixel index bits
    for i in 0..16 {
        let s = table[i] as usize;
        let mut m = Etc1ModifierTable[code[s] as usize][(j & 1) as usize] as i32;
        if (k & 1) != 0 {
            m = -m;
        }
        outbuf[WriteOrderTable[i] as usize] = applicate_color(c[s], m);
        j >>= 1;
        k >>= 1;
    }
}

pub fn decode_etc2_block(data: &[u8], outbuf: &mut [[u8; 4]]) {
    let mut j = u16::from_be_bytes([data[6], data[7]]) as u32;
    let mut k = u16::from_be_bytes([data[4], data[5]]) as u32;
    let mut c: [[u8; 3]; 3] = [[0; 3]; 3];
    if data[3] & 2 != 0 {
        let r = data[0] & 0xf8;
        let dr = ((data[0] as i16) << 3 & 0x18) - ((data[0] as i16) << 3 & 0x20);
        let g = data[1] & 0xf8;
        let dg = ((data[1] as i16) << 3 & 0x18) - ((data[1] as i16) << 3 & 0x20);
        let b = data[2] & 0xf8;
        let db = ((data[2] as i16) << 3 & 0x18) - ((data[2] as i16) << 3 & 0x20);
        if ((r as i16) + dr < 0) || ((r as i16) + dr) > 255 {
            c[0][0] = (data[0] << 3 & 0xc0) | (data[0] << 4 & 0x30) | (data[0] >> 1 & 0xc) | (data[0] & 3);
            c[0][1] = (data[1] & 0xf0) | data[1] >> 4;
            c[0][2] = (data[1] & 0x0f) | data[1] << 4;
            c[1][0] = (data[2] & 0xf0) | data[2] >> 4;
            c[1][1] = (data[2] & 0x0f) | data[2] << 4;
            c[1][2] = (data[3] & 0xf0) | data[3] >> 4;
            let d: u8 = Etc2DistanceTable[(((data[3] >> 1) & 6) | (data[3] & 1)) as usize];
            let color_set: [[u8; 4]; 4] = [
                applicate_color_raw(c[0]),
                applicate_color(c[1], d as i32),
                applicate_color_raw(c[1]),
                applicate_color(c[1], -(d as i32)),
            ];
            k <<= 1;
            for i in 0..16 {
                outbuf[WriteOrderTable[i] as usize] = color_set[((k & 0x02) | (j & 0x01)) as usize];
                j >>= 1;
                k >>= 1;
            }
        } else if ((g as i16) + dg < 0) || ((g as i16) + dg) > 255 {
            c[0][0] = (data[0] << 1 & 0xf0) | (data[0] >> 3 & 0xf);
            c[0][1] = (data[0] << 5 & 0xe0) | (data[1] & 0x10);
            c[0][1] |= c[0][1] >> 4;
            c[0][2] = (data[1] & 8) | (data[1] << 1 & 6) | data[2] >> 7;
            c[0][2] |= c[0][2] << 4;
            c[1][0] = (data[2] << 1 & 0xf0) | (data[2] >> 3 & 0xf);
            c[1][1] = (data[2] << 5 & 0xe0) | (data[3] >> 3 & 0x10);
            c[1][1] |= c[1][1] >> 4;
            c[1][2] = (data[3] << 1 & 0xf0) | (data[3] >> 3 & 0xf);
            let mut d = (data[3] & 4) | (data[3] << 1 & 2);
            if c[0][0] > c[1][0] || (c[0][0] == c[1][0] && (c[0][1] > c[1][1] || (c[0][1] == c[1][1] && c[0][2] >= c[1][2]))) {
                d += 1;
            }
            d = Etc2DistanceTable[d as usize];
            let color_set = [
                applicate_color(c[0], d as i32),
                applicate_color(c[0], -(d as i32)),
                applicate_color(c[1], d as i32),
                applicate_color(c[1], -(d as i32)),
            ];
            k = k << 1;
            for i in 0..16 {
                outbuf[WriteOrderTable[i] as usize] = color_set[((k & 0x02) | (j & 0x01)) as usize];
                j >>= 1;
                k >>= 1;
            }
        } else if ((b as i16) + db < 0) || ((b as i16) + db) > 255 {
            c[0][0] = (data[0] << 1 & 0xfc) | (data[0] >> 5 & 3);
            c[0][1] = (data[0] << 7 & 0x80) | (data[1] & 0x7e) | (data[0] & 1);
            c[0][2] = (data[1] << 7 & 0x80) | (data[2] << 2 & 0x60) | (data[2] << 3 & 0x18) | (data[3] >> 5 & 4);
            c[0][2] |= c[0][2] >> 6;
            c[1][0] = (data[3] << 1 & 0xf8) | (data[3] << 2 & 4) | (data[3] >> 5 & 3);
            c[1][1] = (data[4] & 0xfe) | data[4] >> 7;
            c[1][2] = (data[4] << 7 & 0x80) | (data[5] >> 1 & 0x7c);
            c[1][2] |= c[1][2] >> 6;
            c[2][0] = (data[5] << 5 & 0xe0) | (data[6] >> 3 & 0x1c) | (data[5] >> 1 & 3);
            c[2][1] = (data[6] << 3 & 0xf8) | (data[7] >> 5 & 0x6) | (data[6] >> 4 & 1);
            c[2][2] = data[7] << 2 | (data[7] >> 4 & 3);
            let mut i = 0;
            for y in 0..4 {
                for x in 0..4 {
                    let r = clamp((x * (c[1][0] as i32 - c[0][0] as i32) + y * (c[2][0] as i32 - c[0][0] as i32) + 4 * c[0][0] as i32 + 2) >> 2);
                    let g = clamp((x * (c[1][1] as i32 - c[0][1] as i32) + y * (c[2][1] as i32 - c[0][1] as i32) + 4 * c[0][1] as i32 + 2) >> 2);
                    let b = clamp((x * (c[1][2] as i32 - c[0][2] as i32) + y * (c[2][2] as i32 - c[0][2] as i32) + 4 * c[0][2] as i32 + 2) >> 2);
                    outbuf[i] = color(r, g, b, 255);
                    i += 1;
                }
            }
        } else {
            let code = [data[3] >> 5, (data[3] >> 2) & 0x7];
            let table = Etc1SubblockTable[(data[3] & 0x01) as usize];
            c[0][0] = r | r >> 5;
            c[0][1] = g | g >> 5;
            c[0][2] = b | b >> 5;
            c[1][0] = (r as i16 + dr) as u8;
            c[1][1] = (g as i16 + dg) as u8;
            c[1][2] = (b as i16 + db) as u8;
            c[1][0] |= c[1][0] >> 5;
            c[1][1] |= c[1][1] >> 5;
            c[1][2] |= c[1][2] >> 5;
            for i in 0..16 {
                let s = table[i];
                let m = Etc1ModifierTable[code[s as usize] as usize][(j & 0x01) as usize];
                let m = {
                    if (k & 0x01) > 0 {
                        -(m as i32)
                    } else {
                        m as i32
                    }
                };
                outbuf[WriteOrderTable[i] as usize] = applicate_color(c[s as usize], m);
                j >>= 1;
                k >>= 1;
            }
        }
    } else {
        let code = [data[3] >> 5, data[3] >> 2 & 0x07];
        let table = Etc1SubblockTable[(data[3] & 0x01) as usize];
        c[0][0] = (data[0] & 0xf0) | data[0] >> 4;
        c[1][0] = (data[0] & 0x0f) | data[0] << 4;
        c[0][1] = (data[1] & 0xf0) | data[1] >> 4;
        c[1][1] = (data[1] & 0x0f) | data[1] << 4;
        c[0][2] = (data[2] & 0xf0) | data[2] >> 4;
        c[1][2] = (data[2] & 0x0f) | data[2] << 4;
        for i in 0..16 {
            let s = table[i];
            let m = Etc1ModifierTable[code[s as usize] as usize][(j & 0x01) as usize];
            let m = {
                if (k & 0x01) > 0 {
                    -(m as i32)
                } else {
                    m as i32
                }
            };
            outbuf[WriteOrderTable[i] as usize] = applicate_color(c[s as usize], m);
            j >>= 1;
            k >>= 1;
        }
    }
}

pub fn decode_etc2a8_block(data: &[u8], outbuf: &mut [[u8; 4]]) {
    if (data[1] & 0xf0) != 0 {
        let multiplier = data[1] >> 4;
        let table = Etc2AlphaModTable[(data[1] & 0xf) as usize];
        let mut l = u64::from_be_bytes([data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7]]);
        for i in 0..16 {
            let offset = WriteOrderTableRev[i] * 4 + 3;
            let raw = (data[0] as i32) + (multiplier as i32) * (table[(l & 0x7) as usize] as i32);
            let ptr = outbuf.as_mut_ptr() as *mut u8;
            unsafe {
                *ptr.add(offset as usize) = clamp(raw);
            }
            l >>= 3;
        }
    } else {
        let ptr = outbuf.as_mut_ptr() as *mut u8;
        for i in 0..16 {
            unsafe { *ptr.add(3 + 4 * i) = data[0] }
        }
    }
}

pub fn decode_etc1(data: &[u8], w: usize, h: usize, image: &mut [u8]) {
    let num_blocks_x = (w + 3) / 4;
    let num_blocks_y = (h + 3) / 4;
    let mut buffer = [[0; 4]; 16];
    let mut offset = 0;
    for by in 0..num_blocks_y {
        for bx in 0..num_blocks_x {
            decode_etc1_block(&data[offset..data.len()], &mut buffer);
            copy_block_buffer(bx, by, w, h, 4, 4, &buffer, image);
            offset += 8;
        }
    }
}

pub fn decode_etc2a8(data: &[u8], w: usize, h: usize, image: &mut [u8]) {
    let num_blocks_x = (w + 3) / 4;
    let num_blocks_y = (h + 3) / 4;
    let mut buffer = [[0; 4]; 16];
    let mut offset = 0;
    for by in 0..num_blocks_y {
        for bx in 0..num_blocks_x {
            decode_etc2_block(&data[offset + 8..data.len()], &mut buffer);
            decode_etc2a8_block(&data[offset..data.len()], &mut buffer);
            copy_block_buffer(bx, by, w, h, 4, 4, &buffer, image);
            offset += 16;
        }
    }
}

pub fn decode_etc2(data: &[u8], w: usize, h: usize, image: &mut [u8]) {
    let num_blocks_x = (w + 3) / 4;
    let num_blocks_y = (h + 3) / 4;
    let mut buffer = [[0; 4]; 16];
    let mut offset = 0;
    for by in 0..num_blocks_y {
        for bx in 0..num_blocks_x {
            decode_etc2_block(&data[offset..data.len()], &mut buffer);
            copy_block_buffer(bx, by, w, h, 4, 4, &buffer, image);
            offset += 8;
        }
    }
}
