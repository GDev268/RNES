use std::{borrow::Borrow, collections::HashMap};

use crate::cpu::{StatusFlags, CPU};

///Table Matrix of all opcodes and instructions
pub const lookup_table: Vec<INSTRUCTION> = vec![];

///Opcode Instruction Struct
pub(crate) struct INSTRUCTION {
    pub name: String,
    pub addr_mode: Box<dyn Fn(&mut CPU) -> u8>,
    pub operate: Box<dyn Fn(&mut CPU) -> u8>,
    pub cycles: u8,
}

//Addressing Modes

///Implied Addressing Mode
pub fn imp(cpu: &mut CPU) -> u8 {
    cpu.fetched = cpu.get_accumulator();
    return 0;
}

///Immediate Addressing Mode
pub fn imm(cpu: &mut CPU) -> u8 {
    cpu.program_counter += 1;
    cpu.abs_addr = cpu.program_counter;

    return 0;
}

///Absolute Addressing Mode
pub fn abs(cpu: &mut CPU) -> u8 {
    let low_byte = cpu.read(cpu.program_counter) as u16;
    cpu.program_counter += 1;

    let high_byte = cpu.read(cpu.program_counter) as u16;
    cpu.program_counter += 1;

    cpu.abs_addr = (high_byte << 8) | low_byte;

    return 0;
}

///Absolute X Addressing Mode
pub fn abx(cpu: &mut CPU) -> u8 {
    let low_byte = cpu.read(cpu.program_counter) as u16;
    cpu.program_counter += 1;

    let high_byte = cpu.read(cpu.program_counter) as u16;
    cpu.program_counter += 1;

    cpu.abs_addr = (high_byte << 8) | low_byte;

    cpu.abs_addr += cpu.regx as u16;

    if (cpu.abs_addr & 0xFF00) != (high_byte << 8) {
        return 1;
    } else {
        return 0;
    }
}

///Absolute Y Addressing Mode
pub fn aby(cpu: &mut CPU) -> u8 {
    let low_byte = cpu.read(cpu.program_counter) as u16;
    cpu.program_counter += 1;

    let high_byte = cpu.read(cpu.program_counter) as u16;
    cpu.program_counter += 1;

    cpu.abs_addr = ((high_byte << 8) | low_byte) as u16;

    cpu.abs_addr += cpu.regx as u16;

    if (cpu.abs_addr & 0xFF00) != (high_byte << 8) {
        return 1;
    } else {
        return 0;
    }
}

///Relative Addressing Mode
pub fn rel(cpu: &mut CPU) -> u8 {
    cpu.rel_addr = cpu.read(cpu.program_counter) as u16;
    cpu.program_counter += 1;

    if (cpu.rel_addr & 0x80) != 0 {
        cpu.rel_addr |= 0xFF00;
    }

    return 0;
}

///Zero Page Addressing Mode
pub fn zp0(cpu: &mut CPU) -> u8 {
    cpu.abs_addr = cpu.read(cpu.program_counter) as u16;
    cpu.program_counter += 1;

    cpu.abs_addr &= 0x00FF;
    return 0;
}

///Zero Page X Addressing Mode
pub fn zpx(cpu: &mut CPU) -> u8 {
    cpu.abs_addr = (cpu.read(cpu.program_counter) + cpu.regx) as u16;
    cpu.program_counter += 1;

    cpu.abs_addr &= 0x00FF;

    return 0;
}

///Zero Page Y Addressing Mode
pub fn zpy(cpu: &mut CPU) -> u8 {
    cpu.abs_addr = (cpu.read(cpu.program_counter) + cpu.regy) as u16;
    cpu.program_counter += 1;

    cpu.abs_addr &= 0x00FF;

    return 0;
}

///Indirect X Addressing Mode
pub fn indx(cpu: &mut CPU) -> u8 {
    let instruction = cpu.read(cpu.program_counter);
    cpu.program_counter += 1;

    let low_byte = cpu.read(((instruction + cpu.regx) as u16) & 0x00FF) as u16;
    let high_byte = cpu.read(((instruction + (cpu.regx + 1)) as u16) & 0x00FF) as u16;

    cpu.abs_addr = (high_byte << 8) | low_byte;

    return 0;
}

