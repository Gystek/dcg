# Tagging, checking out and undoing changes

As commit hashes are long and unpratical to use, you can use aliases
(or tags) to refer to them with the `dcg tag` command. By default, the
tag points to the head of the current branch:

```
$ dcg log --oneline
3e47a9d8 Change foo.txt
73164688 Add foo.txt
$ dcg tag first-change
$ cat .dcg/refs/tags/first-change
3e47a9d84ea32f65bda68452fcfaaef06b0136e1d0e4a6f60bc3771fa0936dd6
```

You can also explicitely reference a commit:

```
$ dcg tag initial-commit \
  731646889b7fe63b79f648687a30d2861edd92fe7c3cd1f2c485e0a605367624
```
