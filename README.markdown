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
  - [x] Merge
  - [ ] Linguist
    - [x] Filetype linguist
	- [x] Binary file identification
	- [ ] Extend configuration
  - [x] Linear diff
  - [ ] Diff optimisation
	- [x] ε-reduction
	- [x] Reduction to graph problem
	- [ ] Diff heuristics
  - [ ] Custom parsers
  - [ ] Git
	- [ ] Index
	  - [ ] Diff storage
	    - [ ] Code files
	    - [ ] Plain text/unidentified files
	    - [ ] Binary files
- [ ] Frontend
  - [ ] Diff pretty-printer
  - [ ] Merge conflict pretty-printer
  - [ ] Git commands
	- [ ] Config and setup
      - [ ] init
	  - [ ] config
	  - [ ] ignore file
	  - [ ] attributes file (linguist override)
	- [ ] Index operations
	  - [ ] add
	  - [ ] rm
	  - [ ] (mv)
	  - [ ] status
	  - [ ] diff
	- [ ] Revision and branching operations
	  - [ ] commit
	  - [ ] tag
	  - [ ] log
	  - [ ] reset
	  - [ ] branch
	  - [ ] checkout
	  - [ ] merge
	- [ ] Remotes
	  - [ ] general remote management
	  - [ ] push
	  - [ ] pull
  - [ ] User manual

Internals
---------

See the article, Lean code and Haskell prototype in the
[theory/](./theory/) folder.

Licence
-------

dcg is licensed under the GNU General Public License version 2.0 only.  
The full text of the licence can be accessed via [this link](https://www.gnu.org/licenses/old-licenses/gpl-2.0.txt)
and is also included in the [licence file](./COPYING) of this software package.
