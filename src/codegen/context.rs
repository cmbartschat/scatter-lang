use std::borrow::Cow;

use crate::{
    codegen::target::CodegenTarget,
    intrinsics::get_intrinsic_codegen_name,
    program::{NamespaceId, Program},
};

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
        if let Some(codegen_name) = get_intrinsic_codegen_name(v) {
            Cow::Borrowed(codegen_name)
        } else {
            Self::scoped_name(self.namespace, v)
        }
    }

    pub fn resolve_name_reference(&self, v: &'a str) -> Cow<'a, str> {
        if let Some(codegen_name) = get_intrinsic_codegen_name(v) {
            Cow::Borrowed(codegen_name)
        } else {
            match self.program.resolve_function(self.namespace, v) {
                Some((namespace, original_name)) => Self::scoped_name(namespace, original_name),
                #[expect(clippy::unreadable_literal)]
                None => Self::scoped_name(111111, "unresolved"),
            }
        }
    }
}
