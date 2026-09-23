fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap() == "windows" {
        let mut res = winres::WindowsResource::new();
        // 设置图标路径，假设您把图标命名为 icon.ico 并放在项目根目录
        res.set_icon("icon.ico");
        res.compile().unwrap();
    }
}
