#[macro_use] extern crate graphplan;
use graphplan::{Proposition, Action, GraphPlan, SimpleSolver};
use std::fmt::{self, format};
use strum::IntoEnumIterator; // 0.17.1
use strum_macros::EnumIter; // 0.17.1

/*

  (:types
    location locatable - object
		bot cup - locatable
    robot - bot
    kitchen_stuff
  )

  (:predicates
		(on ?obj - locatable ?loc - location)
		(holding ?arm - locatable ?cupcake - locatable)
    (arm-empty)
    (path ?location1 - location ?location2 - location)
    (is ?obj - locatable ?type - kitchen_stuff)
  )

 */

#[derive(Debug, EnumIter, Clone, Hash, PartialEq, Eq, Ord, PartialOrd)]
pub enum Location {
    Cupboard,
    Table,
    Plate,
    Fridge
}

#[derive(Debug, EnumIter, Clone, Hash, PartialEq, Eq, Ord, PartialOrd, )]
pub enum Object {
    Teacup,
    Coffeecup,
    Arm,
    Plate
}

impl fmt::Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Cupboard => write!(f, "cupboard"),
            Self::Table => write!(f, "table"),
            Self::Plate => write!(f, "plate"),
            Self::Fridge => write!(f, "fridge"),
            _ => write!(f, "unknown"),
        }
    }
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Teacup => write!(f, "teacup"),
            Self::Coffeecup => write!(f, "coffecup"),
            Self::Arm => write!(f, "arm"),
            _ => write!(f, "unknown"),
        }
    }
}

fn on_object_location(obj: Object, loc: Location) -> String {
    format!("on {} {}", obj, loc)
}

fn path_loc1_loc2(loc1: Location, loc2: Location) -> String {
    format!("path {} {}", loc1, loc2)
}


// Helper trait to iterate over enum variants
trait EnumValues {
    fn values() -> Vec<Self>
    where
        Self: std::marker::Sized;
}

// Implement EnumValues trait for Object enum
impl EnumValues for Object {
    fn values() -> Vec<Self> {
        vec![
            Object::Arm,
            Object::Coffeecup,
            Object::Teacup,
            Object::Plate
            // Add more Object variants here
        ]
    }
}

// Implement EnumValues trait for Location enum
impl EnumValues for Location {
    fn values() -> Vec<Self> {
        vec![
            Location::Table,
            Location::Cupboard,
            Location::Fridge,
            Location::Plate
            // Add more Location variants here
        ]
    }
}

fn prefix_object_location(prefix: String) -> Vec<String> {
    let mut result = Vec::new();

    for object_variant in Object::values() {
        for location_variant in Location::values() {
            let string = format!("{} {} {}", prefix, object_variant, location_variant);
            result.push(string);
        }
    }

    result
}

fn prefix_object_object(prefix: String) -> Vec<String> {
    let mut result = Vec::new();

    for object_variant in Object::values() {
        for location_variant in Object::values() {
            let string = format!("{} {} {}", prefix, object_variant, location_variant);
            result.push(string);
        }
    }

    result
}

fn prefix_location_location(prefix: String) -> Vec<String> {
    let mut result = Vec::new();

    for object_variant in Location::values() {
        for location_variant in Location::values() {
            let string = format!("{} {} {}", prefix, object_variant, location_variant);
            result.push(string);
        }
    }

    result
}


fn generate_propositions(statements: Vec<String>) -> Vec<Proposition<String>> {
    statements
        .into_iter()
        .map(|e|
            Proposition::from(e))
        .collect()
}



/*
  (:action pick-up
    :parameters
     (?arm - bot
      ?cupcake - locatable
      ?loc - location)
    :precondition
     (and
        (on ?arm ?loc)
        (on ?cupcake ?loc)
        (arm-empty)
      )
    :effect
     (and
        (not (on ?cupcake ?loc))
        (holding ?arm ?cupcake)
        (not (arm-empty))
     )
  )
 */
#[derive(Default)]
struct ActionPickup {
    parameters: Vec<String>,
    preconditions: Vec<Proposition<String>>,
    effects: Vec<Proposition<String>>
  }

impl ActionPickup {
    pub fn new(param1: String, param2: String, param3: String) -> Self {
        // generate preconditions from params
        let p1 = format!("on {} {}", param1, param3);
        let p2 = format!("on {} {}", param2, param3);
        let p3 = "arm-empty".to_string();
        let precond1 = Proposition::from(p1);
        let precond2 = Proposition::from(p2);
        let precond3 = Proposition::from(p3);


        let effect1 = precond1.negate();
        let e2 = format!("holding {} {}", param1, param2);
        let effect2 = Proposition::from(e2);
        let effect3 = precond3.negate();

        ActionPickup {
            parameters: vec![param1, param2, param3],
            preconditions: vec![precond1, precond2, precond3],
            effects: vec![effect1, effect2, effect3]
        }
    }
}


