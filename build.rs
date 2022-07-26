use std::process::Command;

fn main() {
    // build the dashboard
    Command::new("npm")
        .args(["clean-install"])
        .current_dir("dashboard")
        .status()
        .unwrap();
    Command::new("npm")
        .args(["run", "build"])
        .current_dir("dashboard")
        .status()
        .unwrap();
    // copy the build output to ./static
    Command::new("cp")
        .args(["-r", "dashboard/out", "static"])
        .status()
        .unwrap();
}
