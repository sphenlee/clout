use clout::{error, warn, status, info, debug, trace};

pub fn main() {
    clout::init()
        .with_verbose(4)
        .with_quiet(false)
        .with_silent(false)
        .with_use_color(clout::UseColor::Auto)
        .done()
        .expect("clout failed to init");

    error!("an error");
    warn!("a warning");
    status!("a normal message");
    info!("useful info");
    debug!("debug info");
    trace!("tracing");

    println!("done!");

    clout::shutdown().expect("failed to shutdown clout");
}
