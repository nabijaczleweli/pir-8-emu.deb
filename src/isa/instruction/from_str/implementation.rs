use self::super::super::{InstructionLoadImmediateWideRegisterPair, AluOperationShiftOrRotateDirection, AluOperationShiftOrRotateType, InstructionJumpCondition,
                         InstructionPortDirection, InstructionMadrDirection, InstructionStckDirection, InstructionRegisterPair, AluOperation, Instruction};
use self::super::super::super::super::util::{parse_with_prefix, limit_to_width};
use self::super::super::super::GeneralPurposeRegisterBank;
use self::super::ParseInstructionError;
use std::convert::TryFrom;
use std::str::FromStr;
use std::usize;


impl Instruction {
    #[cfg_attr(rustfmt, rustfmt_skip)]
    pub(in self::super::super) fn from_str_impl(s: &str, registers: &GeneralPurposeRegisterBank) -> Result<Instruction, ParseInstructionError> {
        if let Some(idx) = s.find(is_invalid_character) {
            return Err(ParseInstructionError::InvalidCharacter(idx));
        }

        let mut tokens = s.split_whitespace();
        let instruction = parse_instruction(&mut tokens, s, registers)?;

        if let Some(tok) = tokens.next() {
            return Err(ParseInstructionError::TooManyTokens((tok.as_ptr() as usize) - (s.as_ptr() as usize) + 1));
        }

        Ok(instruction)
    }
}

impl FromStr for AluOperation {
    type Err = ParseInstructionError;

    /// Parse ALU operation in assembly instruction format
    ///
    /// The input string must be ASCII and contain no vertical whitespace
    ///
    /// # Examples
    ///
    /// ```
    /// # use pir_8_emu::isa::instruction::{AluOperationShiftOrRotateDirection, AluOperationShiftOrRotateType, AluOperation};
    /// # use std::str::FromStr;
    /// assert_eq!(AluOperation::from_str("XOR"),
    ///            Ok(AluOperation::Xor));
    ///
    /// assert_eq!(AluOperation::from_str("SOR RIGHT ASF"),
    ///            Ok(AluOperation::ShiftOrRotate {
    ///                d: AluOperationShiftOrRotateDirection::Right,
    ///                tt: AluOperationShiftOrRotateType::Asf,
    ///            }));
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(idx) = s.find(is_invalid_character) {
            return Err(ParseInstructionError::InvalidCharacter(idx));
        }

        let mut tokens = s.split_whitespace();
        let operation = parse_alu_operation(&mut tokens, s, usize::MAX)?;

        if let Some(tok) = tokens.next() {
            return Err(ParseInstructionError::TooManyTokens((tok.as_ptr() as usize) - (s.as_ptr() as usize) + 1));
        }

        Ok(operation)
    }
}

fn is_invalid_character(c: char) -> bool {
    c == '\n' || c == '\x0B' || !c.is_ascii()
}