///Indirect Y Addressing Mode
pub fn indy(cpu: &mut CPU) -> u8 {
    let instruction = cpu.read(cpu.program_counter);
    cpu.program_counter += 1;

    let low_byte = cpu.read((instruction as u16) & 0x00FF) as u16;
    let high_byte = cpu.read(((instruction + 1) as u16) & 0x00FF) as u16;

    cpu.abs_addr = (high_byte << 8) | low_byte;
    cpu.abs_addr += cpu.regy as u16;

    if (cpu.abs_addr & 0xFF00) != (high_byte << 8) as u16 {
        return 1;
    } else {
        return 0;
    }
}

//Opcodes

/// Add Memory to Accumulator With Carry<br>
/// Executes the equation A + M + C<br>
/// Uses the check_if_zero_or_negative_u16() function to trigger the Flags N (Negative) and Z (Zero)<br>
/// Uses the overflow equation to trigger the Flag V (Overflow)<br>
/// !(A^M) & (A^R)
pub fn adc(cpu: &mut CPU) {
    cpu.fetch();

    let value = cpu.get_accumulator() as u16
        + cpu.fetched as u16
        + cpu.get_flag(crate::cpu::StatusFlags::C) as u16;

    cpu.clear_flags(
        StatusFlags::N as u8 | StatusFlags::V as u8 | StatusFlags::Z as u8 | StatusFlags::C as u8,
    );

    check_if_zero_or_negative_u16(cpu, value);

    cpu.set_flag(
        StatusFlags::V,
        ((!(cpu.get_accumulator() as u16 ^ cpu.fetched as u16)
            & (cpu.get_accumulator() as u16 ^ value))
            & 0x0080)
            != 0,
    );

    cpu.set_flag(StatusFlags::C, value > 0x00FF);
    cpu.acu = (value & 0x00FF) as u8;
}

/// Subtraction with Borrow In<br>
/// Executes the equation A−M−(1−C)<br>
/// Uses the check_if_zero_or_negative_u16() function to trigger the Flags N (Negative) and Z (Zero)<br>
/// Uses the overflow equation to trigger the Flag V (Overflow)<br>
/// !(A^M) & (A^R)
pub fn sbc(cpu: &mut CPU) {
    cpu.fetch();

    let value = cpu.get_accumulator() as u16
        + (cpu.fetched ^ 0x00FF) as u16
        + cpu.get_flag(crate::cpu::StatusFlags::C) as u16;

    cpu.clear_flags(
        StatusFlags::N as u8 | StatusFlags::V as u8 | StatusFlags::Z as u8 | StatusFlags::C as u8,
    );

    check_if_zero_or_negative_u16(cpu, value);

    cpu.set_flag(
        StatusFlags::V,
        ((!(cpu.get_accumulator() as u16 ^ cpu.fetched as u16)
            & (cpu.get_accumulator() as u16 ^ value))
            & 0x0080)
            != 0,
    );

    cpu.set_flag(StatusFlags::C, value > 0x00FF);
    cpu.acu = (value & 0x00FF) as u8;
}

/// "AND" Memory with Accumulator<br>
/// Executes the equation A & M<br>
/// Uses the check_if_zero_or_negative_u16() function to trigger the Flags N (Negative) and Z (Zero)<br>
pub fn and(cpu: &mut CPU) {
    cpu.fetch();

    let value = cpu.get_accumulator() & cpu.fetched;

    cpu.clear_flags(StatusFlags::N as u8 | StatusFlags::Z as u8);

    check_if_zero_or_negative_u8(cpu, value);

    cpu.acu = value as u8;
}

