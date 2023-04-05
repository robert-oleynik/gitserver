use std::cell::RefCell;
use std::rc::Rc;

use tf_bindgen::codegen::{resource, Construct};
use tf_bindgen::value::{IntoValue, Value};
use tf_bindgen::Scope;
use tf_kubernetes::kubernetes::resource::kubernetes_persistent_volume;

use super::local_volume_claim::{LocalVolumeClaim, LocalVolumeClaimBuilder};

#[derive(Construct)]
pub struct LocalVolume {
    #[id]
    name: String,
    #[scope]
    scope: Rc<dyn Scope>,
    volume_ref: RefCell<Option<Value<String>>>,
}

pub struct LocalVolumeBuilder {
    name: String,
    scope: Rc<dyn Scope>,
    storage: Option<Value<String>>,
    storage_class: Option<Value<String>>,
    mount_path: Option<Value<String>>,
    node: Option<Value<String>>,
}

impl LocalVolume {
    /// Creates a new builder used to provision a local volume.
    pub fn create<C: Scope + 'static>(
        scope: &Rc<C>,
        name: impl Into<String>,
    ) -> LocalVolumeBuilder {
        LocalVolumeBuilder {
            name: name.into(),
            scope: scope.clone(),
            storage: None,
            storage_class: None,
            mount_path: None,
            node: None,
        }
    }

    /// Returns a perconfigured claim builder to claim this local resource. Will use `name` as name
    /// of the claim.
    pub fn claim(self: &Rc<Self>, name: impl Into<String>) -> LocalVolumeClaimBuilder {
        let mut builder = LocalVolumeClaim::create(self, name);
        builder.volume_name(self.volume_ref.borrow().clone().unwrap());
        builder
    }
}

impl LocalVolumeBuilder {
    pub fn storage(&mut self, value: impl IntoValue<String>) -> &mut Self {
        self.storage = Some(value.into_value());
        self
    }

    pub fn storage_class(&mut self, value: impl IntoValue<String>) -> &mut Self {
        self.storage_class = Some(value.into_value());
        self
    }

    pub fn mount_path(&mut self, value: impl IntoValue<String>) -> &mut Self {
        self.mount_path = Some(value.into_value());
        self
    }

    pub fn node(&mut self, value: impl IntoValue<String>) -> &mut Self {
        self.node = Some(value.into_value());
        self
    }

    pub fn build(&mut self) -> Rc<LocalVolume> {
        let this = Rc::new(LocalVolume {
            name: self.name.clone(),
            scope: self.scope.clone(),
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
