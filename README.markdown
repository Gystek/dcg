dcg
===

dcg aims at being a fast distributed revision control system
analysing files syntactically rather than linearly.

Manual
------

You can find the user manual [here](./manual/). You can view it as markdown on Github
or build it with [mdbook](https://rust-lang.github.io/mdBook/).

Languages
---------

Dcg supports [23 languages](./linguist.toml). It will support more in the future,
among others by allowing custom parsers to be defined.

Unsupported/unrecognized languages are *diff*ed as plain text.

Roadmap
-------

The next goal is to have Git's essential features. See the [roadmap](./ROADMAP.markdown).

Known issues
------------

Dcg is slow and still very work-in-progress. Contributions are welcome.

Internals
---------

See the article, Lean code and Haskell prototype in the
[theory/](./theory/) folder.

Meaning of "dcg"
----------------

Dcg can mean many things depending on how well it performs:
- "dendrochronologit": from ["dendrochronology"](https://en.wikipedia.org/wiki/Dendrochronology) and "Git"
- "diffing code like Git": because that's what it does (poorly)
- "*diffeur à chier de Gustek*": most of the time (I'll leave it up to the user whether or not to translate this)

Licence
-------

dcg is licensed under the GNU General Public License version 2.0 only.  
The full text of the licence can be accessed via [this link](https://www.gnu.org/licenses/old-licenses/gpl-2.0.txt)
and is also included in the [licence file](./COPYING) of this software package.
