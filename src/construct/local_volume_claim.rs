use std::cell::{Ref, RefCell};
use std::rc::Rc;

use derive_builder::Builder;
use tf_bindgen::codegen::{resource, Construct};
use tf_bindgen::value::Value;
use tf_bindgen::Scope;

use tf_kubernetes::kubernetes::resource::kubernetes_persistent_volume_claim;

#[derive(Builder, Construct)]
#[builder(build_fn(private, name = "build_"))]
pub struct LocalVolumeClaim {
    #[id]
    #[builder(private)]
    name: String,
    #[scope]
    #[builder(private)]
    scope: Rc<dyn Scope>,
    #[builder(setter(into))]
    namespace: Value<Option<String>>,
    #[builder(setter(into))]
    storage: String,
    #[builder(setter(into))]
    volume_name: Value<Option<String>>,
    #[builder(setter(skip), default = "RefCell::new(Value::from(None::<String>))")]
    claim_ref: RefCell<Value<Option<String>>>,
}

impl LocalVolumeClaim {
    pub fn create<C: Scope + 'static>(
        scope: &Rc<C>,
        name: impl Into<String>,
    ) -> LocalVolumeClaimBuilder {
        let mut builder = LocalVolumeClaimBuilder::default();
        builder.scope(scope.clone()).name(name.into());
        builder
    }

    /// Returns a Terraform value reference to the name of the generated volume claim.
    pub fn claim(&self) -> Ref<'_, Value<Option<String>>> {
        self.claim_ref.borrow()
    }
}

impl LocalVolumeClaimBuilder {
    pub fn build(&mut self) -> Rc<LocalVolumeClaim> {
        let this = Rc::new(self.build_().expect("missing field"));
        let name = &this.name;

        let claim = resource! {
            &this, resource "kubernetes_persistent_volume_claim" "claim" {
                metadata {
                    namespace = this.namespace.clone()
                    name = format!("{name}-pvc")
                }
                spec {
                    volume_name = this.volume_name.clone()
                    access_modes = [
                        "ReadWriteOnce"
                    ]
                    resources {
                        requests = crate::map!{
                            "storage" = &this.storage
                        }
                    }
                }
            }
        };
        this.claim_ref.replace((&claim.metadata[0].name).into());
        this
    }
}
