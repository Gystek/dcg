dcg
===

dcg aims at being a fast distributed revision control system
analysing files syntactically rather than linearly.

Roadmap
-------

- [ ] Backend
  - [x] `Node` to IR
  - [x] Diff/patch
  - [x] Extract data/metadata from `Node`s
  - [x] Formatting preservation
  - [x] IR serialisation
  - [ ] Merge
- [ ] Frontend
  - [ ] Diff pretty-printer
  - [ ] Git idk

Internals
---------

See the article, Lean code and Haskell prototype in the
[theory/](./theory/) folder.

Licence
-------

dcg is licensed under the GNU General Public License version 2.0 only.  
The full text of the licence can be accessed via [this link](https://www.gnu.org/licenses/old-licenses/gpl-2.0.txt)
and is also included in the [licence file](./COPYING) of this software package.
