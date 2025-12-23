use std::collections::HashMap;

use crate::{
    lang::{Function, ImportNaming, Module},
    path::CanonicalPathBuf,
};

#[derive(Debug)]
pub struct NamespaceImport {
    pub id: NamespaceId,
    pub naming: ImportNaming,
}

#[derive(Default, Debug)]
pub struct Namespace {
    pub path: Option<CanonicalPathBuf>,
    pub imports: Vec<NamespaceImport>,
    pub functions: HashMap<String, Function>,
}

pub type NamespaceId = usize;

#[derive(Debug)]
pub struct Program {
    pub namespaces: Vec<Namespace>,
    _x: (),
}

impl Program {
    pub fn new() -> Self {
        Program {
            namespaces: vec![],
            _x: (),
        }
    }

    #[allow(dead_code)]
    pub fn new_from_module(ast: &Module) -> Self {
        let mut res = Program {
            namespaces: vec![],
            _x: (),
        };
        let id = res.allocate_namespace();
        res.add_functions(id, &ast.functions);
        res
    }

    pub fn allocate_namespace(&mut self) -> NamespaceId {
        self.namespaces.push(Namespace::default());
        self.namespaces.len() - 1
    }

    pub fn get_namespace_mut(&mut self, namespace: NamespaceId) -> &mut Namespace {
        &mut self.namespaces[namespace]
    }

    pub fn get_namespace(&self, namespace: NamespaceId) -> &Namespace {
        &self.namespaces[namespace]
    }

    pub fn add_functions(&mut self, namespace: NamespaceId, functions: &[Function]) {
        let namespace = &mut self.namespaces[namespace];

        for f in functions {
            namespace.functions.insert(f.name.clone(), f.clone());
        }
    }

    pub fn add_imports(&mut self, namespace: NamespaceId, imports: Vec<NamespaceImport>) {
        let namespace = &mut self.namespaces[namespace];
        namespace.imports.extend(imports);
    }

    fn resolve_function_in_namespace(
        &self,
        id: NamespaceId,
        name: &str,
    ) -> Option<(NamespaceId, &str)> {
        self.namespaces[id]
            .functions
            .get(name)
            .map(|f| (id, f.name.as_ref()))
    }

    pub fn resolve_function(
        &self,
        current_id: NamespaceId,
        name: &str,
    ) -> Option<(NamespaceId, &str)> {
        let current = &self.namespaces[current_id];

        if let Some(same_namespace) = self.resolve_function_in_namespace(current_id, name) {
            return Some(same_namespace);
        }

        for import in &current.imports {
            match &import.naming {
                ImportNaming::Wildcard => {
                    if let Some(other_namespace) =
                        self.resolve_function_in_namespace(import.id, name)
                    {
                        return Some(other_namespace);
                    }
                }
                ImportNaming::Named(names) => {
                    let is_named = names.iter().any(|f| f == name);
                    if is_named
                        && let Some(other_namespace) =
                            self.resolve_function_in_namespace(import.id, name)
                    {
                        return Some(other_namespace);
                    }
                }
                ImportNaming::Scoped(prefix) => {
                    if name.starts_with(prefix) && name[prefix.len()..].starts_with('.') {
                        let trailing = &name[(prefix.len() + 1)..];
                        if let Some(other_namespace) =
                            self.resolve_function_in_namespace(import.id, trailing)
                        {
                            return Some(other_namespace);
                        }
                    }
                }
            }
        }

        None
    }
}
