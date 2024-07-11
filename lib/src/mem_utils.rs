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