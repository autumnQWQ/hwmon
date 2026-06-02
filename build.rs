fn main() {
    let ico_path = std::path::Path::new("F:/hwmon/hwmon.ico");

    if ico_path.exists() {
        winresource::WindowsResource::new()
            .set_icon(ico_path.to_str().unwrap())
            .compile()
            .unwrap();
        println!("cargo:warning=Embedded icon: hwmon.ico");
    }

    println!("cargo:rerun-if-changed=hwmon.png");
    println!("cargo:rerun-if-changed=hwmon.ico");
}
