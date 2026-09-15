# minigit
Implementation of (some of) Git in Rust.

I started working on this project when I was inspired by what
[Cursor was doing with Git](https://cursor.com/blog/git-at-any-scale) and got
to learn about how neat Git really is.

This project is in the form of a library + a CLI with *some* of the original
Git commands (and some that I came up with). Concretely:
```
Usage: minigit <command> [<args>]

The stupid implementation of the stupid content tracker.

Options:
  --help, help      display usage information

Commands:
  hash-object       Compute object ID and optionally create an object from a
                    file
  ls-buckets        List buckets in the repository
  ls-objects        List objects in the repository
  parse-object      Reads an object from a file or stdin and parses it
```

## AI Disclaimer
~90% of the code is human-written.
