Require Import Arith.
Require Import Lia.

Section Utils.
  Axiom dependent_funext : forall {A} {B : A -> Type} (f g : forall x, B x),
    (forall x, f x = g x) -> f = g.

  Axiom propext : forall {P Q : Prop}, P <-> Q -> P = Q.

  Axiom UIP : forall {P : Prop} (x y : P), x = y.

  Lemma funext {A B} (f g : A -> B) : (forall x, f x = g x) -> f = g.
  Proof. apply dependent_funext. Qed.

  Lemma pred_ext {A} {P Q : A -> Prop} : (forall x, P x <-> Q x) -> P = Q.
  Proof.
    eauto using funext, propext.
  Qed.

End Utils.
Ltac funext := (apply funext; intro).
Ltac pred_ext := (apply pred_ext; intro).

Inductive regex {M : Type} : Type :=
| r_zero
| r_one
| r_singleton (x : M)
| r_concat (r1 r2 : regex)
| r_union (r1 r2 : regex)
| r_star (r : regex).
Arguments regex M : clear implicits.

Module Type MONOID.
  Parameter M : Set.
  Parameter unit : M.
  Parameter concat : M -> M -> M.

  Axiom assoc : forall a b c, concat (concat a b) c = concat a (concat b c).
  Axiom unit_l : forall a, concat unit a = a.
  Axiom unit_r : forall a, concat a unit = a.
End MONOID.

Module NatMonoid : MONOID.
  Definition M := nat.
  Definition unit := 0.
  Definition concat x y := x + y.

  Proposition assoc : forall a b c, (a + b) + c = a + (b + c).
  Proof. lia. Qed.
  Proposition unit_l : forall a, 0 + a = a. Proof. lia. Qed.
  Proposition unit_r : forall a, a + 0 = a. Proof. lia. Qed.
End NatMonoid.

Module RationalSets (M : MONOID).
  Include M.

  Definition set_empty : M -> Prop := fun m => False.
  Definition set_unit : M -> Prop := fun m => m = unit.

  Definition set_concat (s1 s2 : M -> Prop) : M -> Prop :=
    fun m => exists m1 m2, m = concat m1 m2 /\ s1 m1 /\ s2 m2.

  Definition set_union (s1 s2 : M -> Prop) : M -> Prop :=
    fun m => s1 m \/ s2 m.

  Fixpoint set_exponent (s : M -> Prop) (k : nat) : M -> Prop :=
    match k with
    | O => set_unit
    | S k' => set_concat s (set_exponent s k')
    end.

  Definition set_star (s : M -> Prop) : M -> Prop :=
    fun m => exists k, set_exponent s k m.

  Fixpoint denot (r : regex M) : M -> Prop :=
    match r with
    | r_zero => set_empty
    | r_one => set_unit
    | r_singleton x => fun m => m = x
    | r_concat r1 r2 => set_concat (denot r1) (denot r2)
    | r_union r1 r2 => set_union (denot r1) (denot r2)
    | r_star r => set_star (denot r)
    end.

  (* Lemmas to prove: *)
  (* set_unit, set_concat is a monoid *)
  (* if the monoid is commutative, so is set_concat *)
  (* set_union, set_empty is a commutative monoid *)
  (* set_empty annihilates concat *)
  (* union is idempotent *)
  (* both unfolding rules for star *)

  Lemma set_concat_assoc s1 s2 s3 :
    set_concat (set_concat s1 s2) s3 = set_concat s1 (set_concat s2 s3).
  Proof.
    unfold set_concat.
    pred_ext; split.
    - intros [?[?[->[[?[?[->[]]]]]]]].
      eexists _, _; split; [|split; eauto].
      apply assoc.
    - intros [?[?[->[?[?[?[->[]]]]]]]].
      eexists _, _; split; [rewrite assoc|]; eauto 10.
  Qed.

  Lemma set_concat_comm :
    (forall x y, concat x y = concat y x) ->
    forall s1 s2, set_concat s1 s2 = set_concat s2 s1.
  Proof.
    intros.
    unfold set_concat.
    pred_ext.
    split; intros [?[?[->[]]]]; eauto.
  Qed.

  Lemma set_unit_l s : set_concat set_unit s = s.
  Proof.
    unfold set_unit, set_concat.
    pred_ext; split.
    - intros [?[?[?[]]]]. subst. rewrite unit_l. eauto.
    - eauto using unit_l.
  Qed.

  Lemma set_unit_r s : set_concat s set_unit = s.
  Proof.
    unfold set_unit, set_concat.
    pred_ext; split.
    - intros [?[?[?[]]]]. subst. rewrite unit_r. eauto.
    - eauto using unit_r.
  Qed.

  Lemma set_exponent_one s : set_exponent s 1 = s.
  Proof. apply set_unit_r. Qed.

  Lemma set_exponent_unfold_l s k :
    set_exponent s (S k) = set_concat s (set_exponent s k).
  Proof. reflexivity. Qed.

  Lemma set_exponent_unfold_r s k :
    set_exponent s (S k) = set_concat (set_exponent s k) s.
  Proof.
    induction k; simpl in *.
    - rewrite set_unit_l, set_unit_r. reflexivity.
    - rewrite set_concat_assoc, IHk. reflexivity.
  Qed.

  Lemma set_union_assoc s1 s2 s3 : set_union (set_union s1 s2) s3 = set_union s1 (set_union s2 s3).
  Proof. pred_ext. unfold set_union. tauto. Qed.

  Lemma set_union_comm s1 s2 : set_union s1 s2 = set_union s2 s1.
  Proof. pred_ext. unfold set_union. tauto. Qed.

  Lemma set_union_idemp s : set_union s s = s.
  Proof. pred_ext. unfold set_union. tauto. Qed.

  Lemma set_empty_l s : set_union set_empty s = s.
  Proof. pred_ext. unfold set_empty, set_union. tauto. Qed.

  Lemma set_empty_r s : set_union s set_empty = s.
  Proof. pred_ext. unfold set_empty, set_union. tauto. Qed.

  Lemma set_annihil_l s : set_concat set_empty s = set_empty.
  Proof.
    pred_ext.
    unfold set_empty, set_concat.
    split; [intros [?[?[?[[]]]]] | intros []].
  Qed.

  Lemma set_annihil_r s : set_concat s set_empty = set_empty.
  Proof.
    pred_ext.
    unfold set_empty, set_concat.
    split; [intros [?[?[?[?[]]]]] | intros []].
  Qed.

  Lemma set_star_unfold_l s : set_star s = set_union set_unit (set_concat s (set_star s)).
  Proof.
    unfold set_star, set_union, set_unit, set_concat.
    pred_ext; split.
    - intros [[|k] ?]; eauto.
      destruct H as (? & ? & ? & ? & ?).
      eauto 10.
    - intros [|[?[?[?[?[]]]]]].
      + exists 0. eauto.
      + eexists (S _). simpl. unfold set_concat. eauto.
  Qed.

  Lemma set_star_unfold_r s : set_star s = set_union set_unit (set_concat (set_star s) s).
  Proof.
    unfold set_star, set_union, set_unit, set_concat.
    pred_ext; split.
    - intros [[|k] ?]; eauto.
      rewrite set_exponent_unfold_r in H.
      destruct H as (? & ? & ? & ? & ?).
      eauto 10.
    - intros [|[?[?[?[[]]]]]].
      + exists 0. eauto.
      + eexists (S _). rewrite set_exponent_unfold_r. unfold set_concat. eauto.
  Qed.

End RationalSets.
