import Mathlib.Order.Basic
import Mathlib.Tactic.Lemma
import Mathlib.Tactic.Linarith
import Aesop

section definitions
variable (Data Meta : Type)

inductive Side : Type where
  | left : Side
  | right : Side
deriving instance DecidableEq for Side

inductive Syn : Type where
  | κ : Option Data → Syn
  | τ : Meta → Syn → Syn → Syn
deriving instance DecidableEq for Syn

inductive Diff : Type where
  | ε : Diff
  | tε : Meta → Diff → Diff → Diff
  | μ : Syn Data Meta → Syn Data Meta → Diff
  | tμ : Meta → Meta → Diff → Diff → Diff
  | tπ : Side → Meta → Syn Data Meta → Diff → Diff
  | tβ : Side → Diff → Diff
deriving instance DecidableEq for Diff
end definitions

namespace Syn
variable {Data Meta : Type} [DecidableEq Data] [DecidableEq Meta]

def size : Syn Data Meta → Nat
  | .κ none => 0
  | .κ _ => 1
  | .τ _ x y => 1 + size x + size y
end Syn

namespace Diff
variable {Data Meta : Type} [DecidableEq Data] [DecidableEq Meta]

def w : Diff Data Meta → Nat
  | .ε => 0
  | .tε _ x y => x.w + y.w
  | .μ x y => 1 + x.size + y.size
  | .tμ _ _ x y => 1 + x.w + y.w
  | .tπ _ _ t d => 1 + d.w + t.size
  | .tβ _ d => 1 + d.w

def size : Diff Data Meta → Nat
  | .ε => 1
  | .tε _ x y => 1 + x.size + y.size
  | .μ x y => 1 + 1 + x.size + y.size
  | .tμ _ _ x y => 1 + x.size + y.size
  | .tπ _ _ t d => 1 + d.size + t.size
  | .tβ _ d => 1 + d.size

instance lt : LT (Diff Data Meta) :=
  ⟨fun x y => x.w < y.w⟩

def minw (x y : Diff Data Meta) : Diff Data Meta :=
  if x.w < y.w then
    x
  else
    y

@[aesop norm]
def merge : Diff Data Meta → Diff Data Meta → Option (Diff Data Meta)
  | .ε, x => .some x
  | x, .ε => .some x
  | .tε i l r, .tε j l' r' => if i = j then .tε i <$> merge l l' <*> merge r r' else .none
  | .tε i l r, .tμ i' j l' r' => if i = i' then .tμ i j <$> merge l l' <*> merge r r' else .none
  | .tμ i' j l' r', .tε i l r => if i = i' then .tμ i j <$> merge l l' <*> merge r r' else .none
  | .tε i l r, .tπ side j t d => .tπ side j t <$> merge (.tε i l r) d
  | .tπ side j t d, .tε i l r => .tπ side j t <$> merge (.tε i l r) d
  | .tε _ _ r, .tβ .left d => .tβ .left <$> merge r d
  | .tβ .left d, .tε _ _ r => .tβ .left <$> merge r d
  | .tε _ l _, .tβ .right d => .tβ .right <$> merge l d
  | .tβ .right d, .tε _ l _ => .tβ .right <$> merge l d
  | .tπ i s t d, .tπ i' s' t' d' => if i = i' && s = s' && t = t' then .tπ i s t <$> merge d d' else .none
  | .tβ s d, .tβ s' d' => if s = s' then .tβ s <$> merge d d' else .none
  | x, y => if x = y then .some x else .none
  termination_by x y => x.size + y.size
  decreasing_by all_goals simp [Diff.size]; linarith
end Diff

namespace Syn
variable {Data Meta : Type} [DecidableEq Data] [DecidableEq Meta]

def diff : Syn Data Meta → Syn Data Meta → Diff Data Meta
  | .κ x, .κ y => if x = y then .ε else .μ (.κ x) (.κ y)
  | .τ a x y, .τ b x' y' => let di := .tε a (diff x x') (diff y y');
                            let dm := .μ (.τ a x y) (.τ b x' y');
                            let dtm := .tμ a b (diff x x') (diff y y')
                            let dal := .tπ .left b x' (diff (.τ a x y) y');
                            let dar := .tπ .right b y' (diff (.τ a x y) x');
                            let ddl := .tβ .left (diff y (.τ b x' y'));
                            let ddr := .tβ .right (diff x (.τ b x' y'));
                            if a = b then
                              List.foldl Diff.minw di [dal, dar, ddl, ddr]
                            else
                              List.foldl Diff.minw dm [dtm, dal, dar, ddl, ddr]
  | .κ a, .τ t x y => let dm := .μ (.κ a) (.τ t x y);
                      let dal := .tπ .left t x (diff (.κ a) y);
                      let dar := .tπ .right t y (diff (.κ a) x);
                      List.foldl Diff.minw dm [dal, dar]
  | .τ t x y, .κ a => let dm := .μ (.τ t x y) (.κ a);
                      let ddl := .tβ .left (diff y (.κ a));
                      let ddr := .tβ .right (diff x (.κ a));
                      List.foldl Diff.minw dm [ddl, ddr]