fn parse_instruction<'i, I: Iterator<Item = &'i str>>(itr: &mut I, orig_str: &str, registers: &GeneralPurposeRegisterBank)
                                                      -> Result<Instruction, ParseInstructionError> {
    static VALID_TOKENS: &[&str] = &["LOAD",
                                     "JMPZ",
                                     "JMPP",
                                     "JMPG",
                                     "JMPC",
                                     "JMZG",
                                     "JMZL",
                                     "JMPL",
                                     "JUMP",
                                     "SAVE",
                                     "ALU",
                                     "MOVE",
                                     "MADR",
                                     "PORT",
                                     "COMP",
                                     "STCK",
                                     "CLRF",
                                     "HALT",
                                     "[raw instruction literal]"];

    match itr.next() {
        Some(tok) => {
            let start_pos = (tok.as_ptr() as usize) - (orig_str.as_ptr() as usize);

            // LOAD start
            if tok.eq_ignore_ascii_case("LOAD") {
                parse_instruction_load(itr, orig_str, start_pos + 4 + 1, registers)
            }
            // LOAD end

            // JUMP start
            else if tok.eq_ignore_ascii_case("JMPZ") {
                Ok(Instruction::Jump(InstructionJumpCondition::Jmpz))
            } else if tok.eq_ignore_ascii_case("JMPP") {
                Ok(Instruction::Jump(InstructionJumpCondition::Jmpp))
            } else if tok.eq_ignore_ascii_case("JMPG") {
                Ok(Instruction::Jump(InstructionJumpCondition::Jmpg))
            } else if tok.eq_ignore_ascii_case("JMPC") {
                Ok(Instruction::Jump(InstructionJumpCondition::Jmpc))
            } else if tok.eq_ignore_ascii_case("JMZG") {
                Ok(Instruction::Jump(InstructionJumpCondition::Jmzg))
            } else if tok.eq_ignore_ascii_case("JMZL") {
                Ok(Instruction::Jump(InstructionJumpCondition::Jmzl))
            } else if tok.eq_ignore_ascii_case("JMPL") {
                Ok(Instruction::Jump(InstructionJumpCondition::Jmpl))
            } else if tok.eq_ignore_ascii_case("JUMP") {
                Ok(Instruction::Jump(InstructionJumpCondition::Jump))
            }
            // JUMP end

            // SAVE start
            else if tok.eq_ignore_ascii_case("SAVE") {
                Ok(Instruction::Save { rrr: parse_register(itr, orig_str, start_pos + 4 + 1, registers)?.1 })
            }
            // SAVE end

            // ALU end
            else if tok.eq_ignore_ascii_case("ALU") {
                Ok(Instruction::Alu(parse_alu_operation(itr, orig_str, start_pos + 3 + 1)?))
            }
            // ALU end

            // MOVE end
            else if tok.eq_ignore_ascii_case("MOVE") {
                let (qqq_pos, qqq) = parse_register(itr, orig_str, start_pos + 4 + 1, registers)?;
                Ok(Instruction::Move {
                    qqq: qqq,
                    rrr: parse_register(itr, orig_str, qqq_pos + 1 + 1, registers)?.1,
                })
            }
            // MOVE end

            // MADR start
            else if tok.eq_ignore_ascii_case("MADR") {
                parse_instruction_madr(itr, orig_str, start_pos + 4 + 1)
            }
            // MADR end

            // PORT end
            else if tok.eq_ignore_ascii_case("PORT") {
                parse_instruction_port(itr, orig_str, start_pos + 4 + 1, registers)
            }
            // PORT end

            // COMP start
            else if tok.eq_ignore_ascii_case("COMP") {
                Ok(Instruction::Comp { rrr: parse_register(itr, orig_str, start_pos + 4 + 1, registers)?.1 })
            }
            // COMP end

            // STCK start
            else if tok.eq_ignore_ascii_case("STCK") {
                parse_instruction_stck(itr, orig_str, start_pos + 4 + 1)
            }
            // STCK end

            // CLRF start
            else if tok.eq_ignore_ascii_case("CLRF") {
                Ok(Instruction::Clrf)
            }
            // CLRF end

            // HALT start
            else if tok.eq_ignore_ascii_case("HALT") {
                Ok(Instruction::Halt)
            }
            // HALT end

            // Raw/restricted start
            else if let Some(raw) = parse_with_prefix::<u8>(tok) {
                Ok(Instruction::from(raw))
            }
            // Raw/restricted end
            else {
                Err(ParseInstructionError::UnrecognisedToken(start_pos + 1, VALID_TOKENS))
            }
        }
        None => Err(ParseInstructionError::EmptyString),
    }
}

fn parse_instruction_madr<'i, I: Iterator<Item = &'i str>>(itr: &mut I, orig_str: &str, pos: usize) -> Result<Instruction, ParseInstructionError> {
    fn map(d: InstructionMadrDirection, r: InstructionRegisterPair) -> Instruction {
        Instruction::Madr { d: d, r: r }
    }

    static VALID_TOKENS: &[&str] = &["WRITE", "READ"];

    parse_instruction_direction_register_pair(itr,
                                              orig_str,
                                              pos,
                                              VALID_TOKENS,
                                              &[("WRITE", |r| map(InstructionMadrDirection::Write, r)), ("READ", |r| map(InstructionMadrDirection::Read, r))])
}

