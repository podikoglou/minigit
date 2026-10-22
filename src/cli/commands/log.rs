use std::{
    collections::{BinaryHeap, HashSet},
    env,
};

use argh::FromArgs;
use minigit::{
    MinigitError, Repo,
    object::{Object, commit::Commit, hash::ObjectHash},
};

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "log")]
/// Show commit logs
pub struct LogCommand {}

/// Newtype over [Commit] used in the context of the `log` command's priority queue.
///
/// Comparison between [QueueEntry] values depends solely on the commit time.
///
/// NOTE: This could perhaps be moved somewhere else later or we may be able to achieve the same
/// thing differently.
struct QueueEntry(Box<Commit>, ObjectHash);

impl PartialEq for QueueEntry {
    fn eq(&self, other: &Self) -> bool {
        self.0.committer.1 == other.0.committer.1
    }
}

impl Eq for QueueEntry {}

impl PartialOrd for QueueEntry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for QueueEntry {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.committer.1.cmp(&other.0.committer.1)
    }
}

impl Into<QueueEntry> for (Box<Commit>, ObjectHash) {
    fn into(self) -> QueueEntry {
        QueueEntry(self.0, self.1)
    }
}

impl LogCommand {
    pub fn run(self) -> Result<(), MinigitError> {
        let repo = Repo::open(env::current_dir()?)?;

        let mut queue: BinaryHeap<QueueEntry> = BinaryHeap::default();
        let mut visited: HashSet<ObjectHash> = HashSet::default();

        let head = repo
            .store
            .reference("HEAD".into())
            .and_then(|head| repo.store.resolve_reference(&head.pointer))
            .and_then(|head| repo.store.object(head.clone()).map(|obj| (obj, head)))
            .and_then(|(head, hash)| {
                head.try_as_commit()
                    .ok_or(MinigitError::InvalidHead)
                    .map(|commit| (commit, hash))
            })
            .map_err(|_| MinigitError::InvalidHead)?;

        queue.push(head.into());

        while !queue.is_empty() {
            // we just ensured it's not empty, so we can unwrap.
            let QueueEntry(commit, hash) = queue.pop().unwrap();

            for parent_hash in &commit.parents {
                if visited.contains(parent_hash) {
                    continue;
                }

                let parent = repo.store.object(parent_hash.clone())?;
                let parent = parent.try_as_commit().ok_or(MinigitError::UnexpectedType)?;

                queue.push((parent, parent_hash.clone()).into());
                visited.insert(parent_hash.clone());
            }

            show_commit(&commit, &hash);
        }

        Ok(())
    }
}

fn show_commit(
    Commit {
        tree,
        parents,
        author,
        committer,
        extra,
        description,
    }: &Commit,
    hash: &ObjectHash,
) {
    println!("commit {hash}");
    println!("Author: {}", author.0);
    println!("Date:   {}", author.1.format("%a %b %d %T %z"));
    println!("\n{description}");
}
