#set page(
    paper: "a4",
    number-align: center,
    numbering: "1",
	columns: 2,
)
#set heading(numbering: "1.")
#set par(first-line-indent: 2em)
#set text(font: "New Computer Modern")

#show math.equation: set block(breakable: true)

#let mplus = $xor$

#import "@preview/ctheorems:1.1.3": *
#show: thmrules.with(qed-symbol: smallcaps("q.e.d."))

#let theorem = thmbox("theorem", "Theorem",
                      fill: rgb("#eeffee"))
#let lemma = thmbox("theorem", "Lemma")
#let proof = thmproof("proof", "Proof")

#import "@preview/fletcher:0.5.4" as fletcher: diagram, node, edge

#import "@preview/curryst:0.4.0": rule, proof-tree

#let appendix(body) = {
  set heading(numbering: "A.1", supplement: [Appendix])
  counter(heading).update(0)
  body
}

#place(
  top + center,
  scope: "parent",
  float: true,
align(center, text(17pt)[
	*Dating tree rings: CST-based version control*
	])
)

#place(
  top + center,
  scope: "parent",
  float: true,
	align(center, text(11pt)[
    Gustek \ #link("mailto:gustek@riseup.net")[`gustek@riseup.net`]
]))

#align(center)[
    #set par(justify: false)
    *Abstract* \
]

= Introduction

The use of version control systems (VCS) is ubiquitous in
the software development industry. The core of a VCS can
be identified as two main processes: the calculation of
changes between two versions of the program, and the
merging of the said changes when they exist across
different branches. Most VCSs --- such as Git @git, which is almost universally used in open source
software projects --- perform this first step in a
similar fashion as `diff(1)`; that is, linearly. This
strategy, although simple to implement, is unsatisfactory
and suboptimal. This arises from three problems, two of
which happen to be the very problems VCSs have to solve.
Consider the following versions of a file:

```rs
...
really_long_function_name(5);
...
```

```rs
...
really_long_function_name(6);
...
```

Using a linear _diff_ algorithm, the whole line is
considered changed even if only a single character of the 
line has actually been modified.

Let us now consider @abcmerge. Version `a` is the "base" 
version of the file, whereas version `b` and `c` each 
succeed it on a different branch.

#figure(
	grid(
		columns: (1fr, 1fr, 1fr),
		align(center)[
			version `a`:
			```rs
			...
			5 + 6
			...
			```
		],
		align(center)[
			version `b`:
			```rs
			...
			5 + 7
			...
			```
		],
		align(center)[
			version `c`:
			```rs
			...
			5 - 6
			...
			```
		]
	) + diagram(
		node-stroke: .1em,
		spacing: 2.5em,
		node((0, 0), `a`),
		node((1, 0), `b`),
		node((1, 1), `c`),
		edge((0, 0), (1, 0), "-|>"),
		edge((0, 0), (1, 1), "-|>"),
	),
	placement: auto,
	scope: "parent",
	caption: [Position of `a`, `b` and `c` in the history]
) <abcmerge>

If we try and merge version `b` and `c` while the changes 
they describe have been calculated linearly, a merge 
conflict will occur, as the same line has been changed in 
two different ways, although there is no real conflict on 
a syntactical level. Such conflicts, especially when 
multiplied --- as they tend to be --- are very 
time-cosuming to fix and greatly impair productivity,
requiring human intervention on a task that should be 
peformed automatically.

The third problem line-based VCS (or _diff_ programs in 
general) exhibit is the lack of clarity for the user. See 
the example in @minorchange:

#figure(
	grid(
		columns: (1fr, 1fr),
		align(center)[
			version 1:
			```rs
			fn f(a: i32, x: i32) -> i32 {
				(if x % 2 == 0 {
					-1
				} else {
					x
				}) + a
			}

			fn main() {
				let v = vec![1, 2, 3];
				let x = v.iter().fold(0, f);
			}
			```
		],
		align(center)[
			version 2:
			```rs
			fn f(a: i32, x: i32) -> i32 {
				(if x % 2 == 0 {
					2
				} else {
					x
				}) + a
			}

			fn main() {
				let v = vec![1, 2, 3];
				let x = v.iter().fold(0, f);
			}
			```
		]
	),
	placement: auto,
	scope: "parent",
	caption: [minor change between two Rust files]
) <minorchange>

