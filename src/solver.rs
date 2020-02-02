use std::fmt::{Debug, Display};
use std::hash::Hash;
use std::iter::FromIterator;
use std::collections::{HashMap, HashSet, BTreeSet, VecDeque};
use log::{debug};
use itertools::Itertools;
use crate::proposition::Proposition;
use crate::action::Action;
use crate::pairset::{pairs_from_sets};
use crate::layer::{MutexPairs};
use crate::plangraph::{PlanGraph, Solution};


pub trait GraphPlanSolver<ActionId: Hash + Ord + Clone + Debug,
                          PropositionId: Hash + Ord + Clone + Debug + Display> {
    /// Searches a plangraph for a sequence of collection of actions
    /// that satisfy the goals
    fn search(&self, plangraph: &PlanGraph<ActionId, PropositionId>) -> Option<Solution<ActionId, PropositionId>>;
}

#[derive(Default)]
pub struct SimpleSolver;

type GoalIndex = usize;
type Attempts<ActionId, PropositionId> = HashMap<usize, BTreeSet<Action<ActionId, PropositionId>>>;

#[derive(Clone, Debug, PartialEq)]
struct ActionCombination<ActionId: Hash + PartialEq + Clone + Debug ,
                         PropositionId: Hash + Eq + PartialEq + Clone + Debug + Display>(
    HashMap<GoalIndex, Action<ActionId, PropositionId>>
);

impl<ActionId: Clone + Hash + Eq + Debug,
     PropositionId: Clone + Hash + Eq + Debug + Display>
    ActionCombination<ActionId, PropositionId> {
    pub fn as_set(&self) -> HashSet<Action<ActionId, PropositionId>> {
        self.0.values()
            .cloned()
            .collect()
    }

    #[allow(dead_code)]
    pub fn as_vec(&self) -> Vec<Action<ActionId, PropositionId>> {
        self.0.values()
            .cloned()
            .collect()
    }
}

#[derive(Clone, Debug)]
struct GoalSetActionGenerator<ActionId: Debug + Hash + Clone + Eq + Ord,
                              PropositionId: Debug + Display + Hash + Clone + Eq + Ord> {
    goals: Vec<Proposition<PropositionId>>,
    actions: BTreeSet<Action<ActionId, PropositionId>>,
    mutexes: Option<MutexPairs<Action<ActionId, PropositionId>>>,
}

impl<ActionId: Ord + Clone + Hash + Debug,
     PropositionId: Ord + Clone + Hash + Debug + Display>
    GoalSetActionGenerator<ActionId, PropositionId> {
    pub fn new(goals: Vec<Proposition<PropositionId>>,
               actions: BTreeSet<Action<ActionId, PropositionId>>,
               mutexes: Option<MutexPairs<Action<ActionId, PropositionId>>>,)
               -> GoalSetActionGenerator<ActionId, PropositionId> {
        GoalSetActionGenerator {goals, actions, mutexes}
    }
}

impl<ActionId: Ord + Clone + Hash + Debug,
     PropositionId: Ord + Clone + Hash + Debug + Display>
    IntoIterator for GoalSetActionGenerator<ActionId, PropositionId> {
    type Item = ActionCombination<ActionId, PropositionId>;
    type IntoIter = ActionCombinationIterator<ActionId, PropositionId>;

    fn into_iter(self) -> Self::IntoIter {
        ActionCombinationIterator::new(self)
    }
}

#[derive(Clone, Debug)]
struct ActionCombinationIterator<ActionId: Ord + Clone + Hash + Debug,
                                 PropositionId: Ord + Clone + Hash + Debug + Display> {
    meta: GoalSetActionGenerator<ActionId, PropositionId>, // defines goals we are trying achieve
    attempts: Attempts<ActionId, PropositionId>, // previous attempts to meet a goal
    goals_met: bool, // flag indicating all goals are met or restart
    accum: HashMap<GoalIndex, Action<ActionId, PropositionId>> // combination of actions
}

