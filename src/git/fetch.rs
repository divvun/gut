use super::common;
use super::models::GitCredential;
use git2::{AutotagOption, Error, FetchOptions, Repository};
use std::io::{self, Write};
use std::str;

// https://github.com/rust-lang/git2-rs/blob/master/examples/fetch.rs
pub fn fetch_branch(
    repo: &Repository,
    branch: &str,
    remote_name: &str,
    cred: Option<GitCredential>,
) -> Result<(), Error> {
    log::info!("Fetching {} for repo", branch);
    let mut remote = repo.find_remote(remote_name)?;

    let remote_callbacks = common::create_remote_callback(&cred)?;

    let mut fo = git2::FetchOptions::new();
    fo.remote_callbacks(remote_callbacks);

    remote.fetch(&[branch], Some(&mut fo), None)?;

    Ok(())
}

pub fn fetch(
    repo: &Repository,
    remote_name: &str,
    cred: Option<GitCredential>,
) -> Result<(), Error> {
    let mut remote = repo.find_remote(remote_name)?;

    let mut cb = common::create_remote_callback(&cred)?;

    //let mut fo = git2::FetchOptions::new();
    //fo.remote_callbacks(remote_callbacks);
    //remote.fetch(&[] as &[&str], Some(&mut fo), None)?;

    // This callback gets called for each remote-tracking branch that gets
    // updated. The message we output depends on whether it's a new one or an
    // update.
    cb.update_tips(|refname, a, b| {
        if a.is_zero() {
            println!("[new]     {:20} {}", b, refname);
        } else {
            println!("[updated] {:10}..{:10} {}", a, b, refname);
        }
        true
    });

    // Here we show processed and total objects in the pack and the amount of
    // received data. Most frontends will probably want to show a percentage and
    // the download rate.
    cb.transfer_progress(|stats| {
        if stats.received_objects() == stats.total_objects() {
            print!(
                "Resolving deltas {}/{}\r",
                stats.indexed_deltas(),
                stats.total_deltas()
            );
        } else if stats.total_objects() > 0 {
            print!(
                "Received {}/{} objects ({}) in {} bytes\r",
                stats.received_objects(),
                stats.total_objects(),
                stats.indexed_objects(),
                stats.received_bytes()
            );
        }
        io::stdout().flush().unwrap();
        true
    });
    cb.sideband_progress(|data| {
        print!("remote: {}", str::from_utf8(data).unwrap());
        io::stdout().flush().unwrap();
        true
    });
    // Download the packfile and index it. This function updates the amount of
    // received data and the indexer stats which lets you inform the user about
    // progress.
    let mut fo = FetchOptions::new();
    fo.remote_callbacks(cb);
    remote.download(&[] as &[&str], Some(&mut fo))?;

    {
        // If there are local objects (we got a thin pack), then tell the user
        // how many objects we saved from having to cross the network.
        let stats = remote.stats();
        if stats.local_objects() > 0 {
            println!(
                "\rReceived {}/{} objects in {} bytes (used {} local \
                 objects)",
                stats.indexed_objects(),
                stats.total_objects(),
                stats.received_bytes(),
                stats.local_objects()
            );
        } else {
            println!(
                "\rReceived {}/{} objects in {} bytes",
                stats.indexed_objects(),
                stats.total_objects(),
                stats.received_bytes()
            );
        }
    }

    // Disconnect the underlying connection to prevent from idling.
    remote.disconnect()?;

    // Update the references in the remote's namespace to point to the right
    // commits. This may be needed even if there was no packfile to download,
    // which can happen e.g. when the branches have been changed but all the
    // needed objects are available locally.
    remote.update_tips(None, true, AutotagOption::Unspecified, None)?;

    Ok(())
}
