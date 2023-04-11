use std::rc::Rc;

use tf_bindgen::codegen::{resource, Construct};
use tf_bindgen::{Scope, Value};
use tf_kubernetes::kubernetes::resource::kubernetes_ingress_v1::{self, *};

#[derive(Clone)]
pub struct IngressServiceConfig {
    pub path: String,
    pub service_name: String,
    pub service_port: i64,
}

#[derive(Construct)]
#[construct(builder)]
#[allow(dead_code)]
pub struct Ingress {
    #[construct(id)]
    name: String,
    #[construct(scope)]
    scope: Rc<dyn Scope>,
    #[construct(setter(into_value))]
    namespace: Value<String>,
    #[construct(setter(into))]
    services: Vec<IngressServiceConfig>,
}

impl IngressBuilder {
    pub fn build(&mut self) -> Rc<Ingress> {
        let this = Rc::new(Ingress {
            name: self.name.clone(),
            scope: self.scope.clone(),
            namespace: self.namespace.clone().expect("missing field namespace"),
            services: self.services.clone().expect("missing field services"),
        });
        let name = &self.name;

        let paths: Vec<_> = this
            .services
            .iter()
            .map(|config| {
                let port = KubernetesIngressV1SpecRuleHttpPathBackendServicePort::builder()
                    .number(config.service_port)
                    .build();
                let service = KubernetesIngressV1SpecRuleHttpPathBackendService::builder()
                    .name(&config.service_name)
                    .port(port)
                    .build();
                let backend = KubernetesIngressV1SpecRuleHttpPathBackend::builder()
                    .service(service)
                    .build();
                KubernetesIngressV1SpecRuleHttpPath::builder()
                    .path_type("Prefix")
                    .path(&config.path)
                    .backend(backend)
                    .build()
            })
            .collect();
        resource! {
            &this, resource "kubernetes_ingress_v1" "ingress" {
                metadata {
                    namespace = &this.namespace
                    name = format!("{name}-ingress")
                }
                spec {
                    rule {
                        http {
                            path = paths
                        }
                    }
                }
            }
        };
        this
    }
}