pub fn asl(cpu: &mut CPU) {
    cpu.fetch();

    let value = (cpu.fetched as u16) << 1;

    cpu.clear_flags(StatusFlags::C as u8 | StatusFlags::N as u8 | StatusFlags::Z as u8);

    cpu.set_flag(StatusFlags::C, (value & 0xFF00) > 0);

    check_if_zero_or_negative_u16(cpu, value);

    let imp: Box<dyn Fn(&mut CPU) -> u8> = Box::new(imp);

    if std::ptr::eq(&*lookup_table[cpu.cur_opcode as usize].addr_mode, &*imp) {
        cpu.acu = (value & 0x00FF) as u8;
    } else {
        cpu.write(cpu.abs_addr, (value & 0x00FF) as u8)
    }
}

/// "AND" Memory with Accumulator<br>
/// Executes the equation A & M<br>
/// BIT sets the Z flag as though the value in the address tested were ANDed with the accumulator. The N and V flags are set to match bits 7 and 6 respectively in the value stored at the tested address.<br>
// How to calculate the most significant bits:
//     7 bit
// 7 6 5 4 3 2 1 0 (binary indexes)
// 1 0 0 0 0 0 0 0 (binary) = 0x80 (hexadecimal)
/// Uses the check_if_zero_or_negative_u16() function to trigger the Flags N (Negative) and Z (Zero)<br>
pub fn bit(cpu: &mut CPU) {
    cpu.fetch();

    let value = cpu.get_accumulator() & cpu.fetched;

    cpu.set_flag(StatusFlags::Z, (value & 0x00FF) != 0);
    cpu.set_flag(StatusFlags::V, (cpu.fetched & 0x40) != 0);
    cpu.set_flag(StatusFlags::N, (cpu.fetched & 0x80) != 0);
}

pub fn bcc(cpu: &mut CPU) {
    if cpu.get_flag(StatusFlags::C) == 0 {
        cpu.cycles += 1;

        cpu.abs_addr = cpu.program_counter + cpu.rel_addr;

        if (cpu.abs_addr & 0x00FF) != (cpu.program_counter & 0xFF00) {
            cpu.cycles += 1;
        }

        cpu.program_counter = cpu.abs_addr;
    }
}

pub fn bcs(cpu: &mut CPU) {
    if cpu.get_flag(StatusFlags::C) == 1 {
        cpu.cycles += 1;

        cpu.abs_addr = cpu.program_counter + cpu.rel_addr;

        if (cpu.abs_addr & 0x00FF) != (cpu.program_counter & 0xFF00) {
            cpu.cycles += 1;
        }

        cpu.program_counter = cpu.abs_addr;
    }
}

pub fn beq(cpu: &mut CPU) {
    if cpu.get_flag(StatusFlags::Z) == 1 {
        cpu.cycles += 1;

        cpu.abs_addr = cpu.program_counter + cpu.rel_addr;

        if (cpu.abs_addr & 0x00FF) != (cpu.program_counter & 0xFF00) {
            cpu.cycles += 1;
        }

        cpu.program_counter = cpu.abs_addr;
    }
}

pub fn bmi(cpu: &mut CPU) {
    if cpu.get_flag(StatusFlags::N) == 1 {
        cpu.cycles += 1;

        cpu.abs_addr = cpu.program_counter + cpu.rel_addr;

        if (cpu.abs_addr & 0x00FF) != (cpu.program_counter & 0xFF00) {
            cpu.cycles += 1;
        }

        cpu.program_counter = cpu.abs_addr;
    }
}

pub fn bne(cpu: &mut CPU) {
    if cpu.get_flag(StatusFlags::Z) == 0 {
        cpu.cycles += 1;

        cpu.abs_addr = cpu.program_counter + cpu.rel_addr;

        if (cpu.abs_addr & 0x00FF) != (cpu.program_counter & 0xFF00) {
            cpu.cycles += 1;
        }

        cpu.program_counter = cpu.abs_addr;
    }
}

pub fn bpl(cpu: &mut CPU) {
    if cpu.get_flag(StatusFlags::N) == 0 {
        cpu.cycles += 1;

        cpu.abs_addr = cpu.program_counter + cpu.rel_addr;

        if (cpu.abs_addr & 0x00FF) != (cpu.program_counter & 0xFF00) {
            cpu.cycles += 1;
        }

        cpu.program_counter = cpu.abs_addr;
    }
}

