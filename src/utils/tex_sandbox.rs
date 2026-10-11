//! Run untrusted TeX with no access to application files or the network.
use std::path::Path;
use tokio::process::Command;

pub fn command(engine: &str, workdir: &Path) -> Command {
    let mut cmd = Command::new("bwrap");
    cmd.args([
        "--unshare-all",
        "--die-with-parent",
        "--new-session",
        "--clearenv",
    ]);
    for path in [
        "/usr",
        "/lib",
        "/lib64",
        "/etc/fonts",
        "/etc/texmf",
        "/var/lib/texmf",
        "/var/cache/fontconfig",
    ] {
        if Path::new(path).exists() {
            cmd.args(["--ro-bind", path, path]);
        }
    }
    cmd.arg("--bind")
        .arg(workdir)
        .arg("/work")
        .args([
            "--tmpfs",
            "/tmp",
            "--proc",
            "/proc",
            "--dev",
            "/dev",
            "--chdir",
            "/work",
            "--setenv",
            "HOME",
            "/tmp",
            "--setenv",
            "PATH",
            "/usr/bin:/bin",
            "--setenv",
            "openin_any",
            "p",
            "--setenv",
            "openout_any",
            "p",
            "--",
            "/usr/bin/sh",
            "-c",
            "ulimit -t 30; ulimit -v 1048576; ulimit -f 131072; exec \"$@\"",
            "tex-sandbox",
        ])
        .arg(engine)
        .arg("-no-shell-escape")
        .kill_on_drop(true);
    cmd
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;

    #[tokio::test]
    #[ignore = "requires Linux bubblewrap user namespaces"]
    async fn sandbox_preserves_work_files_and_hides_application_state() {
        let work = tempfile::tempdir().expect("temporary work directory");
        let engine = work.path().join("engine");
        std::fs::write(&engine, b"#!/usr/bin/sh\nset -eu\ntest -z \"${LENSISKU_SECURITY_TEST_SECRET:-}\"\ntest ! -e /home/user\ntest ! -e /etc/passwd\ntest -r input.tex\nprintf 'sandbox-ok' > output.pdf\n").expect("fixture engine");
        std::fs::set_permissions(&engine, std::fs::Permissions::from_mode(0o755))
            .expect("executable");
        std::fs::write(work.path().join("input.tex"), "legitimate input").expect("input");
        let result = command("/work/engine", work.path())
            .env("LENSISKU_SECURITY_TEST_SECRET", "synthetic-secret")
            .output()
            .await
            .expect("bubblewrap execution");
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        assert_eq!(
            std::fs::read(work.path().join("output.pdf")).expect("output"),
            b"sandbox-ok"
        );
    }
}
