git-wire
========

A git custom subcommand
which injects part of other repositoriy's source code
into the repository in a declarative manner.

Instration
----------

- If your environment have rust installed:

    ```
    $ cargo install --git https://github.com/msr1k/git-wire.git
    ```

    Or, you can build from source if you clone this repository.

- In other cases:

    Currently not supported.


Preparation
-----------

Create a file named `.gitwire` at the root of the repository with following JSON format.

```json
[
  {
    "url": "url-of-the-repository",
    "rev": "revision (commit hash or branch name or tag name)",
    "src": "source directory of the target repository",
    "dst": "directory where to put the `src` on this repositry"
  },
  ...
]
```

Commands
--------

### sync

Sync sources depending on the definition of the `.gitwire`.

Please note that it always clears the destination before sync started for each item.

    $ git wire sync

### check

Check sources depending on the definition of the `.gitwire`.

If there are some differences, this command repots each of them all,
and returns with exit code 1, otherwise returns with 0.

    $ git wire check

## License

The MIT License
