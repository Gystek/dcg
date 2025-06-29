# Introduction

Almost everyone uses version control systems (VCSs) (such as Git). Most of
these VCSs calculate changes between files in a linear fashion,
ie. line by line. This approach has several problems, most notably
lack of precision in *diff*s, phantom changes (ie. changes that do not
actually affect the structure of the program yet register as changes),
lack of readability and the infamous merge conflicts.

Dcg is different. Instead of performing *diff* operations linearily,
it computes differences between files based on their concrete syntax
tree (CST) structure: dcg *understands* programming language syntax.

Git can be described as a content-addressable filesystem around which
a VCS has been wrapped. On the contrary, dcg is a VCS which happens to
make use of content-addressable storage. Git reasons and stores
snapshot in files whereas dcg only stores *diff*s. This prevents
considerable overhead, which cannot be avoided even when using
content-addressable storage systems if snapshots are stored instead of
*diff*s.

Dcg ~~will~~ is also ~~be~~ distributed, just like Git is.

Dcg can be referred to as the slow combination of
[difft](http://github.com/Wilfred/difftastic),
[mergiraf](https://mergiraf.org) and [Git](https://git-scm.com).

## Installation

As the first stable version hasn't yet been released, dcg has to be
installed from source:

```
$ git clone https://github.com/Gystek/dcg
$ cd dcg/
$ cargo install
```
