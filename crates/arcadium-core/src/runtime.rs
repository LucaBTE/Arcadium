use std::{
    error::Error,
    fs::File,
    io::{self, Read},
};

use wasmtime::{Engine, Instance, Module, Store};

use zip::ZipArchive;

use crate::installed_game::InstalledGame;

const INIT_EXPORT: &str = "arcadium_init";

pub fn initialize_game(game: &InstalledGame) -> Result<i32, Box<dyn Error>> {
    let wasm_bytes = read_wasm_from_package(game)?;

    let engine = Engine::default();

    let module = Module::from_binary(&engine, &wasm_bytes)?;

    let mut store = Store::new(&engine, ());

    let instance = Instance::new(&mut store, &module, &[])?;

    let init = instance
        .get_typed_func::<(), i32>(&mut store, INIT_EXPORT)
        .map_err(|_| {
            invalid_data(format!(
                "WASM module does not export required function '{}'",
                INIT_EXPORT
            ))
        })?;

    let result = init.call(&mut store, ())?;

    Ok(result)
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
