fn main() {
    slint_build::compile("ui/main_window.slint").expect("Slint build failed");
    //slint_build::compile("ui/login_window.slint").expect("Slint build failed");
}