fn parse_instruction_load<'i, I: Iterator<Item = &'i str>>(itr: &mut I, orig_str: &str, pos: usize, registers: &GeneralPurposeRegisterBank)
                                                           -> Result<Instruction, ParseInstructionError> {
    static VALID_TOKENS: &[&str] = &["IMM", "IND"];

    match itr.next() {
        Some(tok) => {
            let start_pos = (tok.as_ptr() as usize) - (orig_str.as_ptr() as usize);

            if tok.eq_ignore_ascii_case("IMM") {
                parse_instruction_load_immediate(itr, orig_str, start_pos + 3 + 1, registers)
            } else if tok.eq_ignore_ascii_case("IND") {
                Ok(Instruction::LoadIndirect { rrr: parse_register(itr, orig_str, start_pos + 3 + 1, registers)?.1 })
            } else {
                Err(ParseInstructionError::UnrecognisedToken(start_pos + 1, VALID_TOKENS))
            }
        }
        None => Err(ParseInstructionError::MissingToken(pos, VALID_TOKENS)),
    }
}

fn parse_instruction_load_immediate<'i, I: Iterator<Item = &'i str>>(itr: &mut I, orig_str: &str, pos: usize, registers: &GeneralPurposeRegisterBank)
                                                                     -> Result<Instruction, ParseInstructionError> {
    static VALID_TOKENS: &[&str] = &["BYTE", "WIDE"];

    match itr.next() {
        Some(tok) => {
            let start_pos = (tok.as_ptr() as usize) - (orig_str.as_ptr() as usize);

            if tok.eq_ignore_ascii_case("BYTE") {
                Ok(Instruction::LoadImmediateByte { rrr: parse_register(itr, orig_str, start_pos + 4 + 1, registers)?.1 })
            } else if tok.eq_ignore_ascii_case("WIDE") {
                Ok(Instruction::LoadImmediateWide { rr: parse_instruction_load_immediate_wide_register_pair(itr, orig_str, start_pos + 4 + 1)? })
            } else {
                Err(ParseInstructionError::UnrecognisedToken(start_pos + 1, VALID_TOKENS))
            }
        }
        None => Err(ParseInstructionError::MissingToken(pos, VALID_TOKENS)),
    }
}

fn parse_instruction_port<'i, I: Iterator<Item = &'i str>>(itr: &mut I, orig_str: &str, pos: usize, registers: &GeneralPurposeRegisterBank)
                                                           -> Result<Instruction, ParseInstructionError> {
    static VALID_TOKENS: &[&str] = &["IN", "OUT"];

    match itr.next() {
        Some(tok) => {
            let start_pos = (tok.as_ptr() as usize) - (orig_str.as_ptr() as usize);

            if tok.eq_ignore_ascii_case("IN") {
                Ok(Instruction::Port {
                    d: InstructionPortDirection::In,
                    rrr: parse_register(itr, orig_str, start_pos + 2 + 1, registers)?.1,
                })
            } else if tok.eq_ignore_ascii_case("OUT") {
                Ok(Instruction::Port {
                    d: InstructionPortDirection::Out,
                    rrr: parse_register(itr, orig_str, start_pos + 3 + 1, registers)?.1,
                })
            } else {
                Err(ParseInstructionError::UnrecognisedToken(start_pos + 1, VALID_TOKENS))
            }
        }
        None => Err(ParseInstructionError::MissingToken(pos, VALID_TOKENS)),
    }
}

fn parse_instruction_stck<'i, I: Iterator<Item = &'i str>>(itr: &mut I, orig_str: &str, pos: usize) -> Result<Instruction, ParseInstructionError> {
    fn map(d: InstructionStckDirection, r: InstructionRegisterPair) -> Instruction {
        Instruction::Stck { d: d, r: r }
    }

    static VALID_TOKENS: &[&str] = &["PUSH", "POP"];

    parse_instruction_direction_register_pair(itr,
                                              orig_str,
                                              pos,
                                              VALID_TOKENS,
                                              &[("PUSH", |r| map(InstructionStckDirection::Push, r)), ("POP", |r| map(InstructionStckDirection::Pop, r))])
}

fn parse_instruction_direction_register_pair<'i, I: Iterator<Item = &'i str>>(itr: &mut I, orig_str: &str, pos: usize, tokens: &'static [&'static str],
                                                                              mapping: &[(&str, fn(InstructionRegisterPair) -> Instruction)])
                                                                              -> Result<Instruction, ParseInstructionError> {
    match itr.next() {
        Some(tok) => {
            let start_pos = (tok.as_ptr() as usize) - (orig_str.as_ptr() as usize);

            for (dir, map) in mapping {
                if tok.eq_ignore_ascii_case(dir) {
                    return Ok(map(parse_instruction_register_pair(itr, orig_str, start_pos + dir.len() + 1)?));
                }
            }

            Err(ParseInstructionError::UnrecognisedToken(start_pos + 1, tokens))
        }
        None => Err(ParseInstructionError::MissingToken(pos, tokens)),
    }
}

