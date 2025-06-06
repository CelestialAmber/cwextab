extern crate alloc;

use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use crate::{
    mem_utils
};

/// Enum holding the data for each action type.
/// 
/// Note: Some entries have padding bytes due to 4 byte struct alignment. This causes issues on earlier versions of
/// MWCC (around 1.0 to 2.6) which don't properly zero all the data out :p
#[derive(Debug, Clone)]
pub enum ExActionData {
    EndOfList,
    Branch {
        target_offset: u16,
    },
    DestroyLocal {
        local_offset: u16,
        dtor_address: u32,
    },
    DestroyLocalCond {
        condition: u16,
        local_offset: u16,
        pad4: u16,
        dtor_address: u32,
    },
    DestroyLocalPointer {
        local_pointer: u16,
        dtor_address: u32,
    },
    DestroyLocalArray {
        local_array: u16,
        elements: u16,
        element_size: u16,
        dtor_address: u32,
    },
    DestroyBase {
        object_pointer: u16,
        member_offset: u32,
        dtor_address: u32,
    },
    DestroyMember {
        object_pointer: u16,
        member_offset: u32,
        dtor_address: u32,
    },
    DestroyMemberCond {
        condition: u16,
        object_pointer: u16,
        member_offset: u32,
        pad8: u16,
        dtor_address: u32,
    },
    DestroyMemberArray {
        object_pointer: u16,
        member_offset: u32,
        elements: u32,
        element_size: u32,
        dtor_address: u32,
    },
    DeletePointer {
        object_pointer: u16,
        dtor_address: u32,
    },
    DeletePointerCond {
        condition: u16,
        object_pointer: u16,
        pad4: u16,
        dtor_address: u32,
    },
    CatchBlock {
        pad0: u16,
        catch_type: u32,
        catch_pc_offset: u16,
        cinfo_ref: u16,
    },
    ActiveCatchBlock {
        cinfo_ref: u16,
    },
    Terminate,
    Specification {
        specs: u16,
        pc_offset: u32,
        cinfo_ref: u32,
        spec: Vec<u32>,
    },
    CatchBlock32 {
        pad0: u16,
        catch_type: u32,
        catch_pc_offset: u32,
        cinfo_ref: u32,
    },
}

/// Base enum for exception actions.
#[derive(Debug, Copy, Clone)]
pub enum ExAction {
    EndOfList,
    Branch,
    DestroyLocal,
    DestroyLocalCond,
    DestroyLocalPointer,
    DestroyLocalArray,
    DestroyBase,
    DestroyMember,
    DestroyMemberCond,
    DestroyMemberArray,
    DeletePointer,
    DeletePointerCond,
    CatchBlock,
    ActiveCatchBlock,
    Terminate,
    Specification,
    CatchBlock32,
}

impl ExAction {
    pub fn to_int(&self) -> i32 {
        match self {
            ExAction::EndOfList => 0,
            ExAction::Branch => 1,
            ExAction::DestroyLocal => 2,
            ExAction::DestroyLocalCond => 3,
            ExAction::DestroyLocalPointer => 4,
            ExAction::DestroyLocalArray => 5,
            ExAction::DestroyBase => 6,
            ExAction::DestroyMember => 7,
            ExAction::DestroyMemberCond => 8,
            ExAction::DestroyMemberArray => 9,
            ExAction::DeletePointer => 10,
            ExAction::DeletePointerCond => 11,
            ExAction::CatchBlock => 12,
            ExAction::ActiveCatchBlock => 13,
            ExAction::Terminate => 14,
            ExAction::Specification => 15,
            ExAction::CatchBlock32 => 16,
        }
    }

    pub fn from_int(val: i32) -> Option<ExAction> {
        let result: ExAction = match val {
            0 => ExAction::EndOfList,
            1 => ExAction::Branch,
            2 => ExAction::DestroyLocal,
            3 => ExAction::DestroyLocalCond,
            4 => ExAction::DestroyLocalPointer,
            5 => ExAction::DestroyLocalArray,
            6 => ExAction::DestroyBase,
            7 => ExAction::DestroyMember,
            8 => ExAction::DestroyMemberCond,
            9 => ExAction::DestroyMemberArray,
            10 => ExAction::DeletePointer,
            11 => ExAction::DeletePointerCond,
            12 => ExAction::CatchBlock,
            13 => ExAction::ActiveCatchBlock,
            14 => ExAction::Terminate,
            15 => ExAction::Specification,
            16 => ExAction::CatchBlock32,
            _ => {
                //The action value is invalid, return None
                return None;
            }
        };
        Some(result)
    }

