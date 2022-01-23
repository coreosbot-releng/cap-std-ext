use anyhow::Result;
use cap_std_ext::cmdext::CapStdExtCommandExt;
use cap_std_ext::dirext::CapStdExtDirExt;
use rustix::fd::FromFd;
use std::io::Write;
use std::os::unix::prelude::PermissionsExt;
use std::path::Path;
use std::{process::Command, sync::Arc};

#[test]
fn take_fd() -> Result<()> {
    let mut c = Command::new("/bin/bash");
    c.arg("-c");
    c.arg("wc -c <&5");
    let (r, w) = rustix::io::pipe()?;
    let r = Arc::new(r);
    let mut w = cap_std::fs::File::from_fd(w.into());
    c.take_fd_n(r.clone(), 5);
    write!(w, "hello world")?;
    drop(w);
    c.stdout(std::process::Stdio::piped());
    let s = c.output()?;
    assert!(s.status.success());
    assert_eq!(s.stdout.as_slice(), b"11\n");
    Ok(())
}

#[test]
fn optionals() -> Result<()> {
    let td = cap_tempfile::tempdir(cap_std::ambient_authority())?;

    // file
    assert!(td.open_optional("bar")?.is_none());
    assert_eq!(td.remove_file_optional("bar")?, false);
    td.write("bar", "testcontents")?;
    assert_eq!(td.read("bar")?.as_slice(), b"testcontents");
    assert_eq!(td.remove_file_optional("bar")?, true);

    // directory
    assert!(td.open_dir_optional("somedir")?.is_none());
    td.create_dir("somedir")?;
    assert!(td.open_dir_optional("somedir")?.is_some());
    Ok(())
}

#[test]
fn link_tempfile() -> Result<()> {
    let td = cap_tempfile::tempdir(cap_std::ambient_authority())?;
    let p = Path::new("foo");
    let mut f = td.new_linkable_file(p).unwrap();
    assert!(td.metadata_optional(p).unwrap().is_none());
    writeln!(f, "hello world").unwrap();
    drop(f);
    // Verify we didn't write the file
    assert!(td.metadata_optional(p)?.is_none());

    let mut f = td.new_linkable_file(p).unwrap();
    writeln!(f, "hello world").unwrap();
    f.emplace().unwrap();
    assert_eq!(td.metadata(p)?.permissions().mode(), 0o600);
    Ok(())
}
