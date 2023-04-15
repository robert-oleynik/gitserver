use std::rc::Rc;

use tf_bindgen::codegen::{resource, Construct};
use tf_bindgen::value::Value;
use tf_bindgen::Scope;
use tf_kubernetes::kubernetes::resource::{
    kubernetes_cluster_role, kubernetes_cluster_role_binding, kubernetes_config_map,
    kubernetes_deployment, kubernetes_secret, kubernetes_service, kubernetes_service_account,
};

use super::ingress::IngressServiceConfig;

#[derive(Construct)]
#[construct(builder)]
pub struct Jenkins {
    #[construct(id)]
    name: String,
    #[construct(scope)]
    scope: Rc<dyn Scope>,
    #[construct(setter(into_value))]
    namespace: Value<String>,
    #[construct(setter(into))]
    path: String,
    #[construct(setter(into))]
    domain: String,
}

impl Jenkins {
    pub fn ingress(&self) -> IngressServiceConfig {
        IngressServiceConfig {
            rewrite: false,
            path: self.path.clone(),
            service_name: format!("{}-service", self.name),
            service_port: 8080,
        }
    }
}

impl JenkinsBuilder {
    pub fn build(&mut self) -> Rc<Jenkins> {
        let this = Rc::new(Jenkins {
            name: self.name.clone(),
            scope: self.scope.clone(),
            path: self.path.clone().expect("missing field 'path'"),
            domain: self.domain.clone().expect("missing field 'domain'"),
            namespace: self.namespace.clone().expect("missing field 'namespace'"),
        });

        let name = &this.name;
        let path = &this.path;
        let domain = &this.domain;
        let labels = crate::map! {
            "app" = name
        };

        let cluster_role = resource! {
            &this, resource "kubernetes_cluster_role" "jenkins-admin" {
                metadata {
                    name = name
                }
                rule {
                    api_groups = [""]
                    resources = ["*"]
                    verbs = ["*"]
                }
            }
        };

        let service_account = resource! {
            &this, resource "kubernetes_service_account" "jenkins-admin" {
                metadata {
                    namespace = &this.namespace
                    name = name
                }
            }
        };

        resource! {
            &this, resource "kubernetes_cluster_role_binding" "jenkins-admin" {
                metadata {
                    name = name
                }
                role_ref {
                    api_group = "rbac.authorization.k8s.io"
                    kind = "ClusterRole"
                    name = &cluster_role.metadata[0].name
                }
                subject {
                    kind = "ServiceAccount"
                    name = &cluster_role.metadata[0].name
                    namespace = &this.namespace
                }
            }
        };

        let casc = resource! {
            &this, resource "kubernetes_secret" "jenkins-casc" {
                metadata {
                    namespace = &this.namespace
                    name = format!("{name}-casc")
                }
                data = crate::map! {
                    "casc.yaml" = format!(r#"
unclassified:
  location:
    url: https://{domain}{path}
"#),
                    "install-plugins.sh" = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/script/jenkins/install-plugins.sh"))
                }
            }
        };

        let config = resource! {
            &this, resource "kubernetes_config_map" "jenkins" {
                metadata {
                    namespace = &this.namespace
                    name = name
                }
                data = crate::map! {
                    "JENKINS_OPTS" = format!("--prefix={}", this.path),
                    "JAVA_OPTS" = format!("-Djenkins.install.runSetupWizard=false -Dcasc.reload.token={name}"),
                    "CASC_JENKINS_CONFIG" = "/config"
                }
            }
        };

        resource! {
            &this, resource "kubernetes_deployment" "jenkins-server" {
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
                            security_context {
                                fs_group = "1000"
                                run_as_user = "1000"
                                run_as_group = "100"
                                run_as_non_root = true
                            }
                            service_account_name = &service_account.metadata[0].name
                            init_container {
                                name = "install-plugins"
                                image = "jenkins/jenkins:2.400"
                                command = ["bash", "/config/install-plugins.sh"]
                                volume_mount {
                                    name = "jenkins-data"
                                    mount_path = "/var/jenkins_home"
                                }
                                volume_mount {
                                    name = "jenkins-casc"
                                    mount_path = "/config"
                                }
                                env_from {
                                    config_map_ref {
                                        name = &config.metadata[0].name
                                    }
                                }
                            }
                            container {
                                name = "jenkins"
                                image = "jenkins/jenkins:2.400"
                                port {
                                    name = "http"
                                    container_port = 8080
                                }
                                port {
                                    name = "jnlp"
                                    container_port = 50000
                                }
                                liveness_probe {
                                    http_get {
                                        path = format!("{}/login", this.path)
                                        port = "8080"
                                    }
                                    period_seconds = 12
                                    timeout_seconds = 5
                                    failure_threshold = 10
                                }
                                readiness_probe {
                                    http_get {
                                        path = format!("{}/login", this.path)
                                        port = "8080"
                                    }
                                    period_seconds = 12
                                    timeout_seconds = 5
                                    failure_threshold = 10
                                }
                                volume_mount {
                                    name = "jenkins-data"
                                    mount_path = "/var/jenkins_home"
                                }
                                volume_mount {
                                    name = "jenkins-casc"
                                    mount_path = "/config"
                                }
                                env_from {
                                    config_map_ref {
                                        name = &config.metadata[0].name
                                    }
                                }
                            }
                            volume {
                                name = "jenkins-data"
                                empty_dir{}
                            }
                            volume {
                                name = "jenkins-casc"
                                secret {
                                    secret_name = &casc.metadata[0].name
                                }
                            }
                        }
                    }
                }
            }
        };

        resource! {
            &this, resource "kubernetes_service" "jenkins" {
                metadata {
                    namespace = &this.namespace
                    name = format!("{name}-service")
                }
                spec {
                    r#type = "ClusterIP"
                    selector = &labels
                    port {
                        port = 8080
                    }
                }
            }
        };

        this
    }
}
