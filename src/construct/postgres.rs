use std::rc::Rc;

use derive_builder::Builder;
use tf_bindgen::codegen::Construct;
use tf_bindgen::value::Value;
use tf_bindgen::Scope;

#[derive(Builder, Construct)]
#[builder(build_fn(private, name = "build_"))]
pub struct Postgres {
    #[id]
    #[builder(private)]
    name: String,
    #[scope]
    #[builder(private)]
    __m_scope: Rc<dyn Scope>,
    #[builder(setter(into))]
    namespace: Value<Option<String>>,
}

impl Postgres {
    /// Creates a new builder used to build this construct.
    pub fn create<C: Scope + 'static>(scope: &Rc<C>, name: impl Into<String>) -> PostgresBuilder {
        let mut builder = PostgresBuilder::default();
        builder.__m_scope(scope.clone()).name(name.into());
        builder
    }
}

impl PostgresBuilder {
    pub fn build(&mut self) -> Rc<Postgres> {
        let postgres = Rc::new(self.build_().expect("missing field"));

        // TODO: Setup

        postgres
    }
}