fn main() {
    /*
    	(:init
		(on arm table)
		(on coffeecup cupboard)
        (on teacup cupboard)

		(arm-empty)
		(path table cupboard)
        (path cupboard plate)
        (path table fridge)
        (path plate table)
        (path cupboard fridge)
        (path fridge cupboard)
        (path fridge plate)
	)
    */

    // let x = prefix_object_location("hello".to_string(),
    //             Object::Arm, Location);
    // dbg!(&x);

    // for object in Object::iter() {
    //     for location in Location::iter() {
    //         println!("{:?}", object);
    //     }
    // }

    let gen_pickup_object_location = prefix_object_location("pickup".to_string());
    dbg!(&gen_pickup_object_location);
    let props = generate_propositions(gen_pickup_object_location);
    dbg!(&props);

    let gen_on_object_location = prefix_object_location("on".to_string());
    let props2 = generate_propositions(gen_on_object_location);
    dbg!(&props2);

    // let on_arm_table = on_object_location(Object::Arm, Location::Table);
    // let on_coffeecup_cupboard = on_object_location(Object::Coffeecup, Location::Cupboard);
    // let on_teacup_cupboard = on_object_location(Object::Teacup, Location::Cupboard);
    // let path_table_cupboard = path_loc1_loc2(Location::Table, Location::Cupboard);
    // let path_cupboard_plate = path_loc1_loc2(Location::Cupboard, Location::Plate);
    // let path_table_fridge = path_loc1_loc2(Location::Table, Location::Fridge);
    // let path_plate_table = path_loc1_loc2(Location::Plate, Location::Table);
    // let path_cupboard_fridge = path_loc1_loc2(Location::Cupboard, Location::Fridge);
    // let path_fridge_cupboard = path_loc1_loc2(Location::Fridge, Location::Cupboard);
    // let path_fridge_plate = path_loc1_loc2(Location::Fridge, Location::Plate);
    // let prop_on_arm_table: Proposition<_> = Proposition::from(on_arm_table);
    // let prop_on_coffeecup_cupboard: Proposition<_> = Proposition::from(on_coffeecup_cupboard);
    // let prop_on_teacup_cupboard: Proposition<_> = Proposition::from(on_teacup_cupboard);
    // let prop_path_table_cupboard: Proposition<_> = Proposition::from(path_table_cupboard);
    // let prop_path_cupboard_plate: Proposition<_> = Proposition::from(path_cupboard_plate);
    // let prop_path_table_fridge: Proposition<_> = Proposition::from(path_table_fridge);
    // let prop_path_plate_table: Proposition<_> = Proposition::from(path_plate_table);
    // let prop_path_cupboard_fridge: Proposition<_> = Proposition::from(path_cupboard_fridge);
    // let prop_path_fridge_cupboard: Proposition<_> = Proposition::from(path_fridge_cupboard);
    // let prop_path_fridge_plate: Proposition<_> = Proposition::from(path_fridge_plate);

    let arm_empty = format!("arm-empty");
    let prop_arm_empty: Proposition<_> = Proposition::from(arm_empty);
    let prop_not_arm_empty: Proposition<_> = prop_arm_empty.negate();


    let p1 = Proposition::from("tired");
    let not_p1 = p1.negate();

    let p2 = Proposition::from("dog needs to pee");
    let not_p2 = p2.negate();

    let p3 = Proposition::from("at work");
    let p4 = p3.negate();


    /*
    (:action pick-up
        :parameters
        (?arm - bot
        ?cupcake - locatable
        ?loc - location)
        :precondition
        (and
            (on ?arm ?loc)
            (on ?cupcake ?loc)
            (arm-empty)
        )
        :effect
        (and
            (not (on ?cupcake ?loc))
            (holding ?arm ?cupcake)
            (not (arm-empty))
        )
    )
    */
    let action_pickup_arm = ActionPickup::new("arm".to_string(), "cupcake".to_string(), "table".to_string());

    // let pickup_arm_coffeecup_cupboard = format!("pickup arm {}, {}",
                                                                // Object::Coffeecup,
                                                                // Location::Cupboard);
    // let action_pickup_arm_coffeecup_cupboard = Action::new(
    //     pickup_arm_coffeecup_cupboard,
    //     fragset!{&prop_on_arm_table},
    //     fragset!{&prop_path_fridge_cupboard}
    // );

    let a1 = Action::new(
        "drink coffee",
        fragset!{[&p1]},
        fragset!{[&not_p1]}
    );

    let a2 = Action::new(
        "walk dog",
        fragset!{[&p2, &not_p1]},
        fragset!{[&not_p2]},
    );

    let a3 = Action::new(
        "go to work",
        fragset!{[&not_p1, &not_p2]},
        fragset!{[&p3]},
    );


/*
(:goal
    (and
		(on coffeecup plate)
        (on teacup plate)
    )
	)
 */

    let domain = GraphPlan::create_domain(
        fragset!{[&p1, &p2, &p4]},
        fragset!{[&not_p1, &not_p2, &p3]},
        fragset!{[&a1, &a2, &a3]}
    );
    let mut pg = GraphPlan::from_domain(&domain);

    println!("Plan:");

    for step in pg.search::<SimpleSolver>().unwrap() {
        for action in step {
            println!("- {:?}", action.id);
        }
    }
}
