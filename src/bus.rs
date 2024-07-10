use std::{cell::RefCell, rc::Rc};

use crate::cpu::CPU;

pub(crate) struct BUS {
    cpu: Rc<RefCell<CPU>>,
    ram:[u8;2048]
}

impl BUS {
    pub fn new() -> Rc<RefCell<Self>> {
        let mut bus = Rc::new(RefCell::new(BUS{
            cpu: Rc::new(RefCell::new(CPU::new())),
            ram: [Default::default();2048]
        }));

        bus.borrow_mut().cpu.borrow_mut().connect_bus(Rc::downgrade(&bus));

        bus
    }

    pub fn write(&mut self,address:u16,data:u8) {
        self.ram[address as usize] = data;
    }

    pub fn read(&self,address:u16) -> u8 {
        self.ram[address as usize]
    }

}