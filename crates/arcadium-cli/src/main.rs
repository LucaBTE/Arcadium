//redirect the call to the core of the app (receives 'cargo run')
fn main() -> std::io::Result<()> {
    arcadium_core::run()
}
