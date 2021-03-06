use std::collections::VecDeque;
use std::iter::FromIterator;
use std::sync::{Arc, Mutex};
use std::thread;

use error_chain::ChainedError;
use slog::Logger;

use codealong::Repo;

use crate::error::*;
use crate::ui::ProgressPool;

/// Clone and/or fetch all repos
pub fn initialize_repos(
    matches: &clap::ArgMatches,
    repos: Vec<Repo>,
    logger: &Logger,
) -> Result<()> {
    info!(logger, "Initializing {} repos", repos.len());
    let num_threads = std::cmp::min(
        matches
            .value_of("concurrency")
            .unwrap_or_else(|| "6")
            .parse::<i32>()?,
        repos.len() as i32,
    );
    let m = Arc::new(ProgressPool::new(
        repos.len() as u64,
        matches.is_present("progress"),
    ));
    m.set_message("Repos initialized");
    let repos = Arc::new(Mutex::new(VecDeque::from_iter(repos)));
    for _ in 0..num_threads {
        let repos = repos.clone();
        let m = m.clone();
        let mut pb = m.add();
        let root_logger = logger.clone();
        thread::spawn(move || loop {
            let repo = {
                let mut repos = repos.lock().unwrap();
                repos.pop_front()
            };
            if let Some(repo) = repo {
                let logger = root_logger.new(o!("repo" => repo.repo_info().name.to_owned()));
                pb.reset(repo.repo_info().name.to_owned());
                pb.set_message("fetching");
                let cb = |cur: usize, total: usize| {
                    pb.set_length(total as u64);
                    pb.set_position(cur as u64);
                };
                info!(logger, "Initializing repo");
                match repo.init(Some(Box::new(cb))) {
                    Ok(_) => pb.set_message("finished"),
                    Err(e) => {
                        error!(logger, "error initializing"; "error" => e.display_chain().to_string())
                    }
                };
                m.inc(1);
            } else {
                pb.finish();
                break;
            }
        });
    }
    m.join_and_clear()?;
    Ok(())
}
