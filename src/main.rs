#![allow(clippy::unusual_byte_groupings)]

use itertools::Itertools;
use std::fs;
use std::fs::File;
use std::io::Write;

const ROM_BYTES: usize = 128;

fn src_to_bits(reg: &str) -> Option<u8> {
    Some(match reg {
        "r0" => 0b000_00_000,
        "r1" => 0b000_01_000,
        "r2" => 0b000_10_000,
        "r3" => 0b000_11_000,
        _ => return None,
    })
}

fn special_to_bits(reg: &str) -> Option<u8> {
    Some(match reg {
        "r0" => 0b000_00_000,
        "r1" => 0b000_01_000,
        "r2" => 0b000_10_000,
        "r3" => 0b000_11_000,
        "pc" => 0b000_00_001,
        "adr" => 0b000_01_001,
        "sp" => 0b000_10_001,
        "sr" => 0b000_11_001,
        _ => return None,
    })
}

fn half_imm_to_bits(reg: &str) -> Option<u8> {
    Some(match reg {
        "1" => 0b000_00_000,
        "2" => 0b000_01_000,
        "3" => 0b000_10_000,
        "4" => 0b000_11_000,
        _ => return None,
    })
}

fn imm_to_bits(reg: &str) -> Option<u8> {
    Some(match reg {
        "0" => 0b0000_0000,
        "1" => 0b0000_0001,
        "2" => 0b0000_0010,
        "3" => 0b0000_0011,
        "4" => 0b0000_0100,
        "5" => 0b0000_0101,
        "6" => 0b0000_0110,
        "7" => 0b0000_0111,
        "8" => 0b0000_1000,
        "9" => 0b0000_1001,
        "10" => 0b0000_1010,
        "11" => 0b0000_1011,
        "12" => 0b0000_1100,
        "13" => 0b0000_1101,
        "14" => 0b0000_1110,
        "15" => 0b0000_1111,
        _ => return None,
    })
}

fn status_to_bits(reg: &str) -> Option<u8> {
    Some(match reg {
        "0000" => 0b0000_0000,
        "0001" => 0b0000_0001,
        "0010" => 0b0000_0010,
        "0011" => 0b0000_0011,
        "0100" => 0b0000_0100,
        "0101" => 0b0000_0101,
        "0110" => 0b0000_0110,
        "0111" => 0b0000_0111,
        "1000" => 0b0000_1000,
        "1001" => 0b0000_1001,
        "1010" => 0b0000_1010,
        "1011" => 0b0000_1011,
        "1100" => 0b0000_1100,
        "1101" => 0b0000_1101,
        "1110" => 0b0000_1110,
        "1111" => 0b0000_1111,
        _ => return None,
    })
}

fn dst_to_bits(reg: &str) -> Option<u8> {
    Some(match reg {
        "r0" => 0b00000_00_0,
        "r1" => 0b00000_01_0,
        "r2" => 0b00000_10_0,
        "r3" => 0b00000_11_0,
        _ => return None,
    })
}

