extern crate alloc;

use alloc::string::String;
use alloc::{format, vec};
use alloc::vec::Vec;

use crate::{
    actions::{ExceptionAction, ExActionData, PCAction, Relocation},
    mem_utils
};

/// Struct containing all the data from the decoded exception table.
#[derive(Debug, Clone)]
pub struct ExceptionTableData {
    pub flag_val: u16, //0x0-1
    //Flag bits (16 bit value)
    pub has_elf_vector: bool,    //bit 1
    pub large_frame: bool,       //bit 3
    pub has_frame_pointer: bool, //bit 4
    pub saved_cr: bool,          //bit 5
    pub fpr_save_range: u32,     //bits 6-10
    pub gpr_save_range: u32,     //bits 11-15

    pub et_field: u16, //0x2-3

    pub pc_actions: Vec<PCAction>,
    pub exception_actions: Vec<ExceptionAction>,
    pub relocations: Vec<Relocation>,
}

impl ExceptionTableData {
    pub fn new() -> Self {
        Self {
            flag_val: 0,
            has_elf_vector: false,
            large_frame: false,
            has_frame_pointer: false,
            saved_cr: false,
            fpr_save_range: 0,
            gpr_save_range: 0,
            et_field: 0,
            pc_actions: vec![],
            exception_actions: vec![],
            relocations: vec![],
        }
    }

    pub fn calculate_flag_values(&mut self) {
        self.has_elf_vector = ((self.flag_val >> 1) & 1) == 1;
        self.large_frame = ((self.flag_val >> 3) & 1) == 1;
        self.has_frame_pointer = ((self.flag_val >> 4) & 1) == 1;
        self.saved_cr = ((self.flag_val >> 5) & 1) == 1;
        self.fpr_save_range = ((self.flag_val >> 6) & 0b11111) as u32;
        self.gpr_save_range = ((self.flag_val >> 11) & 0b11111) as u32;
    }

    /// Determines if this table has any actions that have uninitialized padding.
    pub fn has_uninitialized_action_padding(&self) -> bool {
        let length = self.exception_actions.len() as usize;

        //Check each action
        for _i in 0..length {
            let action = &self.exception_actions[_i];
            //If the action has padding, and one of the bytes is nonzero, return early
            if let Some(offset) = action.get_struct_padding_offset() {
                let index = offset as usize;
                if action.bytes[index] != 0 || action.bytes[index + 1] != 0 {
                    return true;
                }
            }
        }

        return false;
    }

    /// Returns the raw data for this exception table. If clear_padding is set to
    /// true, any uninitialized padding bytes in exception actions is cleared.
    pub fn get_table_data(&self, clear_padding : bool) -> Vec<u8> {
        let mut bytes : Vec<u8> = vec![];

        //Add header values
        mem_utils::write_uint16_vec(self.flag_val, &mut bytes);
        mem_utils::write_uint16_vec(self.et_field, &mut bytes);

        //Add PC actions
        for _i in 0..self.pc_actions.len() {
            let action = &self.pc_actions[_i];
            //Recalculate range size
            let start_pc: u32 = action.start_pc;
            let range_size: u16 = ((action.end_pc - action.start_pc) / 4) as u16;
            let action_offset: u16 = action.action_offset as u16;
            mem_utils::write_uint32_vec(start_pc, &mut bytes);
            mem_utils::write_uint16_vec(range_size, &mut bytes);
            mem_utils::write_uint16_vec(action_offset, &mut bytes);
        }

        //Write the terminator
        mem_utils::write_uint32_vec(0, &mut bytes);

        //Add exception actions
        for _i in 0..self.exception_actions.len() {
            let action = &self.exception_actions[_i];
            //Append the action data to the list
            let mut action_data: Vec<u8> = action.get_exaction_bytes(clear_padding);
            bytes.append(&mut action_data);
        }

        return bytes;
    }

