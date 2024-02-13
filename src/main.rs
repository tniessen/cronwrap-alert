fn main() -> std::process::ExitCode {
    use clap::Parser;
    cronwrap::main(cronwrap::Args::parse()).into()
}