fn parse_instr(line: &str) -> Option<u8> {
    Some(match line.split_whitespace().collect_vec().as_slice() {
        // Arithmetic
        #[allow(clippy::identity_op)]
        ["add", src] => 0b000_00_000 | src_to_bits(src)?,
        ["adc", src] => 0b000_00_001 | src_to_bits(src)?,
        ["sub", src] => 0b000_00_010 | src_to_bits(src)?,
        ["sbc", src] => 0b000_00_011 | src_to_bits(src)?,
        ["cmp", src] => 0b000_00_100 | src_to_bits(src)?,
        ["cpc", src] => 0b000_00_101 | src_to_bits(src)?,
        ["mul", src] => 0b000_00_110 | src_to_bits(src)?,
        ["div", src] => 0b000_00_111 | src_to_bits(src)?,

        // Logic
        ["lsl", half_imm] => 0b001_00_000 | half_imm_to_bits(half_imm)?,
        ["rol", half_imm] => 0b001_00_001 | half_imm_to_bits(half_imm)?,
        ["lsr", half_imm] => 0b001_00_010 | half_imm_to_bits(half_imm)?,
        ["ror", half_imm] => 0b001_00_011 | half_imm_to_bits(half_imm)?,
        ["and", reg] => 0b001_00_100 | src_to_bits(reg)?,
        ["or", reg] => 0b001_00_101 | src_to_bits(reg)?,
        ["xor", reg] => 0b001_00_110 | src_to_bits(reg)?,

        ["neg"] => 0b001_00_111,
        ["not"] => 0b001_01_111,
        ["dec"] => 0b001_10_111,
        ["inc"] => 0b001_11_111,

        // Store/load
        ["lds", imm] => 0b010_0_0000 | imm_to_bits(imm)?,
        ["sts", imm] => 0b010_1_0000 | imm_to_bits(imm)?,

        // Special ---

        // Immediate to A
        ["stl", imm] => 0b100_0_0000 | imm_to_bits(imm)?,
        ["sth", imm] => 0b100_1_0000 | imm_to_bits(imm)?,

        // Immediate to addr
        ["sdl", imm] => 0b101_0_0000 | imm_to_bits(imm)?,
        ["sdh", imm] => 0b101_1_0000 | imm_to_bits(imm)?,

        // Jump, status register:= 0b0000_NVZC
        ["jmp", "b"] => 0b110_00_000,
        ["brvs", "b"] => 0b110_00_001, // overflow set
        ["brcs", "b"] => 0b110_00_010, // carry set
        ["brcc", "b"] => 0b110_00_011, // carry clear
        ["breq", "b"] => 0b110_00_100, // eq
        ["brne", "b"] => 0b110_00_101, // ne
        ["brns", "b"] => 0b110_00_110, // negative set
        ["brnc", "b"] => 0b110_00_111, // negative clear

        ["jmp", "f"] => 0b110_10_000,
        ["brvs", "f"] => 0b110_10_001, // overflow set
        ["brcs", "f"] => 0b110_10_010, // carry set
        ["brcc", "f"] => 0b110_10_011, // carry clear
        ["breq", "f"] => 0b110_10_100, // eq
        ["brne", "f"] => 0b110_10_101, // ne
        ["brns", "f"] => 0b110_10_110, // negative set
        ["brnc", "f"] => 0b110_10_111, // negative clear

        ["ssr", status] => 0b_110_1_0000 | status_to_bits(status)?,

        ["mov", src, dst] => 0b111_00_00_0 | special_to_bits(src)? | dst_to_bits(dst)?,

        _ => return None,
    })
}

const SIZE_X: isize = 32;
const STRIDE_X: isize = -2;
const OFFSET_X: isize = -2;
const SIZE_Y: isize = 4;
const STRIDE_Y: isize = 4;
const OFFSET_Y: isize = -15;
const STRIDE_Z: isize = -2;
const OFFSET_Z: isize = 0;

fn write_byte(x: isize, y: isize, b: u8) -> String {
    (0..8)
        .map(|z| z * STRIDE_Z + OFFSET_Z)
        .zip((0..8).rev().map(|m| (b >> m) & 1 != 0))
        .map(|(z, set)| {
            if set {
                format!(
                    "setblock ~{x} ~{y} ~{z} minecraft:redstone_wall_torch[facing=east] replace\n"
                )
            } else {
                format!("setblock ~{x} ~{y} ~{z} minecraft:air replace\n")
            }
        })
        .collect()
}

fn main() {
    let input = fs::read_to_string("resources/input.rasm").unwrap();

    let mut opcodes = vec![0; ROM_BYTES];

    for (i, line) in input.lines().enumerate() {
        match parse_instr(line) {
            Some(v) => opcodes[i] = v,
            None => {
                eprintln!("Line {i} does not contain a valid instruction `{line}`.");
                return;
            }
        }
    }

    let mut file = File::create("tododododododood").unwrap();

    let mut i = 0;
    for y in (0..SIZE_Y).map(|y| y * STRIDE_Y + OFFSET_Y) {
        for x in (0..SIZE_X).map(|x| x * STRIDE_X + OFFSET_X) {
            write!(file, "{}", write_byte(x, y, opcodes[i])).unwrap();
            i += 1;
        }
    }
}
