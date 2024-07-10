use std::{borrow::Borrow, cell::RefCell, rc::Weak};

use crate::{bus::BUS, opcode::{lookup_table, imp}};

pub struct CPU {
    //CPU Registers
    pub regx: u8,             //X REGISTER
    pub regy: u8,             //Y REGISTER
    pub acu: u8,              //ACCUMULATOR REGISTER
    pub stack_pointer: u8,    //STACK POINTER
    pub program_counter: u16, //PROGRAM COUNTER
    pub status: u8,           //STATUS REGISTER

    //Assist Variables
    pub fetched: u8,
    pub abs_addr: u16,
    pub rel_addr: u16,
    pub temp_op: u16,
    pub cur_opcode: u8,
    pub cycles: u8,
    pub clock_count: u32,

    bus: Option<Weak<RefCell<BUS>>>,
}

//CPU Status Flags
pub enum StatusFlags {
    C = 1 << 0, //Carry
    Z = 1 << 1, //Zero
    I = 1 << 2, //Interrupt Disable
    D = 1 << 3, //Decimal

    B = 1 << 4, //Break
    G = 1 << 5, //Unsued Flag: GFlag
    V = 1 << 6, //Overflow
    N = 1 << 7, //Negative
}

impl CPU {
    //Constructor
    pub fn new() -> Self {
        Self {
            regx: 0,
            regy: 0,
            acu: 0,
            stack_pointer: 0,
            program_counter: 0,
            status: 0,

            fetched: 0,
            abs_addr: 0,
            rel_addr: 0,
            temp_op: 0,
            cur_opcode: 0,
            cycles: 0,
            clock_count: 0,

            bus: None,
        }
    }

    pub fn read(&self, address: u16) -> u8 {
        if let Some(bus) = &self.bus {
            if let Some(bus) = bus.upgrade() {
                return bus.as_ref().borrow().read(address);
            }
        }

        0
    }

    pub fn write(&mut self, address: u16, data: u8) {
        if let Some(bus) = &mut self.bus {
            if let Some(bus) = bus.upgrade() {
                bus.borrow_mut().write(address, data);
            }
        }
    }

    pub fn get_stack_address(&self) -> u16 {
        0x0100 as u16 + self.stack_pointer as u16
    }

    pub fn get_accumulator(&self) -> u8 {
        return self.acu;
    }

    pub fn get_register_x(&self) -> u8 {
        return self.regx;
    }

    pub fn get_register_y(&self) -> u8 {
        return self.regy;
    }

    pub fn get_program_counter(&self) -> u16 {
        return self.program_counter;
    }

    //Interface Signals

    ///Executes every update but will only trigger when the cycles are off
    pub fn clock(&mut self) {
        if self.cycles == 0 {
            self.cur_opcode = self.read(self.program_counter);

            self.set_flag(StatusFlags::G, true);

            self.program_counter += 1;

            self.cycles = lookup_table[self.cur_opcode as usize].cycles;

            let addr_mode_cycles = (lookup_table[self.cur_opcode as usize].addr_mode)(self);

            let operate_cycles = (lookup_table[self.cur_opcode as usize].operate)(self);

            self.cycles += addr_mode_cycles & operate_cycles;

            self.set_flag(StatusFlags::G, true);
        }

        self.clock_count += 1;
        self.cycles -= 1
    }

