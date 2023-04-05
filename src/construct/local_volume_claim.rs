use std::cell::{Ref, RefCell};
use std::rc::Rc;

use tf_bindgen::codegen::{resource, Construct};
use tf_bindgen::value::{IntoValue, Value};
use tf_bindgen::Scope;

use tf_kubernetes::kubernetes::resource::kubernetes_persistent_volume_claim;

#[derive(Construct)]
pub struct LocalVolumeClaim {
    #[id]
    name: String,
    #[scope]
    scope: Rc<dyn Scope>,
    claim_ref: RefCell<Option<Value<String>>>,
}

pub struct LocalVolumeClaimBuilder {
    name: String,
    scope: Rc<dyn Scope>,
    namespace: Option<Value<String>>,
    storage: Option<Value<String>>,
    volume_name: Option<Value<String>>,
}

impl LocalVolumeClaim {
    pub fn create<C: Scope + 'static>(
        scope: &Rc<C>,
        name: impl Into<String>,
    ) -> LocalVolumeClaimBuilder {
        LocalVolumeClaimBuilder {
            name: name.into(),
            scope: scope.clone(),
            namespace: None,
            storage: None,
            volume_name: None,
        }
    }

    /// Returns a Terraform value reference to the name of the generated volume claim.
    pub fn claim(&self) -> Ref<'_, Option<Value<String>>> {
        self.claim_ref.borrow()
    }
}

impl LocalVolumeClaimBuilder {
    pub fn storage(&mut self, value: impl IntoValue<String>) -> &mut Self {
        self.storage = Some(value.into_value());
        self
    }

    pub fn namespace(&mut self, value: impl IntoValue<String>) -> &mut Self {
        self.namespace = Some(value.into_value());
        self
    }

    pub fn volume_name(&mut self, value: impl IntoValue<String>) -> &mut Self {
        self.volume_name = Some(value.into_value());
        self
    }

    pub fn build(&mut self) -> Rc<LocalVolumeClaim> {
        let this = Rc::new(LocalVolumeClaim {
            name: self.name.clone(),
            scope: self.scope.clone(),
            claim_ref: RefCell::new(None),
        });
        let name = &this.name;
        let namespace = self.namespace.as_ref().expect("missing field 'namespace'");
        let storage = self.storage.as_ref().expect("missing field 'storage'");
        let volume_name = self
            .volume_name
            .as_ref()
            .expect("missing field 'volume_name'");

        let claim = resource! {
            &this, resource "kubernetes_persistent_volume_claim" "claim" {
                metadata {
                    namespace = namespace
                    name = format!("{name}-pvc")
                }
                spec {
                    volume_name = volume_name
                    access_modes = [
                        "ReadWriteOnce"
                    ]
                    resources {
                        requests = crate::map!{
                            "storage" = storage
                        }
                    }
                }
            }
        };
        this.claim_ref
            .replace(Some((&claim.metadata[0].name).into_value()));
        this
    }
}
