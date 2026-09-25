pub mod catchable;
pub mod evaluation;
pub mod messages;
pub mod spec;

pub use evaluation::Features;
pub use messages::Messages;

#[cfg(test)]
mod tests {
    use std::{borrow::Cow, sync::Arc};

    use reqwest::Url;
    use scratch_test_interpreter::Limits;
    use scratch_test_model::{
        ProjectDoc,
        blocks::{BlockKindUnit, EventBlockKindUnit},
    };

    use crate::{
        Features,
        evaluation::{Context, PSpec, SelectableSource, SingleEvaluation},
        spec::{self, Array, RealMapping, RuntimeAny, RuntimeValue},
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

    #[test]
    fn test_urls() {
        let url: Url = "https://example.org:80/abc".parse().unwrap();
        let x = url.host_str().unwrap();
        assert_eq!("example.org", x);
        assert_eq!("https", url.scheme());
        assert_eq!(Some(80), url.port_or_known_default());
    }

    #[tokio::test]
    async fn single_eval() {
        let _ = dotenvy::dotenv();
        // env_logger::init();

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
        let eval = SingleEvaluation::new(
            entities,
            13.into(),
            &selectable,
            Features::all(),
            Arc::new(Ok),
        )
        .unwrap();
        let eval = eval.run_to_end_with_early_return().await;
        /*panic!(
            "{:?} {:#?}\n\n\n{entities:#?}",
            eval.value(),
            eval.all_values()
        );*/
    }

    #[derive(Debug, PartialEq, PartialOrd, Clone)]
    pub enum InitialBlockAmbiguity {
        No,
        Multiple,
    }
    const GREEN_FLAG: BlockKindUnit =
        BlockKindUnit::Event(EventBlockKindUnit::EventWhenflagclicked);

    fn find_initial_block(
        doc: &ProjectDoc,
    ) -> Result<&scratch_test_model::Id, InitialBlockAmbiguity> {
        let mut green_flags = doc
            .ids_with_opcodes()
            .filter_map(|(id, opcode)| (opcode == GREEN_FLAG).then_some(id));
        let first = green_flags.next();
        if green_flags.next().is_some() {
            Err(InitialBlockAmbiguity::Multiple)
        } else {
            first.ok_or(InitialBlockAmbiguity::No)
        }
    }

    #[tokio::test]
    async fn t() {
        let _ = dotenvy::dotenv();
        env_logger::init();

        let s = std::fs::read_to_string("data/dyn_loop.json").unwrap();
        let spec: spec::Bast3StSpec = serde_json::from_str(&s).unwrap();

        let json_doc = scratch_test_model::json_from_sb3_file("a1.sb3").unwrap();
        let doc = scratch_test_model::ProjectDoc::from_json(&json_doc).unwrap();
        let initial_block = find_initial_block(&doc).unwrap().clone();

        let fallback = ();
        let ctx = Context {
            max_list_length: 100,
            limits: Limits::RESTRICTIVE,
            doc,
            initial_block,
            entities: spec.entities().clone().into(),
            allowed_network: Arc::new(Ok),
        };

        let data = PSpec::new(&ctx, &spec, RealMapping::default(), fallback).await;

        panic!("{data:#?}");
    }
}
