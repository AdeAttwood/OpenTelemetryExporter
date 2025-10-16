pub mod macros {
    #[macro_export]
    macro_rules! otel_log {
        // Basic: severity, logger, content
        ($severity:expr, $logger:expr, $content:expr) => {{
            use std::time::SystemTime;

        let mut log = $logger.create_log_record();
        log.set_severity_number($severity);
        log.set_body($content.into());
        log.set_timestamp(SystemTime::now());
        $logger.emit(log);
        }};

        // With attributes: severity, logger, content, attrs: [...]
        ($severity:expr, $logger:expr, $content:expr, attrs: [$(($key:expr, $val:expr)),* $(,)?]) => {{
            use std::time::SystemTime;

        let mut log = $logger.create_log_record();
        log.set_severity_number($severity);
        log.set_body($content.into());
        log.set_timestamp(SystemTime::now());
        $(
        log.add_attribute($key, $val);
        )*
        $logger.emit(log);
        }};

        // With time and attributes
        ($severity:expr, $logger:expr, $content:expr, time: $time:expr, attrs: [$(($key:expr, $val:expr)),* $(,)?]) => {{
            let mut log = $logger.create_log_record();
        log.set_severity_number($severity);
        log.set_body($content.into());
        log.set_timestamp($time.into());
        $(
        log.add_attribute($key, $val);
        )*
        $logger.emit(log);
        }};
    }

    #[macro_export]
    macro_rules! info {
        ($logger:expr, $content:expr) => {
            $crate::otel_log!(opentelemetry::logs::Severity::Info, $logger, $content)
        };
        ($logger:expr, $content:expr, attrs: [$($attr:tt)*]) => {
            $crate::otel_log!(opentelemetry::logs::Severity::Info, $logger, $content, attrs: [$($attr)*])
        };
        ($logger:expr, $content:expr, time: $time:expr, attrs: [$($attr:tt)*]) => {
            $crate::otel_log!(opentelemetry::logs::Severity::Info, $logger, $content, time: $time, attrs: [$($attr)*])
        };
    }

    #[macro_export]
    macro_rules! error {
        ($logger:expr, $content:expr) => {
            $crate::otel_log!(opentelemetry::logs::Severity::Error, $logger, $content)
        };
        ($logger:expr, $content:expr, attrs: [$($attr:tt)*]) => {
            $crate::otel_log!(opentelemetry::logs::Severity::Error, $logger, $content, attrs: [$($attr)*])
        };
        ($logger:expr, $content:expr, time: $time:expr, attrs: [$($attr:tt)*]) => {
            $crate::otel_log!(opentelemetry::logs::Severity::Error, $logger, $content, time: $time, attrs: [$($attr)*])
        };
    }

    pub(crate) use error;
    pub(crate) use info;
}
