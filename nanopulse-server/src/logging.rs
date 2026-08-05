use std::collections::HashMap;

use chrono::Utc;
use tracing::field::{Field, Visit};
use tracing_subscriber::{
    layer::{Context, Layer},
    registry::LookupSpan,
};

use crate::integration::internal::{Event, Log, publish};

pub struct LoggingLayer;

impl LoggingLayer {
    pub fn new() -> Self {
        Self {}
    }
}

struct HashMapVisitor<'a>(&'a mut HashMap<String, String>);

impl<'a> Visit for HashMapVisitor<'a> {
    fn record_str(&mut self, field: &Field, value: &str) {
        self.0.insert(field.name().to_owned(), value.to_owned());
    }

    fn record_bool(&mut self, field: &Field, value: bool) {
        self.0.insert(field.name().to_owned(), value.to_string());
    }

    fn record_i64(&mut self, field: &Field, value: i64) {
        self.0.insert(field.name().to_owned(), value.to_string());
    }

    fn record_u64(&mut self, field: &Field, value: u64) {
        self.0.insert(field.name().to_owned(), value.to_string());
    }

    fn record_f64(&mut self, field: &Field, value: f64) {
        self.0.insert(field.name().to_owned(), value.to_string());
    }

    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        self.0
            .insert(field.name().to_owned(), format!("{:?}", value));
    }
}

impl<S> Layer<S> for LoggingLayer
where
    S: tracing::Subscriber + for<'a> LookupSpan<'a>,
{
    fn on_new_span(
        &self,
        attrs: &tracing_core::span::Attributes<'_>,
        id: &tracing_core::span::Id,
        ctx: Context<'_, S>,
    ) {
        let mut fields = HashMap::new();
        let mut visitor = HashMapVisitor(&mut fields);
        attrs.record(&mut visitor);

        if let Some(span_ref) = ctx.span(id) {
            span_ref
                .extensions_mut()
                .insert::<HashMap<String, String>>(fields);
        }
    }

    fn on_event(&self, event: &tracing::Event<'_>, ctx: Context<'_, S>) {
        let mut all_fields: HashMap<String, String> = HashMap::new();

        if let Some(scope) = ctx.event_scope(event) {
            for span in scope.from_root() {
                if let Some(storage) = span.extensions().get::<HashMap<String, String>>() {
                    all_fields.extend(storage.iter().map(|(k, v)| (k.clone(), v.clone())));
                }
            }
        }

        let mut event_fields = HashMap::new();
        let mut visitor = HashMapVisitor(&mut event_fields);
        event.record(&mut visitor);

        all_fields.extend(event_fields);

        let message = all_fields.remove("message").unwrap_or_default();
        let level = event.metadata().level().as_str();
        let target = event.metadata().target();
        let workspace_name = all_fields
            .get("workspace_name")
            .cloned()
            .unwrap_or_default();

        if !workspace_name.is_empty() {
            publish(
                &workspace_name,
                Event {
                    log: Some(Log {
                        timestamp: Utc::now(),
                        level: level.into(),
                        target: target.into(),
                        attributes: all_fields,
                        message,
                    }),
                    ..Default::default()
                },
            );
        }
    }
}
