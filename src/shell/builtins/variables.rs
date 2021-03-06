use std::rc::Rc;

use phf::*;
use rand::Rng;

use crate::shell::{current_dir_str, value::Value, Shell};

type BulitinVar = fn(&mut Shell) -> Value;

static BUILTIN_VARS: phf::Map<&'static str, BulitinVar> = phf_map! {
    "pid" => pid,
    "home" => home,
    "user" => user,
    "hostname" => hostname,
    "os" => os,
    "family" => family,
    "arch" => arch,
    "distro" => distro,
    "desktop" => desktop,
    "status" => status,
    "pwd" => pwd,
    "version" => version,
    "random" => random,
    "lines" => lines,
    "columns" => columns,
    "null" => null,
    "config" => config,
    "args" => args,
    "pi" => pi,
};

pub fn get_var(shell: &mut Shell, name: &str) -> Option<Value> {
    BUILTIN_VARS.get(name).map(|var| var(shell))
}

pub fn is_builtin(name: &str) -> bool {
    BUILTIN_VARS.contains_key(name)
}

pub fn args(shell: &mut Shell) -> Value {
    Value::List(Rc::new(
        shell
            .args
            .iter()
            .map(|s| Value::String(Rc::new(s.to_string())))
            .collect(),
    ))
}

pub fn config(shell: &mut Shell) -> Value {
    Value::String(Rc::new(shell.config_path().to_string_lossy().to_string()))
}

pub fn null(_: &mut Shell) -> Value {
    Value::Null
}

pub fn user(_: &mut Shell) -> Value {
    Value::String(Rc::new(whoami::username()))
}

pub fn pid(_: &mut Shell) -> Value {
    Value::Int(std::process::id() as i64)
}

pub fn hostname(_: &mut Shell) -> Value {
    Value::String(Rc::new(whoami::devicename()))
}

pub fn home(shell: &mut Shell) -> Value {
    Value::String(Rc::new(shell.home_dir().to_string_lossy().to_string()))
}

pub fn os(_: &mut Shell) -> Value {
    Value::String(Rc::new(std::env::consts::OS.to_string()))
}

pub fn arch(_: &mut Shell) -> Value {
    Value::String(Rc::new(std::env::consts::ARCH.to_string()))
}

pub fn distro(_: &mut Shell) -> Value {
    Value::String(Rc::new(whoami::distro()))
}

pub fn desktop(_: &mut Shell) -> Value {
    Value::String(Rc::new(whoami::desktop_env().to_string()))
}

pub fn status(shell: &mut Shell) -> Value {
    Value::Int(shell.exit_status)
}

pub fn pwd(_: &mut Shell) -> Value {
    Value::String(Rc::new(current_dir_str()))
}

pub fn version(_: &mut Shell) -> Value {
    Value::String(Rc::new(env!("CARGO_PKG_VERSION").to_string()))
}

pub fn family(_: &mut Shell) -> Value {
    Value::String(Rc::new(std::env::consts::FAMILY.to_string()))
}

pub fn random(_: &mut Shell) -> Value {
    let mut rng = rand::thread_rng();
    Value::Int(rng.gen_range(0..i64::MAX))
}

pub fn lines(_: &mut Shell) -> Value {
    let (_, h) = crossterm::terminal::size().unwrap();
    Value::Int(h as i64)
}

pub fn columns(_: &mut Shell) -> Value {
    let (w, _) = crossterm::terminal::size().unwrap();
    Value::Int(w as i64)
}

pub fn pi(_: &mut Shell) -> Value {
    Value::Float(std::f64::consts::PI)
}