The difference between both versions as calculated by a 
linear algorithm is the replacement of the line 
containing ```rs -1``` by one containing ```rs 2```. It 
would be difficult for the user to figure out what the 
change represents and he couldn't have more information 
on the actual nature of the change, given that the linear 
_diff_ is not syntax-aware.

In this paper, we study the computation of _diffs_ (ie. 
collections of changes between two versions of a program) 
of arborescent structures and the merging of such diffs. 
By applying such computations to syntax trees, the 
problems highlighted in the previous examples would be 
solved, as the difference between the two lines of code 
of the first example would be reduced to $5 --> 
6$ (resulting in smaller _diff_ files), there would be no
merge conflicts in the history described by @abcmerge, 
given that version `b` modifies an _operand_ whereas 
version `c` changes an _operator_, and syntax-awareness 
would allow for helpful contextualisation when displaying 
_diff_ files to the user.

In this article, we tackle the issue of producing an 
optimal _diff_ for recursive structures. For doing so, we 
introduce an expressive language for representing 
structural changes and present algorithms for calculating 
changes and applying them to recursive structures. We 
prove the correction of these algorithms and discuss both 
theoretical and practical optimisations. We also bring 
forth an algorithm for merging structural changes, 
proving the correction thereof. Finally, we compare the 
performance of our solution to linear diffs and existing 
structural analysers in real-world situations and review 
the existing literature and implementations on this topic.

= Diffs and trees

The algorithms we describe here are process binary
trees $Tau$ defined as follows:

$ Tau ::&= kappa : A --> Tau \
        &| tau_i : Tau --> Tau --> Tau "where" i : B $

The types $A$ and $B$ respectively represent a "data" 
type and a "metadata" type for the trees. The only
constraint placed upon them is that there exists an
equivalence relation for each of them.

However, most parsers return the children of nodes as a
_list_ of trees (ie. concrete syntax trees as rose trees). We thus define a conversion function
from such a tree (written $Tau_R$) to a binary tree $Tau$
and backwards.
We also define two utilitary values: $"cons"_B$, the
metadata marker for a converted cons cell and $"nil"_Tau$,
a special variant of $kappa$. In describing the conversion
algorithm, we use linked list with the usual `cons` and
`nil` functions. Let $c_(r->b) : Tau_R --> Tau$ and
$c_(b->r) : Tau --> Tau_R$ respectively be the conversion 
function from rose trees to binary trees and vice-versa:

