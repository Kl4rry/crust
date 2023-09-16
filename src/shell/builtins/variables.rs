use std::{rc::Rc, time, time::Duration};

use phf::*;
use rand::Rng;

use crate::{
    parser::{lexer::token::span::Span, shell_error::ShellErrorKind},
    shell::{
        current_dir_str,
        value::{SpannedValue, Value},
        Shell,
    },
};

pub type GetBuiltin = fn(&mut Shell) -> Value;
pub type SetBuiltin = fn(&mut Shell, SpannedValue);

pub struct Builtins(GetBuiltin, Option<SetBuiltin>);

static BUILTIN_VARS: phf::Map<&'static str, Builtins> = phf_map! {
    "pid" => Builtins(pid, None),
    "home" => Builtins(home, None),
    "user" => Builtins(user, None),
    "hostname" => Builtins(hostname, None),
    "os" => Builtins(os, None),
    "family" => Builtins(family, None),
    "arch" => Builtins(arch, None),
    "distro" => Builtins(distro, None),
    "desktop" => Builtins(desktop, None),
    "?" => Builtins(status, None),
    "pwd" => Builtins(pwd, None),
    "version" => Builtins(version, None),
    "random" => Builtins(random, None),
    "lines" => Builtins(lines, None),
    "columns" => Builtins(columns, None),
    "null" => Builtins(null, None),
    "config" => Builtins(config, None),
    "args" => Builtins(args, None),
    "pi" => Builtins(pi, None),
    "tau" => Builtins(tau, None),
    "e" => Builtins(e, None),
    "unix_epoch" => Builtins(epoch, None),
    "path_sep" => Builtins(path_sep, None),
    "interactive" => Builtins(interactive, None),
    "print_ast" => Builtins(get_print_ast, Some(set_print_ast)),
};

pub fn is_builtin(name: &str) -> bool {
    BUILTIN_VARS.contains_key(name)
}

#[inline(always)]
pub fn get_var(shell: &mut Shell, name: &str) -> Option<Value> {
    BUILTIN_VARS.get(name).map(|var| var.0(shell))
}

pub enum SetResult {
    Success,
    NotFound(SpannedValue),
    Error(ShellErrorKind),
}

#[inline(always)]
pub fn set_var(shell: &mut Shell, name: &str, span: Span, value: SpannedValue) -> SetResult {
    let Some(builtin) = BUILTIN_VARS.get(name) else {
        return SetResult::NotFound(value);
    };

    let Some(set) = builtin.1 else {
        return SetResult::Error(ShellErrorKind::ReadOnlyVar(name.into(), span));
    };

    set(shell, value);
    SetResult::Success
}

pub fn set_print_ast(shell: &mut Shell, value: SpannedValue) {
    shell.print_ast = value.value.truthy();
}

pub fn get_print_ast(shell: &mut Shell) -> Value {
    shell.print_ast.into()
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
