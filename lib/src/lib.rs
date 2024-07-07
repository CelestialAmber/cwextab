use anyhow::Result;
use anyhow::bail;

fn read_byte(data: &[u8], offset: &mut i32, update_offset: bool) -> u8 {
    let index = *offset as usize;
    let b = data[index];
    if update_offset {
        *offset += 1;
    }
    b
}

fn read_uint16(data: &[u8], offset: &mut i32, update_offset: bool) -> u16 {
    let index = *offset as usize;
    let bytes = data[index..index + 2].try_into().unwrap();
    if update_offset {
        *offset += 2;
    }
    u16::from_be_bytes(bytes)
}

fn read_uint32(data: &[u8], offset: &mut i32, update_offset: bool) -> u32 {
    let index = *offset as usize;
    let bytes = data[index..index + 4].try_into().unwrap();
    if update_offset {
        *offset += 4;
    }
    u32::from_be_bytes(bytes)
}

#[derive(Copy, Clone)]
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
    
    pub fn from_int(val: i32) -> Result<ExAction> {
        let result : ExAction = match val {
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
                bail!("Invalid action value {}", val);
            },
        };
        Ok(result)
    }
    
    pub fn has_dtor_ref(action : ExAction) -> bool {
        match action {
            ExAction::EndOfList => {
                println!("Warning: null action passed");
                false
            },
            ExAction::Branch | ExAction::CatchBlock | ExAction::ActiveCatchBlock |
            ExAction::Terminate | ExAction::Specification | ExAction::CatchBlock32 => false,
            _ => true,
        }
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
    
    fn convert_to_string(&self) -> String {
        String::from(Self::ACTION_NAMES[self.to_int() as usize])
    }
}

#[derive(Clone)]
pub struct ExceptionAction {
    //General values
    pub action_offset: u32,
    pub action_type: ExAction, //0x0
    pub action_param: u8,      //0x1
    pub has_end_bit: bool,      //true if action type byte has bit 7 set (type & 0x80)
    pub bytes: Vec<u8>,
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
}

impl Default for ExceptionAction {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone)]
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

pub struct ExceptionTableData {
    pub flag_val: u16, //0x0-1
    //Flag bits (16 bit value)
    pub has_elf_vector: bool,    //bit 1
    pub large_frame: bool,      //bit 3
    pub has_frame_pointer: bool, //bit 4
    pub saved_cr: bool,         //bit 5
    pub fpr_save_range: u32,     //bits 6-10
    pub gpr_save_range: u32,     //bits 11-15

    pub et_field: u16, //0x2-3

    pub pc_actions: Vec<PCAction>,
    pub exception_actions: Vec<ExceptionAction>,
    pub func_names: Vec<String>,
}

