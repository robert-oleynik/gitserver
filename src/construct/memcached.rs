use std::rc::Rc;

use tf_bindgen::codegen::{resource, Construct};
use tf_bindgen::{Scope, Value};
use tf_kubernetes::kubernetes::resource::{kubernetes_deployment, kubernetes_service};

#[derive(Construct)]
#[construct(builder)]
pub struct Memcached {
    #[construct(id)]
    name: String,
    #[construct(scope)]
    scope: Rc<dyn Scope>,
    #[construct(setter(into_value))]
    namespace: Value<String>,
    #[construct(setter(into_value))]
    memory_limit: Value<String>,
}

impl MemcachedBuilder {
    pub fn build(&mut self) -> Rc<Memcached> {
        let this = Rc::new(Memcached {
            name: self.name.clone(),
            scope: self.scope.clone(),
            namespace: self.namespace.clone().expect("missing field 'namespace'"),
            memory_limit: self
                .memory_limit
                .clone()
                .expect("missing field 'memory_limit'"),
        });

        let name = &this.name;
        let labels = crate::map! {
            "app" = format!("memcached-{name}")
        };

        resource! {
            &this, resource "kubernetes_service" "memcached" {
                metadata {
                    namespace = &this.namespace
                    name = name
                }
                spec {
                    selector = &labels
                    port {
                        port = 11211
                    }
                }
            }
        };

        resource! {
            &this, resource "kubernetes_deployment" "memcached" {
                metadata {
                    namespace = &this.namespace
                    name = name
                }
                spec {
                    replicas = "1"
                    selector {
                        match_labels = &labels
                    }
                    template {
                        metadata {
                            labels = &labels
                        }
                        spec {
                            container {
                                name = "memcached"
                                image = "memcached:1.6.19-alpine"
                                port {
                                    container_port = 11211
                                }
                                liveness_probe {
                                    tcp_socket {
                                        port = "11211"
                                    }
                                    success_threshold = 1
                                    failure_threshold = 10
                                    period_seconds = 12
                                    timeout_seconds = 5
                                }
                                resources {
                                    requests = crate::map! {
                                        "memory" = &this.memory_limit
                                    }
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
