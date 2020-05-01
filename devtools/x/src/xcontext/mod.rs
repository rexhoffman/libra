// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//remove pub once conversion is complete.
pub mod execution_location;
//pub will still be needed here for config.
pub mod project_metadata;

use project_metadata::Config;
use shaku::{module, Component, Container, ContainerBuilder, Interface};
use std::{path::Path, sync::Arc};

use once_cell::sync::OnceCell;

/// Reference to the root of the project (location of root Cargo.toml)
///
/// This is a non-ideal implementation as it's generally better to have no
/// sensing of information for the construction of components in the container
/// done by the components themselves.   Some exceptions can be made for "top level"
/// components that exist in the main method as a specialized configuration of that
/// container.
pub trait ProjectRoot: Interface {
    fn root(&self) -> &Path;
}

#[derive(Component)]
#[shaku(interface = ProjectRoot)]
struct ProjectRootImpl {
    #[shaku(default = Box::from(execution_location::project_root()))]
    root: Box<Path>,
}

impl ProjectRoot for ProjectRootImpl {
    fn root(&self) -> &Path {
        self.root.as_ref()
    }
}

/// Reference to the path of this execution, the working dir.
pub trait ExecutionPath: Interface {
    fn path(&self) -> &Path;
}
#[derive(Component)]
#[shaku(interface = ExecutionPath)]
struct ExecutionPathImpl {
    #[shaku(default = Box::from(execution_location::locate_project().unwrap().as_path()))]
    path: Box<Path>,
}

impl ExecutionPath for ExecutionPathImpl {
    fn path(&self) -> &Path {
        self.path.as_ref()
    }
}

/// Is the root of the project the current execution path.
pub trait RootExecutionPath: Interface {
    fn the_same(&self) -> bool;
}

/// This struct can be thought of as a representation of a factory
/// project_root == execution_path -> is_project_root_execution_path;
#[derive(Component)]
#[shaku(interface = RootExecutionPath)]
pub struct RootExecutionPathImpl {
    // these fields are unused but exist as tracking informantion, coi as it stands now
    // can't define a Provider without making it context aware -- allowing the provider
    // access to the entire context and breaking the ability to track a dependency graph
    // through that provider.  Not good.  So this approach allows us to track the direct deps.
    #[shaku(inject)]
    #[allow(dead_code)]
    project_root: Arc<dyn ProjectRoot>,
    #[shaku(inject)]
    #[allow(dead_code)]
    execution_path: Arc<dyn ExecutionPath>,
    // this is the relevant field calculated and cached from the prior fields.
    the_same: OnceCell<bool>,
}

impl RootExecutionPath for RootExecutionPathImpl {
    fn the_same(&self) -> bool {
        self.the_same
    }
}

impl RootExecutionPathImpl {
    fn new(project_root: Arc<dyn ProjectRoot>, execution_path: Arc<dyn ExecutionPath>) -> Self {
        Self {
            the_same: project_root.root() == execution_path.path(),
            project_root,
            execution_path,
        }
    }
}

/// Is the root of the project the current execution path.
pub trait ProjectConfig: Interface {
    fn config(&self) -> &Config;
}

#[derive(Component)]
#[shaku(interface = ProjectConfig)]
pub struct ProjectConfigImpl {
    // these fields are unused but exist as tracking informantion, coi as it stands now
    // can't define a Provider without making it context aware -- allowing the provider
    // access to the entire context and breaking the ability to track a dependency graph
    // through that provider.  Not good.  So this approach allows us to track the direct deps.
    #[shaku(inject)]
    #[allow(dead_code)]
    project_root: Arc<dyn ProjectRoot>,
    // this is the relevant field calculated and cached from the prior fields.
    config: OnceCell<Config>,
}

impl ProjectConfig for ProjectConfigImpl {
    fn config(&self) -> &Config {
        &self.config
    }
}

impl ProjectConfigImpl {
    fn new(project_root: Arc<dyn ProjectRoot>) -> Self {
        Self {
            config: project_metadata::Config::from_file(project_root.root().join("x.toml"))
                .expect("expecting x.toml in the project root"),
            project_root,
        }
    }
}

module! {
    ExecutionContextModule {
        components = [RootExecutionPathImpl, ProjectRootImpl, ExecutionPathImpl, ProjectConfigImpl],
        providers = []
    }
}

#[test]
fn xcontext_component_test() {
    println!("before container");

    let container: Container<ExecutionContextModule> = ContainerBuilder::new().build();

    // cool get a managed instance via it's dependencies getting construct
    let root_path_is_execution_path: &dyn RootExecutionPath = container.resolve_ref();

    println!(
        "Root path is execution path {:?}",
        root_path_is_execution_path.the_same()
    );

    println!("before reference");

    // get one of the paths...
    let expath: &dyn ExecutionPath = container.resolve_ref();

    println!("before use");
    println!("{:?}", expath.path());

    // get the other path...
    let propath: &dyn ProjectRoot = container.resolve_ref();

    println!("{:?}", propath.root());

    // get the config...
    let config: &dyn ProjectConfig = container.resolve_ref();

    println!("{:?}", config.config());
}