impl<ActionId: Ord + Debug + Hash + Clone,
     PropositionId: Ord + Debug + Display + Hash + Clone>
    ActionCombinationIterator<ActionId, PropositionId> {
    pub fn new(action_combinations: GoalSetActionGenerator<ActionId, PropositionId>) -> ActionCombinationIterator<ActionId, PropositionId> {
        ActionCombinationIterator {
            meta: action_combinations,
            attempts: Attempts::new(),
            goals_met: false,
            accum: HashMap::new(),
        }
    }
}

impl<ActionId: Ord + Clone + Hash + Debug + PartialEq,
     PropositionId: Ord + Clone + Hash + Debug + Display + PartialEq>
    Iterator for ActionCombinationIterator<ActionId, PropositionId> {
    type Item = ActionCombination<ActionId, PropositionId>;

    fn next(&mut self) -> Option<Self::Item> {
        let goals = &self.meta.goals;
        let actions = &self.meta.actions;
        let goal_len = goals.len();

        let mut stack = VecDeque::new();

        // If the goals have already been met, we need to look for a
        // new combination that also meets the goals
        if self.goals_met {
            // Remove the previous action used to satisfy the last
            // goal and start the loop from the last goal. This will
            // yield a new combination or recursively back track.
            self.goals_met = false;
            let goal_idx = goal_len - 1;
            self.accum.remove(&goal_idx);
            stack.push_front(goal_idx);
        } else {
            stack.push_front(0);
        }

        while let Some(goal_idx) = stack.pop_front() {
            let available_actions = if let Some(acts) = self.attempts.get(&goal_idx) {
                acts.to_owned()
            } else {
                let goal = &goals[goal_idx];
                debug!("Working on goal {:?}", goal);
                let mut available = BTreeSet::new();
                // Only actions that produce the goal and are not
                // mutex with any other actions and have not
                // already been attempted in combination with the
                // other attempted actions and are not mutex with
                // any other action
                for a in actions {
                    // Early continue since the later checks are
                    // more expensive
                    if !a.effects.contains(goal) {
                        continue
                    };

                    // Early break if we already know this action
                    // works for the goal from previous attempts
                    if self.accum.get(&goal_idx).map(|i| i == a).unwrap_or(false) {
                        available.insert(a.clone());
                        break
                    };

                    // Check if this action is mutex with any of
                    // the other accumulated actions
                    let acts = self.accum.clone();
                    let pairs = pairs_from_sets(
                        hashset!{a.clone()},
                        ActionCombination(acts).as_set()
                    );
                    debug!("Checking pairs: {:?} against mutexes: {:?}", pairs, &self.meta.mutexes);

                    if let Some(muxes) = &self.meta.mutexes {
                        if muxes.intersection(&pairs).next().is_none() {
                            available.insert(a.clone());
                        }
                    };
                };
                available
            };

            if available_actions.is_empty() {
                if goal_idx == 0 {
                    // Complete fail
                    break;
                }
                debug!("Unable to find actions for goal {:?}. Going back to previous goal...", goal_idx);
                // Clear attempts for this goal so finding an action
                // can be retried with a new set of actions
                self.attempts.remove(&goal_idx);
                // Backtrack to the previous goal
                stack.push_front(goal_idx - 1);
            } else {
                let next_action = available_actions.iter().next().unwrap();
                // TODO only add the action if the goal isn't met yet
                self.accum.insert(goal_idx, next_action.clone());

                // Add to previous attempts in case we need to backtrack
                let mut remaining_actions = available_actions.clone();
                remaining_actions.remove(&next_action);
                self.attempts.insert(goal_idx, remaining_actions);

                // TODO Implement minimal action sets i.e handle if
                // this action staisfies more than one
                // goal. Otherwise, the speed of finding a solution is
                // dependent on the ordering of goals
                // From the paper: no action can be
                // removed from A so that the add effects of the
                // actions remaining still contain G

                // Proceed to the next goal
                if goal_idx < goal_len - 1 {
                    stack.push_front(goal_idx + 1);
                } else {
                    self.goals_met = true;
                }
            };
        };

        if self.goals_met {
            Some(ActionCombination(self.accum.clone()))
        } else {
            None
        }
    }
}


