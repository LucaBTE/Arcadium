use std::{
    error::Error,
    fs::File,
    io::{self, Read},
};

use wasmtime::{Caller, Engine, Linker, Module, Store, TypedFunc};

use zip::ZipArchive;

use crate::{host::HostState, installed_game::InstalledGame, score_store::ScoreStore};

const INIT_EXPORT: &str = "arcadium_init";
const UPDATE_EXPORT: &str = "arcadium_update";
const SHUTDOWN_EXPORT: &str = "arcadium_shutdown";

pub struct GameRuntime {
    store: Store<HostState>,

    init: TypedFunc<(), i32>,
    update: TypedFunc<f32, ()>,
    shutdown: TypedFunc<(), ()>,
}

impl GameRuntime {
    pub fn load(game: &InstalledGame) -> Result<Self, Box<dyn Error>> {
        let wasm_bytes = read_wasm_from_package(game)?;

        let engine = Engine::default();

        let module = Module::from_binary(&engine, &wasm_bytes)?;

        let mut runtime = Self::instantiate(&engine, &module)?;
        runtime
            .store
            .data_mut()
            .set_score_store(ScoreStore::for_game(&game.metadata.id));
        Ok(runtime)
    }

    fn instantiate(engine: &Engine, module: &Module) -> Result<Self, Box<dyn Error>> {
        let mut store = Store::new(engine, HostState::default());
        let mut linker = Linker::new(engine);
        linker.func_wrap(
            "arcadium",
            "screen_width",
            |caller: Caller<'_, HostState>| i32::from(caller.data().width()),
        )?;
        linker.func_wrap(
            "arcadium",
            "screen_height",
            |caller: Caller<'_, HostState>| i32::from(caller.data().height()),
        )?;
        linker.func_wrap(
            "arcadium",
            "draw_char",
            |mut caller: Caller<'_, HostState>, x: i32, y: i32, character: i32| {
                caller.data_mut().draw_char(x, y, character);
            },
        )?;
        linker.func_wrap(
            "arcadium",
            "draw_cell",
            |mut caller: Caller<'_, HostState>,
             x: i32,
             y: i32,
             character: i32,
             foreground: i32,
             background: i32| {
                caller
                    .data_mut()
                    .draw_cell(x, y, character, foreground, background);
            },
        )?;
        linker.func_wrap(
            "arcadium",
            "key_pressed",
            |caller: Caller<'_, HostState>, key: i32| i32::from(caller.data().key_pressed(key)),
        )?;
        linker.func_wrap("arcadium", "load_score", |caller: Caller<'_, HostState>| {
            caller.data().load_score()
        })?;
        linker.func_wrap(
            "arcadium",
            "save_score",
            |caller: Caller<'_, HostState>, score: i64| i32::from(caller.data().save_score(score)),
        )?;
        linker.func_wrap(
            "arcadium",
            "request_exit",
            |mut caller: Caller<'_, HostState>| caller.data_mut().request_exit(),
        )?;
        let instance = linker.instantiate(&mut store, module)?;
        let init = instance
            .get_typed_func::<(), i32>(&mut store, INIT_EXPORT)
            .map_err(|error| invalid_data(format!("Expected arcadium_init() -> i32: {error}")))?;
        let update = instance
            .get_typed_func::<f32, ()>(&mut store, UPDATE_EXPORT)
            .map_err(|error| {
                invalid_data(format!("Expected arcadium_update(f32) -> (): {error}"))
            })?;
        let shutdown = instance
            .get_typed_func::<(), ()>(&mut store, SHUTDOWN_EXPORT)
            .map_err(|error| {
                invalid_data(format!("Expected arcadium_shutdown() -> (): {error}"))
            })?;

        Ok(Self {
            store,
            init,
            update,
            shutdown,
        })
    }

    pub fn resize(&mut self, width: u16, height: u16) {
        self.store.data_mut().resize(width, height);
    }

    pub fn begin_frame(&mut self) {
        self.store.data_mut().begin_frame();
    }

    pub fn press_key(&mut self, key: i32) {
        self.store.data_mut().press_key(key);
    }

    pub fn screen(&self) -> &HostState {
        self.store.data()
    }

    pub fn exit_requested(&self) -> bool {
        self.store.data().exit_requested()
    }

    pub fn init(&mut self) -> Result<i32, Box<dyn Error>> {
        let result = self.init.call(&mut self.store, ())?;

        Ok(result)
    }

    pub fn update(&mut self, delta_seconds: f32) -> Result<(), Box<dyn Error>> {
        self.update.call(&mut self.store, delta_seconds)?;

        Ok(())
    }

    pub fn shutdown(&mut self) -> Result<(), Box<dyn Error>> {
        self.shutdown.call(&mut self.store, ())?;

        Ok(())
    }
}

fn read_wasm_from_package(game: &InstalledGame) -> Result<Vec<u8>, Box<dyn Error>> {
    let file = File::open(&game.source_path)?;

    let mut archive = ZipArchive::new(file)?;

    let mut wasm_file = archive.by_name(&game.entry).map_err(|_| {
        invalid_data(format!(
            "ADM entry '{}' was not found in package",
            game.entry
        ))
    })?;

    let mut wasm_bytes = Vec::new();

    wasm_file.read_to_end(&mut wasm_bytes)?;

    if wasm_bytes.is_empty() {
        return Err(invalid_data("WASM entry is empty").into());
    }

    Ok(wasm_bytes)
}

