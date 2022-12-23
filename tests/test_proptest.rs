// use proptest::prelude::*;

// use xot::{Node, Text, Value, Xot};

// fn arb_text_node<'a>(xot: &'a mut Xot<'a>) -> impl Strategy<Value = Node> + 'a {
//     ".*".prop_map(|s| xot.new_text(&s))
// }
// fn arb_xml(xot: &mut Xot) -> impl Strategy<Value = Node> {
//     // let leaf = prop_oneof![
//     //     any::<bool>().prop_map(Json::Bool),
//     //     any::<f64>().prop_map(Json::Number),
//     //     ".*".prop_map(Json::String),
//     // ];
//     // leaf.prop_recursive(
//     //     8,   // 8 levels deep
//     //     256, // Shoot for maximum size of 256 nodes
//     //     10,  // We put up to 10 items per collection
//     //     |inner| {
//     //         prop_oneof![
//     //             // Take the inner strategy and make the two recursive cases.
//     //             prop::collection::vec(inner.clone(), 0..10).prop_map(Json::Array),
//     //             prop::collection::hash_map(".*", inner, 0..10).prop_map(Json::Map),
//     //         ]
//     //     },
//     // )
// }