#[cfg(test)]
mod goal_set_action_generator_test {
    use super::*;

    #[test]
    fn single_goal() {
        let p1 = Proposition::from("coffee");
        let p2 = Proposition::from("caffeinated");
        let goals = vec![p2.clone()];
        let mut actions = BTreeSet::new();
        let a1 = Action::new(
            String::from("drink coffee"),
            hashset!{&p1},
            hashset!{&p2}
        );
        actions.insert(a1.clone());
        let mutexes = Some(MutexPairs::new());
        let expected = ActionCombination(hashmap!{0 => a1});
        let actual = GoalSetActionGenerator::new(goals, actions, mutexes)
            .into_iter()
            .next()
            .unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn multiple_goals() {
        let p1 = Proposition::from("coffee");
        let p2 = Proposition::from("caffeinated");
        let p3 = Proposition::from("breakfast");
        let p4 = Proposition::from("full");
        let goals = vec![p2.clone(), p4.clone()];
        let mut actions = BTreeSet::new();
        let a1 = Action::new(
            String::from("drink coffee"),
            hashset!{&p1},
            hashset!{&p2}
        );
        actions.insert(a1.clone());

        let a2 = Action::new(
            String::from("eat breakfast"),
            hashset!{&p3},
            hashset!{&p4}
        );
        actions.insert(a2.clone());

        let mutexes = Some(MutexPairs::new());
        let expected = ActionCombination(hashmap!{0 => a1.clone(), 1 => a2.clone()});
        let actual = GoalSetActionGenerator::new(goals, actions, mutexes)
            .into_iter()
            .next()
            .unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn yields_all() {
        let p1 = Proposition::from("tea");
        let p2 = Proposition::from("coffee");
        let p3 = Proposition::from("caffeinated");
        let p4 = Proposition::from("scone");
        let p5 = Proposition::from("muffin");
        let p6 = Proposition::from("full");
        let goals = vec![p3.clone(), p6.clone()];
        let mut actions = BTreeSet::new();

        let a1 = Action::new(
            String::from("drink coffee"),
            hashset!{&p2},
            hashset!{&p3}
        );
        actions.insert(a1.clone());

        let a2 = Action::new(
            String::from("drink tea"),
            hashset!{&p1},
            hashset!{&p3}
        );
        actions.insert(a2.clone());

        let a3 = Action::new(
            String::from("eat scone"),
            hashset!{&p4},
            hashset!{&p6}
        );
        actions.insert(a3.clone());

        let a4 = Action::new(
            String::from("eat muffin"),
            hashset!{&p5},
            hashset!{&p6}
        );
        actions.insert(a4.clone());

        let mutexes = Some(MutexPairs::new());
        let expected = vec![
            vec![a1.clone(), a4.clone()],
            vec![a1.clone(), a3.clone()],
            vec![a2.clone(), a4.clone()],
            vec![a2.clone(), a3.clone()],
        ];
        let generator = GoalSetActionGenerator::new(goals, actions, mutexes);
        let actual: Vec<Vec<Action<_, _>>> = generator.into_iter()
            .map(|combo| {let mut i = combo.as_vec(); i.sort(); i})
            .collect();

        assert_eq!(expected, actual);
    }
}

type SearchStack<ActionId, PropositionId> = VecDeque<(usize, Vec<Proposition<PropositionId>>, Option<ActionCombinationIterator<ActionId, PropositionId>>)>;

impl<ActionId: Ord + Clone + Hash + Debug,
     PropositionId: Ord + Clone + Hash + Debug + Display>
    GraphPlanSolver<ActionId, PropositionId> for SimpleSolver {
    fn search(&self, plangraph: &PlanGraph<ActionId, PropositionId>) -> Option<Solution<ActionId, PropositionId>> {
        let mut success = false;
        let mut plan = Vec::new();
        let mut failed_goals_memo: HashSet<(usize, Vec<Proposition<PropositionId>>)> = HashSet::new();

        // Initialize the loop
        let mut stack: SearchStack<ActionId, PropositionId> = VecDeque::new();
        let init_goals = Vec::from_iter(plangraph.goals.clone());
        let init_layer_idx = plangraph.layers.len() - 1;
        let init_action_gen = None;

        stack.push_front((init_layer_idx, init_goals, init_action_gen));

        while let Some((idx, goals, action_gen)) = stack.pop_front() {
            debug!("Working on layer {:?} with goals {:?}", idx, goals);
            // Check if the goal set is unsolvable at level idx
            if failed_goals_memo.contains(&(idx, goals.clone())) {
                // Continue to previous layer (the next element in the queue)
                continue;
            }

            // Note: This is a btreeset so ordering is guaranteed
            let actions = plangraph.actions_at_layer(idx - 1)
                .expect("Failed to get actions");
            let mutexes = plangraph.mutex_actions.get(&(idx - 1)).cloned();
            let mut gen = action_gen
                .or_else(|| Some(GoalSetActionGenerator::new(goals.clone(),
                                                             actions.clone(),
                                                             mutexes).into_iter()))
                .unwrap();

            if let Some(goal_actions) = gen.next() {
                debug!("Actions: {:?} for goals: {:?}", goal_actions.as_set(), goals);
                // If we are are on the second to last proposition
                // layer, we are done
                if (idx - 2) == 0 {
                    plan.push(goal_actions.as_set());
                    debug!("Found plan! {:?}", plan);
                    success = true;
                    break;
                } else {
                    plan.push(goal_actions.as_set());
                    let next_goals = goal_actions.as_set().into_iter()
                        .flat_map(|action| action.reqs)
                        .unique()
                        .collect();
                    // Add this layer back into the queue incase we need to backtrack
                    stack.push_front((idx, goals, Some(gen)));
                    stack.push_front((idx - 2, next_goals, None));
                };
            } else {
                debug!("Unable to find actions for goals {:?} from actions {:?}",
                       goals, actions);
                // Record the failed goals at level idx
                failed_goals_memo.insert((idx, goals.clone()));
                // Remove the last step in the plan from which this
                // set of goals comes from
                plan.pop();
                // Backtrack to previous layer and goalset or nothing
                // (the next element in the queue)
            }
        };

        if success {
            // Since this solver goes from the last layer to the
            // first, we need to reverse the plan
            plan.reverse();
            Some(plan)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod simple_solver_test {
    use super::*;

    #[test]
    fn solver_works() {
        let p1 = Proposition::from("tired");
        let not_p1 = p1.negate();
        let p2 = Proposition::from("dog needs to pee");
        let not_p2 = p2.negate();

        let a1 = Action::new(
            String::from("coffee"),
            hashset!{&p1},
            hashset!{&not_p1}
        );

        let a2 = Action::new(
            String::from("walk dog"),
            hashset!{&p2, &not_p1},
            hashset!{&not_p2},
        );

        let goals = hashset!{not_p1, not_p2};

        let mut pg = PlanGraph::new(
            hashset!{p1, p2},
            goals,
            hashset!{a1.clone(), a2.clone()}
        );
        pg.extend();
        pg.extend();
        debug!("Plangraph: {:?}", pg);

        let solver = SimpleSolver::default();
        let expected = vec![hashset!{a1.clone()}, hashset!{a2.clone()}];
        let plan = solver.search(&pg);
        debug!("Plan: {:?}", plan);
        let actual = PlanGraph::format_plan(plan.unwrap());
        assert_eq!(expected, actual);
    }
}

#[cfg(test)]
/// Prove that BTreeSet preserves ordering
mod btreeset_test {
    use std::collections::BTreeSet;

    #[test]
    fn is_sorted () {
        let mut b = BTreeSet::new();
        b.insert(2);
        b.insert(3);
        b.insert(1);
        assert_eq!(b.into_iter().collect::<Vec<_>>(), vec![1, 2, 3]);

        let mut b = BTreeSet::new();
        b.insert(1);
        b.insert(2);
        b.insert(3);
        assert_eq!(b.into_iter().collect::<Vec<_>>(), vec![1, 2, 3]);
    }
}
