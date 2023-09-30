use std::{rc::Rc, time, time::Duration};

use phf::*;
use rand::Rng;

use crate::{
    parser::{ast::context::Context, lexer::token::span::Span, shell_error::ShellErrorKind},
    shell::{
        current_dir_str,
        value::{SpannedValue, Value},
    },
};

pub type GetBuiltin = fn(&mut Context) -> Value;
pub type SetBuiltin = fn(&mut Context, SpannedValue) -> Result<(), ShellErrorKind>;

pub struct Builtins(GetBuiltin, Option<SetBuiltin>);

static BUILTIN_VARS: phf::Map<&'static str, Builtins> = phf_map! {
    "?" => Builtins(status, None),
    ">" => Builtins(get_input_stream, None),
    "arch" => Builtins(arch, None),
    "args" => Builtins(args, None),
    "columns" => Builtins(columns, None),
    "config" => Builtins(config, None),
    "desktop" => Builtins(desktop, None),
    "distro" => Builtins(distro, None),
    "e" => Builtins(e, None),
    "family" => Builtins(family, None),
    "home" => Builtins(home, None),
    "hostname" => Builtins(hostname, None),
    "interactive" => Builtins(interactive, None),
    "lines" => Builtins(lines, None),
    "null" => Builtins(null, None),
    "os" => Builtins(os, None),
    "path_sep" => Builtins(path_sep, None),
    "pi" => Builtins(pi, None),
    "pid" => Builtins(pid, None),
    "print_ast" => Builtins(get_print_ast, Some(set_print_ast)),
    "prompt" => Builtins(get_prompt, Some(set_prompt)),
    "pwd" => Builtins(pwd, None),
    "random" => Builtins(random, None),
    "tau" => Builtins(tau, None),
    "unix_epoch" => Builtins(epoch, None),
    "user" => Builtins(user, None),
    "version" => Builtins(version, None),
};

pub fn is_builtin(name: &str) -> bool {
    BUILTIN_VARS.contains_key(name)
}

#[inline(always)]
pub fn get_var(ctx: &mut Context, name: &str) -> Option<Value> {
    BUILTIN_VARS.get(name).map(|var| var.0(ctx))
}

pub enum SetResult {
    Success,
    NotFound(SpannedValue),
    Error(ShellErrorKind),
}

#[inline(always)]
pub fn set_var(ctx: &mut Context, name: &str, span: Span, value: SpannedValue) -> SetResult {
    let Some(builtin) = BUILTIN_VARS.get(name) else {
        return SetResult::NotFound(value);
    };

    let Some(set) = builtin.1 else {
        return SetResult::Error(ShellErrorKind::ReadOnlyVar(name.into(), span));
    };

    if let Err(err) = set(ctx, value) {
        return SetResult::Error(err);
    }

    SetResult::Success
}

pub fn set_prompt(ctx: &mut Context, value: SpannedValue) -> Result<(), ShellErrorKind> {
    match value.value {
        Value::Closure(closure) => {
            ctx.shell.prompt = Some(closure);
            Ok(())
        }
        // TODO make error nicer
        _ => Err(ShellErrorKind::Basic(
            "Type Error",
            "Prompt must be a closure".into(),
        )),
    }
}

pub fn get_input_stream(ctx: &mut Context) -> Value {
    ctx.input.take().unpack()
}

pub fn get_prompt(ctx: &mut Context) -> Value {
    ctx.shell
        .prompt
        .clone()
        .map(Value::Closure)
        .unwrap_or(Value::Null)
}

pub fn set_print_ast(ctx: &mut Context, value: SpannedValue) -> Result<(), ShellErrorKind> {
    ctx.shell.print_ast = value.value.truthy();
    Ok(())
}

pub fn get_print_ast(ctx: &mut Context) -> Value {
    ctx.shell.print_ast.into()
}

pub fn args(ctx: &mut Context) -> Value {
    Value::from(
        ctx.shell
            .args
            .iter()
            .map(|s| Value::String(Rc::new(s.to_string())))
            .collect::<Vec<_>>(),
    )
}

pub fn config(ctx: &mut Context) -> Value {
    Value::from(ctx.shell.config_path().to_string_lossy().to_string())
}

pub fn null(_: &mut Context) -> Value {
    Value::Null
}

pub fn user(_: &mut Context) -> Value {
    Value::from(whoami::username())
}

pub fn pid(_: &mut Context) -> Value {
    Value::Int(std::process::id() as i64)
}

pub fn hostname(_: &mut Context) -> Value {
    Value::from(whoami::devicename())
}

pub fn home(ctx: &mut Context) -> Value {
    Value::from(ctx.shell.home_dir().to_string_lossy().to_string())
}

pub fn os(_: &mut Context) -> Value {
    Value::from(std::env::consts::OS.to_string())
}

pub fn arch(_: &mut Context) -> Value {
    Value::from(std::env::consts::ARCH.to_string())
}

pub fn distro(_: &mut Context) -> Value {
    Value::from(whoami::distro())
}

pub fn desktop(_: &mut Context) -> Value {
    Value::from(whoami::desktop_env().to_string())
}

pub fn status(ctx: &mut Context) -> Value {
    Value::Int(ctx.shell.exit_status)
}

pub fn pwd(_: &mut Context) -> Value {
    Value::from(current_dir_str())
}

pub fn version(_: &mut Context) -> Value {
    Value::from(env!("CARGO_PKG_VERSION").to_string())
}

pub fn family(_: &mut Context) -> Value {
    Value::from(std::env::consts::FAMILY.to_string())
}

pub fn random(_: &mut Context) -> Value {
    let mut rng = rand::thread_rng();
    Value::Int(rng.gen_range(0..i64::MAX))
}

pub fn lines(_: &mut Context) -> Value {
    let (_, h) = crossterm::terminal::size().unwrap();
    Value::Int(h as i64)
}

pub fn columns(_: &mut Context) -> Value {
    let (w, _) = crossterm::terminal::size().unwrap();
    Value::Int(w as i64)
}

pub fn pi(_: &mut Context) -> Value {
    Value::Float(std::f64::consts::PI)
}

pub fn tau(_: &mut Context) -> Value {
    Value::Float(std::f64::consts::TAU)
}

pub fn e(_: &mut Context) -> Value {
    Value::Float(std::f64::consts::E)
}

pub fn epoch(_: &mut Context) -> Value {
    let start = time::SystemTime::now();
    let since_the_epoch = start
        .duration_since(time::UNIX_EPOCH)
        .unwrap_or(Duration::ZERO);
    let duration =
        since_the_epoch.as_secs() * 1000 + since_the_epoch.subsec_nanos() as u64 / 1_000_000;
    Value::from(duration as i64)
}

pub fn path_sep(_: &mut Context) -> Value {
    #[cfg(unix)]
    let sep = '/';
    #[cfg(windows)]
    let sep = '\\';
    Value::from(String::from(sep))
}

pub fn interactive(ctx: &mut Context) -> Value {
    Value::from(ctx.shell.interactive)
}
