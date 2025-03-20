// Kleene algebra trait with operations:
// - star
// - plus
// - times
// - one
// - zero

use std::collections::HashSet;

pub trait Kleene {
    fn zero() -> Self;
    fn one() -> Self;
    fn plus(self, other: Self) -> Self;
    fn times(self, other: Self) -> Self;
    fn star(self) -> Self;
}

impl Kleene for bool {
    fn zero() -> Self { false }
    fn one() -> Self { true }
    fn plus(self, other: Self) -> Self { self || other }
    fn times(self, other: Self) -> Self { self && other }
    fn star(self) -> Self { true }
}

#[derive(Debug, Clone, PartialEq)]
enum Regex<T> {
    Atom(T),
    Zero,
    One,
    Plus(Box<Regex<T>>, Box<Regex<T>>),
    Times(Box<Regex<T>>, Box<Regex<T>>),
    Star(Box<Regex<T>>),
}

impl<T:std::fmt::Display> std::fmt::Display for Regex<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Regex::Atom(c) => write!(f, "{}", c),
            Regex::Zero => write!(f, "0"),
            Regex::One => write!(f, "1"),
            Regex::Plus(a, b) => write!(f, "({} + {})", a, b),
            Regex::Times(a, b) => write!(f, "({} · {})", a, b),
            Regex::Star(a) => write!(f, "({})*", a),
        }
    }
}

impl<T> Kleene for Regex<T> {
    fn zero() -> Self { Regex::Zero }
    fn one() -> Self { Regex::One }
    fn plus(self, other: Self) -> Self {
        match (self, other) {
            (Regex::Zero, x) | (x, Regex::Zero) => x,
            (a, b) => Regex::Plus(Box::new(a), Box::new(b))
        }
    }
    fn times(self, other: Self) -> Self {
        match (self, other) {
            (Regex::Zero, _) | (_, Regex::Zero) => Regex::Zero,
            (Regex::One, x) | (x, Regex::One) => x,
            (a, b) => Regex::Times(Box::new(a), Box::new(b))
        }
    }
    fn star(self) -> Self {
        match self {
            Regex::Zero | Regex::One => Regex::One,
            Regex::Star(x) => Regex::Star(x),
            x => Regex::Star(Box::new(x))
        }
    }
}

// Kleene's algorithm for converting a NFA to a Kleene algebra
// Takes a start state and computes the Kleene element for going from the start state to any other state
fn nfa_to_kleene<S:Clone+Eq+std::hash::Hash,K:Kleene+Clone>(nfa: &[(S,K,S)], start: S) -> K {
    // The algorithm works by eliminating all states except the start state
    // The final answer is then the self-loop of the start state

    let mut nfa: Vec<(&S, K, &S)> = nfa.iter().map(|(from, k, to)| (from, k.clone(), to)).collect();
    
    let mut states_todo = nfa.iter().flat_map(|(from, _, to)| vec![*from, *to]).collect::<HashSet<_>>();
    states_todo.remove(&start);

    while !states_todo.is_empty() {
        let state = *states_todo.iter().next().unwrap();
        states_todo.remove(&state);
        let mut new_nfa : Vec<(&S, K, &S)> = vec![];
        let mut incoming : Vec<(&S, K, &S)> = vec![];
        let mut outgoing : Vec<(&S, K, &S)> = vec![];
        let mut self_loops : Vec<(&S, K, &S)> = vec![];

        for (from, k, to) in nfa.iter() {
            let edge: (&S, K, &S) = (from, k.clone(), to);
            if from == &state && to == &state {
                self_loops.push(edge);
            } else if from == &state {
                outgoing.push(edge);
            } else if to == &state {
                incoming.push(edge);
            } else {
                new_nfa.push(edge);
            }
        }
        // Add up the self loops
        let self_loop = self_loops.iter().map(|(_, k, _)| k).fold(K::zero(), |acc, k| acc.plus(k.clone())).star();
        // Insert all the shortcut edges into the new NFA
        for (from, k1, _) in incoming.iter() {
            for (_, k2, to) in outgoing.iter() {
                new_nfa.push((*from, k1.clone().times(self_loop.clone().times(k2.clone())), *to));
            }
        }
        nfa = new_nfa;
    }
    let mut answer = K::zero();
    for (from, k, to) in nfa.iter() {
        assert!(**from == start);
        assert!(**to == start);
        answer = answer.plus(k.clone());
    }
    answer.star()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nfa_to_kleene() {
        // Create a simple NFA with 3 states
        let nfa = vec![
            (0, Regex::Atom('a'), 1),  // State 0 to 1 with label 'a'
            (1, Regex::Atom('b'), 2),  // State 1 to 2 with label 'b' 
            (2, Regex::Atom('c'), 0),  // State 2 back to 0 with label 'c'
            (1, Regex::Atom('d'), 1),  // Self loop on state 1 with label 'd'
        ];

        let result = nfa_to_kleene(&nfa, 0);
        
        // assert that it's equal to ((a · ((d)* · (b · c))))* as a string
        assert_eq!(result.to_string(), "((a · ((d)* · (b · c))))*");
    }
}
