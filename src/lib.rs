#![allow(unused)]

pub mod catchable;
pub mod evaluation;
pub mod spec;

pub use evaluation::Features;

#[cfg(test)]
mod tests {
    use crate::{
        evaluation::SelectableSource,
        spec::{Array, Numeric, RuntimeAny, RuntimeValue},
    };

    pub struct SelectableData {
        output: RuntimeAny,
    }
    impl SelectableSource for SelectableData {
        async fn request<'a>(
            &'a self,
            selector: &spec::Selector,
            allowed_features: Features,
        ) -> Result<&'a spec::RuntimeAny, spec::FatalError> {
            return Ok(&self.output);
        }
    }

    use super::*;

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
        let selectable = SelectableData {
            output: RuntimeAny::from(output),
        };
        let mut eval =
            evaluation::SingleEvaluation::new(entities, 13.into(), &selectable, Features::all())
                .unwrap();
        let eval = eval.run_to_end().await;
        panic!(
            "{:?} {:#?}\n\n\n{entities:#?}",
            eval.value(),
            eval.all_values()
        );
    }
}