$ c_(r->b)(kappa_R (x)) &= kappa(x) \
c_(r->b)(tau_(R i)("cons"(x, "nil"))) &= tau_i (c_(r->b)
(x), "nil"_Tau) \
c_(r->b)(tau_(R i)("cons"(x, x'))) &= tau_i (c_(r->b)
(x), c_(r->b)(tau_(R "cons"_B)(x'))) \
c_(r->b)(tau_(R i)("nil")) &= tau_i ("nil"_Tau, 
"nil"_Tau) $

$ c_(b->r)(kappa(x)) &= kappa_R (x) \
c_(b->r)(tau_i (x, "nil"_Tau)) &= tau_(R i)("cons"(c_
(b->r)(x), "nil")) \
c_(b->r)(tau_i (x,y)) &= tau_(R i)(c_(b->r)(x)::c_(b->r) '
(y)) $

where $c_(b->r) ' : Tau --> "list" Tau_R$ is a utility 
function that is defined as follows:

$ c_(b->r) '("nil"_T) &= "nil" \
c_(b->r) '(tau_("cons"_B) (x, y)) &= "cons"(c_(b->r)(x), 
c_(b->r) '(y)) $

All the cases that are unmatched by the $c_(b->r)$ (and
incidentally $c_(b->r) '$) function correspond to badly-formed binary trees and should return an error when encountered

#lemma("Conversion correctness")[
    $forall t : Tau_R, c_(b->r)(c_(r->b)(t)) = t$
] <lemmaconv>

#proof[
    See @lemmaconv_proof
]

We can now define a _diff_ type $Delta$ to represent
changes between binary trees. It can be seen that its
structure is much more complex than that of
unidimensional (ie. linear) diffs.

$ Delta ::&= epsilon : Delta \
          &| t_(epsilon i) : Delta --> Delta --> Delta \
          &| mu : Tau --> Tau --> Delta \
          &| t_(mu i->j) : Delta --> Delta --> Delta \
          &| pi_(tack.l i) : Tau --> Delta --> Delta \
          &| pi_(tack.r i) : Delta --> Tau --> Delta \
          &| beta_tack.l : Delta --> Delta \
          &| beta_tack.r : Delta --> Delta $

$epsilon$ indicates the absence of change between two
binary trees. $t_epsilon$ indicates an equality in
node type (and thus that the computation of changes
follows on the next level). $mu$ formalises the
_modification_ of a node, while $t_mu$ signifies
the modfication of the node _type_ between the left
and right trees (and indicates the lower-level changes).
$pi_tack.l$ and $pi_tack.r$ indicate the addition of
a depth level, defining an arbitrary tree as the 
respectively left and right child of the new node and
indicating the calculated changes for the new node's
(respectively) right and left child. Conversely, 
$beta_tack.l$ and $beta_tack.r$ indicate the deletion
of a node and the continuation of the computation on the 
right and the left, respectively, discarding the 
other-hand child.

We define a weight function $w : Delta --> NN$ on diffs, indicative of the cost of applying (and storing) the _diff_
(nb. $|x|$ is the size of $x : Tau$).

$ w(epsilon) &= 0 \
w(t_(epsilon i)(x, y)) &= w(x) + w(y) \
w(mu(x, y)) &= 1 + |x| + |y| \
w(t_(mu i->j)(x, y)) &= 1 + w(x) + w(y) \
w(pi_(tack.l"/"tack.r i)(t, delta)) &= 1 + |t|+ w(delta)\
w(beta_(tack.l"/"tack.r i)(delta)) &= 1 + w(delta) $

We also define a $min_w : Delta --> Delta --> Delta$ function, yielding the _diff_ having the smallest weight of the two, along with its generalisation for every $n in NN^*$, $min_w : Delta^n --> Delta$.

= Diffing and patching

== Principle

If we represent trees and diffs as an arithmetical 
system, we can define the _diff_ operation as an external 
substraction $- : Tau --> Tau --> Delta$, such that 
$delta = y - x$. We can then define the _patch_ operation 
as an external addition $+ : Tau --> Delta --> Tau$, such 
that $x + delta = y$. It then follows that $x + (y - x) = 
y$. The _diff_ function can be described as 
"$epsilon$-potent", given that $x - x = epsilon$.

It is worth noting that the _patch_ function is not 
actually defined on $Tau --> Delta --> Tau$, rather on 
$Tau --> Delta_t --> Tau$, where $Delta_t$ is the set of 
diffs applicable to a specific tree $t$, on which we can 
place the following bound: ${epsilon; mu (t, u) | u : Tau} subset Delta_t$.

== Algorithms

We thus define the _diff_ function $d : Tau --> Tau --> Delta$:

$ d(kappa(x), kappa(y)) &= cases(
    epsilon "if" x = y,
    mu(x, y) "else"
) \
d(tau_i (x, y), tau_j (x', y')) &= cases(
    min_w (delta_epsilon, delta_(pi_tack.l),
    delta_(pi_tack.r), delta_(beta_tack.l),
    delta_(beta_tack.r)) "if" i = j,
    min_w (delta_mu, delta_(t mu), delta_(pi_tack.l),
    delta_(pi_tack.r), delta_(beta_tack.l),
    delta_(beta_tack.r)) "else"
) \
"where" & delta_epsilon = t_(epsilon i) (d(x, x'), d(y, y')) \
		& delta_(t mu) = t_(mu i -> j) (d(x, x'), d(y, y')) \
        & delta_mu = mu(tau_i (x, y), tau_j (x', y')) \
        & delta_(pi_tack.l) = pi_(tack.l j) (x', d(tau_i (x, y), y')) \
        & delta_(pi_tack.r) = pi_(tack.r j) (d(tau_i (x, y), x'), y') \
        & delta_(beta_tack.l) = beta_tack.l (d(y, tau_j (x', y'))) \
  "and" & delta_(beta_tack.r) = beta_tack.r (d(x, tau_j (x', y'))) \
d(kappa(a), tau_i (x, y)) &= "min"_w (delta_mu,
                                      delta_(pi_tack.l),
                                      delta_(pi_tack.r))\
"where" & delta_mu = mu(kappa(a), tau_i (x, y)) \
        & delta_(pi_tack.l) = pi_(tack.l i)(x, d(kappa(a), y)) \
"and"   & delta_(pi_tack.r) = pi_(tack.r i)(y, d(kappa(a), x)) \
d(tau_i (x, y), kappa(a)) &= "min"_w (delta_mu,
                                      delta_(beta_tack.l),
                                      delta_beta_tack.r)\
                        
"where" & delta_mu = mu(tau_i (x, y), kappa(a)) \
        & delta_beta_tack.l = beta_tack.l (d(y, kappa(a)))\
"and"   & delta_beta_tack.r = beta_tack.r (d(x, kappa(a)) $

The _diff_ output by $d$ is optimal in size:
#lemma("diff optimality")[
		$
		t,t': Tau \
		delta = d(t, t') \
		delta' : Delta, p(t, delta') = t' \
		w(delta') >= w(delta).
		$
] <optimal>
#proof[
	See @optimal_proof.
]

