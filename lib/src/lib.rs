#![no_std]
extern crate alloc;

use alloc::vec;
use alloc::vec::Vec;
use thiserror::Error;

mod actions;
mod extab;
mod mem_utils;

pub use actions::{ExAction, ExActionData, ExceptionAction, PCAction, Relocation};
pub use extab::{ExceptionTableData};

#[derive(Error, Debug)]
pub enum ExtabDecodeError {
    #[error("Data array should at least be 8 bytes long. Given array is {0} bytes long.")]
    ArrayTooSmall(u32),
    #[error("Invalid action value {0} at offset 0x{1:X}")]
    InvalidActionValue(u32, u32),
    #[error("Table is 8 bytes long but terminator is not zero.")]
    InvalidSmallTableTerminator,
    #[error("Internal error")]
    Internal,
}

struct ExtabDecoder {
    extab_data: ExceptionTableData,
    offset: i32,
    data: Vec<u8>,
    length: i32,
}

impl ExtabDecoder {
    fn new() -> Self {
        Self {
            extab_data: ExceptionTableData::new(),
            offset: 0,
            data: vec![],
            length: 0,
        }
    }

    fn parse_exception_table(&mut self, bytes: &[u8]) -> Result<(), ExtabDecodeError> {
        self.offset = 0;
        self.data = Vec::from(bytes);
        self.length = self.data.len() as i32;

        //If the array is empty, return an error.
        if self.length < 8 {
            return Err(ExtabDecodeError::ArrayTooSmall(self.length as u32));
        }

        //Parse the header flag value
        self.extab_data.flag_val = mem_utils::read_uint16(&self.data, &mut self.offset, true);
        self.extab_data.calculate_flag_values();
        self.extab_data.et_field = mem_utils::read_uint16(&self.data, &mut self.offset, true);

        //Check whether the table is 8 bytes but the terminator isn't zero. If so,
        //throw an error.
        let terminator = mem_utils::read_uint32(&self.data, &mut self.offset, false);
        if self.length == 8 && terminator != 0 {
            return Err(ExtabDecodeError::InvalidSmallTableTerminator);
        }

        //Parse range entries until we hit the terminator (32 bit zero value)
        while mem_utils::read_uint32(&self.data, &mut self.offset, false) != 0 {
            let mut pcaction = PCAction::new();
            pcaction.start_pc = mem_utils::read_uint32(&self.data, &mut self.offset, true);
            let range_size: u32 =
                (mem_utils::read_uint16(&self.data, &mut self.offset, true) as u32) * 4; //range size is encoded as size >> 2
            pcaction.end_pc = pcaction.start_pc + range_size;
            pcaction.action_offset =
                mem_utils::read_uint16(&self.data, &mut self.offset, true) as u32;
            self.extab_data.pc_actions.push(pcaction);
        }

        self.offset += 4; //Skip the terminator

        //If there are still bytes remaining, there are action entries to process
        while self.offset < self.length {
            //Console.WriteLine("Offset: " + offset);
            self.parse_action_entry()?;
        }

        Ok(())
    }

    fn parse_action_entry(&mut self) -> Result<(), ExtabDecodeError> {
        let mut exaction = ExceptionAction::new();
        exaction.action_offset = self.offset as u32;
        let action_type_byte = mem_utils::read_byte(&self.data, &mut self.offset, true);
        exaction.has_end_bit = (action_type_byte & 0x80) != 0;
        let action_type_value: u32 = (action_type_byte & 0x7F) as u32;
        let result = ExAction::from_int(action_type_value as i32);
        exaction.action_type = match result {
            Some(action) => action,
            None => {
                return Err(ExtabDecodeError::InvalidActionValue(
                    action_type_value,
                    exaction.action_offset,
                ))
            }
        };
        exaction.action_param = mem_utils::read_byte(&self.data, &mut self.offset, true);

        //Since the way action data is stored is too varied, we just store the remaining data as a byte
        //array to be used later.
        let mut size: i32;

        match exaction.action_type {
            ExAction::EndOfList => {
                size = 0;
            }
            ExAction::Branch => {
                size = 2;
            }
            ExAction::DestroyLocal => {
                size = 6;
            }
            ExAction::DestroyLocalCond => {
                size = 10;
            }
            ExAction::DestroyLocalPointer => {
                size = 6;
            }
            ExAction::DestroyLocalArray => {
                size = 10;
            }
            ExAction::DestroyBase | ExAction::DestroyMember => {
                size = 10;
            }
            ExAction::DestroyMemberCond => {
                size = 14;
            }
            ExAction::DestroyMemberArray => {
                size = 18;
            }
            ExAction::DeletePointer => {
                size = 6;
            }
            ExAction::DeletePointerCond => {
                size = 10;
            }
            ExAction::CatchBlock => {
                size = 10;
            }
            ExAction::ActiveCatchBlock => {
                size = 2;
            }
            ExAction::Terminate => {
                size = 0;
            }
            ExAction::Specification => {
                size = 10;
                //Calculate the length of the array, and add it to the base size
                let length = mem_utils::read_uint16(&self.data, &mut self.offset, false) as i32;
                size += length * 4;
            }
            ExAction::CatchBlock32 => {
                size = 14;
            }
        }

        let start_index = self.offset as usize;
        let end_index = (self.offset + size) as usize;
        exaction.bytes = self.data[start_index..end_index].into();
        self.offset += size;

        //Check if the action entry has a dtor reference. If so, get the relocation information from it,
        //and add it to the list.
        if exaction.has_dtor_ref() {
            let (offset, addr) = match exaction.get_dtor_relocation() {
                Some(val) => val,
                None => {
                    //If None was returned even though the action should have a reference, return an error
                    return Err(ExtabDecodeError::Internal);
                }
            };

            let reloc_offset: u32 = (start_index as u32) + offset;
            let reloc = Relocation { offset: reloc_offset, address: addr };
            self.extab_data.relocations.push(reloc);
        }

        self.extab_data.exception_actions.push(exaction);
        Ok(())
    }
}

/// Decodes the provided exception table data.
///
/// Returns 'None' if the table is not valid.
pub fn decode_extab(data: &[u8]) -> Result<ExceptionTableData, ExtabDecodeError> {
    let mut decoder = ExtabDecoder::new();
    decoder.parse_exception_table(data)?;
    Ok(decoder.extab_data)
}
