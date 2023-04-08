use std::cell::RefCell;
use std::rc::Rc;

use tf_bindgen::codegen::{resource, Construct};
use tf_bindgen::value::{IntoValue, Value};
use tf_bindgen::Scope;
use tf_kubernetes::kubernetes::resource::kubernetes_persistent_volume;

use super::local_dir_volume_claim::{LocalDirVolumeClaim, LocalDirVolumeClaimBuilder};

#[derive(Construct)]
#[construct(builder)]
#[allow(dead_code)]
pub struct LocalDirVolume {
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

impl LocalDirVolume {
    /// Returns a perconfigured claim builder to claim this local resource. Will use `name` as name
    /// of the claim.
    pub fn claim(self: &Rc<Self>, name: impl Into<String>) -> LocalDirVolumeClaimBuilder {
        let mut builder = LocalDirVolumeClaim::create(self, name);
        builder
            .volume_name(self.volume_ref.borrow().clone().unwrap())
            .storage(&self.storage)
            .storage_class(&self.storage_class);
        builder
    }
}

impl LocalDirVolumeBuilder {
    pub fn build(&mut self) -> Rc<LocalDirVolume> {
        let this = Rc::new(LocalDirVolume {
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
        let name = &this.name;

        let volume = resource! {
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
                    storage_class_name = &this.storage_class
                    persistent_volume_source {
                        host_path {
                            path = &this.mount_path
                            r#type = "DirectoryOrCreate"
                        }
                    }
                    node_affinity {
                        required {
                            node_selector_term {
                                match_expressions {
                                    key = "kubernetes.io/hostname"
                                    operator = "In"
                                    values = [&this.node]
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
