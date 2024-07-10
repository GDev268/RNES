use bus::BUS;
use cpu::CPU;

mod bus;
mod cpu;
mod opcode;

fn main() {
    let mut bus = BUS::new();

    let address: u16 = 0x1234;

    println!("Hello, world! {:?}", address >= 0x4232);
}
