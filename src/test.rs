use super::{Executor, Mode};

#[test]
fn calculate() {
    let mut executor = Executor::new(Mode::Script);

    assert_eq!(
        {
            executor.evaluate_program("5 8 add".to_string());
            executor.pop_stack().get_number()
        },
        13f64
    );

    assert_eq!(
        {
            executor.evaluate_program("8 3 sub".to_string());
            executor.pop_stack().get_number()
        },
        5f64
    );

    assert_eq!(
        {
            executor.evaluate_program("5 8 mul".to_string());
            executor.pop_stack().get_number()
        },
        40f64
    );

    assert_eq!(
        {
            executor.evaluate_program("10 5 div".to_string());
            executor.pop_stack().get_number()
        },
        2f64
    );

    assert_eq!(
        {
            executor.evaluate_program("3 2 pow".to_string());
            executor.pop_stack().get_number()
        },
        9f64
    );
}

#[test]
fn variables() {
    let mut executor = Executor::new(Mode::Script);

    assert_eq!(
        {
            executor.evaluate_program("5987 (x) var x".to_string());
            executor.pop_stack().get_number()
        },
        5987f64
    );

    assert_eq!(
        {
            executor.evaluate_program("5987 (x) var x 1 add (x) var x".to_string());
            executor.pop_stack().get_number()
        },
        5988f64
    );
}

#[test]
fn control_if() {
    let mut executor = Executor::new(Mode::Script);

    assert_eq!(
        {
            executor.evaluate_program("(true) (false) 10 2 div 5 equal if".to_string());
            executor.pop_stack().get_bool()
        },
        true
    );

    assert_eq!(
        {
            executor.evaluate_program("(true) (false) 10 2 div 4 equal if".to_string());
            executor.pop_stack().get_bool()
        },
        false
    );
}

#[test]
fn control_while() {
    let mut executor = Executor::new(Mode::Script);

    assert_eq!(
        {
            executor
                .evaluate_program("5 (i) var (i 1 add (i) var) (i 10 less) while i".to_string());
            executor.pop_stack().get_number()
        },
        10f64
    );
}
