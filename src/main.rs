use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use clap::Parser;
use cli::{Cli, Command};
use construct::gitea::Gitea;
use construct::ingress::Ingress;
use tf_bindgen::{cli::Terraform, Stack};
use tf_kubernetes::kubernetes::resource::{kubernetes_namespace, kubernetes_storage_class};
use tf_kubernetes::kubernetes::Kubernetes;

mod cli;
mod construct;
mod helper;

use construct::local_dir_volume::LocalDirVolume;
use construct::postgres::Postgres;

pub fn init() -> Rc<Stack> {
    let stack = Stack::new("gitserver");

    Kubernetes::create(&stack)
        .config_path("~/.kube/config")
        .build();

    let namespace = tf_bindgen::codegen::resource! {
        &stack, resource "kubernetes_namespace" "gitserver" {
            metadata {
                name = "gitserver"
            }
        }
    };
    let namespace = &namespace.metadata[0].name;

    let local_storage_class = tf_bindgen::codegen::resource! {
        &stack, resource "kubernetes_storage_class" "local_storage" {
            metadata {
                name = "local-storage"
            }
            storage_provisioner = "kubernetes.io/no-provisioner"
            volume_binding_mode = "WaitForFirstConsumer"
        }
    };

    let pgdata_volume = LocalDirVolume::create(&stack, "gitserver-pgdata")
        .storage("10Gi")
        .storage_class(&local_storage_class.metadata[0].name)
        .mount_path("/mnt/data1")
        .node("minikube")
        .build();
    let pgdata = pgdata_volume.claim("pgdata").namespace(namespace).build();

    let giteadata_volume = LocalDirVolume::create(&stack, "gitserver-giteadata")
        .storage("10Gi")
        .storage_class(&local_storage_class.metadata[0].name)
        .mount_path("/mnt/data2")
        .node("minikube")
        .build();
    let giteadata = giteadata_volume
        .claim("giteadata")
        .namespace(namespace)
        .build();

    Postgres::create(&stack, "db")
        .namespace(namespace)
        .volume_claim(pgdata.claim().clone().unwrap())
        .build();
    let gitea = Gitea::create(&stack, "gitea")
        .namespace(namespace)
        .volume_claim(giteadata.claim().clone().unwrap())
        .build();
    Ingress::create(&stack, "gitserver")
        .namespace(namespace)
        .services(vec![("/git", gitea.ingress())])
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
    #[cfg(unix)]
    {
        let terminate = Arc::new(AtomicBool::new(false));
        signal_hook::consts::TERM_SIGNALS
            .iter()
            .map(|signal| (signal, Arc::clone(&terminate)))
            .for_each(|(signal, hook)| {
                signal_hook::flag::register(*signal, hook).expect("register signal");
            });
        let mut child = command.spawn()?;
        while !terminate.load(Ordering::Relaxed) {
            if let Some(status) = child.try_wait()? {
                std::process::exit(status.code().unwrap_or(0));
            }
            std::thread::sleep(Duration::from_millis(200));
        }
        let _ = child.kill();
    }
    #[cfg(not(unix))]
    {
        todo!()
    }
    Ok(())
}