    const ACTION_NAMES: [&'static str; 17] = [
        "NULL",
        "BRANCH",
        "DESTROYLOCAL",
        "DESTROYLOCALCOND",
        "DESTROYLOCALPOINTER",
        "DESTROYLOCALARRAY",
        "DESTROYBASE",
        "DESTROYMEMBER",
        "DESTROYMEMBERCOND",
        "DESTROYMEMBERARRAY",
        "DELETEPOINTER",
        "DELETEPOINTERCOND",
        "CATCHBLOCK (Small)",
        "ACTIVECATCHBLOCK",
        "TERMINATE",
        "SPECIFICATION",
        "CATCHBLOCK (Large)",
    ];

    pub fn convert_to_string(&self) -> String {
        String::from(Self::ACTION_NAMES[self.to_int() as usize])
    }
}

/// Struct for exception actions.
#[derive(Debug, Clone)]
pub struct ExceptionAction {
    //General values
    pub action_offset: u32,
    pub action_type: ExAction, //0x0
    pub action_param: u8,      //0x1
    pub has_end_bit: bool,     //true if action type byte has bit 7 set (type & 0x80)
    pub bytes: Vec<u8>, //0x2-
}

impl ExceptionAction {
    pub fn new() -> Self {
        Self {
            action_offset: 0,
            action_type: ExAction::EndOfList,
            action_param: 0,
            has_end_bit: false,
            bytes: vec![],
        }
    }

    /// Returns whether this action has a destuctor reference or not.
    pub fn has_dtor_ref(&self) -> bool {
        match self.action_type {
            ExAction::EndOfList
            | ExAction::Branch
            | ExAction::CatchBlock
            | ExAction::ActiveCatchBlock
            | ExAction::Terminate
            | ExAction::Specification
            | ExAction::CatchBlock32 => false,
            _ => true,
        }
    }

    /// Calculates the offset of the dtor function address value in this action entry.
    /// If the entry does not have one, this function returns none.
    fn get_dtor_address_value_offset(&self) -> Option<u32> {
        let offset: u32 =
        match self.action_type {
            ExAction::DestroyLocal => 2,
            ExAction::DestroyLocalCond => 6,
            ExAction::DestroyLocalPointer => 2,
            ExAction::DestroyLocalArray => 6,
            ExAction::DestroyBase
            | ExAction::DestroyMember => 6,
            ExAction::DestroyMemberCond => 10,
            ExAction::DestroyMemberArray => 14,
            ExAction::DeletePointer => 2,
            ExAction::DeletePointerCond => 6,
            _ => return None,
        };

        Some(offset)
    }

    /// Returns the offset off the padding data in the action data, if any.
    /// Note: if the data has padding, it is always 2 bytes.
    pub fn get_struct_padding_offset(&self) -> Option<u32> {
        let offset: u32 =
        match self.action_type {
            ExAction::DestroyLocalCond => 4,
            ExAction::DestroyMemberCond => 8,
            ExAction::DeletePointerCond => 4,
            ExAction::CatchBlock
            | ExAction::CatchBlock32 => 0,
            _ => return None,
        };

        Some(offset + 2) //Add 2 to get the actual offset
    }

    /// Calculates this action's action type byte.
    fn calculate_action_type_byte(&self) -> u8 {
        let mut action_type_byte : u8 = 0;
        if self.has_end_bit {
            action_type_byte |= 0x80;
        }
        action_type_byte |= self.action_type.to_int() as u8;
        return action_type_byte;
    }

    /// Returns the relocation data for the dtor function in this action entry, if any.
    pub fn get_dtor_relocation(&self) -> Option<(u32, u32)> {
        if !self.has_dtor_ref() {
            //If the action entry doesn't have a dtor reference, return none
            return None;
        }

        let offset: u32 = match self.get_dtor_address_value_offset() {
            Some(val) => val,
            None => {
                return None;
            }
        };

        let address: u32 = mem_utils::read_uint32(&self.bytes, &mut (offset as i32), true);
        Some((offset, address))
    }

