use std::{collections::HashMap, rc::Rc};

use crate::{
    parser::{ast::statement::function::Function, shell_error::ShellErrorKind},
    shell::value::Value,
};

#[derive(Default, Debug)]
struct Inner {
    variables: HashMap<Rc<str>, (bool, Value)>,
    functions: HashMap<Rc<str>, Rc<(Rc<Function>, Frame)>>,
    parent: Option<Frame>,
    index: usize,
}

#[derive(Default, Debug)]
pub struct Frame(Rc<Inner>);

impl Frame {
    pub fn new(
        variables: HashMap<Rc<str>, (bool, Value)>,
        functions: HashMap<Rc<str>, Rc<(Rc<Function>, Frame)>>,
    ) -> Self {
        Self(Rc::new(Inner {
            variables,
            functions,
            parent: None,
            index: 0,
        }))
    }

    pub fn push(
        self,
        variables: HashMap<Rc<str>, (bool, Value)>,
        functions: HashMap<Rc<str>, Rc<(Rc<Function>, Frame)>>,
    ) -> Frame {
        Self(Rc::new(Inner {
            variables,
            functions,
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
    pub fn add_var(&mut self, name: Rc<str>, value: Value) {
        self.add_var_raw(name, value, false);
    }

    /// Add new env var in this scope
    pub fn add_env_var(&mut self, name: Rc<str>, value: Value) {
        debug_assert!(matches!(
            &value,
            Value::Bool(_) | Value::Int(_) | Value::Float(_) | Value::String(_)
        ));
        self.add_var_raw(name, value, true);
    }

    #[inline(always)]
    fn add_var_raw(&mut self, name: Rc<str>, value: Value, export: bool) {
        // safe because we never give out references to variable values
        unsafe {
            let inner = Rc::get_mut_unchecked(&mut self.0);
            inner.variables.insert(name, (export, value));
        };
    }

    pub fn get_function(&self, name: &str) -> Option<Rc<(Rc<Function>, Frame)>> {
        self.0.functions.get(name).cloned()
    }

    pub fn add_function(&mut self, name: Rc<str>, func: Rc<(Rc<Function>, Frame)>) {
        unsafe {
            let inner = Rc::get_mut_unchecked(&mut self.0);
            inner.functions.insert(name, func);
        };
    }

    pub fn env(&self) -> Vec<(String, String)> {
        let mut vars = HashMap::new();
        for frame in self.clone() {
            for (name, (export, var)) in &frame.0.variables {
                if *export && !vars.contains_key(&**name) {
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
