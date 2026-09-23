use std::cell::Cell;

#[link(wasm_import_module = "arcadium")]
unsafe extern "C" {
    fn screen_width() -> i32;
    fn screen_height() -> i32;
    fn draw_char(x: i32, y: i32, character: i32);
    fn key_pressed(key: i32) -> i32;
}

const LEFT: i32 = 2;
const RIGHT: i32 = 3;

thread_local! {
    static X: Cell<f32> = const { Cell::new(0.0) };
}

#[unsafe(no_mangle)]
pub extern "C" fn arcadium_init() -> i32 {
    X.set(unsafe { screen_width() } as f32 / 2.0);
    1
}

#[unsafe(no_mangle)]
pub extern "C" fn arcadium_update(delta_seconds: f32) {
    // Imports are provided by Arcadium; the guest never accesses host memory.
    unsafe {
        let width = screen_width();
        let height = screen_height();
        if width <= 0 || height <= 0 {
            return;
        }
        let direction = key_pressed(RIGHT) - key_pressed(LEFT);
        let x = (X.get() + direction as f32 * 60.0 * delta_seconds).clamp(0.0, (width - 1) as f32);
        X.set(x);
        draw_char(x.round() as i32, height / 2, '@' as i32);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn arcadium_shutdown() {
    X.set(0.0);
}
