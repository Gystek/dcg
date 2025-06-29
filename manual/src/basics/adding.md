# Adding files

Now that we have setup an empty dcg repository, we can learn how to
add files to the index. Throughout the remainder of this manual, it
will be assumed that there is a dcg repository somewhere in the
current directory's hierarchy.

Let's run `dcg status` once. This command displays the status of the
current repository's index:

```
$ dcg status
$
```

Unsurprisingly, nothing shows up. Let's create a new file and add it
to the index with `dcg add`:

```
$ echo "Test file" > foo.txt
$ dcg add foo.txt
$ dcg status
A    foo.txt
```

The `A` here stands for "added". We will see what other status markers
are possible in the next chapter.

You can also delete the file with `dcg rm`:

```
$ dcg rm foo.txt
$ dcg status
$
```

As you can see, the index is now empty.

You can `dcg add` directories, which contents will be recursively
added to the index.

```
$ mkdir test
$ echo "Test file" > test/foo.txt
$ echo "Another one" > test/bar.txt
$ dcg add test
$ dcg status
A    test/foo.txt
A    test/bar.txt
```

Paths given to `dcg add` can also contain globs:

```
$ echo "Test file" > foo.txt
$ echo "Another one" > bar.txt
$ dcg add *.txt
$ dcg status
A    foo.txt
A    bar.txt
```
