git-wire
========

A git subcommand
which wires part of other repositoriy's source code
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

Optionally, you can select a method, `shallow` or `partial`, to checkout src like below.

```json
[
  {
    "url": "url-of-the-repository",
    "rev": "revision (commit hash or branch name or tag name)",
    "src": "source directory of the target repository",
    "dst": "directory where to put the `src` on this repositry",
    "mtd": "shallow/partial"
  },
  ...
]
```

Where `"shallow"` is default behavior, it gets all the files in specified `rev` at once,
it inherently requires more memory and temporary storage than `"partial"`,
but it might be faster if there are many files to `sync`.

While `"partial"` only gets files under a specified directory.
It is sperior than `"shallow"` in terms of memory and temporary storage consumption,
but since it performs downloading for each file one by one (it is mere git command behavior),
it could take much time particularly as the number of files grows.
(In the worst case, you might get an error)

If you `sync` only small number of files of hugely large repository, `"partial"` could be better choice,
but, if it's not, `"shallow"` is more appropriate in many cases,
though it varies depending on the target repository to sync.


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


A sample .gitwire
-----------------

This `.gitwire` sample wires this repository's `src` at revision v1.0.0 into `src_v1.0.0` directory.
https://github.com/msr1k/git-wire/blob/main/.gitignore

## Changelog

- v1.1.1

    Change default checkout method from `"partial"` to `"shallow"`,
    since it seems that, in most cases, `"shallow"` is faster and stabler.

- v1.1.0

    Added optional `"mtd"` (method) setting which can control the way to chekcout target source code.

- v1.0.1

    Replace one dependent crate with one of others to reduce unwanted dependencies.  
    (No functionality changes have been made from v1.0.0.)

- v1.0.0

    Initial Version.


License
-------

The MIT License