fn parse_instruction_load_immediate_wide_register_pair<'i, I: Iterator<Item = &'i str>>(
    itr: &mut I, orig_str: &str, pos: usize)
    -> Result<InstructionLoadImmediateWideRegisterPair, ParseInstructionError> {
    static VALID_TOKENS: &[&str] = &["A&B", "C&D", "X&Y", "ADR"];

    match itr.next() {
        Some(tok) => {
            let start_pos = (tok.as_ptr() as usize) - (orig_str.as_ptr() as usize);

            if tok.eq_ignore_ascii_case("A&B") {
                Ok(InstructionLoadImmediateWideRegisterPair::Ab)
            } else if tok.eq_ignore_ascii_case("C&D") {
                Ok(InstructionLoadImmediateWideRegisterPair::Cd)
            } else if tok.eq_ignore_ascii_case("X&Y") {
                Ok(InstructionLoadImmediateWideRegisterPair::Xy)
            } else if tok.eq_ignore_ascii_case("ADR") {
                Ok(InstructionLoadImmediateWideRegisterPair::Adr)
            } else {
                Err(ParseInstructionError::UnrecognisedToken(start_pos + 1, VALID_TOKENS))
            }
        }
        None => Err(ParseInstructionError::MissingToken(pos, VALID_TOKENS)),
    }
}

fn parse_instruction_register_pair<'i, I: Iterator<Item = &'i str>>(itr: &mut I, orig_str: &str, pos: usize)
                                                                    -> Result<InstructionRegisterPair, ParseInstructionError> {
    static VALID_TOKENS: &[&str] = &["A&B", "C&D"];

    match itr.next() {
        Some(tok) => {
            let start_pos = (tok.as_ptr() as usize) - (orig_str.as_ptr() as usize);

            if tok.eq_ignore_ascii_case("A&B") {
                Ok(InstructionRegisterPair::Ab)
            } else if tok.eq_ignore_ascii_case("C&D") {
                Ok(InstructionRegisterPair::Cd)
            } else {
                Err(ParseInstructionError::UnrecognisedToken(start_pos + 1, VALID_TOKENS))
            }
        }
        None => Err(ParseInstructionError::MissingToken(pos, VALID_TOKENS)),
    }
}

fn parse_alu_operation<'i, I: Iterator<Item = &'i str>>(itr: &mut I, orig_str: &str, pos: usize) -> Result<AluOperation, ParseInstructionError> {
    static VALID_TOKENS: &[&str] = &["ADD", "SUB", "ADDC", "SUBC", "OR", "XOR", "AND", "NOT", "SOR", "[raw operation literal]"];

    match itr.next() {
        Some(tok) => {
            let start_pos = (tok.as_ptr() as usize) - (orig_str.as_ptr() as usize);

            if tok.eq_ignore_ascii_case("ADD") {
                Ok(AluOperation::Add)
            } else if tok.eq_ignore_ascii_case("SUB") {
                Ok(AluOperation::Sub)
            } else if tok.eq_ignore_ascii_case("ADDC") {
                Ok(AluOperation::AddC)
            } else if tok.eq_ignore_ascii_case("SUBC") {
                Ok(AluOperation::SubC)
            } else if tok.eq_ignore_ascii_case("OR") {
                Ok(AluOperation::Or)
            } else if tok.eq_ignore_ascii_case("XOR") {
                Ok(AluOperation::Xor)
            } else if tok.eq_ignore_ascii_case("AND") {
                Ok(AluOperation::And)
            } else if tok.eq_ignore_ascii_case("NOT") {
                Ok(AluOperation::Not)
            } else if tok.eq_ignore_ascii_case("SOR") {
                parse_alu_operation_shift_or_rotate(itr, orig_str, start_pos + 3 + 1)
            } else if let Some(raw) = parse_with_prefix::<u8>(tok).and_then(|n| limit_to_width(n, 4)) {
                Ok(AluOperation::try_from(raw).expect("Wrong raw instruction slicing for ALU op parse"))
            } else {
                Err(ParseInstructionError::UnrecognisedToken(start_pos + 1, VALID_TOKENS))
            }
        }
        None => {
            if pos == usize::MAX {
                Err(ParseInstructionError::EmptyString)
            } else {
                Err(ParseInstructionError::MissingToken(pos, VALID_TOKENS))
            }
        }
    }
}

