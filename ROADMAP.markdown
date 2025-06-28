Roadmap
=======

Migrated from [the README](./README.markdown) for readability reasons.

Milestone 2 - (scarce) equivalent to Git
----------------------------------------

- [x] Config and setup
  - [x] `init`
  - [x] config
  - [x] ignore file
- [x] Index operations
  - [x] Indexing/retrieval system
  - [x] `add`
  - [x] `rm`
  - [x] `status`
  - [x] `diff`
- [ ] Revision and branching operations
  - [x] Diff storage
	- [x] Code files
	- [x] Plain text/unidentified files
	- [x] Binary files
  - [ ] `commit`
  - [ ] `tag`
  - [ ] `log`
  - [ ] `reset`
  - [ ] `branch`
  - [ ] `checkout`
  - [ ] `merge`
- [ ] Remotes
  - [ ] general remote management
  - [ ] `push`
  - [ ] `pull`
- [ ] User manual

Milestone 1 - (slow) equivalent to [difft](https://github.com/Wilfred/difftastic) and [mergiraf](https://mergiraf.org/)
-----------------------------------------------------------------------------------------------------------------------

- [x] `Node` to IR
- [x] Diff/patch
- [x] Extract data/metadata from `Node`s
- [x] Formatting preservation
- [x] IR serialisation
- [x] Merge
- [x] Programming language linguist
- [x] Binary file identification
- [x] Basic diff optimisation
  - [x] ε-reduction
  - [x] Reduction to a graph problem
- [x] Plain text file handling
  - [x] Linear diff/patch
  - [x] Linear merge

Milestone 3 - glitter, speed and extensibility
---------------------------------------

- [ ] Diff pretty-printer
- [ ] Extend configuration
  - [ ] attributes file (linguist override)
- [ ] Merge conflict pretty-printer
- [ ] Linguist improvements
  - [ ] Custom parsers
  - [ ] Add language heuristics
- [ ] Diff heuristics (and other optimisations)
- [ ] Various linear diff algorithms
