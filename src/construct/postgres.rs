use std::rc::Rc;

use derive_builder::Builder;
use tf_bindgen::codegen::{resource, Construct};
use tf_bindgen::value::Value;
use tf_bindgen::Scope;
use tf_kubernetes::kubernetes::resource::{kubernetes_service, kubernetes_stateful_set};

#[derive(Builder, Construct)]
#[builder(build_fn(private, name = "build_"))]
pub struct Postgres {
    #[id]
    #[builder(private)]
    name: String,
    #[scope]
    #[builder(private)]
    __m_scope: Rc<dyn Scope>,
    #[builder(setter(into))]
    namespace: Value<Option<String>>,
    #[builder(setter(into))]
    volume_claim: Value<Option<String>>,
}

impl Postgres {
    /// Creates a new builder used to build this construct.
    pub fn create<C: Scope + 'static>(scope: &Rc<C>, name: impl Into<String>) -> PostgresBuilder {
        let mut builder = PostgresBuilder::default();
        builder.__m_scope(scope.clone()).name(name.into());
        builder
    }
}

impl PostgresBuilder {
    pub fn build(&mut self) -> Rc<Postgres> {
        let this = Rc::new(self.build_().expect("missing field"));

        let name = &this.name;
        let labels = crate::map! {
            "app" = format!("postgres-{name}"),
        };

        resource! {
            &this,
            resource "kubernetes_service" "postgres" {
                metadata {
                    namespace = this.namespace.clone()
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
                    namespace = this.namespace.clone()
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
                            }
                            volume {
                                name = "pgdata"
                                persistent_volume_claim {
                                    claim_name = this.volume_claim.clone()
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
