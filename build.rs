fn main() {
    // Only run the winres builder if compiling for Windows
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap() == "windows" {
        let mut res = winres::WindowsResource::new();
        // Set the path to your .ico file relative to the project root
        res.set_icon("favicon.ico");

        // Optionally set metadata properties visible in Windows File Properties
        res.set_manifest(
            r#"
            <assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
              <trustInfo xmlns="urn:schemas-microsoft-com:asm.v1">
                <security>
                  <requestedPrivileges>
                    <requestedExecutionLevel level="asInvoker" uiAccess="false" />
                  </requestedPrivileges>
                </security>
              </trustInfo>
            </assembly>
            "#,
        );

        if let Err(e) = res.compile() {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    }
}
