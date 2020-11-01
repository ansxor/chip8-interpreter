mod chip8;

fn main() {
    let mut prg: chip8::Program = Default::default();
    prg.run_cycle();

    println!("Hello, world!");
}
