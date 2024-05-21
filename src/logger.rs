use flexi_logger::{
    FileSpec, 
    Logger, 
    opt_format
};

pub fn initiate() {
    let logger = Logger::try_with_env().unwrap()
        .format(opt_format)
        .log_to_file(FileSpec::default())
        .print_message();
    logger.start().unwrap();
}