We then define the _patch_ function $p : Tau --> Delta --> Tau$:

$ p(x, epsilon) &= x \
p(x, mu(x, y)) &= y \
p(tau_i (x, y), t_(epsilon i) (delta_x, delta_y)) &= tau_i (p(x, delta_x), p(y, delta_y)) \
p(x, pi_(tack.l i) (x', delta_y)) &= tau_i (x', p(x, delta_y)) \
p(x, pi_(tack.r i) (y', delta_x)) &= tau_i (p(x, delta_x), y') \
p(tau_i (\_, y), beta_tack.l (delta_y)) &= p(y, delta_y) \
p(tau_i (x, \_), beta_tack.r (delta_x)) &= p(x, delta_x) \
p(tau_i (x, y), t_(mu i->j) (delta_x, delta_y)) &= tau_j (p(x,delta_x), p(y, delta_y)) $

One can see that the definition of $p$ does not match the
entirety of $Tau times Delta$. In such cases not defined
here, an implementation
of the algorithm should throw an error, indicating that the
provided _diff_ is incompatible with the tree.

== Correctness

In this section, we shall prove the correctness of the 
_diff_-_patch_ pipeline. For this, we introduce the following
lemmas and relation: $cal(R) subset Tau times Tau times Delta$, defined by the following inference rules. For convenience, we write the proposition
$(x, y, z) in cal(R)$ as $x | y ~> z$.

#figure(align(center, [
	$t | t ~> epsilon$

	$t | t' ~> mu(t, t')$

	#proof-tree(rule(
		[$tau_i (x, y) | tau_j (x', y') ~>
	t_(mu i->j) (delta_x, delta_y)$],
		[$x | x' ~> delta_x$],
		[$y | y' ~> delta_y$]))

	#proof-tree(rule(
		[$tau_i (x, y) | tau_i (x', y') ~> t_(epsilon i)
		(delta_x, delta_y)$],
		[$x | x' ~> delta_x$],
		[$y | y' ~> delta_y$]
	))

	#proof-tree(rule(
		[$t | tau_j (x', y') ~> t_pi_(tack.l j) (x', delta_y)$],
		[$t | y' ~> delta_y$]
	))

	#proof-tree(rule(
		[$t | tau_j (x', y') ~> t_pi_(tack.r j) (delta_x, y')$],
		[$t | x' ~> delta_x$]
	))

	#proof-tree(rule(
		[$tau_i (x, y) | t ~> t_beta_tack.l (delta_y)$],
		[$y | t ~> delta_y$]
	))

	#proof-tree(rule(
		[$tau_i (x, y) | t ~> t_beta_tack.r (delta_x)$],
		[$x | t ~> delta_x$]
	))
]),
caption: [Inference rules for $cal(R)$]
)

The relation $cal(R)$ is the relation
between the input and the output of $d$, allowing for
multiple images for a single input and thus getting rid of
the $min_w$ function in the _diff_ process. We then use it
as a proof device for simpler induction on diffs.

#lemma[
		$forall t,t': Tau, delta: Delta, d(t, t') = delta ==> (t, t', delta) in cal(R)$
] <dspec>
#proof[By case disjunction on $(t,t')$. For every 
		case, we suppose that $delta = d(t, t')$ and we prove 
		that $(t, t', delta) in cal(R)$. \
		We then replace $d(t, t')$ by its expression and 
		simplify the conditions for every case. From this, we 
		can eliminate the two trivial cases involving 
		constants on both sides, $(kappa(x), kappa(x))$ and 
		($kappa(x), kappa(y)$). \
		For all other cases, we apply another case disjunction
		on the output of $min_w$. $cal(R)$ is now trivially
		defined for every case of this new disjunction.
]

