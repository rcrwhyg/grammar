use rhai::{
    serde::{from_dynamic, to_dynamic},
    Dynamic, Engine, EvalAltResult, Map,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct Point {
    x: f64,
    y: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct MyStruct {
    a: i64,
    b: Vec<String>,
    c: bool,
    d: Point,
}

pub fn ser() {
    let x = MyStruct {
        a: 42,
        b: vec!["hello".to_string(), "world".to_string()],
        c: true,
        d: Point { x: 1.0, y: 2.0 },
    };

    println!("MyStruct: {x:#?}");

    // Convert the 'MyStruct' into a 'Dynamic'
    let map = to_dynamic(x).unwrap();

    assert!(map.is::<Map>());
    println!("Dynamic: {map:#?}");
}

pub fn de() {
    let engine = Engine::new();
    let result: Dynamic = engine
        .eval(
            r#"
        #{
            a: 42,
            b: ["hello", "world"],
            c: true,
            d: #{
                x: 1.0,
                y: 2.0
            }
        }
    "#,
        )
        .unwrap();

    println!("Dynamic: {result:#?}");

    // Convert the 'Dynamic' back into a 'MyStruct'
    let x: MyStruct = from_dynamic(&result).unwrap();

    assert_eq!(
        x,
        MyStruct {
            a: 42,
            b: vec!["hello".to_string(), "world".to_string()],
            c: true,
            d: Point { x: 1.0, y: 2.0 },
        }
    );
    println!("MyStruct: {x:#?}");
}

fn main() -> Result<(), Box<EvalAltResult>> {
    ser();
    println!("-----------------");
    de();

    Ok(())
}
