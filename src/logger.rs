use log::{LogRecord, LogLevel, LogMetadata, self, SetLoggerError, LogLevelFilter};

struct SimpleLogger;

impl log::Log for SimpleLogger {
    fn enabled(&self, metadata: &LogMetadata) -> bool {
        metadata.level() <= LogLevel::Info
    }

    fn log(&self, record: &LogRecord) {
        if self.enabled(record.metadata()) {
            println!("{} - {}", record.level(), record.args());
        }
    }
}



fn init() -> Result<(), SetLoggerError> {
   	log::set_logger(|lvl| {
   		lvl.set(LogLevelFilter::Info);
   		Box::new(SimpleLogger)
  	})
}