use std::rc::Rc;

use tf_bindgen::codegen::{resource, Construct};
use tf_bindgen::value::IntoValue;
use tf_bindgen::{Scope, Value};
use tf_kubernetes::kubernetes::resource::{
    kubernetes_config_map, kubernetes_secret, kubernetes_service, kubernetes_stateful_set,
};

use super::ingress::IngressServiceConfig;

#[derive(Construct)]
#[construct(builder)]
#[allow(dead_code)]
pub struct Gitea {
    #[construct(id)]
    name: String,
    #[construct(scope)]
    scope: Rc<dyn Scope>,
    #[construct(setter(into_value))]
    namespace: Value<String>,
    #[construct(setter(into))]
    path: String,
    #[construct(setter(into_value))]
    domain: Value<String>,
    #[construct(setter(into_value))]
    postgres_host: Value<String>,
    #[construct(setter(into_value))]
    volume_claim: Value<String>,
}

impl Gitea {
    pub fn ingress(&self) -> IngressServiceConfig {
        IngressServiceConfig {
            path: self.path.clone(),
            service_name: format!("{}-service", self.name),
            service_port: 3000,
        }
    }
}

impl GiteaBuilder {
    pub fn build(&mut self) -> Rc<Gitea> {
        let this = Rc::new(Gitea {
            name: self.name.clone(),
            scope: self.scope.clone(),
            namespace: self.namespace.clone().expect("missing field 'namespace'"),
            path: self.path.clone().unwrap_or("/".into()),
            domain: self.domain.clone().unwrap_or("localhost".into_value()),
            postgres_host: self
                .postgres_host
                .clone()
                .unwrap_or("localhost".into_value()),
            volume_claim: self
                .volume_claim
                .clone()
                .expect("missing field 'volume_claim'"),
        });

        let name = &this.name;
        let labels = crate::map! {
            "app" = format!("gitea-{name}")
        };

        let service = resource! {
            &this, resource "kubernetes_service" "gitea" {
                metadata {
                    namespace = &this.namespace
                    name = format!("{name}-service")
                }
                spec {
                    selector = &labels
                    port {
                        name = "web"
                        port = 3000
                    }
                }
            }
        };

        let config = resource! {
            &this, resource "kubernetes_config_map" "gitea-config" {
                metadata {
                    namespace = &this.namespace
                    name = name
                }
                data = crate::map!{
                    "USER_GID" = "1000",
                    "USER_UID" = "1000",
                    "GITEA_WORK_DIR" = "/gitea",
                    "GITEA_CUSTOM" = "/gitea/custom",
                    "GITEA__database__DB_TYPE" = "postgres",
                    "GITEA__database__HOST" = &this.postgres_host,
                    "GITEA__database__NAME" = "gitea",
                    "GITEA__database__USER" = "gitea",
                    "GITEA__database__PASSWORD" = "gitea",
                    "GITEA__server__ROOT_URL" = format!("https://%(DOMAIN)s:%(HTTP_PORT)s{}", this.path),
                    "GITEA__server__DOMAIN" = &this.domain
                }
            }
        };
        let init_config = resource! {
            &this, resource "kubernetes_secret" "gitea-init-config" {
                r#type = "Opaque"
                metadata {
                    namespace = &this.namespace
                    name = format!("{name}-init")
                }
                data = crate::map! {
                    "init.sh" = INIT_SCRIPT,
                    "migrate.sh" = MIGRATION_SCRIPT
                }
            }
        };

        resource! {
            &this, resource "kubernetes_stateful_set" "gitea" {
                metadata {
                    namespace = &this.namespace
                    name = name
                }
                spec {
                    replicas = "1"
                    service_name = &service.metadata[0].name
                    selector {
                        match_labels = &labels
                    }
                    template {
                        metadata {
                            labels = &labels
                        }
                        spec {
                            init_container {
                                name = "init"
                                image = "gitea/gitea:1.19.0-rootless"
                                command = ["bash", "/usr/sbin/init.sh"]
                                volume_mount {
                                    name = "giteadata"
                                    mount_path = "/gitea"
                                }
                                env_from {
                                    config_map_ref {
                                        name = &config.metadata[0].name
                                    }
                                }
                                volume_mount {
                                    name = "init-scripts"
                                    mount_path = "/usr/sbin"
                                }
                                security_context {
                                    run_as_user = "0"
                                }
                            }
                            init_container {
                                name = "init-gitea"
                                image = "gitea/gitea:1.19.0-rootless"
                                command = ["bash", "/usr/sbin/migrate.sh"]
                                volume_mount {
                                    name = "giteadata"
                                    mount_path = "/gitea"
                                }
                                env_from {
                                    config_map_ref {
                                        name = &config.metadata[0].name
                                    }
                                }
                                volume_mount {
                                    name = "init-scripts"
                                    mount_path = "/usr/sbin"
                                }
                            }
                            container {
                                name = "gitea"
                                image = "gitea/gitea:1.19.0-rootless"
                                port {
                                    name = "http"
                                    container_port = 3000
                                }
                                port {
                                    name = "ssh"
                                    container_port = 22
                                }
                                volume_mount {
                                    name = "giteadata"
                                    mount_path = "/gitea"
                                }
                                env_from {
                                    config_map_ref {
                                        name = &config.metadata[0].name
                                    }
                                }
                                liveness_probe {
                                    http_get {
                                        path = "/api/healthz"
                                        port = "http"
                                    }
                                }
                            }
                            volume {
                                name = "giteadata"
                                persistent_volume_claim {
                                    claim_name = &this.volume_claim
                                }
                            }
                            volume {
                                name = "init-scripts"
                                secret {
                                    secret_name = &init_config.metadata[0].name
                                }
                            }
                        }
                    }
                }
            }
        };

        this
    }
}

const INIT_SCRIPT: &str = r#"#!/usr/bin/env bash
echo "running as user: $UID"
echo "===== Prepare /gitea ====="
set -xeo pipefail
mkdir -p "/gitea/custom/conf"
chown -R 1000:1000 "/gitea"
echo "DONE"
"#;

const MIGRATION_SCRIPT: &str = r#"#!/usr/bin/env bash
echo "running as user: $UID"
echo "===== Run Migrations ====="
set -xeo pipefail
environment-to-ini
gitea migrate -c /gitea/custom/conf/app.ini
echo "DONE"
"#;
