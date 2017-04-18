use std::collections::HashMap;
fn main() {
    extern crate crates_index;

    let index = crates_index::Index::new("_index".into());
    if !index.exists() {
            index.fetch().expect("Could not fetch crates.io index");
    }
    let mut map: HashMap<_, u32> =  HashMap::new();
    let mut time_map = HashMap::new();
    run("_index".to_string(), &mut time_map);
    for path in index.crate_index_paths() {
        let crate_ = crates_index::Crate::new(path.clone());
        let latest_version = crate_.latest_version();
        println!("crate name: {}", latest_version.name());
        let path_str = path.to_str().unwrap().to_string()["_index/".len()..].to_string();

        if (true ||  time_map.contains_key(&path_str)) {
        for dep in latest_version.dependencies() {
                    //let time = time_map.get(&path_str).unwrap();
        //print_time(time, "Date: ");
            //println!("dep: {}", dep.name());
            let mut count = map.entry(dep.name().to_string()).or_insert(0);
            *count += 1;
        }
        }

        //println!("crate version: {}", latest_version.version());
    }
    let mut sorted : Vec<_> = map.iter().collect();
    let mut total = sorted.len();
    sorted.sort_by(|a, b| a.1.cmp(b.1));
    for (name, count) in sorted {
        println!("{}. {} {}", total, name, count);
        total -= 1;
    }
}

extern crate git2;
extern crate time;

use std::str;
use git2::{Repository, Signature, Commit, ObjectType, Time, DiffOptions};
use git2::{Pathspec, Error, DiffFormat};
struct Args {
    arg_spec: Vec<String>,
}

fn run(repo_path: String, times: &mut HashMap<String, Time>) -> Result<(), Error> {
    return Ok(());
    let repo = try!(Repository::open(repo_path));
    let mut revwalk = try!(repo.revwalk());

    // Prepare the revwalk based on CLI parameters
    //let base = if args.flag_reverse {git2::SORT_REVERSE} else {git2::SORT_NONE};
    revwalk.set_sorting(git2::SORT_TIME);

    try!(revwalk.push_head());

    // Prepare our diff options and pathspec matcher
    let (mut diffopts, mut diffopts2) = (DiffOptions::new(), DiffOptions::new());
    /*for spec in args.arg_spec.iter() {
        println!("{}", spec);
        diffopts.pathspec(spec);
        diffopts2.pathspec(spec);
    }
    let ps = try!(Pathspec::new(args.arg_spec.iter()));
    */
    // Filter our revwalk based on the CLI parameters
    macro_rules! filter_try {
        ($e:expr) => (match $e { Ok(t) => t, Err(e) => continue })
    }
    use time::strptime;
    let t_before = strptime("2017", "%Y").unwrap();
    for id in revwalk {
        let id = filter_try!(id);
        let commit = filter_try!(repo.find_commit(id));
        let parents = commit.parents().len();
        //if args.arg_spec.len() > 0 {
            match commit.parents().len() {
                0 => {
                    let tree = filter_try!(commit.tree());
                    let flags = git2::PATHSPEC_NO_MATCH_ERROR;
                    continue;
                    //return None;
                    //if ps.match_tree(&tree, flags).is_err() { return None }
                }
                _ => {
                    for parent in commit.parents() {
                            let a = parent.tree().unwrap();
                            let b = commit.tree().unwrap();
                            let diff = repo.diff_tree_to_tree(Some(&a), Some(&b), None).unwrap();
                            let m = diff.foreach(&mut |file, _progress| {
                                    let author = commit.author();
                                    let key = file.old_file().path().unwrap().to_str().unwrap().to_string();
                                    let e = times.entry(key);
                                    let t = to_time(&author.when());
                                    if t < t_before {
                                        return false;
                                    }

                                    e.or_insert(author.when());
                                    print_time(&author.when(), "Date:   ");
                                //println!("{:?}", file.old_file().path());
                                true
                            }, None, None, None);
                            if !m.is_ok() { return Ok(()); }

                    }
                    //if !m { continue; /* return None */ }
                }
            }
        //}
        //Some(Ok(commit))
    }

    // print!
    /*for commit in revwalk {
        let commit = try!(commit);
        print_commit(&commit);
    }*/

    Ok(())
}

fn print_commit(commit: &Commit) {
    let author = commit.author();
}

fn to_time(time: &Time) -> time::Tm  {
    let (offset, sign) = match time.offset_minutes() {
        n if n < 0 => (-n, '-'),
        n => (n, '+'),
    };
    let (hours, minutes) = (offset / 60, offset % 60);
    let ts = time::Timespec::new(time.seconds() +
                                 (time.offset_minutes() as i64) * 60, 0);
    time::at(ts)
}

fn print_time(time: &Time, prefix: &str) {
    let (offset, sign) = match time.offset_minutes() {
        n if n < 0 => (-n, '-'),
        n => (n, '+'),
    };
    let (hours, minutes) = (offset / 60, offset % 60);
    let ts = time::Timespec::new(time.seconds() +
                                 (time.offset_minutes() as i64) * 60, 0);
    let time = time::at(ts);

    println!("{}{} {}{:02}{:02}", prefix,
             time.strftime("%a %b %e %T %Y").unwrap(), sign, hours, minutes);

}

fn match_with_parent(repo: &Repository, commit: &Commit, parent: &Commit,
                     opts: &mut DiffOptions) -> Result<bool, Error> {
    let a = try!(parent.tree());
    let b = try!(commit.tree());
    let diff = try!(repo.diff_tree_to_tree(Some(&a), Some(&b), Some(opts)));
    diff.foreach(&mut |file, _progress| { true }, None, None, None);
    Ok(diff.deltas().len() > 0)
}

