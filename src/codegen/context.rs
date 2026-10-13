use std::{borrow::Cow, collections::HashMap};

use crate::{
    codegen::{target::CodegenTarget, terms::enumerate_captures},
    intrinsics::get_intrinsic_codegen_name,
    lang::Block,
    program::{NamespaceId, Program},
};

pub type CodegenError = Cow<'static, str>;
pub type CodegenResultG<T> = Result<T, CodegenError>;
pub type CodegenResult = CodegenResultG<()>;

pub struct CodegenContext<'a> {
    pub namespace: NamespaceId,
    pub program: &'a Program,
    pub target: CodegenTarget,
    pub(crate) vars: HashMap<String, String>,
}

pub enum ResolvedName<T> {
    Function(T),
    Variable(T),
}

impl<'a> CodegenContext<'a> {
    pub fn new(program: &'a Program) -> Self {
        CodegenContext {
            namespace: 0,
            program,
            target: CodegenTarget::default(),
            vars: HashMap::new(),
        }
    }
    pub fn scoped_name(namespace: NamespaceId, v: &'a str) -> Cow<'a, str> {
        Cow::Owned(format!("user_fn_{}_{}", namespace, v))
    }

    pub fn get_scoped_name(&self, v: &'a str) -> Cow<'a, str> {
        Self::scoped_name(self.namespace, v)
    }

    pub fn load_vars(&mut self, body: &Block) {
        self.vars.clear();
        let mut i = 0;
        enumerate_captures(body, &mut |name| {
            if self.vars.contains_key(name) {
                return;
            }
            let mapped_name = format!("var_{i}");
            i += 1;
            self.vars.insert(name.to_owned(), mapped_name);
        });
    }

    pub fn clear_vars(&mut self) {
        self.vars.clear();
    }

    fn maybe_resolve_variable_name(&self, v: &str) -> Option<String> {
        self.vars.get(v).cloned()
    }

    fn maybe_resolve_function_name(&self, v: &str) -> Option<Cow<'a, str>> {
        match self.program.resolve_function(self.namespace, v) {
            Some((namespace, original_name)) => Some(Self::scoped_name(namespace, original_name)),
            None => self.vars.get(v).map(|i| Cow::Owned(i.clone())),
        }
    }

    pub fn resolve_variable_name(&self, v: &str) -> CodegenResultG<String> {
        match self.maybe_resolve_variable_name(v) {
            Some(e) => Ok(e),
            None => Err(format!("Unable to resolve variable: {v}").into()),
        }
    }

    pub fn resolve_function_name(&self, v: &str) -> CodegenResultG<Cow<'a, str>> {
        if let Some(codegen_name) = get_intrinsic_codegen_name(v) {
            return Ok(Cow::Borrowed(codegen_name));
        }

        match self.maybe_resolve_function_name(v) {
            Some(e) => Ok(e),
            None => Err(format!("Unable to resolve function: {v}").into()),
        }
    }

    pub fn resolve_name(&self, v: &'a str) -> CodegenResultG<ResolvedName<Cow<'a, str>>> {
        if let Some(codegen_name) = get_intrinsic_codegen_name(v) {
            return Ok(ResolvedName::Function(Cow::Borrowed(codegen_name)));
        }

        if let Some(name) = self.maybe_resolve_variable_name(v) {
            return Ok(ResolvedName::Variable(name.into()));
        }

        if let Some(name) = self.maybe_resolve_function_name(v) {
            return Ok(ResolvedName::Function(name));
        }

        Err(format!("Unable to resolve name: {v}").into())
    }
}
