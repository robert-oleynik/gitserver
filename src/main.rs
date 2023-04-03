use std::rc::Rc;

use clap::Parser;
use construct::postgres::Postgres;
use tf_bindgen::{cli::Terraform, Stack};

mod cli;
mod construct;
mod helper;

use cli::{Cli, Command};
use tf_kubernetes::kubernetes::resource::kubernetes_namespace::*;
use tf_kubernetes::kubernetes::Kubernetes;

pub fn init() -> Rc<Stack> {
    let stack = Stack::new("gitserver");

    Kubernetes::create(&stack)
        .config_path("~/.kube/config")
        .build();

    let namespace = tf_bindgen::codegen::resource! {
        &stack,
        resource "kubernetes_namespace" "gitserver" {
            metadata {
                name = "gitserver"
            }
        }
    };

    Postgres::create(&stack, "gitea")
        .namespace(&namespace.metadata[0].name)
        .build();

    stack
}

fn main() -> std::io::Result<()> {
    let cli = Cli::parse();

    let stack = init();
    let mut command = match cli.command() {
        Command::Init => Terraform::init(&stack)?,
        Command::Apply => Terraform::apply(&stack)?,
        Command::Destroy => Terraform::destroy(&stack)?,
    };
    let exit_code = command.spawn()?.wait()?.code().unwrap_or(0);
    std::process::exit(exit_code);
}
