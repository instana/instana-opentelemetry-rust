pub const INTERNAL_TAG_ALL_IN_ONE: &str = "all";
pub const INTERNAL_TAG_ENTITY_ID: &str = "eid";
pub const INTERNAL_TAG_SY: &str = "sy";
pub const INTERNAL_TAG_CRID: &str = "crid";
pub const INTERNAL_TAG_CRTP: &str = "crtp";
pub const INTERNAL_TAG_TP: &str = "tp";
pub const INTERNAL_TAG_IA_TID: &str = "ia_tid";
pub const INTERNAL_TAG_IA_PID: &str = "ia_pid";

pub const DEFAULT_MAX_QUEUE_SIZE: u64 = 30720;
pub const DEFAULT_SCHEDULE_DELAY_MILLIS: u64 = 200;
pub const DEFAULT_MAX_EXPORT_BATCH_SIZE: u64 = 400;

pub const DEFAULT_INSTANA_AGENT_PORT: u64 = 42699;
pub const DEFAULT_INSTANA_AGENT_HOST: &str = "localhost";

pub const OTEL_KEY_STATUS_CODE: &str = "status_code";
pub const OTEL_KEY_STATUS_DESCRIPTION: &str = "error";
pub const OTEL_KEY_INSTRUMENTATION_SCOPE_NAME: &str = r#""scope.name""#;
pub const OTEL_KEY_INSTRUMENTATION_SCOPE_VERSION: &str = r#""scope.version""#;
pub const OTEL_KEY_DROPPED_ATTRIBUTES_COUNT: &str = "dropped_attributes_count";
pub const OTEL_KEY_DROPPED_EVENTS_COUNT: &str = "dropped_events_count";
pub const OTEL_KEY_DROPPED_LINKS_COUNT: &str = "dropped_links_count";
