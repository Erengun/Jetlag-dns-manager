// FRB init function — must remain here for flutter_rust_bridge initialization.

#[flutter_rust_bridge::frb(init)]
pub fn init_app() {
    flutter_rust_bridge::setup_default_user_utils();
}