fn invalid_data(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message.into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::host::key;

    fn runtime(wat: &str) -> Result<GameRuntime, Box<dyn Error>> {
        let engine = Engine::default();
        GameRuntime::instantiate(&engine, &Module::new(&engine, wat)?)
    }

    #[test]
    fn imports_delta_persistent_state_and_shutdown() -> Result<(), Box<dyn Error>> {
        let mut game = runtime(
            r#"(module
            (import "arcadium" "screen_width" (func $width (result i32)))
            (import "arcadium" "screen_height" (func $height (result i32)))
            (import "arcadium" "draw_char" (func $draw (param i32 i32 i32)))
            (import "arcadium" "key_pressed" (func $key (param i32) (result i32)))
            (global $x (mut i32) (i32.const 0))
            (func (export "arcadium_init") (result i32)
                (global.set $x (i32.div_u (call $width) (i32.const 2)))
                (i32.const 1))
            (func (export "arcadium_update") (param $dt f32)
                (if (f32.ne (local.get $dt) (f32.const 0.25)) (then unreachable))
                (global.set $x (i32.add (global.get $x)
                    (i32.sub (call $key (i32.const 3)) (call $key (i32.const 2)))))
                (call $draw (global.get $x) (i32.div_u (call $height) (i32.const 2)) (i32.const 64)))
            (func (export "arcadium_shutdown")
                (call $draw (i32.const 0) (i32.const 0) (i32.const 88))))"#,
        )?;
        game.resize(10, 6);
        assert_eq!(game.init()?, 1);
        game.begin_frame();
        game.press_key(key::LEFT);
        game.update(0.25)?;
        assert_eq!(game.screen().screen()[34].character, '@');
        game.begin_frame();
        game.update(0.25)?;
        assert_eq!(game.screen().screen()[34].character, '@');
        game.resize(12, 8);
        game.begin_frame();
        game.press_key(key::RIGHT);
        game.update(0.25)?;
        assert_eq!(game.screen().screen()[53].character, '@');
        game.shutdown()?;
        assert_eq!(game.screen().screen()[0].character, 'X');
        Ok(())
    }

    #[test]
    fn colored_import_reaches_framebuffer() -> Result<(), Box<dyn Error>> {
        let mut game = runtime(
            r#"(module
            (import "arcadium" "draw_cell" (func $draw (param i32 i32 i32 i32 i32)))
            (func (export "arcadium_init") (result i32) i32.const 1)
            (func (export "arcadium_update") (param f32)
                (call $draw (i32.const 1) (i32.const 0) (i32.const 9608)
                    (i32.const 0x22d3ee) (i32.const 0x0b1020)))
            (func (export "arcadium_shutdown")))"#,
        )?;
        game.resize(2, 1);
        game.init()?;
        game.begin_frame();
        game.update(0.016)?;
        let cell = game.screen().screen()[1];
        assert_eq!(cell.character, '█');
        assert_eq!(
            cell.foreground,
            ratatui::style::Color::Rgb(0x22, 0xd3, 0xee)
        );
        assert_eq!(
            cell.background,
            ratatui::style::Color::Rgb(0x0b, 0x10, 0x20)
        );
        Ok(())
    }

    #[test]
    fn score_and_exit_imports_are_available() -> Result<(), Box<dyn Error>> {
        let path = std::env::temp_dir().join(format!("arcadium-abi-score-{}", std::process::id()));
        let _ = std::fs::remove_file(&path);
        let mut game = runtime(
            r#"(module
            (import "arcadium" "load_score" (func $load (result i64)))
            (import "arcadium" "save_score" (func $save (param i64) (result i32)))
            (import "arcadium" "request_exit" (func $exit))
            (func (export "arcadium_init") (result i32) i32.const 1)
            (func (export "arcadium_update") (param f32)
                (if (i64.ne (call $load) (i64.const 0)) (then unreachable))
                (if (i32.ne (call $save (i64.const 5)) (i32.const 1)) (then unreachable))
                (call $exit))
            (func (export "arcadium_shutdown")))"#,
        )?;
        game.store
            .data_mut()
            .set_score_store(ScoreStore::at_path(path.clone()));
        game.init()?;
        game.update(0.0)?;
        assert!(game.exit_requested());
        assert_eq!(ScoreStore::at_path(path.clone()).load(), 5);
        game.shutdown()?;
        std::fs::remove_file(path)?;
        Ok(())
    }

    #[test]
    fn incompatible_exports_and_traps_return_errors() -> Result<(), Box<dyn Error>> {
        assert!(runtime("(module)").is_err());
        let old = r#"(module
            (func (export "arcadium_init") (result i32) i32.const 1)
            (func (export "arcadium_update"))
            (func (export "arcadium_shutdown")))"#;
        let error = runtime(old).err().ok_or("old ABI was accepted")?;
        assert!(error.to_string().contains("arcadium_update(f32)"));
        let mut game = runtime(&old.replace(
            "(func (export \"arcadium_update\"))",
            "(func (export \"arcadium_update\") (param f32) unreachable)",
        ))?;
        assert!(game.update(0.1).is_err());
        assert!(Module::from_binary(&Engine::default(), b"invalid wasm").is_err());
        Ok(())
    }
}