fn parse_alu_operation_shift_or_rotate<'i, I: Iterator<Item = &'i str>>(itr: &mut I, orig_str: &str, pos: usize)
                                                                        -> Result<AluOperation, ParseInstructionError> {
    static VALID_TOKENS: &[&str] = &["LEFT", "RIGHT"];

    match itr.next() {
        Some(tok) => {
            let start_pos = (tok.as_ptr() as usize) - (orig_str.as_ptr() as usize);

            if tok.eq_ignore_ascii_case("LEFT") {
                Ok(AluOperation::ShiftOrRotate {
                    d: AluOperationShiftOrRotateDirection::Left,
                    tt: parse_alu_operation_shift_or_rotate_type(itr, orig_str, start_pos + 4 + 1)?,
                })
            } else if tok.eq_ignore_ascii_case("RIGHT") {
                Ok(AluOperation::ShiftOrRotate {
                    d: AluOperationShiftOrRotateDirection::Right,
                    tt: parse_alu_operation_shift_or_rotate_type(itr, orig_str, start_pos + 5 + 1)?,
                })
            } else {
                Err(ParseInstructionError::UnrecognisedToken(start_pos + 1, VALID_TOKENS))
            }
        }
        None => Err(ParseInstructionError::MissingToken(pos, VALID_TOKENS)),
    }
}

fn parse_alu_operation_shift_or_rotate_type<'i, I: Iterator<Item = &'i str>>(itr: &mut I, orig_str: &str, pos: usize)
                                                                             -> Result<AluOperationShiftOrRotateType, ParseInstructionError> {
    static VALID_TOKENS: &[&str] = &["LSF", "ASF", "RTC", "RTW"];

    match itr.next() {
        Some(tok) => {
            let start_pos = (tok.as_ptr() as usize) - (orig_str.as_ptr() as usize);

            if tok.eq_ignore_ascii_case("LSF") {
                Ok(AluOperationShiftOrRotateType::Lsf)
            } else if tok.eq_ignore_ascii_case("ASF") {
                Ok(AluOperationShiftOrRotateType::Asf)
            } else if tok.eq_ignore_ascii_case("RTC") {
                Ok(AluOperationShiftOrRotateType::Rtc)
            } else if tok.eq_ignore_ascii_case("RTW") {
                Ok(AluOperationShiftOrRotateType::Rtw)
            } else {
                Err(ParseInstructionError::UnrecognisedToken(start_pos + 1, VALID_TOKENS))
            }
        }
        None => Err(ParseInstructionError::MissingToken(pos, VALID_TOKENS)),
    }
}

fn parse_register<'i, I: Iterator<Item = &'i str>>(itr: &mut I, orig_str: &str, pos: usize, registers: &GeneralPurposeRegisterBank)
                                                   -> Result<(usize, u8), ParseInstructionError> {
    static VALID_TOKENS: &[&str] = &["[register letter]"];

    match itr.next() {
        Some(tok) => {
            let start_pos = (tok.as_ptr() as usize) - (orig_str.as_ptr() as usize);

            let mut cc = tok.chars();
            let letter = cc.next().expect("non-empty token didn't have characters");

            if cc.next().is_none() {
                registers.iter()
                    .find(|r| r.letter().eq_ignore_ascii_case(&letter))
                    .map(|r| (start_pos, r.address()))
                    .ok_or_else(|| {
                        ParseInstructionError::UnrecognisedRegisterLetter(start_pos + 1,
                                                                          letter,
                                                                          [registers[0].letter(),
                                                                           registers[1].letter(),
                                                                           registers[2].letter(),
                                                                           registers[3].letter(),
                                                                           registers[4].letter(),
                                                                           registers[5].letter(),
                                                                           registers[6].letter(),
                                                                           registers[7].letter()])
                    })
            } else {
                Err(ParseInstructionError::UnrecognisedToken(start_pos + 1, VALID_TOKENS))
            }
        }
        None => {
            Err(ParseInstructionError::MissingRegisterLetter(pos,
                                                             [registers[0].letter(),
                                                              registers[1].letter(),
                                                              registers[2].letter(),
                                                              registers[3].letter(),
                                                              registers[4].letter(),
                                                              registers[5].letter(),
                                                              registers[6].letter(),
                                                              registers[7].letter()]))
        }
    }
}
