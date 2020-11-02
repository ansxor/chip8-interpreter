mod chip8;

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window(
            "CHIP-8 Interpreter",
            chip8::WINDOW_X_SIZE as u32,
            chip8::WINDOW_Y_SIZE as u32,
        )
        .position_centered()
        .build()
        .unwrap();
    let mut canvas = window.into_canvas().build().unwrap();

    let mut prg: chip8::Program = Default::default();
    prg.set_ins(0, 0x6001);
    prg.set_ins(2, 0x6102);
    prg.set_ins(4, 0x801E);
    prg.set_ins(6, 0xF029);
    prg.set_ins(8, 0xDFF5);
    prg.set_ins(10, 0x6E10);
    prg.set_ins(12, 0xF129);
    prg.set_ins(14, 0xDEE5);
    prg.set_ins(16, 0x1210);

    canvas.set_draw_color(sdl2::pixels::Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.present();
    let mut event_pump = sdl_context.event_pump().unwrap();

    'running: loop {
        prg.run_cycle();
        prg.draw_output(&mut canvas);
        for event in event_pump.poll_iter() {
            match event {
                sdl2::event::Event::Quit { .. }
                | sdl2::event::Event::KeyDown {
                    keycode: Some(sdl2::keyboard::Keycode::Escape),
                    ..
                } => break 'running,
                _ => {}
            }
        }
        // The rest of the game loop goes here...

        canvas.present();
        // ::std::thread::sleep(sdl2::timer::Duration::new(0, 1_000_000_000u32 / 60));
    }
}
