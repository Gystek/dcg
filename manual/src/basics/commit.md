# Recording changes

Now that we have recorded changes to the index, we can record (or
*commit*) them in the revision tree.

```
$ echo "New file" > foo.txt
$ dcg add foo.txt
```

To record changes, we use the `dcg commit` command. We can supply a
commit message by giving it an argument, or write it using a text
editor.

```
$ dcg commit "Add foo.txt"
[master 73164688] Add foo.txt
  1 files created, 0 files deleted and 0 files modified
```

Now that we have recorded the changes to the revision tree, the index
is cleared. If we check its status:

```
$ dcg status
D    foo.txt
```

`foo.txt` appears as `D`eleted, as it was present in the last commit
but does not exist in the current index. If we ran `dcg commit` again,
it would instead say `0 files created, 1 files deleted and 0 files
modified`.

Let's edit `foo.txt` and add it to the index.

```
$ echo "Changed contents" > foo.txt
$ dcg add foo.txt
$ dcg status
M    foo.txt
```

This time, the index says `foo.txt` was `M`odified. We now introduce a
new command that prints the *diff* between the last commit and the
current index:

```
$ dcg diff
foo.txt:
- New file
+ Changed contents
```

As `foo.txt` is a plain text file, there is no syntax to use for
*diff*ing. The file is thus *diff*ed linearily.

Let's commit the new changes:

```
$ dcg commit "Change foo.txt"
[master 3e47a9d8] Change foo.txt
  0 files created, 0 files deleted and 1 files modified
```

We now have two commits. We can check that using the `dcg log`
function. We give it the `--oneline` argument to have a short output:

```
$ dcg log --oneline
3e47a9d8 Change foo.txt
73164688 Add foo.txt
```
