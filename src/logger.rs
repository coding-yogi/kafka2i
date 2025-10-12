use flexi_logger::{
    opt_format, FileSpec, Logger, LoggerHandle
};

pub fn initiate() -> LoggerHandle {
    let _logger = Logger::try_with_env().unwrap()
        .format(opt_format)
        .log_to_file(FileSpec::default().directory("logs"))
        .start().unwrap();

    _logger
}