pub fn bvc(cpu: &mut CPU) {
    if cpu.get_flag(StatusFlags::V) == 0 {
        cpu.cycles += 1;

        cpu.abs_addr = cpu.program_counter + cpu.rel_addr;

        if (cpu.abs_addr & 0x00FF) != (cpu.program_counter & 0xFF00) {
            cpu.cycles += 1;
        }

        cpu.program_counter = cpu.abs_addr;
    }
}

pub fn bvs(cpu: &mut CPU) {
    if cpu.get_flag(StatusFlags::V) == 1 {
        cpu.cycles += 1;

        cpu.abs_addr = cpu.program_counter + cpu.rel_addr;

        if (cpu.abs_addr & 0x00FF) != (cpu.program_counter & 0xFF00) {
            cpu.cycles += 1;
        }

        cpu.program_counter = cpu.abs_addr;
    }
}

pub fn brk(cpu: &mut CPU) {
    cpu.program_counter += 1;

    cpu.set_flag(StatusFlags::I, true);

    cpu.write(
        cpu.get_stack_address(),
        ((cpu.program_counter >> 8) & 0x00FF) as u8,
    );
    cpu.stack_pointer -= 1;

    //Save the program counter low byte into the stack
    cpu.write(
        cpu.get_stack_address(),
        (cpu.program_counter & 0x00FF) as u8,
    );
    cpu.stack_pointer -= 1;

    cpu.set_flag(StatusFlags::B, true);

    cpu.write(cpu.get_stack_address(), cpu.status);
    cpu.stack_pointer -= 1;

    //The program counter is equal to the low_byte in the 0xFFFE RAM address and to the high_byte in the 0xFFFF RAM address
    let low_byte = cpu.read(0xFFFE) as u16;
    let high_byte = cpu.read(0xFFFF) as u16;

    //Execute the same thing to join two bytes into one opcocde/uint_16
    cpu.program_counter = (high_byte << 8) | low_byte;
}

pub fn clc(cpu: &mut CPU) {
    cpu.clear_flags(StatusFlags::C as u8)
}

pub fn cld(cpu: &mut CPU) {
    cpu.clear_flags(StatusFlags::D as u8)
}

pub fn cli(cpu: &mut CPU) {
    cpu.clear_flags(StatusFlags::I as u8)
}

pub fn clv(cpu: &mut CPU) {
    cpu.clear_flags(StatusFlags::V as u8)
}

pub fn cmp(cpu: &mut CPU) {
    cpu.fetch();

    let value = cpu.fetched as u16 - cpu.get_accumulator() as u16;

    cpu.set_flag(
        StatusFlags::C,
        cpu.get_accumulator() as u16 >= cpu.fetched as u16,
    );

    check_if_zero_or_negative_u16(cpu, value);
}

pub fn cpx(cpu: &mut CPU) {
    cpu.fetch();

    let value = cpu.fetched as u16 - cpu.get_register_x() as u16;

    cpu.set_flag(
        StatusFlags::C,
        cpu.get_accumulator() as u16 >= cpu.fetched as u16,
    );

    check_if_zero_or_negative_u16(cpu, value);
}

pub fn cpy(cpu: &mut CPU) {
    cpu.fetch();

    let value = cpu.fetched as u16 - cpu.get_register_y() as u16;

    cpu.set_flag(
        StatusFlags::C,
        cpu.get_accumulator() as u16 >= cpu.fetched as u16,
    );

    check_if_zero_or_negative_u16(cpu, value);
}

pub fn dec(cpu: &mut CPU) {
    cpu.fetch();

    let value = cpu.fetched - 1;

    cpu.write(cpu.abs_addr, value as u8);

    check_if_zero_or_negative_u16(cpu, value as u16)
}

pub fn dex(cpu: &mut CPU) {
    let value = cpu.get_register_x() - 1;

    cpu.regx = value;

    check_if_zero_or_negative_u8(cpu, value);
}

pub fn dey(cpu: &mut CPU) {
    let value = cpu.get_register_y() - 1;

    cpu.regy = value;

    check_if_zero_or_negative_u8(cpu, value);
}

