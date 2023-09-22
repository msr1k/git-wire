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

Optionally, you can select a method, `"shallow"`, `"shallow_no_sparse"`, and `"partial"` to checkout src like below.

```json
[
  {
    "url": "url-of-the-repository",
    "rev": "revision (commit hash or branch name or tag name)",
    "src": "source directory of the target repository",
    "dst": "directory where to put the `src` on this repositry",
    "mtd": "shallow/shallow_no_sparse/partial"
  },
  ...
]
```

### About checkout methods

#### shallow (default)
This method gets all the files under a specified `src` at once from specified `rev`.
In almost all cases, it will be the best choice.

If you omit `"mtd"` key, `"shallow"` method will be used automatically.


#### shallow_no_sparse
`"shallow"` gets all the files managed by that repository at once from specified `rev`,
it inherently requires more memory and temporary storage than `"shallow"` and `"partial"`,
but it must be faster than `"partial"` if there are many files to `sync`.  
(It must not faster than `"shallow"`.)

I assume that it is the alternative method when you face some problems by using `"shallow"`.

#### partial (not recommended)
`"partial"` gets all the files under a specified directory ONE BY ONE.
Since it performs downloading for each file respectively (it is mere git command behavior),
it could be significantly slow as the number of files grows.
(In the worst case, you might get an error)

There is a faint chance that it might be sperior than `"shallow"` in terms of memory consumption.
But basically it seems no motivation to use this method.

Commands
--------

### sync

Sync sources depending on the definition of the `.gitwire`.

Please note that it always clears the destination before sync started for each item.

    $ git wire sync

### check

Check sources depending on the definition of the `.gitwire`.

If there are some differences, this command reports each of them all,
and returns with exit code 1, otherwise returns with 0.

    $ git wire check


A sample .gitwire
-----------------

This `.gitwire` sample wires this repository's `src` at revision v1.0.0 into `src_v1.0.0` directory.
https://github.com/msr1k/git-wire/blob/main/.gitwire

## Changelog

- v1.2.0 (2023/09/22)

    Added sparse-checkout feature into `"shallow"` checkout method.
    In almost all cases it is the best in terms of memory & time consumption.

    And previous `"shallow"` checkout method renamed as `"shallow_no_sparse"`,
    it could be an alternative to `"shallow"` if you encount some problems with `"shallow"`.

- v1.1.3 (2023/09/21)

    Fixed: if target repository placed somewhere including `.git`,
    `git wire check` wrongly ignores all the files in this repository,
    and no changes detected if it exists.

- v1.1.2

    Update all dependent crate's versions.
    Fix a typo.

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