impl ExceptionTableData {
    fn new() -> Self {
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
            func_names: vec![],
        }
    }

    fn calculate_flag_values(&mut self) {
        self.has_elf_vector = ((self.flag_val >> 1) & 1) == 1;
        self.large_frame = ((self.flag_val >> 3) & 1) == 1;
        self.has_frame_pointer = ((self.flag_val >> 4) & 1) == 1;
        self.saved_cr = ((self.flag_val >> 5) & 1) == 1;
        self.fpr_save_range = ((self.flag_val >> 6) & 0b11111) as u32;
        self.gpr_save_range = ((self.flag_val >> 11) & 0b11111) as u32;
    }
    
    fn get_func_name(&self, index : usize) -> Result<String> {
        if index >= self.func_names.len() { bail!("Invalid function array index"); }
        Ok(self.func_names[index].clone())
    }

    fn convert_to_text(&self) -> Option<String> {
        let mut sb = String::from("");

        sb += "Flag values:\n";
        sb += format!("{}",format_args!("Has Elf Vector: {}\n",if self.has_elf_vector { "Yes" } else { "No" })).as_str();
        sb += format!("{}",format_args!("Large Frame: {}\n",if self.large_frame { "Yes" } else { "No" })).as_str();
        sb += format!("{}",format_args!("Has Frame Pointer: {}\n",if self.has_frame_pointer { "Yes" } else { "No" })).as_str();
        sb += format!("{}",format_args!("Saved CR: {}\n", if self.saved_cr { "Yes" } else { "No" })).as_str();

        if self.fpr_save_range != 0 {
            let start_fpr = 31 - (self.fpr_save_range - 1);
            let fpr_string: String = 
            if start_fpr == 31 {
                String::from("fp31")
            } else {
                format!("fp{start_fpr}-fp31")
            };
            sb += format!("Saved FPR range: {fpr_string}\n").as_str();
        }
        if self.gpr_save_range != 0 {
            let start_gpr = 31 - (self.gpr_save_range - 1);
            let gpr_string: String =
            if start_gpr == 31 {
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
                    sb += format!("PC={start_pc:08X}:{end_pc:08X}, Action: {action_offset:06X}\n").as_str();
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
                
                let mut data_offset: i32 = 0;
                let has_dtor_ref = ExAction::has_dtor_ref(action.action_type);

                match action.action_type {
                    ExAction::EndOfList => {}
                    ExAction::Branch => {
                        let target_offset = read_uint16(&action.bytes, &mut data_offset, true);
                        line += format!("Action: {target_offset:06X}").as_str();
                    }
                    ExAction::DestroyLocal => {
                        let local_offset = read_uint16(&action.bytes, &mut data_offset, true);
                        //let dtor_address = read_uint32(&action.bytes, &mut data_offset, true);
                        line += format!("Local: {local_offset:#X}({local_reg_string})").as_str();
                    }
                    ExAction::DestroyLocalCond => {
                        let condition = read_uint16(&action.bytes, &mut data_offset, true);
                        let local_offset = read_uint16(&action.bytes, &mut data_offset, true);
                        //let dtor_address = read_uint32(&action.bytes, &mut data_offset, true);
                        line += format!("Local: {local_offset:#X}({local_reg_string})").as_str();

                        //The action param is used to determine the type of reference for the condition (0: local offset, 1: register)
                        if action.action_param == 0 {
                            //Local offset
                            line += format!("\nCond: {condition:#X}({local_reg_string})").as_str();
                        } else {
                            //Register
                            //In this case, the local offset param is actually the register number
                            line += format!("\nCond: r{condition}").as_str();
                        }
                    }
                    ExAction::DestroyLocalPointer => {
                        let local_pointer = read_uint16(&action.bytes, &mut data_offset, true);
                        //let dtor_address = read_uint32(&action.bytes, &mut data_offset, true);

                        let mode = action.action_param >> 7;
                        if mode == 0 {
                            //Local offset
                            line += format!("Pointer: {local_pointer:#X}({local_reg_string})").as_str();
                        } else {
                            //Register
                            line += format!("Pointer: r{local_pointer}").as_str();
                        }
                    }
                    ExAction::DestroyLocalArray => {
                        let local_array = read_uint16(&action.bytes, &mut data_offset, true);
                        let elements = read_uint16(&action.bytes, &mut data_offset, true);
                        let element_size = read_uint16(&action.bytes, &mut data_offset, true);
                        //let dtor_address = read_uint32(&action.bytes, &mut data_offset, true);
                        line += format!("Array: {local_array:#X}({local_reg_string})\nElements: {elements}\nSize: {element_size}").as_str();
                    }
                    ExAction::DestroyBase | ExAction::DestroyMember => {
                        let object_pointer = read_uint16(&action.bytes, &mut data_offset, true);
                        let member_offset = read_uint32(&action.bytes, &mut data_offset, true);
                        //let dtor_address = read_uint32(&action.bytes, &mut data_offset, true);
                        let mode = action.action_param >> 7;
                        if mode == 0 {
                            line += format!("Member: {object_pointer:#X}({local_reg_string})+{member_offset:#X}").as_str();
                        } else {
                            //Register
                            line += format!("Member: {member_offset:#X}(r{object_pointer})").as_str();
                        }
                    }
                    ExAction::DestroyMemberCond => {
                        let condition = read_uint16(&action.bytes, &mut data_offset, true);
                        let object_pointer = read_uint16(&action.bytes, &mut data_offset, true);
                        let member_offset = read_uint32(&action.bytes, &mut data_offset, true);
                        //let dtor_address = read_uint32(&action.bytes, &mut data_offset, true);
                        let mode = (action.action_param >> 6) & 1;
                        if mode == 0 {
                            line += format!("Member: {object_pointer:#X}({local_reg_string})+{member_offset:#X}").as_str();
                        } else {
                            //Register
                            line += format!("Member: {member_offset:#X}(r{object_pointer})").as_str();
                        }
                        let condition_mode = action.action_param >> 7;
                        if condition_mode == 0 {
                            //Local offset
                            line += format!("\nCond: {condition:#X}({local_reg_string})").as_str();
                        } else {
                            //Register
                            line += format!("\nCond: r{condition}").as_str();
                        }
                    }
                    ExAction::DestroyMemberArray => {
                        let object_pointer = read_uint16(&action.bytes, &mut data_offset, true);
                        let member_offset = read_uint32(&action.bytes, &mut data_offset, true);
                        let elements = read_uint32(&action.bytes, &mut data_offset, true);
                        let element_size = read_uint32(&action.bytes, &mut data_offset, true);
                        //let dtor_address = read_uint32(&action.bytes, &mut data_offset, true);

                        let mode = action.action_param >> 7;
                        if mode == 0 {
                            //Local offset
                            line += format!("Member: {object_pointer:#X}({local_reg_string})+0x{member_offset}").as_str();
                        } else {
                            //Register
                            line += format!("Member: {member_offset:#X}(r{object_pointer})").as_str();
                        }
                        line += format!("\nElements: {elements}\nSize: {element_size}").as_str();
                    }
                    ExAction::DeletePointer => {
                        let object_pointer = read_uint16(&action.bytes, &mut data_offset, true);
                        //let dtor_address = read_uint32(&action.bytes, &mut data_offset, true);
                        let mode = action.action_param >> 7;
                        if mode == 0 {
                            //Local offset
                            line += format!("Pointer: {object_pointer:#X}({local_reg_string})").as_str();
                        } else {
                            //Register
                            line += format!("Pointer: r{object_pointer})").as_str();
                        }
                    }
                    ExAction::DeletePointerCond => {
                        let condition = read_uint16(&action.bytes, &mut data_offset, true);
                        let object_pointer = read_uint16(&action.bytes, &mut data_offset, true);
                        //let dtor_address = read_uint32(&action.bytes, &mut data_offset, true);
                        let mode = (action.action_param >> 6) & 1;
                        if mode == 0 {
                            //Local offset
                            line += format!("Pointer: {object_pointer:#X}({local_reg_string})").as_str();
                        } else {
                            //Register
                            line += format!("Pointer: r{object_pointer})").as_str();
                        }
                        let condition_mode = action.action_param >> 7;
                        if condition_mode == 0 {
                            //Local offset
                            line += format!("\nCond: {condition:#X}({local_reg_string})").as_str();
                        } else {
                            //Register
                            line += format!("\nCond: r{condition}").as_str();
                        }
                    }
                    ExAction::CatchBlock => {
                        data_offset += 2; //unused bytes?
                        let catch_type = read_uint32(&action.bytes, &mut data_offset, true);
                        let catch_pc_offset = read_uint16(&action.bytes, &mut data_offset, true);
                        let cinfo_ref = read_uint16(&action.bytes, &mut data_offset, true);
                        line += format!("Local: {cinfo_ref:#X}({local_reg_string})\nPC: {catch_pc_offset:08X}\ncatch_type_addr: {catch_type:08X}").as_str();
                    }
                    ExAction::ActiveCatchBlock => {
                        let cinfo_ref = read_uint16(&action.bytes, &mut data_offset, true);
                        line += format!("Local: {cinfo_ref:#X}({local_reg_string})").as_str();
                    }
                    ExAction::Terminate => {}
                    ExAction::Specification => {
                        let specs = read_uint16(&action.bytes, &mut data_offset, true);
                        let pc_offset = read_uint32(&action.bytes, &mut data_offset, true);
                        let cinfo_ref = read_uint32(&action.bytes, &mut data_offset, true);
                        line += format!("Local: {cinfo_ref:#X}({local_reg_string})\nPC: {pc_offset:08X}\nTypes: {specs}").as_str();
                    }
                    ExAction::CatchBlock32 => {
                        data_offset += 2; //unused bytes?
                        let catch_type = read_uint32(&action.bytes, &mut data_offset, true);
                        let catch_pc_offset = read_uint32(&action.bytes, &mut data_offset, true);
                        let cinfo_ref = read_uint32(&action.bytes, &mut data_offset, true);
                        line += format!("Local: {cinfo_ref:#X}({local_reg_string})\nPC: {catch_pc_offset:08X}\ncatch_type_addr: {catch_type:08X}").as_str();
                    }
                }
                
                //If the action references a dtor, print it out using the name array
                if has_dtor_ref {
                    let func_name : String = 
                    match self.get_func_name(func_index) {
                        Ok(name) => name,
                        Err(e) => {
                            println!("{}",e);
                            return None;
                        },
                    };
                    
                    line += format!("\nDtor: \"{func_name}\"").as_str();
                    func_index += 1;
                }

                if action.has_end_bit {
                    line += "."
                }; //Add a dot to the end if the has end bit flag is set
                line += "\n";
                sb += line.as_str(); //Print the line
            }
        }

        Some(sb)
    }
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
    
    fn set_func_names(&mut self, func_names : &[&str]){
        self.extab_data.func_names = Vec::from(func_names).iter().map(|v| v.to_string()).collect();
    }

    fn parse_exception_table(&mut self, bytes: &[u8]) -> Result<()> {
        self.offset = 0;
        self.data = Vec::from(bytes);
        self.length = self.data.len() as i32;
        
        //If the array is empty, return an error.
        if self.length < 8 {
            bail!("Error: Data array should at least be 8 bytes long.");
        }    

        //Parse the header flag value
        self.extab_data.flag_val = read_uint16(&self.data, &mut self.offset, true);
        self.extab_data.calculate_flag_values();
        self.extab_data.et_field = read_uint16(&self.data, &mut self.offset, true);
        
        //Check whether the table is 8 bytes but the terminator isn't zero. If so,
        //throw an error.
        let terminator = read_uint32(&self.data, &mut self.offset, false);
        if self.length == 8 && terminator != 0 {
            bail!("Error: Invalid extab table, table is 8 bytes long but terminator is not zero.");
        }

        //Parse range entries until we hit the terminator (32 bit zero value)
        while read_uint32(&self.data, &mut self.offset, false) != 0 {
            let mut pcaction = PCAction::new();
            pcaction.start_pc = read_uint32(&self.data, &mut self.offset, true);
            let range_size: u32 = (read_uint16(&self.data, &mut self.offset, true) as u32) * 4; //range size is encoded as size >> 2
            pcaction.end_pc = pcaction.start_pc + range_size;
            pcaction.action_offset = read_uint16(&self.data, &mut self.offset, true) as u32;
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

    fn parse_action_entry(&mut self) -> Result<()> {
        let mut exaction = ExceptionAction::new();
        exaction.action_offset = self.offset as u32;
        let action_type_byte = read_byte(&self.data, &mut self.offset, true);
        exaction.has_end_bit = (action_type_byte & 0x80) != 0;
        exaction.action_type = ExAction::from_int((action_type_byte & 0x7F) as i32)?;
        exaction.action_param = read_byte(&self.data, &mut self.offset, true);

        //Since the way action data is stored is too varied, we just store the remaining data as a byte
        //array to be used later.
        let mut size: i32;

        match exaction.action_type {
            ExAction::Branch => {
                size = 2;
            }
            ExAction::DestroyLocal => {
                size = 6;
            }
            ExAction::DestroyLocalCond => {
                size = 8;
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
                size = 12;
            }
            ExAction::DestroyMemberArray => {
                size = 18;
            }
            ExAction::DeletePointer => {
                size = 6;
            }
            ExAction::DeletePointerCond => {
                size = 8;
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
                let length = read_uint16(&self.data, &mut self.offset, false) as i32;
                size += length * 4;
            }
            ExAction::CatchBlock32 => {
                size = 14;
            }
            _ => {
                bail!("Error: invalid action value, should not happen");
            }
        }

        let start_index = self.offset as usize;
        let end_index = (self.offset + size) as usize;
        exaction.bytes = self.data[start_index..end_index].into();
        self.offset += size;

        self.extab_data.exception_actions.push(exaction);
        Ok(())
    }

    fn convert_to_text(&self) -> Option<String> {
        let text = self.extab_data.convert_to_text()?;
        Some(text)
    }
}



/// Decodes the provided exception table data.
///
/// Returns 'None' if the table is not valid.
pub fn decode_extab(data : &[u8]) -> Option<ExceptionTableData> {
    let mut decoder = ExtabDecoder::new();
    let result = decoder.parse_exception_table(data);
    if let Err(e) = result {
        println!("Error: Failed to decode extab data:");
        println!("{}",e);
        return None;
    }
    Some(decoder.extab_data)
}

/// Decodes the provided exception table data, and returns
/// it formatted as a string using a provided list of function names.
///
/// Returns 'None' if the table or provided function list is not valid
pub fn decode_extab_to_text(data : &[u8], func_names : &[&str]) -> Option<String> {
    let mut decoder = ExtabDecoder::new();
    decoder.set_func_names(func_names);
    let result = decoder.parse_exception_table(data);
    if let Err(e) = result {
        println!("Error: Failed to decode extab data:");
        println!("{}",e);
        return None;
    }
    let text = decoder.convert_to_text()?;
    Some(text)
}