#lemma[
		$forall t, t' : Tau, delta : Delta, (t, t', delta)
		in cal(R) ==> p(t, delta) = t'$
] <dpspec>
#proof[
		By case disjunction on the different elements of
		$cal(R)$. From then, one can trivially see from the
		definition of $p$ that $(t, t', delta) in cal(R)
		==> p(t, delta) = t'$.
]

We now prove the correctness of the pipeline:
#theorem("Correctness")[
    $forall t, t' : Tau, p(t, d(t, t')) = t'$
] <correctness>
#proof[
	From @dspec and @dpspec, we see that $forall t, t' : Tau,
	delta : Delta, d(t, t') = delta ==> p(t, delta) =  t'$,
	thus $p(t, d(t, t')) = delta$.
]

= Merging

== Principle

If we take up the same arithmetical system as described in
the _diff_/_patch_ part, we can define the _merged diff_ of
$delta_1$ and $delta_2$, $delta_3 = m(delta_1, delta_2)$,
as the _diff_ which, when patched onto the base tree $t$ of
both $delta_1$ and $delta_2$, includes both the changes
described in $delta_1$ and those described in $delta_2$.

== Algorithm

We thus define the merge function $m : Delta --> Delta -->  Delta$:

$ 
	m(epsilon, x) &= x \
	m(x, epsilon) &= x \
	m(t_(epsilon i)(l, r), t_(epsilon i)(l', r'))
	&= t_(epsilon i)(m(l, l'), m(r, r')) \
	m(t_(mu i -> j)(l, r), t_(mu i -> j)(l', r'))
	&= t_(mu i->j)(m(l, l'), m(r, r')) \
	m(t_(epsilon i)(l, r), t_(mu i -> j)(l', r'))
	&= t_(mu i->j)(m(l, l'), m(r, r')) \
	m(t_(mu i -> j)(l', r'), t_(epsilon i)(l, r))
	&= t_(mu i->j)(m(l, l'), m(r, r')) \
	m(t_(epsilon i)(l, r), pi_(tack.l j)(t, delta))
	&= pi_(tack.l j)(t, m(t_(epsilon i)(l, r), delta)) \
	m(t_(epsilon i)(l, r), pi_(tack.r j)(delta, t))
	&= pi_(tack.r j)(m(t_(epsilon i)(l, r), delta), t) \
	m(pi_(tack.l j)(t, delta), t_(epsilon i)(l, r))
	&= pi_(tack.l j)(t, m(t_(epsilon i)(l, r), delta)) \
	m(pi_(tack.r j)(delta, t), t_(epsilon i)(l, r))
	&= pi_(tack.r j)(m(t_(epsilon i)(l, r), delta), t) \
	m(t_(epsilon i)(\_, r), beta_tack.l (delta))
	&= beta_tack.l (m(r, delta)) \
	m(t_(epsilon i)(l, \_), beta_tack.r (delta))
	&= beta_tack.r (m(l, delta)) \
	m(beta_tack.l (delta), t_(epsilon i)(\_, r))
	&= beta_tack.l (m(r, delta)) \
	m(beta_tack.r (delta), t_(epsilon i)(l, \_))
	&= beta_tack.r (m(l, delta)) \
	m(pi_(tack.l\/tack.r i)(t, delta), pi_(tack.l\/tack.r i)(t, delta'))
	&= pi_(tack.l\/tack.r i)(t, m(delta, delta')) \
	m(beta_(tack.l\/tack.r)(delta), beta_(tack.l\/tack.r)(delta'))
	&= beta_(tack.l\/tack.r)(m(delta, delta')) \
	m(x, x) &= x \
$

One can see that the definition of $m$ does not match
the entirety of $Delta^2$. In cases not defined in the
algorithm, a _merge conflict_ has occured and an
implementation of the algorithm should throw an error,
indicating the location of the conflict to allow for
fixing.

== Correctness

= Optimisation

== $epsilon$-reduction

The first theoretical optimisation strategy is
$epsilon$-reduction, that is folding the diffs
that are equivalent to an absence of change into
$epsilon$. Such an optimisation can easily be
defined by the following $epsilon_R : Delta --> Delta$
function:

$
  epsilon_R (t_(epsilon i)(x, y)) &= cases(
	epsilon "if" epsilon_R (x) = epsilon_R (y) = epsilon,
	t_(epsilon i)(epsilon_R (x), epsilon_R (y)) "else"
  ) \
  epsilon_R (mu(x, y)) &= cases(
	epsilon "if" epsilon_R (x) = epsilon_R (y),
	mu(epsilon_R (x), epsilon_R (y)) "else"
  ) \
  epsilon_R (t_(mu i->j)(x, y)) &=
  t_(mu i->j)(epsilon_R (x), epsilon_R (y)) \
  epsilon_R (pi_(tack.l\/tack.r i)(t, delta)) &=
  pi_(tack.l\/tack.r i)(t, epsilon_R (delta)) \
  epsilon_R (beta_(tack.l\/tack.r i)(delta)) &=
  beta_(tack.l\/tack.r i)(epsilon_R (delta)) \
  epsilon_R (epsilon) &= epsilon \
$

== Heuristics

= Pratical considerations

== Implementation strategies

The first strategy we used was treating the diffing problem
as a shortest-path finding problem in a directed acyclic graph (DAG):

#figure(
	diagram(
		node-stroke: .1em,
		spacing: 2.5em,
		node((0, 0), `foo(a,b) | foo(a, c)`),
		node((-2, 1.5), `a | a`, fill: red),
		node((-1.25, 1.5), `b | c`, fill: orange),
		node((-0.5, 1.5), `foo(a, b) | c`),
		node((0.5, 1.5), `foo(a, b) | a`),
		node((1.5, 1.5), `b | foo(a, c)`),
		node((2.75, 1.5), `a | foo(a, c)`),

		node((-2, 2.5), $script(emptyset)$, fill: red),
		node((-1.25, 2.5), $script(emptyset)$, fill: orange),

		edge((0, 0), (-2.25, 1.3), "-|>", label-side: center, $script(t_epsilon (\_, emptyset))$),
		edge((0, 0), (-1.25, 1.5), "-|>", label-side: center, $script(t_epsilon (emptyset, \_))$),
		edge((0, 0), (-0.5, 1.5), "--|>", label-side: center, $script(pi_tack.l (a, \_))$),
		edge((0, 0), (0.5, 1.5), "--|>", label-side: center, $script(pi_tack.r (\_, c))$),
		edge((0, 0), (1.5, 1.5), "--|>", label-side: center, $script(beta_tack.l)$),
		edge((0, 0), (2.75, 1.5), "--|>", label-side: center, $script(beta_tack.r)$),

		edge((-2, 1.5), (-2, 2.5), "-|>", label-side: center, $script(epsilon)$),
		edge((-1.25, 1.5), (-1.25, 2.5), "-|>", label-side: center, $script(mu (b, c))$)
	),
	scope: "parent",
	placement: auto,
	caption: [Graph formulation of the _diff_ problem]
) <graph-diff>

@graph-diff displays the unfolding of the thus conceived diffing process,
implemented using a modified version of the A\* algorithm.
All nodes shown in the figure were pushed onto the min-heap at some point
and dotted edges indicate unvisited paths. Colours indicate that such nodes
were constructed by recursive calls to the `diff` function. The heap we
use minimises $f(n) = g(n) + h(n)$ where $g$ and $h$ are calculated as follows:
$g(n)$ is 0 for $epsilon$ and $t_epsilon$ (and necessarily for the initial
node), $|x| + |y|$ for $mu (x, y)$ and 1 for other constructors. If corresponds
to the previously defined $w$ function when applied to the entire graph.
The heuristic function $h$ is defined by $h(l, r) = min(|l|, |r|)$, where
$l$ and $r$ are respectively the left and right trees the _diff_ is processing.
We also have $h(emptyset) = 0$ and when dealing with recursive (i.e. binary)
constructors, the smallest heuristic value is kept.

When compared to a rather naive implementation (with memoisation of already-_diff_-ed
nodes as sole optimisation), this method has shown to greatly reduce (approximately tenfold)
the time needed to _diff_ the same file pairs.

== Unidentified, plain text and binary files

Unidentified and plain-text files are _diff_-ed linearily using the Histogram
_diff_ algorithm. Linear _diffs_ are merged in a similar fashion as tree _diffs_,
emulating a three-way merge.

Binary files are stored _as-is_ on the filesystem given that it is more expensive both
to compute and to store binary _diffs_. _Diffs_ are stored in the index (and later embedded
in the commits) and the type of each diff is kept in an index-wide registry.

= Performance

== Methodology

== Results

= Related work

= Further research

= Conclusion

#bibliography(title: "References", "biblio.yml")

#show: appendix

= Some proofs

== @lemmaconv <lemmaconv_proof>

== @optimal <optimal_proof>