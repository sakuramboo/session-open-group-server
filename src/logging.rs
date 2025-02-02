use log::LevelFilter;
use log4rs::{
    append::{
        console::ConsoleAppender,
        rolling_file::{policy::compound, RollingFileAppender},
    },
    config::{Appender, Logger, Root},
    encode::pattern::PatternEncoder,
    filter::threshold::ThresholdFilter,
};
use std::str::FromStr;

pub fn init(log_file: Option<String>, log_level: Option<String>) {
    let level = if log_level.is_some() {
        LevelFilter::from_str(&log_level.unwrap()).unwrap()
    } else {
        LevelFilter::Info
    };
    let stdout_appender = {
        let encoder =
            Box::new(PatternEncoder::new("{h({l})} {d(%Y-%m-%d %H:%M:%S%.3f)} [{f}:{L}] {m}{n}"));
        let stdout = ConsoleAppender::builder().encoder(encoder).build();
        let filter = Box::new(ThresholdFilter::new(level));
        Appender::builder().filter(filter).build("stdout", Box::new(stdout))
    };
    let mut root = Root::builder().appender("stdout");
    let sogs = Logger::builder().build("session_open_group_server", level);
    let mut config_builder = log4rs::Config::builder().logger(sogs).appender(stdout_appender);
    if let Some(log_file) = log_file {
        // Rotate log files every ~50MB keeping 1 archived
        let size_trigger = compound::trigger::size::SizeTrigger::new(50_000_000);
        let roller = compound::roll::fixed_window::FixedWindowRoller::builder()
            .build(&format!("{}-archive.{{}}", &log_file), 1)
            .unwrap();
        let roll_policy = compound::CompoundPolicy::new(Box::new(size_trigger), Box::new(roller));
        // Print to the file at the given level
        let file_appender =
            RollingFileAppender::builder().build(&log_file, Box::new(roll_policy)).unwrap();
        let filter = Box::new(ThresholdFilter::new(level));
        let file_appender =
            Appender::builder().filter(filter).build("file", Box::new(file_appender));
        config_builder = config_builder.appender(file_appender);
        root = root.appender("file");
    }
    let root = root.build(level);
    let config = config_builder.build(root).unwrap();
    let _ = log4rs::init_config(config).expect("Couldn't initialize log configuration.");
}
