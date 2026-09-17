#![allow(unused)]

pub mod catchable;
pub mod evaluation;
pub mod spec;

#[cfg(test)]
mod tests {
    use crate::{
        evaluation::SelectableData,
        spec::{Numeric, RuntimeValue},
    };

    use super::*;

    #[tokio::test]
    async fn t() {
        let _ = dotenvy::dotenv();
        env_logger::init();

        let s = std::fs::read_to_string("data/compare.json").unwrap();
        let v: spec::Bast3StSpec = serde_json::from_str(&s).unwrap();
        let entities = v.entities();
        let output = vec![1, 2, 3]
            .iter()
            .map(|x| RuntimeValue::Prim(spec::PrimitiveValue::Str(x.to_string().into())))
            .collect();
        let selectable = SelectableData::new(output);
        let mut eval = evaluation::SingleEvaluation::new(entities, 5, &selectable).unwrap();
        eval.run_to_end().await;
        panic!(
            "{:?} {:#?}\n\n\n{entities:#?}",
            eval.value(),
            eval.all_values()
        );
    }
}
