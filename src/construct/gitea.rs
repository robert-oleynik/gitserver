use std::rc::Rc;

use tf_bindgen::codegen::{resource, Construct};
use tf_bindgen::{Scope, Value};
use tf_kubernetes::kubernetes::resource::{kubernetes_service, kubernetes_stateful_set};

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
    #[construct(setter(into_value))]
    volume_claim: Value<String>,
}

impl GiteaBuilder {
    pub fn build(&mut self) -> Rc<Gitea> {
        let this = Rc::new(Gitea {
            name: self.name.clone(),
            scope: self.scope.clone(),
            namespace: self.namespace.clone().expect("missing field 'namespace'"),
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
                    name = format!("gitea-{name}")
                }
                spec {
                    selector = &labels
                    port {
                        name = "db"
                        port = 5432
                    }
                }
            }
        };

        resource! {
            &this, resource "kubernetes_stateful_set" "gitea" {
                metadata {
                    namespace = &this.namespace
                    name = format!("gitea-{name}")
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
                            container {
                                name = "gitea"
                                image = "gitea/gitea:1.19.0"
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
                                env {
                                    name = "USER_UID"
                                    value = "1000"
                                }
                                env {
                                    name = "USER_GID"
                                    value = "1000"
                                }
                                env {
                                    name = "GITEA__database__DB_TYPE"
                                    value = "postgres"
                                }
                                env {
                                    name = "GITEA__database__NAME"
                                    value = "gitea"
                                }
                                env {
                                    name = "GITEA__database__USER"
                                    value = "gitea"
                                }
                                env {
                                    name = "GITEA__database__PASSWORD"
                                    value = "gitea"
                                }
                            }
                            volume {
                                name = "giteadata"
                                persistent_volume_claim {
                                    claim_name = &this.volume_claim
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
