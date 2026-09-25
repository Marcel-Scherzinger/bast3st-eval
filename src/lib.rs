pub mod catchable;
pub mod evaluation;
pub mod messages;
pub mod spec;

pub use evaluation::Features;
pub use messages::Messages;

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use crate::{
        Features,
        evaluation::{SelectableSource, SingleEvaluation},
        spec::{self, Array, RuntimeAny, RuntimeValue},
    };

    pub struct DummyData {
        data: RuntimeAny,
    }
    impl SelectableSource for DummyData {
        async fn request<'a>(
            &'a self,
            _selector: &spec::Selector,
            _allowed_features: Features,
        ) -> Result<Cow<'a, spec::RuntimeAny>, spec::FatalError> {
            Ok(Cow::Borrowed(&self.data))
        }
    }

    #[tokio::test]
    async fn t() {
        let _ = dotenvy::dotenv();
        env_logger::init();

        let s = std::fs::read_to_string("data/compare.json").unwrap();
        let v: spec::Bast3StSpec = serde_json::from_str(&s).unwrap();
        let entities = v.entities();
        let output: Array = [1, 2, 3]
            .iter()
            .map(|x| RuntimeValue::Prim(spec::PrimitiveValue::Str(x.to_string().into())))
            .collect();
        let selectable = DummyData {
            data: RuntimeAny::from(output),
        };
        let eval =
            SingleEvaluation::new(entities, 13.into(), &selectable, Features::all()).unwrap();
        let eval = eval.run_to_end_with_early_return().await;
        panic!(
            "{:?} {:#?}\n\n\n{entities:#?}",
            eval.value(),
            eval.all_values()
        );
    }
}