    ///The interrupt will execute when the "disable interrupt" (I status Flag) is off
    pub fn interrupt_request(&mut self) {
        if self.get_flag(StatusFlags::I) == 0 {
            //Save the program counter high byte into the stack
            self.write(
                self.get_stack_address(),
                ((self.program_counter >> 8) & 0x00FF) as u8,
            );
            self.stack_pointer -= 1;

            //Save the program counter low byte into the stack
            self.write(
                self.get_stack_address(),
                (self.program_counter & 0x00FF) as u8,
            );
            self.stack_pointer -= 1;

            //Sets the status register into the stack
            self.set_flag(StatusFlags::B, false);
            self.set_flag(StatusFlags::G, true);
            self.set_flag(StatusFlags::I, true);

            self.write(self.get_stack_address(), self.status);
            self.stack_pointer -= 1;

            //The program counter is equal to the low_byte in the 0xFFFE RAM address and to the high_byte in the 0xFFFF RAM address
            let low_byte = self.read(0xFFFE) as u16;
            let high_byte = self.read(0xFFFF) as u16;

            //Execute the same thing to join two bytes into one opcocde/uint_16
            self.program_counter = (high_byte << 8) | low_byte;

            self.cycles = 7;
        }
    }

    ///The non maskable input can't be ignored in contrary to the interrput request but they do the same thing execept
    ///for the program address is 0xFFFA for low byte and 0xFFFB for high byte
    pub fn non_maskable_input(&mut self) {
        //Save the program counter high byte into the stack
        self.write(
            self.get_stack_address(),
            ((self.program_counter >> 8) & 0x00FF) as u8,
        );
        self.stack_pointer -= 1;

        //Save the program counter low byte into the stack
        self.write(
            self.get_stack_address(),
            (self.program_counter & 0x00FF) as u8,
        );
        self.stack_pointer -= 1;

        //Sets the status register into the stack
        self.set_flag(StatusFlags::B, false);
        self.set_flag(StatusFlags::G, true);
        self.set_flag(StatusFlags::I, true);

        self.write(self.get_stack_address(), self.status);
        self.stack_pointer -= 1;

        //The program counter is equal to the low_byte in the 0xFFFA RAM address and to the high_byte in the 0xFFFB RAM address
        let low_byte = self.read(0xFFFA) as u16;
        let high_byte = self.read(0xFFFB) as u16;

        //Execute the same thing to join two bytes into one opcocde/uint_16
        self.program_counter = (high_byte << 8) | low_byte;

        self.cycles = 8;
    }

    ///Resets the registers and pointers and status and sets the program counter to the low_byte in the 0xFFFC RAM address and to the high_byte in the 0xFFFD RAM address 
    pub fn reset(&mut self) {
        self.status = 0x00 | StatusFlags::G as u8;

        self.stack_pointer = 0xFD;
        self.regx = 0;
        self.regy = 0;
        self.acu = 0;

        self.abs_addr = 0x0000;
        self.rel_addr = 0x0000;
        self.fetched = 0x00;
        
        //The program counter is equal to the low_byte in the 0xFFFC RAM address and to the high_byte in the 0xFFFD RAM address
        let low_byte = self.read(0xFFFC) as u16;
        let high_byte = self.read(0xFFFd) as u16;

        //Execute the same thing to join two bytes into one opcocde/uint_16
        self.program_counter = (high_byte << 8) | low_byte;

        self.cycles = 8;
    }

    pub fn complete(&self) -> bool{
        return self.cycles == 0;
    }

    pub fn connect_bus(&mut self, bus: Weak<RefCell<BUS>>) {
        self.bus = Some(bus)
    }

    //Set/Get Status Flags
    pub fn get_flag(&self, flag: StatusFlags) -> u8 {
        let bit = flag as u8;

        if (self.status & bit) > 0 {
            return 1;
        } else {
            return 0;
        }
    }

    pub fn set_flag(&mut self, flag: StatusFlags, enable: bool) {
        let bit = flag as u8;
        if enable {
            self.status |= bit;
        } else {
            self.status &= !bit;
        }
    }

    pub fn clear_flags(&mut self,flags:u8) {
        self.status &= !flags;
    }

    pub fn fetch(&mut self) {
        let imp: Box<dyn Fn(&mut CPU) -> u8> = Box::new(imp);
        if std::ptr::eq(&*lookup_table[self.cur_opcode as usize].addr_mode,&*imp) {
            self.fetched = self.read(self.abs_addr)
        }
    } 
}
