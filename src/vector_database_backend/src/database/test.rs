
#[cfg(test)]
mod tests {
    // use crate::database::{index::{generate_index, Vector}, db::Database};
    // use crate::config::EMBEDDING_LENGTH;
    // use instant_distance::Search;

    // #[test]
    // fn test_index_generate() {
    //     let points = vec![Vector::random(), Vector::random(), Vector::random()];
    //     let values: Vec<String> = vec!["p1".to_string(), "p2".to_string(), "p3".to_string()];

    //     let map = generate_index(points, values);
    //     let mut search = Search::default();

    //     let query = Vector::random();

    //     let closet_point = map.search(&query, &mut search).next().unwrap();

    //     println!("{:?}", closet_point.value);
    // }

    // #[test]
    // fn test_store_and_query() {
    //     let a = Vector::random();
    //     let keys = vec![a.clone(), Vector::random(), Vector::random()];
    //     let b = String::from("p1");
    //     let values: Vec<String> = vec![b.clone(), "p2".to_string(), "p3".to_string()];

    //     let db = Database::new(keys, values);
    //     let mut search = Search::default();

    //     let res = db.query(&a, &mut search, 2);
    //     assert_eq!(res.len(), 2);
    //     assert_eq!(b, res[0]);
    // }

    // #[test]
    // #[should_panic]
    // fn test_from_vec() {
    //     let q: Vec<f32> = vec![1.0, 2.0, 3.0];
    //     let _ = Vector::from(q);
    // }

    // #[test]
    // fn test_from_vec2() {
    //     let q: Vec<f32> = rand::thread_rng().sample_iter(Standard).take(EMBEDDING_LENGTH).collect();
    //     assert_eq!(q.len(), EMBEDDING_LENGTH);
    //     let _ = Vector::from(q);
    // }

}