def patch : Syn Data Meta → Diff Data Meta → Option (Syn Data Meta)
  | x, .ε => x
  | a, .μ x y => if a = x then .some y else .none
  | .τ t x y, .tε t' dx dy => if t = t' then .τ t <$> patch x dx <*> patch y dy else .none
  | x, .tπ .left t x' dy => .τ t x' <$> patch x dy
  | x, .tπ .right t y' dx => .τ t <$> (patch x dx) <*> .some y'
  | .τ _ _ y, .tβ .left dy => patch y dy
  | .τ _ x _, .tβ .right dx => patch x dx
  | .τ t x y, .tμ t0 t1 dx dy => if t0 = t then .τ t1 <$> patch x dx <*> patch y dy else .none
  | _, _ => .none
end Syn

section specification
variable {Data Meta : Type} [DecidableEq Data] [DecidableEq Meta]

@[aesop safe constructors]
inductive DiffRel : Syn Data Meta → Syn Data Meta → Diff Data Meta → Prop where
  | refl : DiffRel t t .ε
  | mod: DiffRel t t' (.μ t t')
  | tmd : DiffRel x x' dx
          → DiffRel y y' dy
          → DiffRel (.τ a x y) (.τ b x' y') (.tμ a b dx dy)
  | tid : DiffRel x x' dx
          → DiffRel y y' dy
          → DiffRel (.τ a x y) (.τ a x' y') (.tε a dx dy)
  | tal : DiffRel t y' dy
          → DiffRel t (.τ b x' y') (.tπ .left b x' dy)
  | tar : DiffRel t x' dx
          → DiffRel t (.τ b x' y') (.tπ .right b y' dx)
  | tdl : DiffRel y t dy
          → DiffRel (.τ a x y) t (.tβ .left dy)
  | tdr : DiffRel x t dx
          → DiffRel (.τ a x y) t (.tβ .right dx)

@[aesop safe constructors]
inductive MergeRel : Diff Data Meta → Diff Data Meta → Diff Data Meta → Prop where
  | refl : MergeRel x x x
  | xε : MergeRel x .ε x
  | εx : MergeRel .ε x x
  | εε : MergeRel x x' mx
       → MergeRel y y' my
       → MergeRel (.tε i x y) (.tε i x' y') (.tε i mx my)
  | εμ : MergeRel x x' mx
       → MergeRel y y' my
       → MergeRel (.tε i x y) (.tμ i j x' y') (.tμ i j mx my)
  | με : MergeRel x x' mx
       → MergeRel y y' my
       → MergeRel (.tμ i j x y) (.tε i x' y') (.tμ i j mx my)
  | επ : MergeRel (.tε i x y) d md
       → MergeRel (.tε i x y) (.tπ s j t d) (.tπ s j t md)
  | πε : MergeRel d (.tε j x y) md
       → MergeRel (.tπ s i t d) (.tε j x y) (.tπ s i t md)
  | εβL : MergeRel r d md
        → MergeRel (.tε _ _ r) (.tβ .left d) (.tβ .left md)
  | εβR : MergeRel l d md
        → MergeRel (.tε _ l _) (.tβ .right d) (.tβ .right md)
  | βεL : MergeRel d r md
        → MergeRel (.tβ .left d) (.tε _ _ r) (.tβ .left md)
  | βεR : MergeRel d l md
        → MergeRel (.tβ .right d) (.tε _ l _) (.tβ .right md)
  | ππ : MergeRel d d' md
       → MergeRel (.tπ i s t d) (.tπ i s t d') (.tπ i s t md)
  | ββ : MergeRel d d' md
       → MergeRel (.tβ s d) (.tβ s d') (.tβ s md)
end specification

section diff_proof
variable {Data Meta : Type}

lemma minw_lemma {δ δ' Δ : Diff Data Meta} :
  δ.minw δ' = Δ →
  δ = Δ ∨ δ' = Δ
:= by
  intro h
  rw [← h]
  simp [Diff.minw]
  cases Nat.decLt δ.w δ'.w <;> aesop

variable [DecidableEq Data] [DecidableEq Meta]

lemma diff_spec {t t' : Syn Data Meta} {δ : Diff Data Meta} :
  Syn.diff t t' = δ →
  DiffRel t t' δ
:= by
revert δ
induction t, t' using Syn.diff.induct
  <;> intro δ h
  <;> simp [Syn.diff] at h
all_goals try (rename _ ≠ _ => neq; simp [neq] at h)
all_goals try (rw [← h]; constructor)
all_goals (
  repeat (cases (minw_lemma h) <;> clear h <;> rename_i h)
    <;> try (rw [← h] at *; constructor <;> aesop)
)

theorem diff_patch_correct_spec {t t' : Syn Data Meta} {δ : Diff Data Meta} :
  DiffRel t t' δ →
  Syn.patch t δ = .some t'
:= by intro h; induction h <;> simp [Syn.patch] <;> try aesop

theorem diff_patch_correct_impl {t t' : Syn Data Meta} :
  Syn.patch t (Syn.diff t t') = .some t'
:= diff_patch_correct_spec (diff_spec rfl)
end diff_proof

section merge_proof
variable [DecidableEq Data] [DecidableEq Meta]

theorem merge_spec {δ1 δ2: Diff Data Meta} :
  Diff.merge δ1 δ2 = .some δ3 → MergeRel δ1 δ2 δ3
:= by sorry

omit [DecidableEq Data] [DecidableEq Meta] in
theorem merge_comm {δ δ' : Diff Data Meta} :
  MergeRel δ δ' δ'' → MergeRel δ' δ δ''
:= by intro h; induction h <;> aesop
end merge_proof
