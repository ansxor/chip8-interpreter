mod chip8;

fn main() {
    let mut prg: chip8::Program = Default::default();
    prg.set_ins(0, 0x6001);
    prg.set_ins(1, 0x6102);
    prg.set_ins(2, 0x801E);
    prg.set_ins(3, 0xF029);
    prg.set_ins(4, 0xDFF5);
    prg.set_ins(5, 0x6E10);
    prg.set_ins(6, 0xF129);
    prg.set_ins(7, 0xDEE5);
    prg.set_ins(8, 0x1210);
    for i in 0..8 {
        prg.run_cycle();
    }

    println!("Hello, world!");
}
