use std::borrow::Cow;

use crate::{
    codegen::target::CodegenTarget,
    intrinsics::get_intrinsic_codegen_name,
    program::{NamespaceId, Program},
};

pub type CodegenError = Cow<'static, str>;
pub type CodegenResultG<T> = Result<T, CodegenError>;
pub type CodegenResult = CodegenResultG<()>;

pub struct CodegenContext<'a> {
    pub namespace: NamespaceId,
    pub program: &'a Program,
    pub target: CodegenTarget,
}

impl<'a> CodegenContext<'a> {
    pub fn scoped_name(namespace: NamespaceId, v: &'a str) -> Cow<'a, str> {
        Cow::Owned(format!("user_fn_{}_{}", namespace, v))
    }

    pub fn get_scoped_name(&self, v: &'a str) -> Cow<'a, str> {
        Self::scoped_name(self.namespace, v)
    }

    pub fn resolve_name(&self, v: &'a str) -> CodegenResultG<Cow<'a, str>> {
        if let Some(codegen_name) = get_intrinsic_codegen_name(v) {
            Ok(Cow::Borrowed(codegen_name))
        } else {
            match self.program.resolve_function(self.namespace, v) {
                Some((namespace, original_name)) => Ok(Self::scoped_name(namespace, original_name)),
                None => Err(format!("Unable to resolve name: {v}").into()),
            }
        }
    }
}
