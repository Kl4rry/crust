use std::{rc::Rc, time, time::Duration};

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
    "tau" => tau,
    "e" => e,
    "epoch" => epoch,
    "path_sep" => path_sep,
    "interactive" => interactive,
};

#[inline(always)]
pub fn get_var(shell: &mut Shell, name: &str) -> Option<Value> {
    BUILTIN_VARS.get(name).map(|var| var(shell))
}

pub fn is_builtin(name: &str) -> bool {
    BUILTIN_VARS.contains_key(name)
}

pub fn args(shell: &mut Shell) -> Value {
    Value::from(
        shell
            .args
            .iter()
            .map(|s| Value::String(Rc::new(s.to_string())))
            .collect::<Vec<_>>(),
    )
}

pub fn config(shell: &mut Shell) -> Value {
    Value::from(shell.config_path().to_string_lossy().to_string())
}

pub fn null(_: &mut Shell) -> Value {
    Value::Null
}

pub fn user(_: &mut Shell) -> Value {
    Value::from(whoami::username())
}

pub fn pid(_: &mut Shell) -> Value {
    Value::Int(std::process::id() as i64)
}

pub fn hostname(_: &mut Shell) -> Value {
    Value::from(whoami::devicename())
}

pub fn home(shell: &mut Shell) -> Value {
    Value::from(shell.home_dir().to_string_lossy().to_string())
}

pub fn os(_: &mut Shell) -> Value {
    Value::from(std::env::consts::OS.to_string())
}

pub fn arch(_: &mut Shell) -> Value {
    Value::from(std::env::consts::ARCH.to_string())
}

pub fn distro(_: &mut Shell) -> Value {
    Value::from(whoami::distro())
}

pub fn desktop(_: &mut Shell) -> Value {
    Value::from(whoami::desktop_env().to_string())
}

pub fn status(shell: &mut Shell) -> Value {
    Value::Int(shell.exit_status)
}

pub fn pwd(_: &mut Shell) -> Value {
    Value::from(current_dir_str())
}

pub fn version(_: &mut Shell) -> Value {
    Value::from(env!("CARGO_PKG_VERSION").to_string())
}

pub fn family(_: &mut Shell) -> Value {
    Value::from(std::env::consts::FAMILY.to_string())
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

pub fn tau(_: &mut Shell) -> Value {
    Value::Float(std::f64::consts::TAU)
}

pub fn e(_: &mut Shell) -> Value {
    Value::Float(std::f64::consts::E)
}

pub fn epoch(_: &mut Shell) -> Value {
    let start = time::SystemTime::now();
    let since_the_epoch = start
        .duration_since(time::UNIX_EPOCH)
        .unwrap_or(Duration::ZERO);
    let duration =
        since_the_epoch.as_secs() * 1000 + since_the_epoch.subsec_nanos() as u64 / 1_000_000;
    Value::from(duration as i64)
}

pub fn path_sep(_: &mut Shell) -> Value {
    #[cfg(unix)]
    let sep = '/';
    #[cfg(windows)]
    let sep = '\\';
    Value::from(String::from(sep))
}

pub fn interactive(shell: &mut Shell) -> Value {
    Value::from(shell.interactive)
}
