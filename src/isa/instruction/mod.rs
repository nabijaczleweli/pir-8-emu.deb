//! An instruction is a single byte, and can include some following immediate values purely for data.


mod instruction;
mod from_str;
mod display;
mod execute;

pub use self::instruction::{InstructionLoadImmediateWideRegisterPair, AluOperationShiftOrRotateDirection, AluOperationShiftOrRotateType,
                            InstructionJumpCondition, InstructionPortDirection, InstructionMadrDirection, InstructionStckDirection, InstructionRegisterPair,
                            AluOperation, Instruction};
pub use self::from_str::ParseInstructionError;
pub use self::display::DisplayInstruction;
