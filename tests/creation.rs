use serde_json::{json, to_string_pretty};

use cozo::{Db, EntityId, Validity};
use cozorocks::DbBuilder;

fn create_db(name: &str) -> Db {
    let builder = DbBuilder::default()
        .path(name)
        .create_if_missing(true)
        .destroy_on_exit(true);
    Db::build(builder).unwrap()
}

fn test_send_sync<T: Send + Sync>(_: &T) {}

#[test]
fn creation() {
    let db = create_db("_test_db");
    test_send_sync(&db);
    assert!(db.current_schema().unwrap().as_array().unwrap().is_empty());
    let res = db.transact_attributes(&json!({
        "attrs": [
            {"put": {"keyword": "person/idd", "cardinality": "one", "type": "string", "index": "identity", "history": false}},
            {"put": {"keyword": "person/first_name", "cardinality": "one", "type": "string", "index": true}},
            {"put": {"keyword": "person/last_name", "cardinality": "one", "type": "string", "index": true}},
            {"put": {"keyword": "person/age", "cardinality": "one", "type": "int"}},
            {"put": {"keyword": "person/friend", "cardinality": "many", "type": "ref"}},
            {"put": {"keyword": "person/weight", "cardinality": "one", "type": "float"}},
            {"put": {"keyword": "person/covid", "cardinality": "one", "type": "bool"}},
        ]
    }))
    .unwrap();
    println!("{}", res);
    let first_id = res["results"][0][0].as_u64().unwrap();
    let last_id = res["results"][6][0].as_u64().unwrap();
    db.transact_attributes(&json!({
        "attrs": [
            {"put": {"id": first_id, "keyword": ":person/id", "cardinality": "one", "type": "string", "index": "identity", "history": false}},
            {"retract": {"id": last_id, "keyword": ":person/covid", "cardinality": "one", "type": "bool"}}
        ]
    })).unwrap();
    assert_eq!(db.current_schema().unwrap().as_array().unwrap().len(), 6);
    println!(
        "{}",
        to_string_pretty(&db.current_schema().unwrap()).unwrap()
    );
    db.transact_triples(&json!({
        "tx": [
            {"put": {
                "_temp_id": "alice",
                "person/first_name": "Alice",
                "person/age": 7,
                "person/last_name": "Amorist",
                "person/id": "alice_amorist",
                "person/weight": 25,
                "person/friend": "eve"}},
            {"put": {
                "_temp_id": "bob",
                "person/first_name": "Bob",
                "person/age": 70,
                "person/last_name": "Wonderland",
                "person/id": "bob_wonderland",
                "person/weight": 100,
                "person/friend": "alice"
            }},
            {"put": {
                "_temp_id": "eve",
                "person/first_name": "Eve",
                "person/age": 18,
                "person/last_name": "Faking",
                "person/id": "eve_faking",
                "person/weight": 50,
                "person/friend": [
                    "alice",
                    "bob",
                    {
                        "person/first_name": "Charlie",
                        "person/age": 22,
                        "person/last_name": "Goodman",
                        "person/id": "charlie_goodman",
                        "person/weight": 120,
                        "person/friend": "eve"
                    }
                ]
            }},
        ]
    }))
    .unwrap();

    println!(
        "{}",
        to_string_pretty(&db.entities_at(None).unwrap()).unwrap()
    );

    let pulled = db
        .pull(
            EntityId::MIN_PERM,
            &json!([
                "_id",
                "person/first_name",
                "person/last_name",
                {"pull":"person/friend", "as": "friends", "recurse": true},
            ]),
            Validity::current(),
        )
        .unwrap();

    println!("{}", to_string_pretty(&pulled).unwrap());

    let query = json!([
        ["_id", "person/first_name", "Eve"],
        ["_id", "person/friend", "?friend"],
        ["?friend", "person/first_name", "?friend_name"]
    ]);
    let mut tx = db.transact().unwrap();
    let vld = Validity::current();
    let query = tx.parse_clauses(&query, vld).unwrap();
    dbg!(&query);
    let compiled = tx.compile_clauses(query, vld).unwrap();
    dbg!(&compiled);
    for x in compiled.iter(&tx) {
        dbg!(x.unwrap());
    }

    // iteration
    // let mut it = db.total_iter();
    // while let Some((k_slice, v_slice)) = it.pair().unwrap() {
    //     let key = EncodedVec::new(k_slice);
    //     let val = key.debug_value(v_slice);
    //     dbg!(key);
    //     dbg!(val);
    //     it.next();
    // }
}