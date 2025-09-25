use opentelemetry::{
    propagation::{text_map_propagator::FieldIter, Extractor, Injector, TextMapPropagator},
    Context,
};
use std::collections::HashSet;

#[derive(Debug)]
pub struct CompositePropagator {
    propagators: Vec<Box<dyn TextMapPropagator + Send + Sync>>,
    fields: Vec<String>,
}

impl CompositePropagator {
    pub fn new(propagators: Vec<Box<dyn TextMapPropagator + Send + Sync>>) -> Self {
        let mut fields = HashSet::new();
        for propagator in &propagators {
            fields.extend(propagator.fields().map(ToString::to_string));
        }

        CompositePropagator {
            propagators,
            fields: fields.into_iter().collect(),
        }
    }
}

impl TextMapPropagator for CompositePropagator {
    /// Encodes the values of the `Context` and injects them into the `Injector`.
    fn inject_context(&self, context: &Context, injector: &mut dyn Injector) {
        for propagator in &self.propagators {
            propagator.inject_context(context, injector)
        }
    }

    /// Retrieves encoded `Context` information using the `Extractor`. If no data was
    /// retrieved OR if the retrieved data is invalid, then the current `Context` is
    /// returned.
    fn extract_with_context(&self, cx: &Context, extractor: &dyn Extractor) -> Context {
        self.propagators
            .iter()
            .fold(cx.clone(), |current_cx, propagator| {
                propagator.extract_with_context(&current_cx, extractor)
            })
    }

    fn fields(&self) -> FieldIter<'_> {
        FieldIter::new(self.fields.as_slice())
    }
}
