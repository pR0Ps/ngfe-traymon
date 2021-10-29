#[cfg(not(target_os = "windows"))]
compile_error!("This program is Windows-exclusive");

fn main() {
    let mut res = winres::WindowsResource::new();
    res.set_icon_with_id("res/active.ico", "1");
    res.set_icon_with_id("res/inactive.ico", "2");

    if let Err(e) = res.compile() {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}
