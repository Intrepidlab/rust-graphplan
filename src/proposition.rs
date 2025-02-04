use std::fmt;
use std::fmt::{Debug, Display};
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;



#[derive(Eq, Ord, PartialOrd, Clone)]
pub struct Proposition<PropositionId> where PropositionId: Hash {
    pub id: PropositionId,
    pub negation: bool,
}

impl<PropositionId: Display + Hash> Debug for Proposition<PropositionId> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}P:{}", if self.negation {"¬"} else {""}, self.id)
    }
}

impl<PropositionId: Hash> Hash for Proposition<PropositionId> {
    fn hash<H>(&self, state: &mut H) where H: Hasher {
        self.negation.hash(state);
        self.id.hash(state);
    }
}

impl<PropositionId: Hash> PartialEq for Proposition<PropositionId> {
    fn eq(&self, other: &Self) -> bool {
        let mut hasher_left = DefaultHasher::new();
        let mut hasher_right = DefaultHasher::new();
        self.hash(&mut hasher_left);
        other.hash(&mut hasher_right);
        hasher_left.finish() == hasher_right.finish()
    }
}

impl<PropositionId: Clone + PartialEq + Hash> Proposition<PropositionId> {
    pub fn new(id: PropositionId, negation: bool) -> Self {
        Proposition {id, negation}
    }

    pub fn negate(&self) -> Self {
        Proposition { id: self.id.clone(), negation: !self.negation }
    }

    pub fn is_negation(&self, prop: &Self) -> bool {
        prop.id == self.id && prop.negation == !self.negation
    }
}

impl From<&'static str> for Proposition<&'static str> {
    fn from(s: &'static str) -> Self {
        Proposition {id: s, negation: false}
    }
}


impl From<String> for Proposition<String> {
    fn from(s: String) -> Self {
        Proposition {id: s, negation: false}
    }
}


#[cfg(test)]
mod proposition_test {
    use super::*;

    #[derive(PartialEq, Clone, Hash, Eq)]
    enum Props {
        A,
        B,
    }

    impl From<Props> for Proposition<Props> {
        fn from(prop: Props) -> Self {
            Self::new(prop, false)
        }
    }

    #[test]
    fn propositions_can_be_negated() {
        // Sanity check
        assert_eq!(Proposition::from("test"), Proposition::from("test"));
        let p1 = Proposition::from("test");

        assert!(false == p1.negation);
        assert!(true == Proposition::from("test").negate().negation);

        let p2 = Proposition::from("test").negate();

        assert!(
            p2.is_negation(&p1),
            "{:?} is not a negation of {:?}", p1, p2
        );

        assert!(p1.is_negation(&p2));
    }

    #[test]
    fn proposition_hashing_works() {
        let set = fragset!{[Proposition::from("caffeinated")]};
        assert!(set.contains(&Proposition::from("caffeinated")));

        let set = fragset!{[Proposition::from("caffeinated").negate()]};
        assert!(set.contains(&Proposition::from("caffeinated").negate()));

        let set = fragset!{[Proposition::from("caffeinated").negate()]};
        assert!(!set.contains(&Proposition::from("caffeinated")));
    }

    #[test]
    fn proposition_ids_are_extensible() {
        let p1 = Proposition::from(Props::A);
        let p2 = Proposition::from(Props::B);
        let set = fragset!{[p1.clone()]};
        assert!(set.contains(&p1));
        assert!(!set.contains(&p2));
    }

}
