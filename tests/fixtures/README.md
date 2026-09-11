# `minigit` Test Fixtures

## Fixtures
- `repo-1`: Contains a basic repository with a single commit, of a single file. Contains three objects: one blob, one tree, one commit.

## Creating a repository fixture
- Create a repository somewhere, initialize it as you like
- `tar -cvf ../repo-<n>.tar`

It is crucical that this is ran inside the repository, so that the repository's
directly is not included in the tar!
