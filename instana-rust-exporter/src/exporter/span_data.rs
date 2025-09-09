use anyhow::{anyhow, Result};
use opentelemetry::{
    trace::{Event, Link},
    Key, KeyValue, Value,
};
use opentelemetry_sdk::trace::SpanData;

pub trait GET {
    fn get_attribute(&self, attribute_name: impl Into<Key> + Copy) -> Result<Value>;
    fn get_attributes(&self) -> Vec<KeyValue>;
    fn get_events(&self) -> Vec<Event>;
    fn get_links(&self) -> Vec<Link>;
}

impl GET for SpanData {
    fn get_attribute<'a>(&self, attribute_name: impl Into<Key> + Copy) -> Result<Value> {
        for keyvalue in &self.attributes {
            if keyvalue.key == attribute_name.into() {
                return Ok(keyvalue.value.clone());
            }
        }
        return Err(anyhow!("Attribute not found"));
    }

    fn get_attributes(&self) -> Vec<KeyValue> {
        return self.attributes.clone();
    }

    fn get_events(&self) -> Vec<Event> {
        return self.events.events.clone();
    }
    fn get_links(&self) -> Vec<Link> {
        return self.links.links.clone();
    }
}

#[cfg(test)]
mod tests {
    use std::time::SystemTime;

    use super::*;
    use opentelemetry::{InstrumentationScope, KeyValue, SpanId};
    use opentelemetry_sdk::trace::{SpanEvents, SpanLinks};

    //use serde::ser::{SerializeSeq,SerializeMap};

    use opentelemetry::trace::{SpanContext, SpanKind, Status};

    #[test]
    fn test_span() {
        let mut vector: Vec<KeyValue> = Vec::new();
        vector.push(KeyValue::new("INTERNAL_TAG_ENTITY_ID", 100));
        vector.push(KeyValue::new("INTERNAL_TAG_CRTP", "ntg"));

        // let vec_event:Vec<Event>=Vec::new();
        // let vec_link:Vec<Link>=Vec::new();
        // let links= SpanLinks {
        //     links: vec_link,
        //     dropped_count: 0,
        // };
        // let events=SpanEvents {
        //     events: vec_event,
        //     dropped_count: 0,
        // };

        /* above  commented way is not possible beacuse the struct SpanEvents and SpanLinks are non-exhaustive */
        let span_data = SpanData {
            span_context: SpanContext::NONE,
            parent_span_id: SpanId::INVALID,

            span_kind: SpanKind::Client,

            name: std::borrow::Cow::Borrowed("testspan"),

            start_time: SystemTime::now(),

            end_time: SystemTime::now(),

            attributes: vector,

            dropped_attributes_count: 100,

            events: SpanEvents::default(),

            links: SpanLinks::default(),

            status: Status::Ok,

            instrumentation_scope: InstrumentationScope::builder("fake-instrumentation").build(),
        };

        let x = span_data.get_attribute("INTERNAL_TAG_CRTP").unwrap();

        assert_eq!(x, "ntg".into());
    }
}