pub fn eor(cpu: &mut CPU) {
    cpu.fetch();

    let value = cpu.get_accumulator() ^ cpu.fetched;

    cpu.acu = value as u8;

    check_if_zero_or_negative_u8(cpu, value);
}

pub fn inc(cpu: &mut CPU) {
    cpu.fetch();

    let value = cpu.fetched as u16 + 1;

    cpu.write(cpu.abs_addr, value as u8);

    check_if_zero_or_negative_u16(cpu, value);
}

pub fn inx(cpu: &mut CPU) {
    let value = cpu.get_register_x() + 1;

    cpu.regx = value;

    check_if_zero_or_negative_u8(cpu, value);
}

pub fn iny(cpu: &mut CPU) {
    let value = cpu.get_register_y() + 1;

    cpu.regy = value;

    check_if_zero_or_negative_u8(cpu, value);
}

pub fn jmp(cpu: &mut CPU) {
    cpu.program_counter = cpu.abs_addr;
}

pub fn jsr(cpu: &mut CPU) {
    cpu.program_counter -= 1;

    cpu.write(
        cpu.get_stack_address(),
        ((cpu.program_counter >> 8) & 0x00FF) as u8,
    );
    cpu.stack_pointer -= 1;

    //Save the program counter low byte into the stack
    cpu.write(
        cpu.get_stack_address(),
        (cpu.program_counter & 0x00FF) as u8,
    );
    cpu.stack_pointer -= 1;

    cpu.program_counter = cpu.abs_addr;
}

pub fn lda(cpu: &mut CPU) {
    cpu.fetch();

    cpu.acu = cpu.fetched;

    check_if_zero_or_negative_u8(cpu, cpu.get_accumulator());
}

pub fn ldx(cpu: &mut CPU) {
    cpu.fetch();

    cpu.regx = cpu.fetched;

    check_if_zero_or_negative_u8(cpu, cpu.get_register_x());
}

pub fn ldy(cpu: &mut CPU) {
    cpu.fetch();

    cpu.regy = cpu.fetched;

    check_if_zero_or_negative_u8(cpu, cpu.get_register_y());
}

pub fn lsr(cpu: &mut CPU) {
    cpu.fetch();

    cpu.set_flag(StatusFlags::C, (cpu.fetched & 0x0001) != 0);

    let value = cpu.fetched as u16 >> 1;

    check_if_zero_or_negative_u16(cpu, value);

    let imp: Box<dyn Fn(&mut CPU) -> u8> = Box::new(imp);

    if std::ptr::eq(&*lookup_table[cpu.cur_opcode as usize].addr_mode, &*imp) {
        cpu.acu = (value & 0x00FF) as u8;
    } else {
        cpu.write(cpu.abs_addr, (value & 0x00FF) as u8)
    }
}

pub fn nop(cpu: &mut CPU) {
    match cpu.cur_opcode {
        0x1C | 0x3C | 0x5C | 0x7C | 0xDC | 0xFC => { //1
        }
        _ => { //0
        }
    }
}



//Extra Functions
///Checks if the value equals to zero or if the value (AND) the most significant bit on an 8-bit value (0x80)
// How to calculate the most significant bits:
//     7 bit
// 7 6 5 4 3 2 1 0 (binary indexes)
// 1 0 0 0 0 0 0 0 (binary) = 0x80 (hexadecimal)
pub fn check_if_zero_or_negative_u16(cpu: &mut CPU, value: u16) {
    if (value & 0x00FF) == 0 {
        cpu.set_flag(StatusFlags::Z, true)
    } else if (value & 0x00FF) & 0x0080 != 0 {
        cpu.set_flag(StatusFlags::N, true)
    }
}

pub fn check_if_zero_or_negative_u8(cpu: &mut CPU, value: u8) {
    if value == 0 {
        cpu.set_flag(StatusFlags::Z, true)
    } else if (value & 0x80) != 0 {
        cpu.set_flag(StatusFlags::N, true)
    }
}