    /// Converts the table into a string, taking in an array of the function
    /// names required for the table.
    ///
    /// Returns 'None' if an error occurs.
    pub fn to_string(&self, func_names: Vec<String>) -> Option<String> {
        let mut sb = String::from("");

        sb += "Flag values:\n";
        sb += format!(
            "{}",
            format_args!(
                "Has Elf Vector: {}\n",
                if self.has_elf_vector { "Yes" } else { "No" }
            )
        )
        .as_str();
        sb += format!(
            "{}",
            format_args!(
                "Large Frame: {}\n",
                if self.large_frame { "Yes" } else { "No" }
            )
        )
        .as_str();
        sb += format!(
            "{}",
            format_args!(
                "Has Frame Pointer: {}\n",
                if self.has_frame_pointer { "Yes" } else { "No" }
            )
        )
        .as_str();
        sb += format!(
            "{}",
            format_args!("Saved CR: {}\n", if self.saved_cr { "Yes" } else { "No" })
        )
        .as_str();

        if self.fpr_save_range != 0 {
            let start_fpr = 31 - (self.fpr_save_range - 1);
            let fpr_string: String = if start_fpr == 31 {
                String::from("fp31")
            } else {
                format!("fp{start_fpr}-fp31")
            };
            sb += format!("Saved FPR range: {fpr_string}\n").as_str();
        }
        if self.gpr_save_range != 0 {
            let start_gpr = 31 - (self.gpr_save_range - 1);
            let gpr_string: String = if start_gpr == 31 {
                String::from("r31")
            } else {
                format!("r{start_gpr}-r31")
            };
            sb += format!("Saved GPR range: {gpr_string}\n").as_str();
        }
        sb += "\n";

        let num_pcactions = self.pc_actions.len();

        //Print exception range entries
        if num_pcactions > 0 {
            sb += "PC actions:\n";
            for i in 0..num_pcactions {
                let action = &self.pc_actions[i];
                let start_pc = action.start_pc;
                let end_pc = action.end_pc;
                let action_offset = action.action_offset;
                if start_pc != end_pc {
                    sb += format!("PC={start_pc:08X}:{end_pc:08X}, Action: {action_offset:06X}\n")
                        .as_str();
                } else {
                    sb += format!("PC={start_pc:08X}, Action: {action_offset:06X}\n").as_str();
                }
            }

            sb += "\n";
        }

        let num_exactions = self.exception_actions.len();

        if num_exactions > 0 {
            sb += "Exception actions:\n";
            let local_reg_string = if self.has_frame_pointer { "FP" } else { "SP" };
            let mut func_index: usize = 0;

            for i in 0..num_exactions {
                let action = &self.exception_actions[i];
                let mut line = String::from("");
                let action_offset = action.action_offset;
                let action_name = action.action_type.convert_to_string();
                line += format!("{action_offset:06X}:\nType: {action_name}\n").as_str();

                let has_dtor_ref = action.has_dtor_ref();
                let exaction_data = action.get_exaction_data();

                match exaction_data {
                    ExActionData::EndOfList => {}
                    ExActionData::Branch { target_offset } => {
                        line += format!("Action: {target_offset:06X}\n").as_str();
                    }
                    ExActionData::DestroyLocal { local_offset, .. } => {
                        line += format!("Local: {local_offset:#X}({local_reg_string})\n").as_str();
                    }
                    ExActionData::DestroyLocalCond {
                        condition,
                        local_offset,
                        ..
                    } => {
                        line += format!("Local: {local_offset:#X}({local_reg_string})\n").as_str();

                        //The action param is used to determine the type of reference for the condition (0: local offset, 1: register)
                        if action.action_param == 0 {
                            //Local offset
                            line += format!("Cond: {condition:#X}({local_reg_string})\n").as_str();
                        } else {
                            //Register
                            //In this case, the local offset param is actually the register number
                            line += format!("Cond: r{condition}\n").as_str();
                        }
                    }
                    ExActionData::DestroyLocalPointer { local_pointer, .. } => {
                        let mode = action.action_param >> 7;
                        if mode == 0 {
                            //Local offset
                            line +=
                                format!("Pointer: {local_pointer:#X}({local_reg_string})\n").as_str();
                        } else {
                            //Register
                            line += format!("Pointer: r{local_pointer}\n").as_str();
                        }
                    }
                    ExActionData::DestroyLocalArray {
                        local_array,
                        elements,
                        element_size,
                        ..
                    } => {
                        line += format!("Array: {local_array:#X}({local_reg_string})\nElements: {elements}\nSize: {element_size}\n").as_str();
                    }
                    ExActionData::DestroyBase {
                        object_pointer,
                        member_offset,
                        ..
                    } => {
                        let mode = action.action_param >> 7;
                        if mode == 0 {
                            line += format!("Member: {object_pointer:#X}({local_reg_string})+{member_offset:#X}\n").as_str();
                        } else {
                            line +=
                                format!("Member: {member_offset:#X}(r{object_pointer})\n").as_str();
                        }
                    }
                    ExActionData::DestroyMember {
                        object_pointer,
                        member_offset,
                        ..
                    } => {
                        let mode = action.action_param >> 7;
                        if mode == 0 {
                            line += format!("Member: {object_pointer:#X}({local_reg_string})+{member_offset:#X}\n").as_str();
                        } else {
                            line +=
                                format!("Member: {member_offset:#X}(r{object_pointer})\n").as_str();
                        }
                    }
                    ExActionData::DestroyMemberCond {
                        condition,
                        object_pointer,
                        member_offset,
                        ..
                    } => {
                        let mode = (action.action_param >> 6) & 1;
                        if mode == 0 {
                            line += format!("Member: {object_pointer:#X}({local_reg_string})+{member_offset:#X}\n").as_str();
                        } else {
                            //Register
                            line +=
                                format!("Member: {member_offset:#X}(r{object_pointer})\n").as_str();
                        }
                        let condition_mode = action.action_param >> 7;
                        if condition_mode == 0 {
                            //Local offset
                            line += format!("Cond: {condition:#X}({local_reg_string})\n").as_str();
                        } else {
                            //Register
                            line += format!("Cond: r{condition}\n").as_str();
                        }
                    }
                    ExActionData::DestroyMemberArray {
                        object_pointer,
                        member_offset,
                        elements,
                        element_size,
                        ..
                    } => {
                        let mode = action.action_param >> 7;
                        if mode == 0 {
                            //Local offset
                            line += format!(
                                "Member: {object_pointer:#X}({local_reg_string})+0x{member_offset}\n"
                            )
                            .as_str();
                        } else {
                            //Register
                            line +=
                                format!("Member: {member_offset:#X}(r{object_pointer})\n").as_str();
                        }
                        line += format!("Elements: {elements}\nSize: {element_size}\n").as_str();
                    }
                    ExActionData::DeletePointer { object_pointer, .. } => {
                        let mode = action.action_param >> 7;
                        if mode == 0 {
                            //Local offset
                            line += format!("Pointer: {object_pointer:#X}({local_reg_string})\n")
                                .as_str();
                        } else {
                            //Register
                            line += format!("Pointer: r{object_pointer})\n").as_str();
                        }
                    }
                    ExActionData::DeletePointerCond {
                        condition,
                        object_pointer,
                        ..
                    } => {
                        let mode = (action.action_param >> 6) & 1;
                        if mode == 0 {
                            //Local offset
                            line += format!("Pointer: {object_pointer:#X}({local_reg_string})\n")
                                .as_str();
                        } else {
                            //Register
                            line += format!("Pointer: r{object_pointer})\n").as_str();
                        }
                        let condition_mode = action.action_param >> 7;
                        if condition_mode == 0 {
                            //Local offset
                            line += format!("Cond: {condition:#X}({local_reg_string})\n").as_str();
                        } else {
                            //Register
                            line += format!("Cond: r{condition}\n").as_str();
                        }
                    }
                    ExActionData::CatchBlock {
                        catch_type,
                        catch_pc_offset,
                        cinfo_ref,
                        ..
                    } => {
                        line += format!("Local: {cinfo_ref:#X}({local_reg_string})\nPC: {catch_pc_offset:08X}\ncatch_type_addr: {catch_type:08X}\n").as_str();
                    }
                    ExActionData::ActiveCatchBlock { cinfo_ref } => {
                        line += format!("Local: {cinfo_ref:#X}({local_reg_string})\n").as_str();
                    }
                    ExActionData::Terminate => {}
                    ExActionData::Specification {
                        specs,
                        pc_offset,
                        cinfo_ref,
                        ..
                    } => {
                        line += format!("Local: {cinfo_ref:#X}({local_reg_string})\nPC: {pc_offset:08X}\nTypes: {specs}\n").as_str();
                    }
                    ExActionData::CatchBlock32 {
                        catch_type,
                        catch_pc_offset,
                        cinfo_ref,
                        ..
                    } => {
                        line += format!("Local: {cinfo_ref:#X}({local_reg_string})\nPC: {catch_pc_offset:08X}\ncatch_type_addr: {catch_type:08X}\n").as_str();
                    }
                }

                //If the action references a dtor, print it out using the name array
                if has_dtor_ref {
                    if func_index >= func_names.len() {
                        line += "Error: Invalid function array index\n";
                    } else {
                        let func_name = func_names[func_index].as_str();
                        line += format!("Dtor: \"{func_name}\"\n").as_str();
                        func_index += 1;
                    }
                }

                if action.has_end_bit {
                    line += "Has end bit\n"
                };
                sb += line.as_str(); //Print the line
            }
        }

        Some(sb)
    }
}