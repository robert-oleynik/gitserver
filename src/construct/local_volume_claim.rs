use std::cell::{Ref, RefCell};
use std::rc::Rc;

use tf_bindgen::codegen::{resource, Construct};
use tf_bindgen::value::{IntoValue, Value};
use tf_bindgen::Scope;

use tf_kubernetes::kubernetes::resource::kubernetes_persistent_volume_claim;

#[derive(Construct)]
#[construct(builder)]
#[allow(dead_code)]
pub struct LocalVolumeClaim {
    #[construct(id)]
    name: String,
    #[construct(scope)]
    scope: Rc<dyn Scope>,
    #[construct(setter(into_value))]
    namespace: Value<String>,
    #[construct(setter(into_value))]
    storage: Value<String>,
    #[construct(setter(into_value))]
    storage_class: Value<String>,
    #[construct(setter(into_value))]
    volume_name: Value<String>,
    #[construct(skip)]
    claim_ref: RefCell<Option<Value<String>>>,
}

impl LocalVolumeClaim {
    /// Returns a Terraform value reference to the name of the generated volume claim.
    pub fn claim(&self) -> Ref<'_, Option<Value<String>>> {
        self.claim_ref.borrow()
    }
}

impl LocalVolumeClaimBuilder {
    pub fn build(&mut self) -> Rc<LocalVolumeClaim> {
        let this = Rc::new(LocalVolumeClaim {
            name: self.name.clone(),
            scope: self.scope.clone(),
            namespace: self.namespace.clone().expect("missing field 'namespace'"),
            storage: self.storage.clone().expect("missing field 'storage'"),
            storage_class: self
                .storage_class
                .clone()
                .expect("missing field 'storage_class'"),
            volume_name: self
                .volume_name
                .clone()
                .expect("missing field 'volume_name'"),
            claim_ref: RefCell::new(None),
        });
        let name = &this.name;
        let namespace = self.namespace.as_ref().expect("missing field 'namespace'");
        let storage = self.storage.as_ref().expect("missing field 'storage'");
        let storage_class = self
            .storage_class
            .as_ref()
            .expect("missing field 'storage_class'");
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
                    storage_class_name = storage_class
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
