use std::rc::Rc;

use tf_bindgen::codegen::{resource, Construct};
use tf_bindgen::value::{IntoValue, Value};
use tf_bindgen::Scope;
use tf_kubernetes::kubernetes::resource::{kubernetes_service, kubernetes_stateful_set};

#[derive(Construct)]
pub struct Postgres {
    #[id]
    name: String,
    #[scope]
    scope: Rc<dyn Scope>,
}

pub struct PostgresBuilder {
    name: String,
    scope: Rc<dyn Scope>,
    namespace: Option<Value<String>>,
    volume_claim: Option<Value<String>>,
}

impl Postgres {
    /// Creates a new builder used to build this construct.
    pub fn create<C: Scope + 'static>(scope: &Rc<C>, name: impl Into<String>) -> PostgresBuilder {
        PostgresBuilder {
            name: name.into(),
            scope: scope.clone(),
            namespace: None,
            volume_claim: None,
        }
    }
}

impl PostgresBuilder {
    pub fn namespace(&mut self, value: impl IntoValue<String>) -> &mut Self {
        self.namespace = Some(value.into_value());
        self
    }

    pub fn volume_claim(&mut self, value: impl IntoValue<String>) -> &mut Self {
        self.volume_claim = Some(value.into_value());
        self
    }

    pub fn build(&mut self) -> Rc<Postgres> {
        let this = Rc::new(Postgres {
            scope: self.scope.clone(),
            name: self.name.clone(),
        });

        let name = &this.name;
        let namespace = self.namespace.as_ref().expect("missing field 'namespace'");
        let volume_claim = self
            .volume_claim
            .as_ref()
            .expect("missing field 'namespace'");
        let labels = crate::map! {
            "app" = format!("postgres-{name}"),
        };

        resource! {
            &this,
            resource "kubernetes_service" "postgres" {
                metadata {
                    namespace = namespace
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

        resource! {
            &this,
            resource "kubernetes_stateful_set" "postgres" {
                metadata {
                    namespace = namespace
                    name = format!("postgres-{name}")
                }
                spec {
                    replicas = "1"
                    service_name = format!("postgres-{name}")
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
                                    mount_path = "/var/lib/postgresql/data/pgdata"
                                }
                                env {
                                    name = "POSTGRES_PASSWORD"
                                    value = "example"
                                }
                            }
                            volume {
                                name = "pgdata"
                                persistent_volume_claim {
                                    claim_name = volume_claim
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
