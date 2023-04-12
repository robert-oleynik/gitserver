use std::rc::Rc;

use tf_bindgen::codegen::{resource, Construct};
use tf_bindgen::value::IntoValue;
use tf_bindgen::{Scope, Value};
use tf_kubernetes::kubernetes::resource::{
    kubernetes_config_map, kubernetes_secret, kubernetes_service, kubernetes_stateful_set,
};

use super::ingress::IngressServiceConfig;

const INIT_SCRIPT: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/script/gitea/init.sh"));
const MIGRATION_SCRIPT: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/script/gitea/migrate.sh"
));

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
    cache_host: Value<String>,
    #[construct(setter(into_value))]
    db_host: Value<String>,
    #[construct(setter(into_value))]
    db_name: Value<String>,
    #[construct(setter(into_value))]
    db_user: Value<String>,
    #[construct(setter(into_value))]
    db_password: Value<String>,
    #[construct(setter(into_value))]
    root_user: Value<String>,
    #[construct(setter(into_value))]
    root_passwd: Value<String>,
    #[construct(setter(into_value))]
    root_email: Value<String>,
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
            cache_host: self.cache_host.clone().unwrap_or("localhost".into_value()),
            db_host: self.db_host.clone().unwrap_or("localhost".into_value()),
            db_name: self.db_name.clone().expect("missing field 'db_name'"),
            db_user: self.db_user.clone().expect("missing field 'db_user'"),
            db_password: self
                .db_password
                .clone()
                .expect("missing field 'db_password'"),
            root_user: self.root_user.clone().expect("missing field 'root_user'"),
            root_passwd: self
                .root_passwd
                .clone()
                .expect("missing field 'root_passwd'"),
            root_email: self.root_email.clone().expect("missing field 'root_email'"),
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

        let init_root_config = resource! {
            &this, resource "kubernetes_secret" "gitea-init-root-config" {
                metadata {
                    namespace = &this.namespace
                    name = name
                }
                data = crate::map! {
                    "INSTALL_LOCK" = "true",
                    "ROOT_USER" = &this.root_user,
                    "ROOT_PASSWD" = &this.root_passwd,
                    "ROOT_EMAIL" = &this.root_email
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
                    "GITEA_APP_INI" = "/gitea/custom/conf/app.ini",
                    "GITEA__database__DB_TYPE" = "postgres",
                    "GITEA__database__HOST" = &this.db_host,
                    "GITEA__database__NAME" = &this.db_name,
                    "GITEA__database__USER" = &this.db_user,
                    "GITEA__database__PASSWD" = &this.db_password,
                    "GITEA__server__ROOT_URL" = format!("https://%(DOMAIN)s{}", this.path),
                    "GITEA__server__DOMAIN" = &this.domain,
                    "GITEA__cache__ADAPTER" = "memcache",
                    "GITEA__cache__HOST" = &this.cache_host
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
                                env_from {
                                    secret_ref {
                                        name = &init_root_config.metadata[0].name
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
                                readiness_probe {
                                    http_get {
                                        path = "/api/healthz"
                                        port = "http"
                                    }
                                }
                                liveness_probe {
                                    http_get {
                                        path = "/api/healthz"
                                        port = "http"
                                    }
                                    success_threshold = 1
                                    failure_threshold = 10
                                    period_seconds = 12
                                    timeout_seconds = 5
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
