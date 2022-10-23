use std::{collections::HashMap, rc::Rc};

use super::stream::ValueStream;
use crate::{
    parser::{
        ast::{variable::Variable, Block},
        shell_error::ShellErrorKind,
    },
    shell::value::Value,
};

#[derive(Debug, Default)]
struct Inner {
    variables: HashMap<String, (bool, Value)>,
    functions: HashMap<String, Rc<(Vec<Variable>, Block)>>,
    #[allow(unused)]
    input: ValueStream,
    parent: Option<Frame>,
    index: usize,
}

#[derive(Debug, Default)]
pub struct Frame(Rc<Inner>);

impl Frame {
    pub fn new(
        variables: HashMap<String, (bool, Value)>,
        functions: HashMap<String, Rc<(Vec<Variable>, Block)>>,
        input: ValueStream,
    ) -> Self {
        Self(Rc::new(Inner {
            variables,
            functions,
            input,
            parent: None,
            index: 0,
        }))
    }

    pub fn push(
        self,
        variables: HashMap<String, (bool, Value)>,
        functions: HashMap<String, Rc<(Vec<Variable>, Block)>>,
        input: ValueStream,
    ) -> Frame {
        Self(Rc::new(Inner {
            variables,
            functions,
            input,
            parent: Some(self),
            index: 0,
        }))
    }

    pub fn index(&self) -> usize {
        self.0.index
    }

    pub fn get_var(&self, name: &str) -> Option<Value> {
        let (_, value) = self.0.variables.get(name)?;
        Some(value.clone())
    }

    /// Update existing variable. Returns the value if the variable does not exist.
    pub fn update_var(
        &mut self,
        name: &str,
        value: Value,
    ) -> Result<Option<Value>, ShellErrorKind> {
        for mut frame in self.clone() {
            // safe because we never give out references to variable values
            unsafe {
                let inner = Rc::get_mut_unchecked(&mut frame.0);
                if let Some((env, heap_value)) = inner.variables.get_mut(name) {
                    if *env
                        && !matches!(
                            &value,
                            Value::Bool(_) | Value::Int(_) | Value::Float(_) | Value::String(_)
                        )
                    {
                        return Err(ShellErrorKind::InvalidEnvVar(value.to_type()));
                    }

                    *heap_value = value;
                    return Ok(None);
                }
            }
        }
        Ok(Some(value))
    }

    /// Add new var in this scope
    pub fn add_var(&mut self, name: String, value: Value) {
        self.add_var_raw(name, value, false);
    }

    /// Add new env var in this scope
    pub fn add_env_var(&mut self, name: String, value: Value) {
        debug_assert!(matches!(
            &value,
            Value::Bool(_) | Value::Int(_) | Value::Float(_) | Value::String(_)
        ));
        self.add_var_raw(name, value, true);
    }

    #[inline(always)]
    fn add_var_raw(&mut self, name: String, value: Value, export: bool) {
        // safe because we never give out references to variable values
        unsafe {
            let inner = Rc::get_mut_unchecked(&mut self.0);
            inner.variables.insert(name, (export, value));
        };
    }

    pub fn get_function(&self, name: &str) -> Option<Rc<(Vec<Variable>, Block)>> {
        self.0.functions.get(name).cloned()
    }

    pub fn add_function(&mut self, name: String, func: Rc<(Vec<Variable>, Block)>) {
        unsafe {
            let inner = Rc::get_mut_unchecked(&mut self.0);
            inner.functions.insert(name, func);
        };
    }

    pub fn env(&self) -> Vec<(String, String)> {
        let mut vars = HashMap::new();
        for frame in self.clone() {
            for (name, (export, var)) in &frame.0.variables {
                if *export && !vars.contains_key(name) {
                    vars.insert(name.to_string(), var.to_string());
                }
            }
        }
        vars.into_iter().collect()
    }
}

impl Clone for Frame {
    fn clone(&self) -> Self {
        Self(Rc::clone(&self.0))
    }
}

impl IntoIterator for Frame {
    type Item = Frame;

    type IntoIter = FrameIter;

    fn into_iter(self) -> Self::IntoIter {
        FrameIter(Some(self))
    }
}

pub struct FrameIter(Option<Frame>);

impl Iterator for FrameIter {
    type Item = Frame;

    fn next(&mut self) -> Option<Self::Item> {
        match self.0 {
            Some(ref mut frame) => {
                let old = frame.clone();
                self.0 = frame.0.parent.clone();
                Some(old)
            }
            None => None,
        }
    }
}
