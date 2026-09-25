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

    #[test]
    fn test_urls() {
        let url: Url = "https://example.org:80/abc".parse().unwrap();
        let x = url.host_str().unwrap();
        assert_eq!("example.org", x);
        assert_eq!("https", url.scheme());
        assert_eq!(Some(80), url.port_or_known_default());
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
        let eval = SingleEvaluation::new(
            entities,
            13.into(),
            &selectable,
            Features::all(),
            Arc::new(Ok),
        )
        .unwrap();
        let eval = eval.run_to_end_with_early_return().await;
        panic!(
            "{:?} {:#?}\n\n\n{entities:#?}",
            eval.value(),
            eval.all_values()
        );
    }
}
