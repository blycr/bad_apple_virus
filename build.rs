fn main() {
    #[cfg(all(target_os = "windows", target_env = "msvc"))]
    {
        println!("cargo:rerun-if-changed=Cargo.toml");
        println!("cargo:rerun-if-changed=apple.ico");
        let mut res = winres::WindowsResource::new();

        res.set_manifest(
            r#"
            <assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
            <dependency>
                <dependentAssembly>
                    <assemblyIdentity
                        type="win32"
                        name="Microsoft.Windows.Common-Controls"
                        version="6.0.0.0"
                        processorArchitecture="*"
                        publicKeyToken="6595b64144ccf1df"
                        language="*"
                    />
                </dependentAssembly>
            </dependency>
            <application xmlns="urn:schemas-microsoft-com:asm.v3">
                <windowsSettings>
                    <dpiAware xmlns="http://schemas.microsoft.com/SMI/2005/WindowsSettings">true</dpiAware>
                    <dpiAwareness xmlns="http://schemas.microsoft.com/SMI/2016/WindowsSettings">PerMonitorV2</dpiAwareness>
                </windowsSettings>
            </application>
            </assembly>
        "#,
        );

        // from: https://upload.wikimedia.org/wikipedia/commons/4/4e/Single_apple.png
        res.set_icon("apple.ico");
        res.compile().unwrap();
    }
}
