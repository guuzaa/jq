use predicates::prelude::*;

pub fn json_matches(pairs: &[(&str, &str)]) -> impl Predicate<str> {
    let mut conditions = vec![
        Box::new(predicate::str::starts_with("{")) as Box<dyn Predicate<str>>,
        Box::new(predicate::str::ends_with("}\n")) as Box<dyn Predicate<str>>,
    ];

    for (key, value) in pairs {
        conditions.push(Box::new(predicate::str::contains(format!("\"{}\"", key))));
        conditions.push(Box::new(predicate::str::contains(format!("\"{}\"", value))));
    }

    predicates::function::function(move |actual: &str| {
        conditions.iter().all(|pred| pred.eval(actual))
    })
}
