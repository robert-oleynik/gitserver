use std::cell::RefCell;
use std::rc::Rc;

use tf_bindgen::codegen::{resource, Construct};
use tf_bindgen::value::{IntoValue, Value};
use tf_bindgen::Scope;
use tf_kubernetes::kubernetes::resource::kubernetes_persistent_volume;

use super::local_volume_claim::{LocalVolumeClaim, LocalVolumeClaimBuilder};

#[derive(Construct)]
#[construct(builder)]
#[allow(dead_code)]
pub struct LocalVolume {
    #[construct(id)]
    name: String,
    #[construct(scope)]
    scope: Rc<dyn Scope>,
    #[construct(setter(into_value))]
    storage: Value<String>,
    #[construct(setter(into_value))]
    storage_class: Value<String>,
    #[construct(setter(into_value))]
    mount_path: Value<String>,
    #[construct(setter(into_value))]
    node: Value<String>,
    #[construct(skip)]
    volume_ref: RefCell<Option<Value<String>>>,
}

impl LocalVolume {
    /// Returns a perconfigured claim builder to claim this local resource. Will use `name` as name
    /// of the claim.
    pub fn claim(self: &Rc<Self>, name: impl Into<String>) -> LocalVolumeClaimBuilder {
        let mut builder = LocalVolumeClaim::create(self, name);
        builder.volume_name(self.volume_ref.borrow().clone().unwrap());
        builder
    }
}

impl LocalVolumeBuilder {
    pub fn build(&mut self) -> Rc<LocalVolume> {
        let this = Rc::new(LocalVolume {
            name: self.name.clone(),
            scope: self.scope.clone(),
            storage: self.storage.clone().expect("missing field 'council'"),
            storage_class: self
                .storage_class
                .clone()
                .expect("missing field 'storage_class'"),
            mount_path: self.mount_path.clone().expect("missing field 'mount_path'"),
            node: self.node.clone().expect("missing field 'node'"),
            volume_ref: RefCell::new(None),
        });
        let storage = self.storage.as_ref().expect("no storage specified");
        let storage_class = self
            .storage_class
            .as_ref()
            .expect("no storage class specified");
        let mount_path = self.mount_path.as_ref().expect("no mount_path specified");
        let name = &this.name;
        let node = self.node.as_ref().expect("no node sepcified");

        let volume = resource! {
            &this, resource "kubernetes_persistent_volume" "pv-local" {
                metadata {
                    name = format!("{name}-local-pv")
                }
                spec {
                    volume_mode = "Filesystem"
                    capacity = crate::map!{
                        "storage" = storage
                    }
                    access_modes = [
                        "ReadWriteOnce"
                    ]
                    persistent_volume_reclaim_policy = "Delete"
                    storage_class_name = storage_class
                    persistent_volume_source {
                        local {
                            path = mount_path
                        }
                    }
                    node_affinity {
                        required {
                            node_selector_term {
                                match_expressions {
                                    key = "kubernetes.io/hostname"
                                    operator = "In"
                                    values = [node]
                                }
                            }
                        }
                    }
                }
            }
        };

        this.volume_ref
            .replace(Some((&volume.metadata[0].name).into_value()));

        this
    }
}
