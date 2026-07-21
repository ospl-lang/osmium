use std::{collections::{HashMap, VecDeque}, hash::Hash, path::PathBuf, sync::{Arc, Mutex, atomic::AtomicUsize}};
use ospl_common::{ast::Statement, inst::{optimized::Inst, symbols::DebugSymbolTable}};
use ospl_compiler::{BuildData, Compiler};

use crate::{BUILD_FOLDER, Log, graph::resolv0::{PkgRef, VersionRuleRef}};

#[derive(Clone)]
pub struct CxxNode {
    pub required_as: String,
    pub o_file: PathBuf,
    pub c_file: PathBuf,
    pub link_with: Vec<String>,
}

pub type ModuleId = u32;

#[derive(Default, Debug)]
pub struct Graph {
    pub modules: HashMap<ModuleId, ModuleNode>,
    
    /// The ID of the program entrypoint
    pub main: ModuleId,
}

#[derive(Default)]
pub struct ModuleNode {
    pub deps: Vec<Requirement>,
    pub cxx_deps: Vec<CxxNode>,
    pub ast: Vec<Statement>,

    /// Uused for anything but logging
    pub meta: ModuleNodeMeta
}

#[derive(Default)]
pub struct ModuleNodeMeta {
    pub name: String,
    pub pkg: String,
}

impl std::fmt::Debug for ModuleNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(f, "{}/{}", self.meta.pkg, self.meta.name)
    }
}

#[derive(Clone, Default, Debug)]
pub struct Requirement {
    pub ident: String,
    pub id: u32,
}

#[derive(Clone, Debug)]
pub struct UnresolvedRequirement {
    pub re: PkgRef,
    pub ver: VersionRuleRef
}

pub fn genmods(graph: &Graph) -> HashMap<u32, GeneratedModule> {
    let keys: Vec<u32> = graph.modules.keys().cloned().collect();
    let order = topo_sort(
        &keys,
        |id| graph.modules[id].deps.iter().map(|e| e.id).collect(),
    ).expect("cycle in dependency graph!");

    let mut finished: HashMap<u32, GeneratedModule> = HashMap::new();

    for node_id in order {
        let node = &graph.modules[&node_id];

        // LogState!(&format!("node {node_id}"));

        Log!(Linking, "module {node:?}");

        let mut input_code = Vec::new();
        for req in &node.deps {
            // Log!(Linking, "with {}", req.ident);
            let requirement = finished[&req.id].clone();
            let requirement = super::wrap_in_iife_declaration(&req.ident, requirement.stmts);
            input_code.push(requirement);
        }

        for cxx in &node.cxx_deps {
            let so_file = cxx.o_file.with_extension("so");

            let mut so_file2 = PathBuf::from(BUILD_FOLDER);
            so_file2.push(cxx.o_file.with_extension("so"));

            let mut obj_file2 = PathBuf::from(BUILD_FOLDER);
            obj_file2.push(cxx.o_file.with_extension("o"));

            let s = super::create_ffi(&cxx.required_as, &so_file);
            input_code.push(s);
        };

        input_code.extend_from_slice(&node.ast);
        finished.insert(node_id, GeneratedModule {
            cxx_deps: node.cxx_deps.clone(),
            file: "(unknown)".to_string(),
            stmts: input_code,
            deps: node.deps.clone()
        });
    }

    return finished;
}

#[derive(Clone)]
pub struct GeneratedModule {
    pub stmts: Vec<Statement>,
    pub file: String,
    pub cxx_deps: Vec<CxxNode>,
    pub deps: Vec<Requirement>,
}

pub fn getmain<'a>(graph: &'a Graph, m: &'a HashMap<u32, GeneratedModule>) -> Option<&'a GeneratedModule> {
    return m.get(&graph.main)
}

pub fn buildmain(graph: &Graph, mut m: HashMap<u32, GeneratedModule>) -> Result<Vec<Inst>, ospl_compiler::CE> {
    let m = m.remove(&graph.main)
        .expect("the entrypoint is missing");

    for cxx in &m.cxx_deps {
        Log!(Invoking, "C compiler on {:?}", cxx.c_file);
        let mut so_file2 = PathBuf::from(BUILD_FOLDER);
        so_file2.push(cxx.o_file.with_extension("so"));

        let mut obj_file2 = PathBuf::from(BUILD_FOLDER);
        obj_file2.push(cxx.o_file.with_extension("o"));

        if !std::process::Command::new("cc")
            .arg("-fPIC")
            .arg("-c")
            .arg(&cxx.c_file)
            .arg("-o")
            .arg(&obj_file2)
            .spawn()
            .expect("failed to summon cc")
            .wait()
            .expect("failed to wait for cc")
            .success()
        { panic!("CC failed to run") }

        if !std::process::Command::new("cc")
            .arg("-shared")
            .arg(obj_file2)
            .arg("-o")
            .arg(&so_file2)
            .spawn()
            .expect("failed to summon cc")
            .wait()
            .expect("failed to wait for cc")
            .success()
        { panic!("CC failed to run") }
    };

    let build_data = Arc::new(BuildData {
        next_resource_id: AtomicUsize::new(0),
        symbols: Mutex::new(DebugSymbolTable::default())
    });

    Log!(Compiling, "everything");
    let mut comp = Compiler::new(build_data);
    let mut root = Vec::new();
    comp.compile_all(&m.stmts, &mut root)?;

    return Ok(root)
}

/// Generic topological sort using Kahn's algorithm.
///
/// nodes:
///   - all graph nodes
///
/// edges:
///   - function returning dependencies for a node
///
/// Returns:
///   - `Some(sorted list)` if DAG
///   - `None` if cycle exists
pub fn topo_sort<N, F>(nodes: &[N], mut edges: F) -> Option<Vec<N>>
where
    N: Eq + Hash + Clone,
    F: FnMut(&N) -> Vec<N>,
{
    let mut indegree: HashMap<N, usize> = HashMap::new();
    let mut adj: HashMap<N, Vec<N>> = HashMap::new();

    // initialize graph
    for node in nodes {
        indegree.entry(node.clone()).or_insert(0);

        for dep in edges(node) {
            adj.entry(dep.clone()).or_default().push(node.clone());
            *indegree.entry(node.clone()).or_insert(0) += 1;
        }
    }

    // queue of nodes with no incoming edges
    let mut q = VecDeque::new();
    for (node, &deg) in &indegree {
        if deg == 0 {
            q.push_back(node.clone());
        }
    }

    let mut result = Vec::with_capacity(nodes.len());

    while let Some(n) = q.pop_front() {
        result.push(n.clone());

        if let Some(children) = adj.get(&n) {
            for c in children {
                let entry = indegree.get_mut(c).unwrap();
                *entry -= 1;

                if *entry == 0 {
                    q.push_back(c.clone());
                }
            }
        }
    }

    if result.len() == nodes.len() {
        Some(result)
    } else {
        None // cycle detected
    }
}