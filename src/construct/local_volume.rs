use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;

use derive_builder::Builder;
use tf_bindgen::codegen::{resource, Construct};
use tf_bindgen::value::Value;
use tf_bindgen::Scope;
use tf_kubernetes::kubernetes::resource::kubernetes_persistent_volume;

use super::local_volume_claim::{LocalVolumeClaim, LocalVolumeClaimBuilder};

#[derive(Builder, Construct)]
#[builder(build_fn(private, name = "build_"))]
pub struct LocalVolume {
    #[id]
    #[builder(private)]
    name: String,
    #[scope]
    #[builder(private)]
    scope: Rc<dyn Scope>,
    #[builder(setter(into))]
    storage: String,
    #[builder(setter(into))]
    mount_path: Value<Option<String>>,
    #[builder(setter(into))]
    nodes: Value<Option<HashSet<String>>>,
    #[builder(setter(skip), default = "RefCell::new(Value::from(None::<String>))")]
    volume_ref: RefCell<Value<Option<String>>>,
}

impl LocalVolume {
    /// Creates a new builder used to provision a local volume.
    pub fn create<C: Scope + 'static>(
        scope: &Rc<C>,
        name: impl Into<String>,
    ) -> LocalVolumeBuilder {
        let mut builder = LocalVolumeBuilder::default();
        builder.scope(scope.clone()).name(name.into());
        builder
    }

    /// Returns a perconfigured claim builder to claim this local resource. Will use `name` as name
    /// of the claim.
    pub fn claim(self: &Rc<Self>, name: impl Into<String>) -> LocalVolumeClaimBuilder {
        let mut builder = LocalVolumeClaim::create(self, name);
        builder.volume_name(self.volume_ref.borrow().clone());
        builder
    }
}

impl LocalVolumeBuilder {
    pub fn build(&mut self) -> Rc<LocalVolume> {
        let this = Rc::new(self.build_().expect("missing field"));
        let name = &this.name;

        resource! {
            &this, resource "kubernetes_persistent_volume" "pv-local" {
                metadata {
                    name = format!("{name}-local-pv")
                }
                spec {
                    volume_mode = "Filesystem"
                    capacity = crate::map!{
                        "storage" = &this.storage
                    }
                    access_modes = [
                        "ReadWriteOnce"
                    ]
                    persistent_volume_reclaim_policy = "Delete"
                    // storage_class_name = "local-storage"
                    persistent_volume_source {
                        local {
                            path = this.mount_path.clone()
                        }
                    }
                    node_affinity {
                        required {
                            node_selector_term {
                                match_expressions {
                                    key = "kubernetes.io/hostname"
                                    operator = "In"
                                    values = this.nodes.clone()
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