    /// Returns the raw bytes for this action, with the option for zeroing out padding bytes.
    pub fn get_exaction_bytes(&self, zero_padding : bool) -> Vec<u8> {
        let mut bytes : Vec<u8> = vec![];

        bytes.push(self.calculate_action_type_byte());
        bytes.push(self.action_param);
        //Push the action specific data to the list, then zero out padding bytes if requested/any exist
        let length : usize = self.bytes.len();
        for _i in 0..length {
            bytes.push(self.bytes[_i]);
        }

        if zero_padding {
            if let Some(padding_offset) = self.get_struct_padding_offset() {
                let offset : usize = padding_offset as usize;
                //Zero out the padding bytes
                bytes[offset] = 0;
                bytes[offset + 1] = 0;
            }
        }

        return bytes;
    }

    /// Decodes the action data from the byte array depending on the set action type, and converts it
    /// to an ExActionData enum containing the decoded data.
    pub fn get_exaction_data(&self) -> ExActionData {
        let mut offset: i32 = 0;

        match self.action_type {
            ExAction::EndOfList => ExActionData::EndOfList {},
            ExAction::Branch => {
                let target_offset = mem_utils::read_uint16(&self.bytes, &mut offset, true);
                ExActionData::Branch { target_offset }
            }
            ExAction::DestroyLocal => {
                let local_offset = mem_utils::read_uint16(&self.bytes, &mut offset, true);
                let dtor_address = mem_utils::read_uint32(&self.bytes, &mut offset, true);
                ExActionData::DestroyLocal {
                    local_offset,
                    dtor_address,
                }
            }
            ExAction::DestroyLocalCond => {
                let condition = mem_utils::read_uint16(&self.bytes, &mut offset, true);
                let local_offset = mem_utils::read_uint16(&self.bytes, &mut offset, true);
                let pad4 = mem_utils::read_uint16(&self.bytes, &mut offset, true);
                let dtor_address = mem_utils::read_uint32(&self.bytes, &mut offset, true);
                ExActionData::DestroyLocalCond {
                    condition,
                    local_offset,
                    pad4,
                    dtor_address,
                }
            }
            ExAction::DestroyLocalPointer => {
                let local_pointer = mem_utils::read_uint16(&self.bytes, &mut offset, true);
                let dtor_address = mem_utils::read_uint32(&self.bytes, &mut offset, true);
                ExActionData::DestroyLocalPointer {
                    local_pointer,
                    dtor_address,
                }
            }
            ExAction::DestroyLocalArray => {
                let local_array = mem_utils::read_uint16(&self.bytes, &mut offset, true);
                let elements = mem_utils::read_uint16(&self.bytes, &mut offset, true);
                let element_size = mem_utils::read_uint16(&self.bytes, &mut offset, true);
                let dtor_address = mem_utils::read_uint32(&self.bytes, &mut offset, true);
                ExActionData::DestroyLocalArray {
                    local_array,
                    elements,
                    element_size,
                    dtor_address,
                }
            }
            ExAction::DestroyBase => {
                let object_pointer = mem_utils::read_uint16(&self.bytes, &mut offset, true);
                let member_offset = mem_utils::read_uint32(&self.bytes, &mut offset, true);
                let dtor_address = mem_utils::read_uint32(&self.bytes, &mut offset, true);
                ExActionData::DestroyBase {
                    object_pointer,
                    member_offset,
                    dtor_address,
                }
            }
            ExAction::DestroyMember => {
                let object_pointer = mem_utils::read_uint16(&self.bytes, &mut offset, true);
                let member_offset = mem_utils::read_uint32(&self.bytes, &mut offset, true);
                let dtor_address = mem_utils::read_uint32(&self.bytes, &mut offset, true);
                ExActionData::DestroyMember {
                    object_pointer,
                    member_offset,
                    dtor_address,
                }
            }
            ExAction::DestroyMemberCond => {
                let condition = mem_utils::read_uint16(&self.bytes, &mut offset, true);
                let object_pointer = mem_utils::read_uint16(&self.bytes, &mut offset, true);
                let member_offset = mem_utils::read_uint32(&self.bytes, &mut offset, true);
                let pad8 = mem_utils::read_uint16(&self.bytes, &mut offset, true);
                let dtor_address = mem_utils::read_uint32(&self.bytes, &mut offset, true);
                ExActionData::DestroyMemberCond {
                    condition,
                    object_pointer,
                    member_offset,
                    pad8,
                    dtor_address,
                }
            }
            ExAction::DestroyMemberArray => {
                let object_pointer = mem_utils::read_uint16(&self.bytes, &mut offset, true);
                let member_offset = mem_utils::read_uint32(&self.bytes, &mut offset, true);
                let elements = mem_utils::read_uint32(&self.bytes, &mut offset, true);
                let element_size = mem_utils::read_uint32(&self.bytes, &mut offset, true);
                let dtor_address = mem_utils::read_uint32(&self.bytes, &mut offset, true);
                ExActionData::DestroyMemberArray {
                    object_pointer,
                    member_offset,
                    elements,
                    element_size,
                    dtor_address,
                }
            }
            ExAction::DeletePointer => {
                let object_pointer = mem_utils::read_uint16(&self.bytes, &mut offset, true);
                let dtor_address = mem_utils::read_uint32(&self.bytes, &mut offset, true);
                ExActionData::DeletePointer {
                    object_pointer,
                    dtor_address,
                }
            }
            ExAction::DeletePointerCond => {
                let condition = mem_utils::read_uint16(&self.bytes, &mut offset, true);
                let object_pointer = mem_utils::read_uint16(&self.bytes, &mut offset, true);
                let pad4 = mem_utils::read_uint16(&self.bytes, &mut offset, true);
                let dtor_address = mem_utils::read_uint32(&self.bytes, &mut offset, true);
                ExActionData::DeletePointerCond {
                    condition,
                    object_pointer,
                    pad4,
                    dtor_address,
                }
            }
            ExAction::CatchBlock => {
                let pad0 = mem_utils::read_uint16(&self.bytes, &mut offset, true);
                let catch_type = mem_utils::read_uint32(&self.bytes, &mut offset, true);
                let catch_pc_offset = mem_utils::read_uint16(&self.bytes, &mut offset, true);
                let cinfo_ref = mem_utils::read_uint16(&self.bytes, &mut offset, true);
                ExActionData::CatchBlock {
                    pad0,
                    catch_type,
                    catch_pc_offset,
                    cinfo_ref,
                }
            }
            ExAction::ActiveCatchBlock => {
                let cinfo_ref = mem_utils::read_uint16(&self.bytes, &mut offset, true);
                ExActionData::ActiveCatchBlock { cinfo_ref }
            }
            ExAction::Terminate => ExActionData::Terminate {},
            ExAction::Specification => {
                let specs = mem_utils::read_uint16(&self.bytes, &mut offset, true);
                let pc_offset = mem_utils::read_uint32(&self.bytes, &mut offset, true);
                let cinfo_ref = mem_utils::read_uint32(&self.bytes, &mut offset, true);

                //Read the specified number of 32 bit values and add them to the list
                let length = specs as i32;
                let mut spec: Vec<u32> = vec![];
                for _i in 0..length {
                    spec.push(mem_utils::read_uint32(&self.bytes, &mut offset, true));
                }
                ExActionData::Specification {
                    specs,
                    pc_offset,
                    cinfo_ref,
                    spec,
                }
            }
            ExAction::CatchBlock32 => {
                let pad0 = mem_utils::read_uint16(&self.bytes, &mut offset, true);
                let catch_type = mem_utils::read_uint32(&self.bytes, &mut offset, true);
                let catch_pc_offset = mem_utils::read_uint32(&self.bytes, &mut offset, true);
                let cinfo_ref = mem_utils::read_uint32(&self.bytes, &mut offset, true);
                ExActionData::CatchBlock32 {
                    pad0,
                    catch_type,
                    catch_pc_offset,
                    cinfo_ref,
                }
            }
        }
    }
}

impl Default for ExceptionAction {
    fn default() -> Self {
        Self::new()
    }
}

/// Struct for pc actions.
#[derive(Debug, Clone)]
pub struct PCAction {
    pub start_pc: u32,
    pub end_pc: u32,
    pub action_offset: u32,
}

impl PCAction {
    pub fn new() -> Self {
        Self {
            start_pc: 0,
            end_pc: 0,
            action_offset: 0,
        }
    }
}

impl Default for PCAction {
    fn default() -> Self {
        Self::new()
    }
}

/// Struct for exception table relocation (always dtor function address)
#[derive(Debug, Clone)]
pub struct Relocation {
    pub offset: u32,
    pub address: u32,
}
