use std::rc::Rc;

use clap::Parser;
use cli::{Cli, Command};
use tf_bindgen::{cli::Terraform, Stack};
use tf_kubernetes::kubernetes::resource::kubernetes_namespace;
use tf_kubernetes::kubernetes::Kubernetes;

mod cli;
mod construct;
mod helper;

use construct::local_volume::LocalVolume;
use construct::postgres::Postgres;

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

    let volume = LocalVolume::create(&stack, "gitserver")
        .storage("10Gi")
        .mount_path("/mnt")
        .node("minikube")
        .build();

    let pgdata = volume
        .claim("pgdata")
        .namespace(&namespace.metadata[0].name)
        .storage("10Gi")
        .build();

    Postgres::create(&stack, "gitea-pg")
        .namespace(&namespace.metadata[0].name)
        .volume_claim(pgdata.claim().clone().unwrap())
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
