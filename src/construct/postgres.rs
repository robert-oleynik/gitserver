use std::rc::Rc;

use tf_bindgen::codegen::{resource, Construct};
use tf_bindgen::value::Value;
use tf_bindgen::Scope;
use tf_kubernetes::kubernetes::resource::{kubernetes_service, kubernetes_stateful_set};

#[derive(Construct)]
#[construct(builder)]
pub struct Postgres {
    #[construct(id)]
    name: String,
    #[construct(scope)]
    scope: Rc<dyn Scope>,
    #[construct(setter(into_value))]
    namespace: Value<String>,
    #[construct(setter(into_value))]
    db_name: Value<String>,
    #[construct(setter(into_value))]
    user: Value<String>,
    #[construct(setter(into_value))]
    password: Value<String>,
    #[construct(setter(into_value))]
    volume_claim: Value<String>,
}

impl PostgresBuilder {
    pub fn build(&mut self) -> Rc<Postgres> {
        let this = Rc::new(Postgres {
            scope: self.scope.clone(),
            name: self.name.clone(),
            namespace: self.namespace.clone().expect("missing field 'namespace'"),
            db_name: self.db_name.clone().expect("missing field 'db_name'"),
            user: self.user.clone().expect("missing field 'user'"),
            password: self.password.clone().expect("missing field 'password'"),
            volume_claim: self
                .volume_claim
                .clone()
                .expect("missing field 'volume_claim'"),
        });

        let name = &this.name;
        let labels = crate::map! {
            "app" = format!("postgres-{name}"),
        };

        let service = resource! {
            &this, resource "kubernetes_service" "postgres" {
                metadata {
                    namespace = &this.namespace
                    name = format!("postgres-{name}")
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

        let user_str: &str = &this.user.get();
        let db_name_str: &str = &this.db_name.get();

        resource! {
            &this, resource "kubernetes_stateful_set" "postgres" {
                metadata {
                    namespace = &this.namespace
                    name = format!("postgres-{name}")
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
                                name = "postgres"
                                image = "postgres:15.2-alpine"
                                port {
                                    name = "db"
                                    container_port = 5432
                                }
                                volume_mount {
                                    name = "pgdata"
                                    mount_path = "/var/lib/postgresql/data"
                                }
                                env {
                                    name = "POSTGRES_DB"
                                    value = &this.db_name
                                }
                                env {
                                    name = "POSTGRES_USER"
                                    value = &this.user
                                }
                                env {
                                    name = "POSTGRES_PASSWORD"
                                    value = &this.password
                                }
                                liveness_probe {
                                    exec {
                                        command = ["psql", "-w", "-U", user_str, "-d", db_name_str, "-c", "SELECT 1"]
                                    }
                                    success_threshold = 1
                                    failure_threshold = 10
                                    period_seconds = 12
                                    timeout_seconds = 5
                                }
                                readiness_probe {
                                    exec {
                                        command = ["psql", "-w", "-U", user_str, "-d", db_name_str, "-c", "SELECT 1"]
                                    }
                                }
                            }
                            volume {
                                name = "pgdata"
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
