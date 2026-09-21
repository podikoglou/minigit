# Fixtures

## Objects
### Creating an object fixture
- Grab object from wherever (could be a regression, so, from a real repo, or could be synthetically constructed)
- Place into `tests/fixtures/objects`, with a file name of `<object-type>-<index>.obj`
- Run `tests/fixtures/scripts/hashes.sh`

## Repositories
- `repo-1`: Repo with 1 commit, of a single file. Contains three objects: one blob, one tree, one commit.
- `repo-2`: Repo with 414 commits of one text file (containing three numbers) each
- `repo-3`: Repo with 4 commits of one text file each, ending with a merge commit that has two parents
- `repo-4`: Repo with 4 commits, and a tag

### Creating a repository fixture
- Create a repository somewhere, initialize it as you like
- `tar -cvf ../repo-<n>.tar`

It is crucical that this is ran inside the repository, so that the repository's
directly is not included in the tar!
