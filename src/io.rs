use std::{
    collections::HashMap,
    env, fs,
    io::{stdin, stdout, BufRead, Write},
};

use rand::prelude::*;

use crate::{compile::Assembly, value::Value, vm::Env, RuntimeError, RuntimeResult};

#[allow(unused_variables)]
pub trait IoBackend {
    fn print_str(&mut self, s: &str);
    fn rand(&mut self) -> f64;
    fn scan_line(&mut self) -> String {
        String::new()
    }
    fn import(&mut self, name: &str, env: &Env) -> RuntimeResult<Vec<Value>> {
        Err(env.error("Import not supported in this environment"))
    }
    fn var(&mut self, name: &str) -> Option<String> {
        None
    }
    fn args(&mut self) -> Vec<String> {
        Vec::new()
    }
    fn file_exists(&self, path: &str) -> bool {
        false
    }
    fn list_dir(&self, path: &str, env: &Env) -> RuntimeResult<Vec<String>> {
        Err(env.error("File IO not supported in this environment"))
    }
    fn is_file(&self, path: &str, env: &Env) -> RuntimeResult<bool> {
        Err(env.error("File IO not supported in this environment"))
    }
    fn read_file(&mut self, path: &str, env: &Env) -> RuntimeResult<Vec<u8>> {
        Err(env.error("File IO not supported in this environment"))
    }
    fn write_file(&mut self, path: &str, contents: Vec<u8>, env: &Env) -> RuntimeResult {
        Err(env.error("File IO not supported in this environment"))
    }
}

impl<'a, T> IoBackend for &'a mut T
where
    T: IoBackend,
{
    fn print_str(&mut self, s: &str) {
        (**self).print_str(s)
    }
    fn rand(&mut self) -> f64 {
        (**self).rand()
    }
    fn scan_line(&mut self) -> String {
        (**self).scan_line()
    }
    fn import(&mut self, name: &str, env: &Env) -> RuntimeResult<Vec<Value>> {
        (**self).import(name, env)
    }
    fn var(&mut self, name: &str) -> Option<String> {
        (**self).var(name)
    }
    fn args(&mut self) -> Vec<String> {
        (**self).args()
    }
    fn file_exists(&self, path: &str) -> bool {
        (**self).file_exists(path)
    }
    fn list_dir(&self, path: &str, env: &Env) -> RuntimeResult<Vec<String>> {
        (**self).list_dir(path, env)
    }
    fn is_file(&self, path: &str, env: &Env) -> RuntimeResult<bool> {
        (**self).is_file(path, env)
    }
    fn read_file(&mut self, path: &str, env: &Env) -> RuntimeResult<Vec<u8>> {
        (**self).read_file(path, env)
    }
    fn write_file(&mut self, path: &str, contents: Vec<u8>, env: &Env) -> RuntimeResult {
        (**self).write_file(path, contents, env)
    }
}

pub struct StdIo {
    imports: HashMap<String, Vec<Value>>,
    rng: SmallRng,
}

impl Default for StdIo {
    fn default() -> Self {
        Self {
            imports: HashMap::new(),
            rng: SmallRng::seed_from_u64(instant::now().to_bits()),
        }
    }
}

impl IoBackend for StdIo {
    fn print_str(&mut self, s: &str) {
        print!("{}", s);
        let _ = stdout().lock().flush();
    }
    fn rand(&mut self) -> f64 {
        self.rng.gen()
    }
    fn scan_line(&mut self) -> String {
        stdin()
            .lock()
            .lines()
            .next()
            .and_then(Result::ok)
            .unwrap_or_default()
    }
    fn import(&mut self, path: &str, _env: &Env) -> RuntimeResult<Vec<Value>> {
        if !self.imports.contains_key(path) {
            let (stack, _) = Assembly::load_file(path)
                .map_err(RuntimeError::Import)?
                .run_with_backend(&mut *self)
                .map_err(RuntimeError::Import)?;
            self.imports.insert(path.into(), stack);
        }
        Ok(self.imports[path].clone())
    }
    fn var(&mut self, name: &str) -> Option<String> {
        env::var(name).ok()
    }
    fn args(&mut self) -> Vec<String> {
        env::args().collect()
    }
    fn file_exists(&self, path: &str) -> bool {
        fs::metadata(path).is_ok()
    }
    fn is_file(&self, path: &str, env: &Env) -> RuntimeResult<bool> {
        fs::metadata(path)
            .map(|m| m.is_file())
            .map_err(|e| env.error(e.to_string()))
    }
    fn list_dir(&self, path: &str, env: &Env) -> RuntimeResult<Vec<String>> {
        let mut paths = Vec::new();
        for entry in fs::read_dir(path).map_err(|e| env.error(e.to_string()))? {
            let entry = entry.map_err(|e| env.error(e.to_string()))?;
            paths.push(entry.path().to_string_lossy().into());
        }
        Ok(paths)
    }
    fn read_file(&mut self, path: &str, env: &Env) -> RuntimeResult<Vec<u8>> {
        fs::read(path).map_err(|e| env.error(e.to_string()))
    }
    fn write_file(&mut self, path: &str, contents: Vec<u8>, env: &Env) -> RuntimeResult {
        fs::write(path, contents).map_err(|e| env.error(e.to_string()))
    }
}
