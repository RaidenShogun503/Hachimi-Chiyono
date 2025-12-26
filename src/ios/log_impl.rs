use log::{Record, Level, Metadata};

struct AppleLogger;

impl log::Log for AppleLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            // Simple syslog fallback or we could use os_log if we bind it.
            // For now, let's just print to stderr/stdout which usually shows up in crashlogs/device logs often
            // but syslog is better.
            use std::ffi::CString;
            unsafe {
                let msg = CString::new(format!("[Hachimi] {}", record.args())).unwrap();
                libc::syslog(libc::LOG_NOTICE, msg.as_ptr());
            }
        }
    }

    fn flush(&self) {}
}

static LOGGER: AppleLogger = AppleLogger;

pub fn init(level: log::LevelFilter) {
    log::set_logger(&LOGGER)
        .map(|()| log::set_max_level(level))
        .unwrap_or_else(|e| eprintln!("Failed to interpret logger: {}", e));
}
