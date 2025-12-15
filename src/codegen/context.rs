use std::borrow::Cow;

use crate::{
    codegen::target::CodegenTarget,
    intrinsics::{get_c_name, get_intrinsic},
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
        if get_intrinsic(v).is_some() {
            Cow::Borrowed(get_c_name(v))
        } else {
            Self::scoped_name(self.namespace, v)
        }
    }

    pub fn resolve_name_reference(&self, v: &'a str) -> Cow<'a, str> {
        if get_intrinsic(v).is_some() {
            Cow::Borrowed(get_c_name(v))
        } else {
            match self.program.resolve_function(self.namespace, v) {
                Some((namespace, original_name)) => Self::scoped_name(namespace, original_name),
                None => Self::scoped_name(111111, "unresolved"),
            }
        }
    }
}
