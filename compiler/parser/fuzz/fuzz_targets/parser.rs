#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &str| {
    struct NoOpReporter;

    impl parser::parser::ErrorReporter for NoOpReporter {
        fn report(&mut self, error: parser::parser::Error) {}
    }

    // fuzzed code goes here
    parser::parser::Parser::new(&mut NoOpReporter, data).parse_file();
});
