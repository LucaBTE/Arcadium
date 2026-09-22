use std::{
    error::Error,
    fs::File,
    io::{self, Read},
};

use wasmtime::{Engine, Instance, Module, Store, TypedFunc};

use zip::ZipArchive;

use crate::installed_game::InstalledGame;

const INIT_EXPORT: &str = "arcadium_init";
const UPDATE_EXPORT: &str = "arcadium_update";
const SHUTDOWN_EXPORT: &str = "arcadium_shutdown";

pub struct GameRuntime {
    store: Store<()>,

    init: TypedFunc<(), i32>,
    update: TypedFunc<(), ()>,
    shutdown: TypedFunc<(), ()>,
}

impl GameRuntime {
    pub fn load(game: &InstalledGame) -> Result<Self, Box<dyn Error>> {
        let wasm_bytes = read_wasm_from_package(game)?;

        let engine = Engine::default();

        let module = Module::from_binary(&engine, &wasm_bytes)?;

        let mut store = Store::new(&engine, ());

        let instance = Instance::new(&mut store, &module, &[])?;

        let init = get_init_function(&instance, &mut store)?;

        let update = get_void_function(&instance, &mut store, UPDATE_EXPORT)?;

        let shutdown = get_void_function(&instance, &mut store, SHUTDOWN_EXPORT)?;

        Ok(Self {
            store,
            init,
            update,
            shutdown,
        })
    }

    pub fn init(&mut self) -> Result<i32, Box<dyn Error>> {
        let result = self.init.call(&mut self.store, ())?;

        Ok(result)
    }

    pub fn update(&mut self) -> Result<(), Box<dyn Error>> {
        self.update.call(&mut self.store, ())?;

        Ok(())
    }

    pub fn shutdown(&mut self) -> Result<(), Box<dyn Error>> {
        self.shutdown.call(&mut self.store, ())?;

        Ok(())
    }
}

fn get_init_function(
    instance: &Instance,
    store: &mut Store<()>,
) -> Result<TypedFunc<(), i32>, Box<dyn Error>> {
    instance
        .get_typed_func::<(), i32>(store, INIT_EXPORT)
        .map_err(|_| {
            invalid_data(format!(
                "WASM module does not export required function '{}'",
                INIT_EXPORT
            ))
            .into()
        })
}

fn get_void_function(
    instance: &Instance,
    store: &mut Store<()>,
    name: &str,
) -> Result<TypedFunc<(), ()>, Box<dyn Error>> {
    instance.get_typed_func::<(), ()>(store, name).map_err(|_| {
        invalid_data(format!(
            "WASM module does not export required function '{}'",
            name
        ))
        .into()
    })
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
