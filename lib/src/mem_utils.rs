extern crate alloc;

use alloc::vec::Vec;

pub fn read_byte(data: &[u8], offset: &mut i32, update_offset: bool) -> u8 {
    let index = *offset as usize;
    let b = data[index];
    if update_offset {
        *offset += 1;
    }
    b
}

pub fn read_uint16(data: &[u8], offset: &mut i32, update_offset: bool) -> u16 {
    let index = *offset as usize;
    let bytes = data[index..index + 2].try_into().unwrap();
    if update_offset {
        *offset += 2;
    }
    u16::from_be_bytes(bytes)
}

pub fn read_uint32(data: &[u8], offset: &mut i32, update_offset: bool) -> u32 {
    let index = *offset as usize;
    let bytes = data[index..index + 4].try_into().unwrap();
    if update_offset {
        *offset += 4;
    }
    u32::from_be_bytes(bytes)
}

pub fn write_uint16_vec(val: u16, dest: &mut Vec<u8>) {
    for _i in 0..2 {
        let byte : u8 = ((val >> (8 * (1 - _i))) & 0xFF) as u8;
        dest.push(byte);
    }
}

pub fn write_uint32_vec(val: u32, dest: &mut Vec<u8>) {
    for _i in 0..4 {
        let byte : u8 = ((val >> (8 * (3 - _i))) & 0xFF) as u8;
        dest.push(byte);
    }
}
