# Setup and configuration

## Configuration

Before anything, dcg must be configured. This section will explain the
configuration format, where the configuration files are located and
all configuration keys.

### Configuration files

Configuration is written in the TOML language. Dcg looks through 6
locations in order. Later configurations override earlier ones. Dcg
visits, in order: `/etc/dcgconfig.toml`,
`$XDG_CONFIG_HOME/dcg/config.toml`, `$HOME/.config/dcg/config.toml`,
`$HOME/.dcgconfig.toml`, `$PWD/.dcg/config.toml` and
`$PWD/.dcgconfig.toml`.

### Configuration keys

More keys will be added before the first stable version is released.

- `user.name`: the user's name. This key is mandatory.
- `user.email`: the user's email address. This key is mandatory.

- `init.default_branch`: the default branch name for new dcg
  repositories. This defaults to `master`.

- `commit.editor`: the editor to use to edit commit messages is no
  message has been supplied on the command line.

## Setup

Now that dcg has been configured, we can learn how to setup a new dcg
repository. The command to setup a new repository is `dcg init`.

You can either initialise the repository in a new folder:

```
$ dcg init my_repo
Initialized new dcg repository in 'my_repo'
```

Or in the current folder:

```
$ dcg init
Initialized new dcg repository in '.'
```

You can also customise the default branch name for this repository
with the `--initial-branch` (or `-b`) option (if it is ommited, the
value specified in the configuration or `master` is used):

```
$ dcg init --initial-branch dev
Initialized new dcg repository in '.'
$ cat ./.dcg/refs/HEAD
dev
```
