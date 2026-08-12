use crate::utils::writer::Writer;
use grep_searcher::SearcherBuilder;
use indoc::formatdoc;
use pretty_assertions::{assert_eq, assert_ne};
use std::collections::HashMap;

use crate::rules::eval_context::eval_context_tests::BasicQueryTesting;
use crate::rules::eval_context::{
    root_scope, simplified_json_from_root, ClauseReport, EventRecord, RecordTracker,
};

use super::*;

//
// All unary function simple tests
//

#[test]
fn test_all_unary_functions() -> Result<()> {
    let path_value = PathAwareValue::try_from("{}")?;
    let non_empty_path_value =
        PathAwareValue::try_from(serde_yaml::from_str::<serde_yaml::Value>(
            r#"
        Resources:
          ec2:
            Type: AWS::EC2::Instance
            Properties:
              ImageId: ami-123456789012
              Tags: []
        "#,
        )?)?;
    let list_value =
        PathAwareValue::try_from(serde_yaml::from_str::<serde_yaml::Value>(r#"[1, 2, 3]"#)?)?;
    let empty_list_value =
        PathAwareValue::try_from(serde_yaml::from_str::<serde_yaml::Value>(r#"[]"#)?)?;
    let string_value = PathAwareValue::try_from(r#""String""#)?;
    let empty_string_value = PathAwareValue::try_from(r#""""#)?;
    let int_value = PathAwareValue::try_from(r#"10"#)?;
    let bool_value = PathAwareValue::try_from(r#"true"#)?;
    let float_value = PathAwareValue::try_from(r#"10.2"#)?;
    let char_range_value = PathAwareValue::try_from(r#"r[a, d)"#)?;
    let int_range_value = PathAwareValue::try_from(r#"r(10, 20)"#)?;
    let float_range_value = PathAwareValue::try_from(r#"r(10.0, 20.5]"#)?;
    let null_value = PathAwareValue::Null(path_value::Path::root());

    type UnaryTest<'test> = Vec<(
        Box<dyn Fn(&QueryResult) -> Result<bool>>,
        Vec<QueryResult>,
        Vec<QueryResult>,
    )>;

    let tests: UnaryTest = vec![
        (
            Box::new(exists_operation),
            // Successful tests
            vec![
                QueryResult::Resolved(Rc::new(path_value.clone())),
                QueryResult::Resolved(Rc::new(non_empty_path_value.clone())),
            ],
            // Failure tests
            vec![QueryResult::UnResolved(UnResolved {
                traversed_to: Rc::new(path_value.clone()),
                reason: None,
                remaining_query: "".to_string(),
            })],
        ),
        (
            Box::new(element_empty_operation),
            // Successful Tests
            vec![
                QueryResult::Resolved(Rc::new(path_value.clone())),
                QueryResult::Resolved(Rc::new(empty_string_value)), // we do check for string empty as well
                QueryResult::Resolved(Rc::new(empty_list_value.clone())),
                QueryResult::UnResolved(UnResolved {
                    remaining_query: "".to_string(),
                    reason: None,
                    traversed_to: Rc::new(path_value.clone()),
                }),
            ],
            // Failure tests
            vec![
                QueryResult::Resolved(Rc::new(non_empty_path_value.clone())),
                QueryResult::Resolved(Rc::new(list_value.clone())),
                QueryResult::Resolved(Rc::new(string_value.clone())),
            ],
        ),
        (
            Box::new(is_string_operation),
            // Success Case
            vec![QueryResult::Resolved(Rc::new(string_value.clone()))],
            // Failure Cases
            vec![
                QueryResult::Resolved(Rc::new(path_value.clone())),
                QueryResult::Resolved(Rc::new(list_value.clone())),
                QueryResult::Resolved(Rc::new(int_value.clone())),
                QueryResult::Resolved(Rc::new(non_empty_path_value.clone())),
                QueryResult::UnResolved(UnResolved {
                    traversed_to: Rc::new(path_value.clone()),
                    reason: None,
                    remaining_query: "".to_string(),
                }),
            ],
        ),
        (
            Box::new(is_int_operation),
            // Success Case
            vec![QueryResult::Resolved(Rc::new(int_value.clone()))],
            // Failure Cases
            vec![
                QueryResult::Resolved(Rc::new(path_value.clone())),
                QueryResult::Resolved(Rc::new(list_value.clone())),
                QueryResult::Resolved(Rc::new(string_value.clone())),
                QueryResult::Resolved(Rc::new(non_empty_path_value.clone())),
                QueryResult::UnResolved(UnResolved {
                    traversed_to: Rc::new(path_value.clone()),
                    reason: None,
                    remaining_query: "".to_string(),
                }),
            ],
        ),
        (
            Box::new(is_list_operation),
            // Success Case
            vec![
                QueryResult::Resolved(Rc::new(list_value.clone())),
                QueryResult::Resolved(Rc::new(empty_list_value.clone())),
            ],
            // Failure Cases
            vec![
                QueryResult::Resolved(Rc::new(path_value.clone())),
                QueryResult::Resolved(Rc::new(int_value.clone())),
                QueryResult::Resolved(Rc::new(int_range_value.clone())),
                QueryResult::Resolved(Rc::new(string_value.clone())),
                QueryResult::Resolved(Rc::new(non_empty_path_value.clone())),
                QueryResult::UnResolved(UnResolved {
                    traversed_to: Rc::new(path_value.clone()),
                    reason: None,
                    remaining_query: "".to_string(),
                }),
            ],
        ),
        (
            Box::new(is_struct_operation),
            // Success Case
            vec![
                QueryResult::Resolved(Rc::new(path_value.clone())),
                QueryResult::Resolved(Rc::new(non_empty_path_value.clone())),
            ],
            // Failure Cases
            vec![
                QueryResult::Resolved(Rc::new(int_value.clone())),
                QueryResult::Resolved(Rc::new(list_value.clone())),
                QueryResult::Resolved(Rc::new(string_value.clone())),
                QueryResult::Resolved(Rc::new(empty_list_value)),
                QueryResult::Resolved(Rc::new(float_value.clone())),
                QueryResult::UnResolved(UnResolved {
                    traversed_to: Rc::new(path_value.clone()),
                    reason: None,
                    remaining_query: "".to_string(),
                }),
            ],
        ),
        (
            Box::new(is_bool_operation),
            // Success Case
            vec![QueryResult::Resolved(Rc::new(bool_value))],
            // Failure Cases
            vec![
                QueryResult::Resolved(Rc::new(path_value.clone())),
                QueryResult::Resolved(Rc::new(list_value.clone())),
                QueryResult::Resolved(Rc::new(string_value.clone())),
                QueryResult::Resolved(Rc::new(non_empty_path_value.clone())),
                QueryResult::UnResolved(UnResolved {
                    traversed_to: Rc::new(path_value.clone()),
                    reason: None,
                    remaining_query: "".to_string(),
                }),
            ],
        ),
        (
            Box::new(is_float_operation),
            // Success Case
            vec![QueryResult::Resolved(Rc::new(float_value.clone()))],
            // Failure Cases
            vec![
                QueryResult::Resolved(Rc::new(path_value.clone())),
                QueryResult::Resolved(Rc::new(list_value.clone())),
                QueryResult::Resolved(Rc::new(string_value.clone())),
                QueryResult::Resolved(Rc::new(int_value.clone())),
                QueryResult::Resolved(Rc::new(non_empty_path_value.clone())),
                QueryResult::UnResolved(UnResolved {
                    traversed_to: Rc::new(path_value.clone()),
                    reason: None,
                    remaining_query: "".to_string(),
                }),
            ],
        ),
        (
            Box::new(is_char_range_operation),
            // Success Case
            vec![QueryResult::Resolved(Rc::new(char_range_value.clone()))],
            // Failure Cases
            vec![
                QueryResult::Resolved(Rc::new(path_value.clone())),
                QueryResult::Resolved(Rc::new(list_value.clone())),
                QueryResult::Resolved(Rc::new(string_value.clone())),
                QueryResult::Resolved(Rc::new(int_value.clone())),
                QueryResult::Resolved(Rc::new(non_empty_path_value.clone())),
                QueryResult::Resolved(Rc::new(float_range_value.clone())),
                QueryResult::Resolved(Rc::new(int_range_value.clone())),
                QueryResult::UnResolved(UnResolved {
                    traversed_to: Rc::new(path_value.clone()),
                    reason: None,
                    remaining_query: "".to_string(),
                }),
            ],
        ),
        (
            Box::new(is_int_range_operation),
            // Success Case
            vec![QueryResult::Resolved(Rc::new(int_range_value))],
            // Failure Cases
            vec![
                QueryResult::Resolved(Rc::new(path_value.clone())),
                QueryResult::Resolved(Rc::new(list_value.clone())),
                QueryResult::Resolved(Rc::new(string_value.clone())),
                QueryResult::Resolved(Rc::new(int_value.clone())),
                QueryResult::Resolved(Rc::new(non_empty_path_value.clone())),
                QueryResult::Resolved(Rc::new(float_range_value.clone())),
                QueryResult::Resolved(Rc::new(char_range_value.clone())),
                QueryResult::UnResolved(UnResolved {
                    traversed_to: Rc::new(path_value.clone()),
                    reason: None,
                    remaining_query: "".to_string(),
                }),
            ],
        ),
        (
            Box::new(is_float_range_operation),
            // Success Case
            vec![QueryResult::Resolved(Rc::new(float_range_value.clone()))],
            // Failure Cases
            vec![
                QueryResult::Resolved(Rc::new(path_value.clone())),
                QueryResult::Resolved(Rc::new(list_value.clone())),
                QueryResult::Resolved(Rc::new(string_value.clone())),
                QueryResult::Resolved(Rc::new(int_value.clone())),
                QueryResult::Resolved(Rc::new(non_empty_path_value.clone())),
                QueryResult::Resolved(Rc::new(char_range_value.clone())),
                QueryResult::UnResolved(UnResolved {
                    traversed_to: Rc::new(path_value.clone()),
                    reason: None,
                    remaining_query: "".to_string(),
                }),
            ],
        ),
        (
            Box::new(is_null_operation),
            // Success Case
            vec![QueryResult::Resolved(Rc::new(null_value.clone()))],
            // Failure Cases
            vec![
                QueryResult::Resolved(Rc::new(path_value.clone())),
                QueryResult::Resolved(Rc::new(list_value)),
                QueryResult::Resolved(Rc::new(string_value)),
                QueryResult::Resolved(Rc::new(int_value)),
                QueryResult::Resolved(Rc::new(non_empty_path_value)),
                QueryResult::Resolved(Rc::new(char_range_value)),
                QueryResult::Resolved(Rc::new(float_value)),
                QueryResult::Resolved(Rc::new(float_range_value)),
                QueryResult::UnResolved(UnResolved {
                    traversed_to: Rc::new(path_value),
                    reason: None,
                    remaining_query: "".to_string(),
                }),
            ],
        ),
    ];

    for (index, (func, successes, failures)) in tests.iter().enumerate() {
        println!("Testing Case #{}", index);
        for (idx, each_success) in successes.iter().enumerate() {
            println!("Testing Success Case {}#{}", index, idx);
            assert!((*func)(each_success)?);
        }
        for (idx, each_failure) in failures.iter().enumerate() {
            println!("Testing Failure Case {}#{}", index, idx);
            assert!(!(*func)(each_failure)?);
        }
    }

    Ok(())
}

#[test]
fn query_empty_and_non_empty() -> Result<()> {
    let path_value = PathAwareValue::try_from(serde_yaml::from_str::<serde_yaml::Value>(
        r#"
        Resources:
           s3:
             Type: AWS::S3::Bucket
           ec2:
             Type: AWS::EC2::Instance
             Properties:
               ImageId: ami-123456789012
        "#,
    )?)?;
    let mut eval = BasicQueryTesting {
        root: Rc::new(path_value),
        recorder: None,
    };

    let query = AccessQuery::try_from("Resources.*[ Type == /Bucket/ ]")?.query;
    let status = unary_operation(
        &query,
        (CmpOperator::Empty, true),
        false,
        "".to_string(),
        None,
        &mut eval,
    )?;
    match status {
        EvaluationResult::QueryValueResult(expected) => {
            assert_eq!(expected.len(), 1);
            let matched = &expected[0].0;
            match matched {
                QueryResult::Resolved(res) => {
                    assert_eq!(res.self_path().0.as_str(), "/Resources/s3");
                }
                _ => unreachable!(),
            }
        }

        EvaluationResult::EmptyQueryResult(..) => unreachable!(),
    }

    let query = AccessQuery::try_from("Resources.*[ Type == /Broker/ ]")?.query;
    let status = unary_operation(
        &query,
        (CmpOperator::Empty, true),
        false,
        "".to_string(),
        None,
        &mut eval,
    )?;
    match status {
        EvaluationResult::QueryValueResult(_) => unreachable!(),
        EvaluationResult::EmptyQueryResult(status, _) => {
            assert_eq!(status, Status::FAIL);
        }
    }

    Ok(())
}

//
// Binary comparison testing of each_lhs_value
//

#[test]
fn each_lhs_value_not_comparable() -> Result<()> {
    let path_value = PathAwareValue::try_from(serde_yaml::from_str::<serde_yaml::Value>(
        r#"
        Parameters:
          allowed_images: [ami-123456789012, ami-01234567890]
        Resources:
           s3:
             Type: AWS::S3::Bucket
           ec2:
             Type: AWS::EC2::Instance
             Properties:
               ImageId: ami-123456789012
        "#,
    )?)?;
    let mut eval = BasicQueryTesting {
        root: Rc::new(path_value),
        recorder: None,
    };

    let query_ec2 = AccessQuery::try_from("Resources.ec2.Properties.ImageId")?.query;
    let lhs = eval.query(&query_ec2)?;
    assert_eq!(lhs.len(), 1);
    let lhs = match &lhs[0] {
        QueryResult::Resolved(val) => val,
        _ => unreachable!(),
    };
    let rhs_query = AccessQuery::try_from("Parameters.allowed_images")?.query;
    let rhs = eval.query(&rhs_query)?;
    let result = each_lhs_compare(compare_eq, Rc::clone(lhs), &rhs)?;

    assert_eq!(result.len(), 1);
    let cmp_result = &result[0];
    match cmp_result {
        RhsComparison::NotComparable(NotComparableWithRhs {
            pair: ComparedPair { rhs: value, .. },
            ..
        }) => {
            let rhs_ptr = match &rhs[0] {
                QueryResult::Resolved(ptr) => &**ptr,
                _ => unreachable!(),
            };

            assert_eq!(rhs_ptr, &**value);
        }

        _ => unreachable!(),
    }

    let result = each_lhs_compare(
        in_cmp(true), // not in operation
        Rc::clone(lhs),
        &rhs,
    )?;

    assert_eq!(result.len(), 1);
    let cmp_result = &result[0];
    match cmp_result {
        RhsComparison::Comparable(ComparisonWithRhs { outcome, .. }) => {
            assert!(!(*outcome));
        }

        _ => unreachable!(),
    }

    let result = each_lhs_compare(
        in_cmp(false), // in operation
        Rc::clone(lhs),
        &rhs,
    )?;

    assert_eq!(result.len(), 1);
    let cmp_result = &result[0];
    match cmp_result {
        RhsComparison::Comparable(ComparisonWithRhs { outcome, .. }) => {
            assert!(*outcome);
        }

        _ => unreachable!(),
    }

    Ok(())
}

#[test]
fn each_lhs_value_eq_compare() -> Result<()> {
    let path_value = PathAwareValue::try_from(serde_yaml::from_str::<serde_yaml::Value>(
        r#"
        Parameters:
          allowed_images: [ami-123456789012, ami-01234567890]
        Resources:
           s3:
             Type: AWS::S3::Bucket
           ec2:
             Type: AWS::EC2::Instance
             Properties:
               ImageId: ami-123456789012
        "#,
    )?)?;
    let mut eval = BasicQueryTesting {
        root: Rc::new(path_value),
        recorder: None,
    };

    let query_ec2 = AccessQuery::try_from("Resources.ec2.Properties.ImageId")?.query;
    let lhs = eval.query(&query_ec2)?;
    assert_eq!(lhs.len(), 1);
    let lhs = match &lhs[0] {
        QueryResult::Resolved(val) => val,
        _ => unreachable!(),
    };
    let rhs_query = AccessQuery::try_from("Parameters.allowed_images[*]")?.query;
    let rhs = eval.query(&rhs_query)?;
    assert_eq!(rhs.len(), 2);
    let result = each_lhs_compare(compare_eq, Rc::clone(lhs), &rhs)?;

    assert_eq!(result.len(), 2);
    for cmp_result in result {
        match cmp_result {
            RhsComparison::Comparable(ComparisonWithRhs {
                pair: ComparedPair { rhs, .. },
                outcome,
            }) => {
                if outcome {
                    match (&**lhs, &*rhs) {
                        (PathAwareValue::String((_, s1)), PathAwareValue::String((_, s2))) => {
                            assert_eq!(s1, s2);
                            assert!(!std::ptr::eq(s1, s2));
                            assert_eq!(s1.as_str(), "ami-123456789012")
                        }
                        (_, _) => unreachable!(),
                    }
                } else {
                    match (&**lhs, &*rhs) {
                        (PathAwareValue::String((_, s1)), PathAwareValue::String((_, s2))) => {
                            assert_ne!(s1, s2);
                            assert!(!std::ptr::eq(s1, s2));
                            assert_eq!(s1.as_str(), "ami-123456789012");
                            assert_eq!(s2.as_str(), "ami-01234567890");
                        }
                        (_, _) => unreachable!(),
                    }
                }
            }

            _ => unreachable!(),
        }
    }

    Ok(())
}

#[test]
fn each_lhs_value_eq_compare_mixed_comparable() -> Result<()> {
    let path_value = PathAwareValue::try_from(serde_yaml::from_str::<serde_yaml::Value>(
        r#"
        Parameters:
          allowed_images: [ami-123456789012, ami-01234567890]
        Resources:
          iam:
            Type: AWS::IAM::Role
            Properties:
              PolicyDocument:
                Statement:
                  - Principal: '*'
                    Effect: Allow
                    Resource: ['s3*']
                  - Principal: [aws-123, aws-345]
                    Effect: Allow
                    Resource: '*'
        "#,
    )?)?;
    let mut eval = BasicQueryTesting {
        root: Rc::new(path_value),
        recorder: None,
    };

    //
    // Equivalent of Resources.*.Properties.PolicyDocument.Statement[*].Principal
    //
    let lhs_query =
        AccessQuery::try_from("Resources.*.Properties.PolicyDocument.Statement[*].Principal")?
            .query;
    let selected_lhs = eval.query(&lhs_query)?;
    assert_eq!(selected_lhs.len(), 2); // 2 statements present

    let rhs_value = PathAwareValue::try_from(r#""*""#)?;
    let rhs_query_result = vec![QueryResult::Resolved(Rc::new(rhs_value))];
    for each_lhs in selected_lhs {
        match &each_lhs {
            QueryResult::Resolved(lhs) => {
                for cmp_result in each_lhs_compare(
                    not_compare(compare_eq, true),
                    Rc::clone(lhs),
                    &rhs_query_result,
                )? {
                    match cmp_result {
                        RhsComparison::Comparable(ComparisonWithRhs { outcome, .. }) => {
                            if !outcome {
                                assert_eq!(lhs.self_path().0.as_str(), "/Resources/iam/Properties/PolicyDocument/Statement/0/Principal");
                            } else {
                                assert!(lhs.self_path().0.starts_with("/Resources/iam/Properties/PolicyDocument/Statement/1/Principal"));
                            }
                        }

                        _ => unreachable!(),
                    }
                }
            }

            _ => unreachable!(),
        }
    }

    Ok(())
}

#[test]
fn each_lhs_value_eq_compare_mixed_single_plus_array_form_correct_exec() -> Result<()> {
    let path_value = PathAwareValue::try_from(serde_yaml::from_str::<serde_yaml::Value>(
        r#"
        Parameters:
          allowed_images: [ami-123456789012, ami-01234567890]
        Resources:
          iam:
            Type: AWS::IAM::Role
            Properties:
              PolicyDocument:
                Statement:
                  - Principal: '*'
                    Effect: Allow
                    Resource: ['s3*']
                  - Principal: [aws-123, aws-345]
                    Effect: Allow
                    Resource: '*'
        "#,
    )?)?;
    let mut eval = BasicQueryTesting {
        root: Rc::new(path_value),
        recorder: None,
    };

    //
    // Equivalent of Resources.*.Properties.PolicyDocument.Statement[*].Principal[*] == '*'
    //
    let lhs_query =
        AccessQuery::try_from("Resources.*.Properties.PolicyDocument.Statement[*].Principal[*]")?
            .query;
    let selected_lhs = eval.query(&lhs_query)?;
    assert_eq!(selected_lhs.len(), 3); // 3 selected values

    let rhs_value = PathAwareValue::try_from(r#""*""#)?;
    let rhs_query_result = vec![QueryResult::Resolved(Rc::new(rhs_value))];
    for each_lhs in selected_lhs {
        match each_lhs {
            QueryResult::Resolved(lhs) => {
                for cmp_result in each_lhs_compare(compare_eq, Rc::clone(&lhs), &rhs_query_result)?
                {
                    match cmp_result {
                        RhsComparison::Comparable(ComparisonWithRhs { outcome, .. }) => {
                            if outcome {
                                assert_eq!(lhs.self_path().0.as_str(), "/Resources/iam/Properties/PolicyDocument/Statement/0/Principal");
                            } else {
                                match lhs.self_path().0.as_str() {
                                    "/Resources/iam/Properties/PolicyDocument/Statement/1/Principal/0" |
                                    "/Resources/iam/Properties/PolicyDocument/Statement/1/Principal/1" => {},
                                    _ => unreachable!()
                                }
                            }
                        }

                        _ => unreachable!(),
                    }
                }
            }

            _ => unreachable!(),
        }
    }

    Ok(())
}

macro_rules! test_case {
    ($rhs_value:expr, $lhs:expr, $eval:ident, $func:expr, $assert:expr) => {
        let lhs_gt_query = AccessQuery::try_from($lhs)?.query;
        let rhs_value = $rhs_value;
        let values = $eval.query(&lhs_gt_query)?;
        for each_lhs in values {
            match each_lhs {
                QueryResult::Resolved(res) => {
                    for cmp_result in each_lhs_compare(
                        $func,
                        res,
                        &[QueryResult::Resolved(Rc::new(rhs_value.clone()))],
                    )? {
                        match cmp_result {
                            RhsComparison::Comparable(ComparisonWithRhs { outcome, .. }) => {
                                assert_eq!(outcome, $assert);
                            }

                            _ => {}
                        }
                    }
                }

                _ => unreachable!(),
            }
        }
    };
}

#[test]
fn binary_comparisons_gt_ge() -> Result<()> {
    let path_value = PathAwareValue::try_from(serde_yaml::from_str::<serde_yaml::Value>(
        r#"
        values:
          int: 10
          ints: [20, 10]
          float: 1.0
          array: [1 ,2]
          string: Hi
    "#,
    )?)?;
    let mut eval = BasicQueryTesting {
        root: Rc::new(path_value),
        recorder: None,
    };

    //
    // Testing gt
    //
    test_case!(
        PathAwareValue::try_from("8")?,
        r#"values.int"#,
        eval,
        crate::rules::path_value::compare_gt,
        true
    );
    test_case!(
        PathAwareValue::try_from("8")?,
        r#"values.ints"#,
        eval,
        crate::rules::path_value::compare_gt,
        true
    );
    test_case!(
        PathAwareValue::try_from("8")?,
        r#"values.int"#,
        eval,
        crate::rules::path_value::compare_ge,
        true
    );
    test_case!(
        PathAwareValue::try_from("8")?,
        r#"values.ints"#,
        eval,
        crate::rules::path_value::compare_ge,
        true
    );
    test_case!(
        PathAwareValue::try_from("10")?,
        r#"values.ints"#,
        eval,
        crate::rules::path_value::compare_ge,
        true
    );

    test_case!(
        PathAwareValue::try_from("15")?,
        r#"values.int"#,
        eval,
        crate::rules::path_value::compare_gt,
        false
    );

    test_case!(
        PathAwareValue::try_from("0.5")?,
        r#"values.float"#,
        eval,
        crate::rules::path_value::compare_gt,
        true
    );
    test_case!(
        PathAwareValue::try_from("1.5")?,
        r#"values.float"#,
        eval,
        crate::rules::path_value::compare_gt,
        false
    );
    test_case!(
        PathAwareValue::try_from("1.0")?,
        r#"values.float"#,
        eval,
        crate::rules::path_value::compare_ge,
        true
    );

    test_case!(
        PathAwareValue::try_from(r#""Hi""#)?,
        r#"values.string"#,
        eval,
        crate::rules::path_value::compare_ge,
        true
    );
    test_case!(
        PathAwareValue::try_from(r#""Di""#)?,
        r#"values.string"#,
        eval,
        crate::rules::path_value::compare_gt,
        true
    );
    test_case!(
        PathAwareValue::try_from(r#""Ji""#)?,
        r#"values.string"#,
        eval,
        crate::rules::path_value::compare_gt,
        false
    );
    Ok(())
}

#[test]
fn binary_comparisons_lt_le() -> Result<()> {
    let path_value = PathAwareValue::try_from(serde_yaml::from_str::<serde_yaml::Value>(
        r#"
        values:
          int: 10
          ints: [20, 10]
          float: 1.0
          array: [1 ,2]
          string: Hi
    "#,
    )?)?;
    let mut eval = BasicQueryTesting {
        root: Rc::new(path_value),
        recorder: None,
    };

    //
    // Testing gt
    //
    test_case!(
        PathAwareValue::try_from("8")?,
        r#"values.int"#,
        eval,
        crate::rules::path_value::compare_lt,
        false
    );
    test_case!(
        PathAwareValue::try_from("8")?,
        r#"values.ints"#,
        eval,
        crate::rules::path_value::compare_lt,
        false
    );
    test_case!(
        PathAwareValue::try_from("8")?,
        r#"values.int"#,
        eval,
        crate::rules::path_value::compare_le,
        false
    );
    test_case!(
        PathAwareValue::try_from("8")?,
        r#"values.ints"#,
        eval,
        crate::rules::path_value::compare_le,
        false
    );

    test_case!(
        PathAwareValue::try_from("20")?,
        r#"values.ints"#,
        eval,
        crate::rules::path_value::compare_le,
        true
    );
    test_case!(
        PathAwareValue::try_from("15")?,
        r#"values.int"#,
        eval,
        crate::rules::path_value::compare_lt,
        true
    );

    test_case!(
        PathAwareValue::try_from("0.5")?,
        r#"values.float"#,
        eval,
        crate::rules::path_value::compare_lt,
        false
    );
    test_case!(
        PathAwareValue::try_from("1.0")?,
        r#"values.float"#,
        eval,
        crate::rules::path_value::compare_le,
        true
    );
    test_case!(
        PathAwareValue::try_from("1.5")?,
        r#"values.float"#,
        eval,
        crate::rules::path_value::compare_lt,
        true
    );

    test_case!(
        PathAwareValue::try_from(r#""Hi""#)?,
        r#"values.string"#,
        eval,
        crate::rules::path_value::compare_le,
        true
    );
    test_case!(
        PathAwareValue::try_from(r#""Di""#)?,
        r#"values.string"#,
        eval,
        crate::rules::path_value::compare_lt,
        false
    );
    test_case!(
        PathAwareValue::try_from(r#""Ji""#)?,
        r#"values.string"#,
        eval,
        crate::rules::path_value::compare_lt,
        true
    );
    Ok(())
}

#[test]
fn test_compare_rulegen() -> Result<()> {
    let rulegen_created = r#"
let aws_ec2_securitygroup_resources = Resources.*[ Type == 'AWS::EC2::SecurityGroup' ]
rule aws_ec2_securitygroup when %aws_ec2_securitygroup_resources !empty {
  %aws_ec2_securitygroup_resources.Properties.SecurityGroupEgress == [{"CidrIp":"0.0.0.0/0","IpProtocol":-1},{"CidrIpv6":"::/0","IpProtocol":-1}]
}"#;
    let template = r#"
Resources:

  # SecurityGroups
  ## Alb Security Groups

  rFrontendAppSpecificSg:
    Type: AWS::EC2::SecurityGroup
    Properties:
      GroupDescription: Frontend Security Group
      GroupName: secgrp-frontend
      SecurityGroupEgress:
        - CidrIp: "0.0.0.0/0"
          IpProtocol: -1
        - CidrIpv6: "::/0"
          IpProtocol: -1
      VpcId: vpc-123abc
    "#;
    let rules = RulesFile::try_from(rulegen_created)?;
    let value = PathAwareValue::try_from(serde_yaml::from_str::<serde_yaml::Value>(template)?)?;
    let mut root = root_scope(&rules, Rc::new(value));
    //let mut tracker = RecordTracker::new(&mut root);
    let status = eval_rules_file(&rules, &mut root, None)?;
    assert_eq!(status, Status::PASS);
    Ok(())
}

#[test]
fn block_guard_pass() -> Result<()> {
    let path_value = PathAwareValue::try_from(serde_yaml::from_str::<serde_yaml::Value>(
        r#"
        Resources:
          iam:
            Type: AWS::IAM::Role
            Properties:
              PolicyDocument:
                Statement:
                  - Principal: '*'
                    Effect: Allow
                    Resource: ['s3*']
                  - Principal: [aws-123, aws-345]
                    Effect: Allow
                    Resource: '*'
          ecs:
            Type: AWS::ECS::Task
            Properties:
              Role:
                Ref: iam
        "#,
    )?)?;

    let block_clauses = GuardClause::try_from(
        r#"Resources[ Type == /Role/ ].Properties.PolicyDocument {
      Statement[*] {
         Principal != '*' <<No wildcard allowed for Principals>>
      }
    }
    "#,
    )?;

    let mut tracker = RecordTracker::new();
    let mut eval = BasicQueryTesting {
        root: Rc::new(path_value),
        recorder: Some(&mut tracker),
    };
    let status = eval_guard_clause(&block_clauses, &mut eval, ClauseRole::Assertion)?;
    assert_eq!(status, Status::FAIL);
    let top = tracker.extract();
    match top.container.as_ref() {
        Some(record) => {
            assert!(matches!(
                record,
                RecordType::BlockGuardCheck(BlockCheck {
                    status: Status::FAIL,
                    ..
                })
            ),);
            //
            // 2 Map Filters, 1 Block Clause
            //
            assert_eq!(top.children.len(), 3);
            let top_child = &top.children[2];
            assert!(matches!(
                top_child.container.as_ref().unwrap(),
                RecordType::BlockGuardCheck(BlockCheck {
                    status: Status::FAIL,
                    ..
                })
            ),);
            assert_eq!(top_child.children.len(), 2); // There are 2 Statements inside PolicyDocument
            for (idx, each) in top_child.children.iter().enumerate() {
                match each.container.as_ref() {
                    Some(inner) => {
                        if idx == 0 {
                            assert!(matches!(
                                inner,
                                RecordType::GuardClauseBlockCheck(BlockCheck {
                                    status: Status::FAIL,
                                    ..
                                })
                            ),);
                            assert_eq!(each.children.len(), 1); // only on principal value
                            let guard_rec = &each.children[0];
                            match guard_rec.container.as_ref().unwrap() {
                                RecordType::ClauseValueCheck(ClauseCheck::Comparison(
                                    ComparisonClauseCheck {
                                        status: Status::FAIL,
                                        custom_message: Some(msg),
                                        message: None,
                                        comparison: (CmpOperator::Eq, true),
                                        from: QueryResult::Resolved(from_q),
                                        to: Some(QueryResult::Resolved(_)),
                                    },
                                )) => {
                                    assert_eq!(msg, "No wildcard allowed for Principals");
                                    assert_eq!(from_q.self_path().0.as_str(), "/Resources/iam/Properties/PolicyDocument/Statement/0/Principal");
                                }
                                _ => unreachable!(),
                            }
                        } else {
                            assert!(matches!(
                                inner,
                                RecordType::GuardClauseBlockCheck(BlockCheck {
                                    status: Status::PASS,
                                    ..
                                })
                            ),);
                            assert_eq!(each.children.len(), 2); // there are 2 principal values
                            for each_clause_check in &each.children {
                                match &each_clause_check.container {
                                    Some(RecordType::ClauseValueCheck(ClauseCheck::Success)) => {}
                                    _ => unreachable!(),
                                }
                            }
                        }
                    }
                    None => unreachable!(),
                }
            }
        }
        None => unreachable!(),
    }

    Ok(())
}

#[test]
fn test_guard_10_compatibility_and_diff() -> Result<()> {
    let value_str = r###"
    Statement:
      - Principal: ['*', 's3:*']
    "###;
    let value = PathAwareValue::try_from(serde_yaml::from_str::<serde_yaml::Value>(value_str)?)?;
    let mut eval = BasicQueryTesting {
        root: Rc::new(value),
        recorder: None,
    };

    //
    // Evaluation differences with 1.0 for Statement.*.Principal == '*'
    //
    // Guard 1.0 this would PASS with at-least one semantics for the payload above. This is where docs
    // need to be consulted to understand that == is at-least-one and != is ALL. Due to this decision certain
    // expressions like ensure that ALL AWS::EC2::Volume Encrypted == true, could not be specified
    //
    // In Guard 2.0 this would FAIL. The reason being that Guard 2.0 goes for explicitness in specifying
    // clauses. By default it asserts for ALL semantics. If you expecting to match at-least one or more
    // you must use SOME keyword that would evaluate correctly. With this support in 2.0 we can
    // support ALL expressions like
    //
    //        AWS::EC2::Volume Properties.Encrypted == true
    //
    // At the same time, one can explicitly express at-least-one or more semantics using SOME
    //
    //         AWS::EC2::Volume SOME Properties.Encrypted == true
    //
    // And finally
    //
    //       AWS::EC2::Volume Properties {
    //             Encrypted !EXISTS or
    //             Encrypted == true
    //       }
    //
    // can be correctly specified. This also makes the intent clear to both the rule author and
    // auditor what was acceptable. Here, it is okay that accept Encrypted was not specified
    // as an attribute or when specified it must be true. This makes it clear to the reader/auditor
    // rather than guess at how Guard engine evaluates.
    //
    // The evaluation engine is purposefully dumb and stupid, defaults to working
    // one way consistently enforcing ALL semantics. Needs to told explicitly to do otherwise
    //

    let clause_str = r#"Statement.*.Principal == '*'"#;
    let clause = GuardClause::try_from(clause_str)?;
    let status = eval_guard_clause(&clause, &mut eval, ClauseRole::Assertion)?;
    assert_eq!(status, Status::FAIL);

    let clause_str = r#"SOME Statement.*.Principal == '*'"#;
    let clause = GuardClause::try_from(clause_str)?;
    let status = eval_guard_clause(&clause, &mut eval, ClauseRole::Assertion)?;
    assert_eq!(status, Status::PASS);

    let value_str = r###"
    Statement:
      - Principal: aws
      - Principal: ['*', 's3:*']
    "###;
    let value = PathAwareValue::try_from(serde_yaml::from_str::<serde_yaml::Value>(value_str)?)?;
    let mut eval = BasicQueryTesting {
        root: Rc::new(value),
        recorder: None,
    };
    //
    // Evaluate the SOME clause again, it must pass with the value as well
    //
    let status = eval_guard_clause(&clause, &mut eval, ClauseRole::Assertion)?;
    assert_eq!(status, Status::PASS);

    Ok(())
}

#[test]
fn block_evaluation() -> Result<()> {
    let value_str = r#"
    Resources:
      apiGw:
        Type: 'AWS::ApiGateway::RestApi'
        Properties:
          EndpointConfiguration: ["PRIVATE"]
          Policy:
            Statement:
              - Action: Allow
                Resource: ['*', "aws:"]
                Condition:
                    'aws:IsSecure': true
                    'aws:sourceVpc': ['vpc-1234']
              - Action: Allow
                Resource: ['*', "aws:"]

    "#;
    let value = serde_yaml::from_str::<serde_yaml::Value>(value_str)?;
    let value = PathAwareValue::try_from(value)?;
    let clause_str = r#"Resources.*[ Type == 'AWS::ApiGateway::RestApi' ].Properties {
        EndpointConfiguration == ["PRIVATE"]
        some Policy.Statement[*] {
            Action == 'Allow'
            Condition[ keys == 'aws:IsSecure' ] !empty
        }
    }
    "#;
    let clause = GuardClause::try_from(clause_str)?;
    let mut eval = BasicQueryTesting {
        root: Rc::new(value),
        recorder: None,
    };
    let status = eval_guard_clause(&clause, &mut eval, ClauseRole::Assertion)?;
    assert_eq!(status, Status::PASS);
    Ok(())
}

#[test]
fn block_evaluation_fail() -> Result<()> {
    let value_str = r#"
    Resources:
      apiGw:
        Type: 'AWS::ApiGateway::RestApi'
        Properties:
          EndpointConfiguration: ["PRIVATE"]
          Policy:
            Statement:
              - Action: Allow
                Resource: ['*', "aws:"]
                Condition:
                    'aws:IsSecure': true
                    'aws:sourceVpc': ['vpc-1234']
              - Action: Allow
                Resource: ['*', "aws:"]
      apiGw2:
        Type: 'AWS::ApiGateway::RestApi'
        Properties:
          EndpointConfiguration: ["PRIVATE"]
          Policy:
            Statement:
              - Action: Allow
                Resource: ['*', "aws:"]

    "#;
    let value = serde_yaml::from_str::<serde_yaml::Value>(value_str)?;
    let value = PathAwareValue::try_from(value)?;
    let mut eval = BasicQueryTesting {
        root: Rc::new(value),
        recorder: None,
    };
    let clause_str = r#"Resources.*[ Type == 'AWS::ApiGateway::RestApi' ].Properties {
        EndpointConfiguration == ["PRIVATE"]
        some Policy.Statement[*] {
            Action == 'Allow'
            Condition[ keys == 'aws:IsSecure' ] !empty
        }
    }
    "#;
    let clause = GuardClause::try_from(clause_str)?;
    let status = eval_guard_clause(&clause, &mut eval, ClauseRole::Assertion)?;
    assert_eq!(status, Status::FAIL);
    Ok(())
}

#[test]
fn variable_projections() -> Result<()> {
    let path_value = PathAwareValue::try_from(serde_yaml::from_str::<serde_yaml::Value>(
        r#"
        Resources:
          s3_bucket:
            Type: AWS::S3::Bucket
          s3_bucket_policy:
            Type: AWS::S3::BucketPolicy
            Properties:
              Bucket:
                Ref: s3_bucket
          s3_bucket_policy_2:
            Type: AWS::S3::BucketPolicy
            Properties:
              Bucket: aws:arn
        "#,
    )?)?;

    let rules_file = RulesFile::try_from(
        r#"
    let policies = Resources[ Type == /BucketPolicy$/ ]
    rule policies_check when %policies not empty { # testing no view projection check
      %policies.Properties.Bucket exists
      %policies.Properties.Bucket not empty # checks both Map not empty/ string not empty
      #
      # checks Ref's value is not empty. This has 2 results, one FAILure for s3_bucket_policy_2
      # one PASS for s3_bucket_policy. Due to some keyword it does PASS
      #
      some %policies.Properties.Bucket.Ref not empty
    }
    "#,
    )?;
    let mut root_scope = root_scope(&rules_file, Rc::new(path_value));
    let status = eval_rules_file(&rules_file, &mut root_scope, None)?;
    assert_eq!(status, Status::PASS);

    Ok(())
}

#[test]
fn variable_projections_failures() -> Result<()> {
    let path_value = PathAwareValue::try_from(serde_yaml::from_str::<serde_yaml::Value>(
        r#"
        Resources:
          s3_bucket:
            Type: AWS::S3::Bucket
          s3_bucket_policy:
            Type: AWS::S3::BucketPolicy
            Properties:
              Bucket:
                Ref: s3_bucket
          s3_bucket_policy_2:
            Type: AWS::S3::BucketPolicy
            Properties:
              Bucket: ""
        "#,
    )?)?;

    let rules_file = RulesFile::try_from(
        r#"
    let policies = Resources[ Type == /BucketPolicy$/ ]
    rule policies_check when %policies not empty { # testing no view projection check
      %policies.Properties.Bucket exists
      %policies.Properties.Bucket not empty # checks both Map not empty/ string not empty
      #
      # checks Ref's value is not empty. This has 2 results, one FAILure for s3_bucket_policy_2
      # one PASS for s3_bucket_policy. Due to some keyword it does PASS
      #
      some %policies.Properties.Bucket.Ref not empty
    }
    "#,
    )?;
    let mut root_scope = root_scope(&rules_file, Rc::new(path_value));
    let status = eval_rules_file(&rules_file, &mut root_scope, None)?;
    assert_eq!(status, Status::FAIL); // for s3_bucket_policy_2.Properties.Bucket == ""

    let top = root_scope.reset_recorder().extract();
    assert_eq!(top.children.len(), 1); // one rule
    let rule = &top.children[0];
    assert_eq!(rule.children.len(), 4); // 1 one for rule condition, 3 for rule clauses
                                        //assert_eq!(matches!(rule_block.container, Some(RecordType::RuleBlock(Status::FAIL))), true);
    for (idx, each_rule_clause) in rule.children.iter().enumerate() {
        if idx == 0 {
            // Condition block
            assert!(matches!(
                each_rule_clause.container,
                Some(RecordType::RuleCondition(Status::PASS))
            ),);
            assert_eq!(each_rule_clause.children.len(), 1); //
            let gbc = &each_rule_clause.children[0];
            assert!(matches!(
                gbc.container,
                Some(RecordType::GuardClauseBlockCheck(BlockCheck {
                    status: Status::PASS,
                    ..
                }))
            ),);
        } else if idx == 2 {
            assert!(matches!(
                each_rule_clause.container,
                Some(RecordType::GuardClauseBlockCheck(BlockCheck {
                    status: Status::FAIL,
                    ..
                }))
            ),);
            assert_eq!(each_rule_clause.children.len(), 2); //
            let failed_clause = &each_rule_clause.children[1];
            assert!(matches!(
                failed_clause.container,
                Some(RecordType::ClauseValueCheck(ClauseCheck::Unary(
                    UnaryValueCheck {
                        comparison: (CmpOperator::Empty, true),
                        value: ValueCheck {
                            status: Status::FAIL,
                            ..
                        }
                    }
                )))
            ),);
        } else {
            assert!(matches!(
                each_rule_clause.container,
                Some(RecordType::GuardClauseBlockCheck(BlockCheck {
                    status: Status::PASS,
                    ..
                }))
            ),);
        }
    }

    Ok(())
}

#[test]
fn query_cross_joins() -> Result<()> {
    let path_value = PathAwareValue::try_from(serde_yaml::from_str::<serde_yaml::Value>(
        r#"
        Resources:
          s3_bucket:
            Type: AWS::S3::Bucket
          s3_bucket_policy:
            Type: AWS::S3::BucketPolicy
            Properties:
              Bucket:
                Ref: s3_bucket
        "#,
    )?)?;
    let rules_files = RulesFile::try_from(
        r#"
    rule s3_cross_query_join {
       let policies = Resources[ Type == /BucketPolicy$/ ].Properties.Bucket.Ref
       Resources.%policies {
         Type == 'AWS::S3::Bucket'
       }
    }
    "#,
    )?;
    let mut root_scope = root_scope(&rules_files, Rc::new(path_value.clone()));
    let status = eval_rules_file(&rules_files, &mut root_scope, None)?;
    assert_eq!(status, Status::PASS);

    let rules_files = RulesFile::try_from(
        r#"
    rule s3_cross_query_join {
       let policies = Resources[ Type == /NotBucketPolicy$/ ].Properties.Bucket.Ref
       Resources.%policies {
         Type == 'AWS::S3::Bucket'
       }
    }
    "#,
    )?;
    let mut root_scope = eval_context::root_scope(&rules_files, Rc::new(path_value));
    let status = eval_rules_file(&rules_files, &mut root_scope, None)?;
    assert_eq!(status, Status::SKIP);

    let path_value = PathAwareValue::try_from(serde_yaml::from_str::<serde_yaml::Value>(
        r#"
        Resources:
          s3_bucket:
            Type: AWS::S3::Bucket
          s3_bucket_policy:
            Type: AWS::S3::BucketPolicy
            Properties:
              Bucket:
                Ref: s3_bucket
          s3_bucket_policy_2:
            Type: AWS::S3::BucketPolicy
            Properties:
              Bucket: aws:arn...
        "#,
    )?)?;

    //
    // NO some present for assignment, hence failure
    //
    let rules_files = RulesFile::try_from(
        r#"
    rule s3_cross_query_join {
       let policies = Resources[ Type == /BucketPolicy$/ ].Properties.Bucket.Ref
       Resources.%policies {
         Type == 'AWS::S3::Bucket'
       }
    }
    "#,
    )?;
    let mut root_scope = eval_context::root_scope(&rules_files, Rc::new(path_value.clone()));
    let status = eval_rules_file(&rules_files, &mut root_scope, None)?;
    assert_eq!(status, Status::FAIL);

    //
    // Using SOME to indicate not all BucketPolicy object will have Bucket References. In
    // our payload s3_bucket_policy_2 is skipped as it does not resolve
    //
    let rules_files = RulesFile::try_from(
        r#"
    rule s3_cross_query_join {
       let policies = some Resources[ Type == /BucketPolicy$/ ].Properties.Bucket.Ref
       Resources.%policies {
         Type == 'AWS::S3::Bucket'
       }
    }
    "#,
    )?;
    let mut root_scope = eval_context::root_scope(&rules_files, Rc::new(path_value.clone()));
    let status = eval_rules_file(&rules_files, &mut root_scope, None)?;
    assert_eq!(status, Status::PASS);

    //
    // Using SOME at the block level will yield the same result
    // s3_bucket_policy_2 is skipped
    //
    let rules_files = RulesFile::try_from(
        r#"
    rule s3_cross_query_join {
       let policies = Resources[ Type == /BucketPolicy$/ ].Properties.Bucket.Ref
       some Resources.%policies {
         Type == 'AWS::S3::Bucket'
       }
    }
    "#,
    )?;
    let mut root_scope = eval_context::root_scope(&rules_files, Rc::new(path_value));
    let status = eval_rules_file(&rules_files, &mut root_scope, None)?;
    assert_eq!(status, Status::PASS);

    Ok(())
}

#[test]
fn cross_rule_clause_when_checks() -> Result<()> {
    let rules_skipped = r#"
    rule skipped when skip !exists {
        Resources.*.Properties.Tags !empty
    }

    rule dependent_on_skipped when skipped {
        Resources.*.Properties exists
    }

    rule dependent_on_dependent when dependent_on_skipped {
        Resources.*.Properties exists
    }

    rule dependent_on_not_skipped when !skipped {
        Resources.*.Properties exists
    }
    "#;

    let input = r#"
    {
        skip: true,
        Resources: {
            first: {
                Type: 'WhackWhat',
                Properties: {
                    Tags: [{ hi: "there" }, { right: "way" }]
                }
            }
        }
    }
    "#;

    let resources = PathAwareValue::try_from(input)?;
    let rules = RulesFile::try_from(rules_skipped)?;
    let mut root = root_scope(&rules, Rc::new(resources));
    let status = eval_rules_file(&rules, &mut root, None)?;
    assert_eq!(status, Status::PASS);
    let mut expectations = HashMap::with_capacity(4);
    expectations.insert("skipped".to_string(), Status::SKIP);
    expectations.insert("dependent_on_skipped".to_string(), Status::SKIP);
    expectations.insert("dependent_on_dependent".to_string(), Status::SKIP);
    expectations.insert("dependent_on_not_skipped".to_string(), Status::PASS);
    let rules_results = root.reset_recorder().extract().children;
    assert_eq!(rules_results.len(), 4);
    for each in rules_results {
        match each.container {
            Some(RecordType::RuleCheck(status)) => {
                assert_eq!(expectations.get(status.name), Some(&status.status));
            }

            _ => unreachable!(),
        }
    }

    let input = r#"
    {
        Resources: {
            first: {
                Type: 'WhackWhat',
                Properties: {
                    Tags: [{ hi: "there" }, { right: "way" }]
                }
            }
        }
    }
    "#;

    let resources = PathAwareValue::try_from(input)?;
    let mut root = root_scope(&rules, Rc::new(resources));
    let status = eval_rules_file(&rules, &mut root, None)?;
    assert_eq!(status, Status::PASS);
    expectations.clear();
    expectations.insert("skipped".to_string(), Status::PASS);
    expectations.insert("dependent_on_skipped".to_string(), Status::PASS);
    expectations.insert("dependent_on_dependent".to_string(), Status::PASS);
    expectations.insert("dependent_on_not_skipped".to_string(), Status::SKIP);

    let rules_results = root.reset_recorder().extract().children;
    assert_eq!(rules_results.len(), 4);
    for each in rules_results {
        match each.container {
            Some(RecordType::RuleCheck(status)) => {
                assert_eq!(expectations.get(status.name), Some(&status.status));
            }

            _ => unreachable!(),
        }
    }

    Ok(())
}

#[test]
fn test_field_type_array_or_single() -> Result<()> {
    let statements = r#"{
        Statement: [{
            Action: '*',
            Effect: 'Allow',
            Resources: '*'
        }, {
            Action: ['api:Get', 'api2:Set'],
            Effect: 'Allow',
            Resources: '*'
        }]
    }
    "#;
    let path_value = PathAwareValue::try_from(statements)?;
    let clause = GuardClause::try_from(r#"Statement[*].Action != '*'"#)?;
    let mut eval = BasicQueryTesting {
        root: Rc::new(path_value),
        recorder: None,
    };
    let status = eval_guard_clause(&clause, &mut eval, ClauseRole::Assertion)?;
    assert_eq!(status, Status::FAIL);

    let statements = r#"{
        Statement: {
            Action: '*',
            Effect: 'Allow',
            Resources: '*'
        }
    }
    "#;
    let path_value = PathAwareValue::try_from(statements)?;
    let mut eval = BasicQueryTesting {
        root: Rc::new(path_value),
        recorder: None,
    };
    let status = eval_guard_clause(&clause, &mut eval, ClauseRole::Assertion)?;
    assert_eq!(status, Status::FAIL);

    let clause = GuardClause::try_from(r#"Statement[*].Action[*] != '*'"#)?;
    let status = eval_guard_clause(&clause, &mut eval, ClauseRole::Assertion)?;
    assert_eq!(status, Status::FAIL);

    // Test old format
    let clause = GuardClause::try_from(r#"Statement.*.Action.* != '*'"#)?;
    let status = eval_guard_clause(&clause, &mut eval, ClauseRole::Assertion)?;
    assert_eq!(status, Status::FAIL);

    let clause = GuardClause::try_from(r#"some Statement[*].Action == '*'"#)?;
    let status = eval_guard_clause(&clause, &mut eval, ClauseRole::Assertion)?;
    assert_eq!(status, Status::PASS);

    let clause = GuardClause::try_from(r#"some Statement[*].Action != '*'"#)?;
    let status = eval_guard_clause(&clause, &mut eval, ClauseRole::Assertion)?;
    assert_eq!(status, Status::FAIL);

    Ok(())
}

#[test]
fn test_for_in_and_not_in() -> Result<()> {
    let statements = r#"
    {
      "mainSteps": [
          {
            "action": "aws:updateAgent"
          },
          {
            "action": "aws:configurePackage"
          }
        ]
    }"#;

    let value = PathAwareValue::try_from(serde_yaml::from_str::<serde_yaml::Value>(statements)?)?;
    let mut eval = BasicQueryTesting {
        root: Rc::new(value),
        recorder: None,
    };

    let clause = GuardClause::try_from(
        r#"mainSteps[*].action !IN ["aws:updateSsmAgent", "aws:updateAgent"]"#,
    )?;
    let status = eval_guard_clause(&clause, &mut eval, ClauseRole::Assertion)?;
    assert_eq!(status, Status::FAIL);

    let clause = GuardClause::try_from(
        r#"mainSteps[*].action IN ["aws:updateSsmAgent", "aws:updateAgent"]"#,
    )?;
    let status = eval_guard_clause(&clause, &mut eval, ClauseRole::Assertion)?;
    assert_eq!(status, Status::FAIL);

    let clause = GuardClause::try_from(
        r#"some mainSteps[*].action IN ["aws:updateSsmAgent", "aws:updateAgent"]"#,
    )?;
    let status = eval_guard_clause(&clause, &mut eval, ClauseRole::Assertion)?;
    assert_eq!(status, Status::PASS);

    Ok(())
}

#[test]
fn test_rule_with_range_test_and_this() -> Result<()> {
    let rule_str = r#"rule check_parameter_validity {
     InputParameter.TcpBlockedPorts[*] {
         this in r[0, 65535] <<[NON_COMPLIANT] Parameter TcpBlockedPorts has invalid value.>>
     }
 }"#;

    let rule = Rule::try_from(rule_str)?;

    let value_str = r#"
    InputParameter:
        TcpBlockedPorts:
            - 21
            - 22
            - 101
    "#;
    let value = PathAwareValue::try_from(serde_yaml::from_str::<serde_yaml::Value>(value_str)?)?;
    let mut eval = BasicQueryTesting {
        root: Rc::new(value),
        recorder: None,
    };
    let status = eval_rule(&rule, &mut eval, ClauseRole::Assertion)?;
    assert_eq!(status, Status::PASS);

    let value_str = r#"
    InputParameter:
        TcpBlockedPorts:
            - 21
            - 22
            - 101
            - 100000
    "#;
    let value = PathAwareValue::try_from(serde_yaml::from_str::<serde_yaml::Value>(value_str)?)?;
    let mut eval = BasicQueryTesting {
        root: Rc::new(value),
        recorder: None,
    };
    let status = eval_rule(&rule, &mut eval, ClauseRole::Assertion)?;
    assert_eq!(status, Status::FAIL);

    Ok(())
}

#[test]
fn test_inner_when_skipped() -> Result<()> {
    let rule_str = r#"
    rule no_wild_card_in_managed_policy {
        Resources[ Type == /ManagedPolicy/ ] {
            when Properties.ManagedPolicyName != /Admin/ {
                Properties.PolicyDocument.Statement[*].Action[*] != '*'
            }
        }
    }
    "#;

    let rule = Rule::try_from(rule_str)?;
    let value_str = r#"
    Resources:
      ReadOnlyAdminPolicy:
        Type: 'AWS::IAM::ManagedPolicy'
        Properties:
          PolicyDocument:
            Statement:
              - Action: '*'
                Effect: Allow
                Resource: '*'
            Version: 2012-10-17
          Description: ''
          ManagedPolicyName: AdminPolicy
      ReadOnlyPolicy:
        Type: 'AWS::IAM::ManagedPolicy'
        Properties:
          PolicyDocument:
            Statement:
              - Action:
                  - 'cloudwatch:*'
                  - '*'
                Effect: Allow
                Resource: '*'
            Version: 2013-10-17
          Description: ''
          ManagedPolicyName: OperatorPolicy
    "#;
    let value = PathAwareValue::try_from(serde_yaml::from_str::<serde_yaml::Value>(value_str)?)?;
    let mut eval = BasicQueryTesting {
        root: Rc::new(value),
        recorder: None,
    };
    let status = eval_rule(&rule, &mut eval, ClauseRole::Assertion)?;
    assert_eq!(status, Status::FAIL);

    let value_str = r#"
    Resources:
      ReadOnlyAdminPolicy:
        Type: 'AWS::IAM::ManagedPolicy'
        Properties:
          PolicyDocument:
            Statement:
              - Action: '*'
                Effect: Allow
                Resource: '*'
            Version: 2012-10-17
          Description: ''
          ManagedPolicyName: AdminPolicy
    "#;
    let value = PathAwareValue::try_from(serde_yaml::from_str::<serde_yaml::Value>(value_str)?)?;
    let mut eval = BasicQueryTesting {
        root: Rc::new(value),
        recorder: None,
    };
    let status = eval_rule(&rule, &mut eval, ClauseRole::Assertion)?;
    assert_eq!(status, Status::SKIP);

    let value_str = r#"
    Resources: {}
    "#;
    let value = PathAwareValue::try_from(serde_yaml::from_str::<serde_yaml::Value>(value_str)?)?;
    let mut eval = BasicQueryTesting {
        root: Rc::new(value),
        recorder: None,
    };
    let status = eval_rule(&rule, &mut eval, ClauseRole::Assertion)?;
    assert_eq!(status, Status::SKIP);

    let value_str = r#"{}"#;
    let value = PathAwareValue::try_from(serde_yaml::from_str::<serde_yaml::Value>(value_str)?)?;
    let mut eval = BasicQueryTesting {
        root: Rc::new(value),
        recorder: None,
    };
    let status = eval_rule(&rule, &mut eval, ClauseRole::Assertion)?;
    assert_eq!(status, Status::FAIL);

    Ok(())
}

#[test]
fn test_multiple_valued_clause_reporting() -> Result<()> {
    struct ReportAssertions {}

    impl<'value> RecordTracer<'value> for ReportAssertions {
        fn start_record(&mut self, _context: &str) -> Result<()> {
            Ok(())
        }

        fn end_record(&mut self, _context: &str, record: RecordType<'value>) -> Result<()> {
            match record {
                RecordType::GuardClauseBlockCheck(BlockCheck {
                    message,
                    status,
                    at_least_one_matches,
                }) => {
                    assert_eq!(message, None);
                    assert_eq!(status, Status::FAIL);
                    assert!(!at_least_one_matches);
                }

                RecordType::ClauseValueCheck(ClauseCheck::Comparison(ComparisonClauseCheck {
                    status,
                    from,
                    to,
                    ..
                })) => {
                    assert!(to.is_some());
                    assert_eq!(status, Status::FAIL);
                    match from {
                        QueryResult::Resolved(res) => {
                            assert!(
                                res.self_path().0.as_str() == "/Resources/second/Properties/Name"
                                    || res.self_path().0.as_str()
                                        == "/Resources/failed/Properties/Name",
                            );
                        }

                        _ => unreachable!(),
                    }
                }

                RecordType::ClauseValueCheck(ClauseCheck::Success) => {}

                RecordType::RuleCheck(NamedStatus { name, status, .. }) => {
                    assert_eq!(name, "name_check");
                    assert_eq!(status, Status::FAIL);
                }

                RecordType::FileCheck(NamedStatus { status, .. }) => {
                    assert_eq!(status, Status::FAIL);
                }

                _ => unreachable!(),
            }
            Ok(())
        }
    }

    let rule = r###"
    rule name_check { Resources.*.Properties.Name == /NAME/ }
    "###;

    let value = r###"
    Resources:
      second:
        Properties:
          Name: FAILEDMatch
      first:
        Properties:
          Name: MatchNAME
      matches:
        Properties:
          Name: MatchNAME
      failed:
        Properties:
          Name: FAILEDMatch
    "###;

    let rules = Rule::try_from(rule)?;
    let values = PathAwareValue::try_from(serde_yaml::from_str::<serde_yaml::Value>(value)?)?;
    let mut asserter = ReportAssertions {};
    let mut root = BasicQueryTesting {
        root: Rc::new(values.clone()),
        recorder: Some(&mut asserter),
    };
    let status = eval_rule(&rules, &mut root, ClauseRole::Assertion)?;
    assert_eq!(status, Status::FAIL);

    let rule = r###"
    let resources = Resources.*
    rule name_check { %resources.Properties.Name == /NAME/ }
    "###;

    let rules = RulesFile::try_from(rule)?;
    let mut root = root_scope(&rules, Rc::new(values));
    let status = eval_rules_file(&rules, &mut root, None)?;
    assert_eq!(status, Status::FAIL);

    Ok(())
}

#[rstest::rstest]
#[case("SubdomainMaster", "Master.PrivateIp", Status::PASS)]
#[case("SubdomainInternal", "Master.PrivateIp", Status::PASS)]
#[case("SubdomainDefault", "Infra1.PrivateIp", Status::PASS)]
#[case("SubdomainDefault", "Infra1.PrivateIp", Status::PASS)]
#[case("Subdomain", "Infra1.PrivateIp", Status::FAIL)]
#[case("SubdomainDefault", "Infra1.PublicIp", Status::FAIL)]
#[case("Subdomain", "Master.PrivateIp", Status::FAIL)]
#[case("SubdomainDefault", "Master.PublicIp", Status::FAIL)]
fn test_in_comparison_operator_for_list_of_lists(
    #[case] name_arg: &str,
    #[case] resource_records_arg: &str,
    #[case] status_arg: Status,
) -> Result<()> {
    let template = formatdoc! {
        r###"
        Resources:
            MasterRecord:
                Type: AWS::Route53::RecordSet
                Properties:
                    HostedZoneName: !Ref 'HostedZoneName'
                    Comment: DNS name for my instance.
                    Name: !Join ['', [!Ref '{}', ., !Ref 'HostedZoneName']]
                    Type: A
                    TTL: "900"
                    ResourceRecords:
                    - !GetAtt '{}'"###,
        name_arg,
        resource_records_arg,
    };

    let rules = r#"
    let aws_route53_recordset_resources = Resources.*[ Type == 'AWS::Route53::RecordSet' ]
    rule aws_route53_recordset when %aws_route53_recordset_resources !empty {
      let targets = [{"Fn::Join": ["",[{"Ref": "SubdomainMaster"},".", {"Ref": "HostedZoneName"}]]}, {"Fn::Join": ["",[{"Ref": "SubdomainWild"},".", {"Ref": "HostedZoneName"}]]}, {"Fn::Join": ["",[{"Ref": 'SubdomainInternal'},".", {"Ref": "HostedZoneName"}]]}, {"Fn::Join": ["",[{"Ref": "SubdomainDefault"},".", {"Ref": "HostedZoneName"}]]}]
      %aws_route53_recordset_resources.Properties.Comment == "DNS name for my instance."
      %aws_route53_recordset_resources.Properties.ResourceRecords IN [[{"Fn::GetAtt": "Master.PrivateIp"}], [{"Fn::GetAtt": "Infra1.PrivateIp"}]]
      %aws_route53_recordset_resources.Properties.Name IN %targets
      %aws_route53_recordset_resources.Properties.Type == "A"
      %aws_route53_recordset_resources.Properties.TTL == "900"
      %aws_route53_recordset_resources.Properties.HostedZoneName == {"Ref": "HostedZoneName"}
    }
    "#;

    let value = PathAwareValue::try_from(serde_yaml::from_str::<serde_yaml::Value>(&template)?)?;
    let rule_eval = RulesFile::try_from(rules)?;
    let mut context = root_scope(&rule_eval, Rc::new(value));
    let status = eval_rules_file(&rule_eval, &mut context, None)?;
    assert_eq!(status, status_arg);

    Ok(())
}

#[rstest::rstest]
#[case(r#"'900'"#, Status::PASS)]
#[case(r#"!!str 900"#, Status::PASS)]
#[case(r#"900"#, Status::FAIL)]
#[case(r#"!!int "900""#, Status::FAIL)]
#[case(r#"!!float "900""#, Status::FAIL)]
fn test_type_conversions(#[case] ttl_arg: &str, #[case] status_arg: Status) -> Result<()> {
    let template = formatdoc! {
        r###"
        Resources:
            MasterRecord:
                Type: AWS::Route53::RecordSet
                Properties:
                    TTL: {}
                    "###,
        ttl_arg,
    };

    let rules = r#"
    let aws_route53_recordset_resources = Resources.*[ Type == 'AWS::Route53::RecordSet' ]
    rule aws_route53_recordset when %aws_route53_recordset_resources !empty {
      %aws_route53_recordset_resources.Properties.TTL == "900"
    }
    "#;

    let value = PathAwareValue::try_from(serde_yaml::from_str::<serde_yaml::Value>(&template)?)?;
    let rule_eval = RulesFile::try_from(rules)?;
    let mut context = root_scope(&rule_eval, Rc::new(value));
    let status = eval_rules_file(&rule_eval, &mut context, None)?;
    assert_eq!(status, status_arg);

    Ok(())
}

#[test]
fn is_bool() -> Result<()> {
    let rule_str = r###"
    rule check_is_bool{
        foo is_bool
    }
    "###;

    let resources_str = r###"
    {
        foo: false
    }
    "###;

    let value = PathAwareValue::try_from(resources_str)?;
    let rules_file = RulesFile::try_from(rule_str)?;
    println!("{:?}", rules_file);
    let mut eval = root_scope(&rules_file, Rc::new(value));
    let status = eval_rules_file(&rules_file, &mut eval, None)?;
    assert_eq!(status, Status::PASS);

    let resources_str = r#"
    {
        foo: "false"
    }
    "#;
    let value = PathAwareValue::try_from(resources_str)?;
    let mut eval = root_scope(&rules_file, Rc::new(value));
    let status = eval_rules_file(&rules_file, &mut eval, None)?;
    assert_eq!(status, Status::FAIL);

    Ok(())
}

#[test]
fn is_int() -> Result<()> {
    let rule_str = r###"
    rule check_is_int{
        foo is_int
    }
    "###;

    let resources_str = r###"
    {
        foo: 1
    }
    "###;

    let value = PathAwareValue::try_from(resources_str)?;
    let rules_file = RulesFile::try_from(rule_str)?;
    println!("{:?}", rules_file);
    let mut eval = root_scope(&rules_file, Rc::new(value));
    let status = eval_rules_file(&rules_file, &mut eval, None)?;
    assert_eq!(status, Status::PASS);

    let resources_str = r#"
    {
        foo: "1"
    }
    "#;
    let value = PathAwareValue::try_from(resources_str)?;
    let mut eval = root_scope(&rules_file, Rc::new(value));
    let status = eval_rules_file(&rules_file, &mut eval, None)?;
    assert_eq!(status, Status::FAIL);

    Ok(())
}

#[test]
fn double_projection_tests() -> Result<()> {
    let rule_str = r###"
    rule check_ecs_against_local_or_metadata {
        let ecs_tasks = Resources.*[
            Type == 'AWS::ECS::TaskDefinition'
            Properties.TaskRoleArn exists
        ]

        let iam_references = some %ecs_tasks.Properties.TaskRoleArn.'Fn::GetAtt'[0]
        when %iam_references !empty {
            let iam_local = Resources.%iam_references
            %iam_local.Type == 'AWS::IAM::Role'
            %iam_local.Properties.PermissionsBoundary exists
        }

        let ecs_task_role_is_string = %ecs_tasks[
            Properties.TaskRoleArn is_string
        ]
        when %ecs_task_role_is_string !empty {
            %ecs_task_role_is_string.Metadata.NotRestricted exists
        }
    }
    "###;

    let resources_str = r#"
    {
        Resources: {
            ecs: {
                Type: 'AWS::ECS::TaskDefinition',
                Metadata: {
                    NotRestricted: true
                },
                Properties: {
                    TaskRoleArn: "aws:arn..."
                }
            },
            ecs2: {
              Type: 'AWS::ECS::TaskDefinition',
              Properties: {
                TaskRoleArn: { 'Fn::GetAtt': ["iam", "arn"] }
              }
            },
            iam: {
              Type: 'AWS::IAM::Role',
              Properties: {
                PermissionsBoundary: "aws:arn"
              }
            }
        }
    }
    "#;

    let value = PathAwareValue::try_from(resources_str)?;
    let rules_file = RulesFile::try_from(rule_str)?;
    let mut eval = root_scope(&rules_file, Rc::new(value));
    let status = eval_rules_file(&rules_file, &mut eval, None)?;
    assert_eq!(status, Status::PASS);

    let resources_str = r#"
    {
        Resources: {
            ecs2: {
              Type: 'AWS::ECS::TaskDefinition',
              Properties: {
                TaskRoleArn: { 'Fn::GetAtt': ["iam", "arn"] }
              }
            }
        }
    }
    "#;
    let value = PathAwareValue::try_from(resources_str)?;
    let mut eval = root_scope(&rules_file, Rc::new(value));
    let status = eval_rules_file(&rules_file, &mut eval, None)?;
    assert_eq!(status, Status::FAIL);

    Ok(())
}

#[test]
fn test_rules_with_some_clauses() -> Result<()> {
    let query = r#"let x = some Resources.*[ Type == 'AWS::IAM::Role' ].Properties.Tags[ Key == /[A-Za-z0-9]+Role/ ]"#;
    let resources = r#"    {
      "Resources": {
          "CounterTaskDefExecutionRole5959CB2D": {
              "Type": "AWS::IAM::Role",
              "Properties": {
                  "AssumeRolePolicyDocument": {
                      "Statement": [
                      {
                          "Action": "sts:AssumeRole",
                          "Effect": "Allow",
                          "Principal": {
                          "Service": "ecs-tasks.amazonaws.com"
                          }
                      }],
                      "Version": "2012-10-17"
                  },
                  "PermissionsBoundary": {"Fn::Sub" : "arn::aws::iam::${AWS::AccountId}:policy/my-permission-boundary"},
                  "Tags": [{ "Key": "TestRole", "Value": ""}]
              },
              "Metadata": {
                  "aws:cdk:path": "foo/Counter/TaskDef/ExecutionRole/Resource"
              }
          },
          "BlankRole001": {
              "Type": "AWS::IAM::Role",
              "Properties": {
                  "AssumeRolePolicyDocument": {
                      "Statement": [
                      {
                          "Action": "sts:AssumeRole",
                          "Effect": "Allow",
                          "Principal": {
                          "Service": "ecs-tasks.amazonaws.com"
                          }
                      }],
                      "Version": "2012-10-17"
                  },
                  "Tags": [{ "Key": "FooBar", "Value": ""}]
              },
              "Metadata": {
                  "aws:cdk:path": "foo/Counter/TaskDef/ExecutionRole/Resource"
              }
          },
          "BlankRole002": {
              "Type": "AWS::IAM::Role",
              "Properties": {
                  "AssumeRolePolicyDocument": {
                      "Statement": [
                      {
                          "Action": "sts:AssumeRole",
                          "Effect": "Allow",
                          "Principal": {
                          "Service": "ecs-tasks.amazonaws.com"
                          }
                      }],
                      "Version": "2012-10-17"
                  }
              },
              "Metadata": {
                  "aws:cdk:path": "foo/Counter/TaskDef/ExecutionRole/Resource"
              }
          }
      }
    }
    "#;
    let value = PathAwareValue::try_from(resources)?;
    let parsed = RulesFile::try_from(query)?;
    let mut eval = root_scope(&parsed, Rc::new(value));
    let selected = eval.resolve_variable("x")?;
    println!("{:?}", selected);
    assert_eq!(selected.len(), 1);

    Ok(())
}

#[test]
fn test_support_for_atleast_one_match_clause() -> Result<()> {
    let clause_some_str = r#"some Tags[*].Key == /PROD/"#;
    let clause_some = GuardClause::try_from(clause_some_str)?;

    let clause_str = r#"Tags[*].Key == /PROD/"#;
    let clause = GuardClause::try_from(clause_str)?;

    let values_str = r#"{
        Tags: [
            {
                Key: "InPROD",
                Value: "ProdApp"
            },
            {
                Key: "NoP",
                Value: "NoQ"
            }
        ]
    }
    "#;
    let values = PathAwareValue::try_from(values_str)?;
    let mut eval = BasicQueryTesting {
        root: Rc::new(values),
        recorder: None,
    };

    let status = eval_guard_clause(&clause_some, &mut eval, ClauseRole::Assertion)?;
    assert_eq!(status, Status::PASS);

    let status = eval_guard_clause(&clause, &mut eval, ClauseRole::Assertion)?;
    assert_eq!(status, Status::FAIL);

    let values_str = r#"{ Tags: [] }"#;
    let values = PathAwareValue::try_from(values_str)?;
    let mut eval = BasicQueryTesting {
        root: Rc::new(values),
        recorder: None,
    };
    let status = eval_guard_clause(&clause_some, &mut eval, ClauseRole::Assertion)?;
    assert_eq!(status, Status::FAIL);

    let status = eval_guard_clause(&clause, &mut eval, ClauseRole::Assertion)?;
    assert_eq!(status, Status::FAIL);

    let values_str = r#"{ }"#;
    let values = PathAwareValue::try_from(values_str)?;
    let mut eval = BasicQueryTesting {
        root: Rc::new(values.clone()),
        recorder: None,
    };
    let status = eval_guard_clause(&clause_some, &mut eval, ClauseRole::Assertion)?;
    assert_eq!(status, Status::FAIL);

    let status = eval_guard_clause(&clause, &mut eval, ClauseRole::Assertion)?;
    assert_eq!(status, Status::FAIL);

    //
    // Trying out the selection filters
    //
    let selection_str = r#"Resources[
        Type == 'AWS::DynamoDB::Table'
        some Properties.Tags[*].Key == /PROD/
    ]"#;
    let resources_str = r#"{
        Resources: {
            ddbSelected: {
                Type: 'AWS::DynamoDB::Table',
                Properties: {
                    Tags: [
                        {
                            Key: "PROD",
                            Value: "ProdApp"
                        }
                    ]
                }
            },
            ddbNotSelected: {
                Type: 'AWS::DynamoDB::Table'
            }
        }
    }"#;
    let _resources = PathAwareValue::try_from(resources_str)?;
    let selection_query = AccessQuery::try_from(selection_str)?;
    let mut eval = BasicQueryTesting {
        root: Rc::new(values),
        recorder: None,
    };
    let selected = eval.query(&selection_query.query)?;
    println!("Selected = {:?}", selected);
    assert_eq!(selected.len(), 1);

    Ok(())
}

#[test]
fn test_map_keys_function() -> Result<()> {
    let value_str = r#"
    Resources:
      apiGw:
        Type: 'AWS::ApiGateway::RestApi'
        Properties:
          EndpointConfiguration: ["PRIVATE"]
          Policy:
            Statement:
              - Action: Allow
                Resource: ['*', "aws:"]
                Condition:
                    'aws:IsSecure': true

    "#;
    let value = serde_yaml::from_str::<serde_yaml::Value>(value_str)?;
    let value = PathAwareValue::try_from(value)?;

    let rule_str = r#"
let api_gw = Resources[ Type == 'AWS::ApiGateway::RestApi' ]
rule check_rest_api_is_private_and_has_access {
    %api_gw {
      Properties.EndpointConfiguration == ["PRIVATE"]
      some Properties.Policy.Statement[*].Condition[ keys == /aws:[sS]ource(Vpc|VPC|Vpce|VPCE)/ ] !empty
    }
}"#;
    let rule = RulesFile::try_from(rule_str)?;
    let mut root = root_scope(&rule, Rc::new(value));
    let status = eval_rules_file(&rule, &mut root, None)?;
    assert_eq!(status, Status::FAIL);

    let value_str = r#"
    Resources:
      apiGw:
        Type: 'AWS::ApiGateway::RestApi'
        Properties:
          EndpointConfiguration: ["PRIVATE"]
          Policy:
            Statement:
              - Action: Allow
                Resource: ['*', "aws:"]
                Condition:
                    'aws:IsSecure': true
                    'aws:sourceVpc': ['vpc-1234']

    "#;
    let value = serde_yaml::from_str::<serde_yaml::Value>(value_str)?;
    let value = PathAwareValue::try_from(value)?;
    let mut root = root_scope(&rule, Rc::new(value));
    let status = eval_rules_file(&rule, &mut root, None)?;
    assert_eq!(status, Status::PASS);

    Ok(())
}

#[test]
fn ensure_all_list_value_access_on_empty_fails() -> Result<()> {
    let resources = r#"Tags: []"#;
    let values = PathAwareValue::try_from(serde_yaml::from_str::<serde_yaml::Value>(resources)?)?;
    let claused_failure_spec = GuardClause::try_from(r#"Tags[*].Key == /Name/"#)?;
    let mut eval = BasicQueryTesting {
        root: Rc::new(values.clone()),
        recorder: None,
    };
    let status = eval_guard_clause(&claused_failure_spec, &mut eval, ClauseRole::Assertion)?;
    assert_eq!(status, Status::FAIL);

    let claused_failure_spec = GuardClause::try_from(r#"some Tags[*].Key == /Name/"#)?;
    let mut eval = BasicQueryTesting {
        root: Rc::new(values.clone()),
        recorder: None,
    };
    let status = eval_guard_clause(&claused_failure_spec, &mut eval, ClauseRole::Assertion)?;
    assert_eq!(status, Status::FAIL);

    let claused_failure_spec = GuardClause::try_from(r#"Tags[*] { Key == /Name/ }"#)?;
    let mut eval = BasicQueryTesting {
        root: Rc::new(values.clone()),
        recorder: None,
    };
    let status = eval_guard_clause(&claused_failure_spec, &mut eval, ClauseRole::Assertion)?;
    assert_eq!(status, Status::FAIL);

    let claused_failure_spec = GuardClause::try_from(r#"some Tags[*] { Key == /Name/ }"#)?;
    let mut eval = BasicQueryTesting {
        root: Rc::new(values.clone()),
        recorder: None,
    };
    let status = eval_guard_clause(&claused_failure_spec, &mut eval, ClauseRole::Assertion)?;
    assert_eq!(status, Status::FAIL);

    let claused_failure_spec = GuardClause::try_from(r#"Tags !empty"#)?;
    let mut eval = BasicQueryTesting {
        root: Rc::new(values.clone()),
        recorder: None,
    };
    let status = eval_guard_clause(&claused_failure_spec, &mut eval, ClauseRole::Assertion)?;
    assert_eq!(status, Status::FAIL);

    let claused_failure_spec = GuardClause::try_from(r#"Tags empty"#)?;
    let mut eval = BasicQueryTesting {
        root: Rc::new(values.clone()),
        recorder: None,
    };
    let status = eval_guard_clause(&claused_failure_spec, &mut eval, ClauseRole::Assertion)?;
    assert_eq!(status, Status::PASS);

    let claused_failure_spec = GuardClause::try_from(r#"Tags[*] !empty"#)?;
    let mut eval = BasicQueryTesting {
        root: Rc::new(values.clone()),
        recorder: None,
    };
    let status = eval_guard_clause(&claused_failure_spec, &mut eval, ClauseRole::Assertion)?;
    assert_eq!(status, Status::FAIL);

    let claused_failure_spec = GuardClause::try_from(r#"Tags[*] empty"#)?;
    let mut eval = BasicQueryTesting {
        root: Rc::new(values),
        recorder: None,
    };
    let status = eval_guard_clause(&claused_failure_spec, &mut eval, ClauseRole::Assertion)?;
    assert_eq!(status, Status::PASS);

    Ok(())
}

#[test]
fn ensure_all_map_values_access_on_empty_fails() -> Result<()> {
    let resources = r#"Resources: {}"#;
    let values = PathAwareValue::try_from(serde_yaml::from_str::<serde_yaml::Value>(resources)?)?;
    let mut eval = BasicQueryTesting {
        root: Rc::new(values.clone()),
        recorder: None,
    };

    let clause_failure_spec = GuardClause::try_from(r#"Resources.*.Properties exists"#)?;
    let status = eval_guard_clause(&clause_failure_spec, &mut eval, ClauseRole::Assertion)?;
    assert_eq!(status, Status::FAIL);

    let clause_failure_spec = GuardClause::try_from(r#"Resources.* { Properties exists }"#)?;
    let status = eval_guard_clause(&clause_failure_spec, &mut eval, ClauseRole::Assertion)?;
    assert_eq!(status, Status::FAIL);

    let clause_failure_spec = GuardClause::try_from(r#"Resources exists"#)?;
    let status = eval_guard_clause(&clause_failure_spec, &mut eval, ClauseRole::Assertion)?;
    assert_eq!(status, Status::PASS);

    //
    // Resources is empty, hence FAIL
    //
    let clause_failure_spec =
        GuardClause::try_from(r#"Resources[ Type == /Bucket/ ].Properties exists"#)?;
    let status = eval_guard_clause(&clause_failure_spec, &mut eval, ClauseRole::Assertion)?;
    assert_eq!(status, Status::SKIP);

    //
    // Resource present filter did not select, SKIP
    //
    let resources = r#"
    Resources:
      ec2:
        Type: AWS::EC2::Instance
        Properties:
          ImageId: ami-1234554657
    "#;
    let _value = PathAwareValue::try_from(serde_yaml::from_str::<serde_yaml::Value>(resources)?)?;
    let mut eval = BasicQueryTesting {
        root: Rc::new(values),
        recorder: None,
    };
    let status = eval_guard_clause(&clause_failure_spec, &mut eval, ClauseRole::Assertion)?;
    assert_eq!(status, Status::SKIP);

    //
    // No resources present
    //
    let resources = r#"{}"#;
    let values = PathAwareValue::try_from(serde_yaml::from_str::<serde_yaml::Value>(resources)?)?;
    let mut eval = BasicQueryTesting {
        root: Rc::new(values),
        recorder: None,
    };
    let clause_failure_spec = GuardClause::try_from(r#"Resources exists"#)?;
    let status = eval_guard_clause(&clause_failure_spec, &mut eval, ClauseRole::Assertion)?;
    assert_eq!(status, Status::FAIL);

    Ok(())
}

fn find_failed_clauses<'event, 'value>(
    root: &'event EventRecord<'value>,
) -> Vec<&'event EventRecord<'value>> {
    match &root.container {
        Some(RecordType::Filter(_)) | Some(RecordType::ClauseValueCheck(ClauseCheck::Success)) => {
            vec![]
        }

        Some(RecordType::ClauseValueCheck(_)) => vec![root],

        _ => {
            let mut acc = Vec::new();
            for child in &root.children {
                acc.extend(find_failed_clauses(child));
            }
            acc
        }
    }
}

#[test]
fn filter_based_join_clauses_failures_and_skips() -> Result<()> {
    let resources = r#"
    Resources:
      function:
        Type: AWS::Lambda::Function
        Properties:
          Code: ''
          Role:
            Ref: iam
      function2:
        Type: AWS::Lambda::Function
        Properties:
          Code: ''
          Role: aws:arn
      iam:
        Type: AWS::IAM::Role
        Properties:
          PolicyDocument:
            Statement:
              Action: '*'
              Resource: '*'
              Effect: Allow
              Principal: ['*']
      iam2:
        Type: AWS::IAM::Role
        Properties:
          PolicyDocument:
            Statement:
              Action: '*'
              Resource: '*'
              Effect: Allow
              Principal: '*'
    "#;

    let rules = r###"
    rule ensure_lambda_role_local_stack {
      let with_refs = some Resources[ Type == 'AWS::Lambda::Function' ].Properties.Role.Ref
      Resources.%with_refs {
         Type == 'AWS::IAM::Role'
         Properties.PolicyDocument.Statement[*] {
           Action != '*'
           Principal != '*'
         }
      }
    }
    "###;

    let rules_file = RulesFile::try_from(rules)?;
    let path_value =
        PathAwareValue::try_from(serde_yaml::from_str::<serde_yaml::Value>(resources)?)?;
    let mut eval = root_scope(&rules_file, Rc::new(path_value));
    let status = eval_rules_file(&rules_file, &mut eval, None)?;
    assert_eq!(status, Status::FAIL);

    let top = eval.reset_recorder().extract();
    let failed_clauses = find_failed_clauses(&top);
    assert_eq!(failed_clauses.len(), 2);
    for each in failed_clauses {
        if let Some(RecordType::ClauseValueCheck(check)) = &each.container {
            match check {
                ClauseCheck::Comparison(ComparisonClauseCheck { status, from, .. }) => {
                    assert_eq!(*status, Status::FAIL);
                    assert!(each.context.contains("Action") || each.context.contains("Principal"),);
                    assert!(from.resolved().map_or(false, |res| {
                        let path = res.self_path().0.as_str();
                        path == "/Resources/iam/Properties/PolicyDocument/Statement/Action"
                            || path
                                == "/Resources/iam/Properties/PolicyDocument/Statement/Principal/0"
                    }))
                }

                _ => unreachable!(),
            }
        }
    }

    //
    // No Lambda resources present, expect SKIP, same rules file
    //

    let resources = r#"
    Resources:
      iam:
        Type: AWS::IAM::Role
        Properties:
          PolicyDocument:
            Statement:
              Action: '*'
              Resource: '*'
              Effect: Allow
              Principal: ['*']
      iam2:
        Type: AWS::IAM::Role
        Properties:
          PolicyDocument:
            Statement:
              Action: '*'
              Resource: '*'
              Effect: Allow
              Principal: '*'
    "#;
    let path_value =
        PathAwareValue::try_from(serde_yaml::from_str::<serde_yaml::Value>(resources)?)?;
    let mut eval = root_scope(&rules_file, Rc::new(path_value));
    let status = eval_rules_file(&rules_file, &mut eval, None)?;
    assert_eq!(status, Status::SKIP);

    //
    // Lambda resources not connected IAM, expect skip
    //
    let resources = r#"
    Resources:
      function2:
        Type: AWS::Lambda::Function
        Properties:
          Code: ''
          Role: aws:arn
      iam:
        Type: AWS::IAM::Role
        Properties:
          PolicyDocument:
            Statement:
              Action: '*'
              Resource: '*'
              Effect: Allow
              Principal: ['*']
    "#;
    let path_value =
        PathAwareValue::try_from(serde_yaml::from_str::<serde_yaml::Value>(resources)?)?;
    let mut eval = eval.reset_root(Rc::new(path_value));
    let status = eval_rules_file(&rules_file, &mut eval, None)?;
    assert_eq!(status, Status::SKIP);

    //
    // Lambda resource present, but have dangling reference
    //

    let resources = r###"
    Resources:
      function:
        Type: AWS::Lambda::Function
        Properties:
          Code: ''
          Role:
            Ref: iamNotThere # dangling reference
      iam:
        Type: AWS::IAM::Role
        Properties:
          PolicyDocument:
            Statement:
              Action: '*'
              Resource: '*'
              Effect: Allow
              Principal: ['*']
    "###;
    let path_value =
        PathAwareValue::try_from(serde_yaml::from_str::<serde_yaml::Value>(resources)?)?;

    let mut eval = eval.reset_root(Rc::new(path_value));

    //
    // Let us track failures and assert on what must be observed
    //
    let status = eval_rules_file(&rules_file, &mut eval, None)?;
    assert_eq!(status, Status::FAIL);

    let top = eval.reset_recorder().extract();
    let failed_clauses = find_failed_clauses(&top);
    assert_eq!(failed_clauses.len(), 1);
    match &failed_clauses[0].container {
        Some(RecordType::ClauseValueCheck(ClauseCheck::MissingBlockValue(check))) => {
            assert_eq!(check.status, Status::FAIL);
            assert_eq!(check.from.resolved(), None);
        }
        _ => unreachable!(),
    }

    Ok(())
}

#[test]
fn filter_based_with_join_pass_use_cases() -> Result<()> {
    let resources = r#"
    Resources:
      function:
        Type: AWS::Lambda::Function
        Properties:
          Code: ''
          Role:
            Ref: iam
      function2:
        Type: AWS::Lambda::Function
        Properties:
          Code: ''
          Role: aws:arn
      iam:
        Type: AWS::IAM::Role
        Properties:
          PolicyDocument:
            Statement:
              Action: '*'
              Resource: '*'
              Effect: Allow
              Principal: ['*']
      iam2:
        Type: AWS::IAM::Role
        Properties:
          PolicyDocument:
            Statement:
              Action: '*'
              Resource: '*'
              Effect: Allow
              Principal: '*'
    "#;

    let rules = r###"
    rule ensure_lambda_role_local_stack {
      let with_refs = some Resources[ Type == 'AWS::Lambda::Function' ].Properties.Role.Ref
      Resources.%with_refs {
         Type == 'AWS::IAM::Role'
         Properties.PolicyDocument.Statement[*] {
           Action == '*'
           Principal == '*'
         }
      }
    }
    "###;

    let path_value =
        PathAwareValue::try_from(serde_yaml::from_str::<serde_yaml::Value>(resources)?)?;
    let rules_file = RulesFile::try_from(rules)?;
    let mut eval = root_scope(&rules_file, Rc::new(path_value));
    let status = eval_rules_file(&rules_file, &mut eval, None)?;
    assert_eq!(status, Status::PASS);

    Ok(())
}

#[test]
fn rule_clause_tests() -> Result<()> {
    let r = r###"
    rule check_all_resources_have_tags_present {
    let all_resources = Resources.*.Properties

    %all_resources.Tags EXISTS
    %all_resources.Tags !EMPTY
}
    "###;
    let rule = RulesFile::try_from(r)?;

    let v = r#"
    {
        "Resources": {
            "vpc": {
                "Type": "AWS::EC2::VPC",
                "Properties": {
                    "CidrBlock": "10.0.0.0/25",
                    "Tags": [
                        {
                            "Key": "my-vpc",
                            "Value": "my-vpc"
                        }
                    ]
                }
            }
        }
    }
    "#;

    let value = PathAwareValue::try_from(v)?;
    let mut eval = root_scope(&rule, Rc::new(value));
    let status = eval_rules_file(&rule, &mut eval, None)?;
    assert_eq!(Status::PASS, status);

    //
    // Tags Empty, FAIL
    //
    let v = r#"
    {
        "Resources": {
            "vpc": {
                "Type": "AWS::EC2::VPC",
                "Properties": {
                    "CidrBlock": "10.0.0.0/25",
                    "Tags": []
                }
            }
        }
    }
    "#;

    let value = PathAwareValue::try_from(v)?;
    let mut eval = eval.reset_root(Rc::new(value));
    let status = eval_rules_file(&rule, &mut eval, None)?;
    assert_eq!(Status::FAIL, status);

    Ok(())
}

#[test]
fn rule_test_type_blocks() -> Result<()> {
    let r = r"
    rule iam_basic_checks {
  AWS::IAM::Role {
    Properties.AssumeRolePolicyDocument.Version == /(\d{4})-(\d{2})-(\d{2})/
    Properties.PermissionsBoundary == /arn:aws:iam::(\d{12}):policy/
    Properties.Tags[*].Value == /[a-zA-Z0-9]+/
    Properties.Tags[*].Key   == /[a-zA-Z0-9]+/
  }
}";

    let value = r#"
    {
        "Resources": {
            "iamrole": {
                "Type": "AWS::IAM::Role",
                "Properties": {
                    "PermissionsBoundary": "arn:aws:iam::123456789012:policy/permboundary",
                    "AssumeRolePolicyDocument": {
                        "Version": "2021-01-10",
                        "Statement": {
                            "Effect": "Allow",
                            "Principal": "*",
                            "Action": "*",
                            "Resource": "*"
                        }
                    }
                }
            },
            "iamRole2": {
              "Type": "AWS::IAM::Role",
              "Properties": {
                  "PermissionsBoundary": "arn:aws:iam::123456789112:policy/permboundary",
                  "AssumeRolePolicyDocument": {
                      "Version": "2021-01-10",
                      "Statement": {
                          "Effect": "Allow",
                          "Principal": "*",
                          "Action": "*",
                          "Resource": "*"
                      }
                  },
                  "Tags": [
                    { "Key": "Key", "Value": "Value" }
                  ]
              }
            }
        }
    }
    "#;

    let root = PathAwareValue::try_from(serde_yaml::from_str::<serde_yaml::Value>(value)?)?;
    let rules_file = RulesFile::try_from(r)?;
    let mut root_context = root_scope(&rules_file, Rc::new(root));
    let status = eval_rules_file(&rules_file, &mut root_context, None)?;
    assert_eq!(Status::FAIL, status);

    let top = root_context.reset_recorder().extract();
    let failed_clause = find_failed_clauses(&top);
    assert_eq!(failed_clause.len(), 2); // For Tag's key and value check for first resource
    for each in failed_clause {
        match &each.container {
            Some(RecordType::ClauseValueCheck(ClauseCheck::Comparison(
                ComparisonClauseCheck {
                    from, status, to, ..
                },
            ))) => {
                assert_eq!(*status, Status::FAIL);
                assert_eq!(from.resolved(), None);
                assert_eq!(*to, None);
            }

            _ => unreachable!(),
        }
    }

    Ok(())
}

#[test]
fn rules_file_tests_the_unituitive_all_clause_that_skips() -> Result<()> {
    let file = r#"
let iam_resources = Resources.*[ Type == "AWS::IAM::Role" ]
rule iam_resources_exists {
    %iam_resources !EMPTY
}

rule iam_basic_checks when iam_resources_exists {
    %iam_resources.Properties.AssumeRolePolicyDocument.Version == /(\d{4})-(\d{2})-(\d{2})/
    %iam_resources.Properties.PermissionsBoundary == /arn:aws:iam::(\d{12}):policy/
    #
    # This check FAILs as it tests using a conjoined call. It is testing that ALL
    # IAM resources have non empty Tags. This FAILs as "iamrole" does not have Tags
    # property specified. Therefore this check overall leads to PASS, which is the
    # correct outcome as specified. See next test on the right way to use this
    #
    when %iam_resources.Properties.Tags EXISTS
         %iam_resources.Properties.Tags !EMPTY {

        %iam_resources.Properties.Tags[*].Value == /[a-zA-Z0-9]+/
        %iam_resources.Properties.Tags[*].Key   == /[a-zA-Z0-9]+/
    }
}"#;

    let value = r#"
    {
        "Resources": {
            "iamrole": {
                "Type": "AWS::IAM::Role",
                "Properties": {
                    "PermissionsBoundary": "arn:aws:iam::123456789012:policy/permboundary",
                    "AssumeRolePolicyDocument": {
                        "Version": "2021-01-10",
                        "Statement": {
                            "Effect": "Allow",
                            "Principal": "*",
                            "Action": "*",
                            "Resource": "*"
                        }
                    }
                }
            },
            "iamRole2": {
              "Type": "AWS::IAM::Role",
              "Properties": {
                  "PermissionsBoundary": "arn:aws:iam::123456789112:policy/permboundary",
                  "AssumeRolePolicyDocument": {
                      "Version": "2021-01-10",
                      "Statement": {
                          "Effect": "Allow",
                          "Principal": "*",
                          "Action": "*",
                          "Resource": "*"
                      }
                  },
                  "Tags": [
                    { "Key": "Key", "Value": "Value" }
                  ]
              }
            }
        }
    }
    "#;

    let root = PathAwareValue::try_from(value)?;
    let rules_file = RulesFile::try_from(file)?;
    let mut root_context = root_scope(&rules_file, Rc::new(root));
    let status = eval_rules_file(&rules_file, &mut root_context, None)?;
    assert_eq!(Status::PASS, status);

    Ok(())
}

#[test]
fn rules_file_tests_simpler_correct_form_using_newer_constructs() -> Result<()> {
    let file = r"
rule iam_basic_checks {
    Resources[ Type == 'AWS::IAM::Role' ] {
        Properties {
            AssumeRolePolicyDocument.Version == /(\d{4})-(\d{2})-(\d{2})/
            PermissionsBoundary == /arn:aws:iam::(\d{12}):policy/
            Tags[*] {
                Key   == /[a-zA-Z0-9]+/
                Value == /[a-zA-Z0-9]+/
            }
        }
    }
}";

    //
    // Missing Tag properties
    //
    let value = r#"
    {
        "Resources": {
            "iamrole": {
                "Type": "AWS::IAM::Role",
                "Properties": {
                    "PermissionsBoundary": "arn:aws:iam::123456789012:policy/permboundary",
                    "AssumeRolePolicyDocument": {
                        "Version": "2021-01-10",
                        "Statement": {
                            "Effect": "Allow",
                            "Principal": "*",
                            "Action": "*",
                            "Resource": "*"
                        }
                    }
                }
            },
            "iamRole2": {
              "Type": "AWS::IAM::Role",
              "Properties": {
                  "PermissionsBoundary": "arn:aws:iam::123456789112:policy/permboundary",
                  "AssumeRolePolicyDocument": {
                      "Version": "2021-01-10",
                      "Statement": {
                          "Effect": "Allow",
                          "Principal": "*",
                          "Action": "*",
                          "Resource": "*"
                      }
                  },
                  "Tags": [
                    { "Key": "Key", "Value": "Value" }
                  ]
              }
            }
        }
    }
    "#;

    let root = PathAwareValue::try_from(value)?;
    let rules_file = RulesFile::try_from(file)?;
    let mut root_context = root_scope(&rules_file, Rc::new(root));

    let status = eval_rules_file(&rules_file, &mut root_context, None)?;
    assert_eq!(Status::FAIL, status);

    let top = root_context.reset_recorder().extract();
    let failed_clause = find_failed_clauses(&top);
    assert_eq!(failed_clause.len(), 1); // There is only one for Tag[*] block
    for each in failed_clause {
        match &each.container {
            Some(RecordType::ClauseValueCheck(ClauseCheck::MissingBlockValue(ValueCheck {
                status,
                from,
                ..
            }))) => {
                assert_eq!(*status, Status::FAIL);
                assert_eq!(from.resolved(), None);
            }

            _ => unreachable!(),
        }
    }

    //
    // Empty Tag properties
    //
    let value = r#"
    {
        "Resources": {
            "iamrole": {
                "Type": "AWS::IAM::Role",
                "Properties": {
                    "PermissionsBoundary": "arn:aws:iam::123456789012:policy/permboundary",
                    "AssumeRolePolicyDocument": {
                        "Version": "2021-01-10",
                        "Statement": {
                            "Effect": "Allow",
                            "Principal": "*",
                            "Action": "*",
                            "Resource": "*"
                        }
                    },
                    Tags: []
                }
            },
            "iamRole2": {
              "Type": "AWS::IAM::Role",
              "Properties": {
                  "PermissionsBoundary": "arn:aws:iam::123456789112:policy/permboundary",
                  "AssumeRolePolicyDocument": {
                      "Version": "2021-01-10",
                      "Statement": {
                          "Effect": "Allow",
                          "Principal": "*",
                          "Action": "*",
                          "Resource": "*"
                      }
                  },
                  "Tags": [
                    { "Key": "Key", "Value": "Value" }
                  ]
              }
            }
        }
    }
    "#;

    let root = PathAwareValue::try_from(value)?;
    let mut root_context = root_context.reset_root(Rc::new(root));
    let status = eval_rules_file(&rules_file, &mut root_context, None)?;
    assert_eq!(Status::FAIL, status);

    let top = root_context.reset_recorder().extract();
    let failed_clause = find_failed_clauses(&top);
    assert_eq!(failed_clause.len(), 1); // There is only one for Tag[*] block
    for each in failed_clause {
        match &each.container {
            Some(RecordType::ClauseValueCheck(ClauseCheck::MissingBlockValue(ValueCheck {
                status,
                from,
                ..
            }))) => {
                assert_eq!(*status, Status::FAIL);
                assert_eq!(from.resolved(), None);
                match from.unresolved_traversed_to() {
                    Some(val) => {
                        assert_eq!(
                            val.self_path().0.as_str(),
                            "/Resources/iamrole/Properties/Tags"
                        );
                    }
                    None => unreachable!(),
                }
            }

            _ => unreachable!(),
        }
    }

    Ok(())
}

const SAMPLE: &str = r#"
    {
        "Statement": [
            {
                "Sid": "PrincipalPutObjectIfIpAddress",
                "Effect": "Allow",
                "Action": "s3:PutObject",
                "Resource": "arn:aws:s3:::my-service-bucket/*",
                "Condition": {
                    "Bool": {"aws:ViaAWSService": "false"}
                }
            },
            {
                "Sid": "ServicePutObject",
                "Effect": "Allow",
                "Action": "s3:PutObject",
                "Resource": "arn:aws:s3:::my-service-bucket/*",
                "Condition": {
                    "Bool": {"aws:ViaAWSService": "true"}
                }
            }
        ]
    }
    "#;

#[test]
fn test_iam_statement_clauses() -> Result<()> {
    let sample = r#"
    {
        "Statement": [
            {
                "Sid": "PrincipalPutObjectIfIpAddress",
                "Effect": "Allow",
                "Action": "s3:PutObject",
                "Resource": "arn:aws:s3:::my-service-bucket/*",
                "Condition": {
                    "Bool": {"aws:ViaAWSService": "false"},
                    "StringEquals": {"aws:SourceVpc": "vpc-12243sc"}
                }
            },
            {
                "Sid": "ServicePutObject",
                "Effect": "Allow",
                "Action": "s3:PutObject",
                "Resource": "arn:aws:s3:::my-service-bucket/*",
                "Condition": {
                    "Bool": {"aws:ViaAWSService": "true"}
                }
            }
        ]
    }
    "#;
    let values = PathAwareValue::try_from(sample)?;
    let mut eval = BasicQueryTesting {
        root: Rc::new(values),
        recorder: None,
    };

    let clause = r#"Statement[
        Condition EXISTS ].Condition.*[
            this is_struct ][ KEYS == /aws:[sS]ource(Vpc|VPC|Vpce|VPCE)/ ] NOT EMPTY"#;
    // let clause = "Condition.*[ KEYS == /aws:[sS]ource(Vpc|VPC|Vpce|VPCE)/ ]";
    let parsed = GuardClause::try_from(clause)?;
    let status = eval_guard_clause(&parsed, &mut eval, ClauseRole::Assertion)?;
    assert_eq!(Status::PASS, status);

    let clause = r#"Statement[ Condition EXISTS
                                     Condition.*[ KEYS == /aws:[sS]ource(Vpc|VPC|Vpce|VPCE)/ ] !EMPTY ] NOT EMPTY
    "#;
    let parsed = GuardClause::try_from(clause)?;
    let status = eval_guard_clause(&parsed, &mut eval, ClauseRole::Assertion)?;
    assert_eq!(Status::PASS, status);

    let parsed = GuardClause::try_from(
        r#"SOME Statement[*].Condition.*[ THIS IS_STRUCT ][ KEYS ==  /aws:[sS]ource(Vpc|VPC|Vpce|VPCE)/ ] NOT EMPTY"#,
    )?;
    let status = eval_guard_clause(&parsed, &mut eval, ClauseRole::Assertion)?;
    assert_eq!(Status::PASS, status);

    let sample = r#"
    {
        "Statement": [
            {
                "Sid": "PrincipalPutObjectIfIpAddress",
                "Effect": "Allow",
                "Action": "s3:PutObject"
            }
        ]
    }"#;
    let value = PathAwareValue::try_from(sample)?;
    let mut eval = BasicQueryTesting {
        root: Rc::new(value),
        recorder: None,
    };
    let status = eval_guard_clause(&parsed, &mut eval, ClauseRole::Assertion)?;
    assert_eq!(status, Status::FAIL);

    let sample = r#"
    {
        "Statement": [
            {
                "Sid": "PrincipalPutObjectIfIpAddress",
                "Effect": "Allow",
                "Action": "s3:PutObject",
                "Condition": {
                    "array": [1, 3, 4]
                }
            }
        ]
    }"#;
    let value = PathAwareValue::try_from(sample)?;
    let mut eval = BasicQueryTesting {
        root: Rc::new(value),
        recorder: None,
    };
    let status = eval_guard_clause(&parsed, &mut eval, ClauseRole::Assertion)?;
    assert_eq!(status, Status::FAIL);

    let sample = r#"
    {
        "Statement": [
            {
                "Sid": "PrincipalPutObjectIfIpAddress",
                "Effect": "Allow",
                "Action": "s3:PutObject",
                "Condition": {
                    "array": [1, 3, 4],
                    "StringEquals": {"aws:SourceVpc": "vpc-12243sc"}
                }
            }
        ]
    }"#;
    let value = PathAwareValue::try_from(sample)?;
    let mut eval = BasicQueryTesting {
        root: Rc::new(value),
        recorder: None,
    };
    let status = eval_guard_clause(&parsed, &mut eval, ClauseRole::Assertion)?;
    assert_eq!(status, Status::PASS);

    let value = PathAwareValue::try_from(SAMPLE)?;
    let parsed = GuardClause::try_from(clause)?;
    let mut eval = BasicQueryTesting {
        root: Rc::new(value),
        recorder: None,
    };
    let status = eval_guard_clause(&parsed, &mut eval, ClauseRole::Assertion)?;
    assert_eq!(Status::FAIL, status);

    Ok(())
}

#[test]
fn test_api_gateway() -> Result<()> {
    let rule = r#"
rule check_rest_api_private {
  AWS::ApiGateway::RestApi {
    # Endpoint configuration must only be private
    Properties.EndpointConfiguration == ["PRIVATE"]

    # At least one statement in the resource policy must contain a condition with the key of "aws:sourceVpc" or "aws:sourceVpce"
    Properties.Policy.Statement[ Condition.*[ KEYS == /aws:[sS]ource(Vpc|VPC|Vpce|VPCE)/ ] !EMPTY ] !EMPTY
  }
}
    "#;

    let rule = Rule::try_from(rule)?;

    let resources = r#"
    {
        "Resources": {
            "apigatewayapi": {
                "Type": "AWS::ApiGateway::RestApi",
                "Properties": {
                    "Policy": {
                        "Version": "2012-10-17",
                        "Statement": [
                            {
                                "Sid": "PrincipalPutObjectIfIpAddress",
                                "Effect": "Allow",
                                "Action": "s3:PutObject",
                                "Resource": "arn:aws:s3:::my-service-bucket/*",
                                "Condition": {
                                    "Bool": {"aws:ViaAWSService": "false"},
                                    "StringEquals": {"aws:SourceVpc": "vpc-12243sc"}
                                }
                            },
                            {
                                "Sid": "ServicePutObject",
                                "Effect": "Allow",
                                "Action": "s3:PutObject",
                                "Resource": "arn:aws:s3:::my-service-bucket/*",
                                "Condition": {
                                    "Bool": {"aws:ViaAWSService": "true"}
                                }
                            }
                        ]
                    },
                    "EndpointConfiguration": ["PRIVATE"]
                }
            }
        }
    }"#;

    let values = PathAwareValue::try_from(resources)?;
    let mut eval = BasicQueryTesting {
        root: Rc::new(values),
        recorder: None,
    };
    let status = eval_rule(&rule, &mut eval, ClauseRole::Assertion)?;
    assert_eq!(status, Status::PASS);

    Ok(())
}

#[test]
fn test_api_gateway_cleaner_model() -> Result<()> {
    let rule = r#"
rule check_rest_api_private {
  AWS::ApiGateway::RestApi {
    Properties {
        # Endpoint configuration must only be private
        EndpointConfiguration == ["PRIVATE"]
        some Policy.Statement[*] {
            Condition.*[ keys == /aws:[sS]ource(Vpc|VPC|Vpce|VPCE)/ ] not empty
        }
    }
  }
}
    "#;

    let rule = Rule::try_from(rule)?;

    let resources = r#"
    {
        "Resources": {
            "apigatewayapi": {
                "Type": "AWS::ApiGateway::RestApi",
                "Properties": {
                    "Policy": {
                        "Version": "2012-10-17",
                        "Statement": [
                            {
                                "Sid": "PrincipalPutObjectIfIpAddress",
                                "Effect": "Allow",
                                "Action": "s3:PutObject",
                                "Resource": "arn:aws:s3:::my-service-bucket/*",
                                "Condition": {
                                    "Bool": {"aws:ViaAWSService": "false"},
                                    "StringEquals": {"aws:SourceVpc": "vpc-12243sc"}
                                }
                            },
                            {
                                "Sid": "ServicePutObject",
                                "Effect": "Allow",
                                "Action": "s3:PutObject",
                                "Resource": "arn:aws:s3:::my-service-bucket/*",
                                "Condition": {
                                    "Bool": {"aws:ViaAWSService": "true"}
                                }
                            }
                        ]
                    },
                    "EndpointConfiguration": ["PRIVATE"]
                }
            }
        }
    }"#;

    let values = PathAwareValue::try_from(resources)?;
    let mut eval = BasicQueryTesting {
        root: Rc::new(values),
        recorder: None,
    };
    let status = eval_rule(&rule, &mut eval, ClauseRole::Assertion)?;
    assert_eq!(status, Status::PASS);

    let resources = r#"
    {
        "Resources": {
            "apigatewayapi": {
                "Type": "AWS::ApiGateway::RestApi",
                "Properties": {
                    "Policy": {
                        "Version": "2012-10-17",
                        "Statement": [
                            {
                                "Sid": "PrincipalPutObjectIfIpAddress",
                                "Effect": "Allow",
                                "Action": "s3:PutObject",
                                "Resource": "arn:aws:s3:::my-service-bucket/*",
                                "Condition": {
                                    "Bool": {"aws:ViaAWSService": "false"},
                                    "Bool": {"aws:SecureTransport": "true"}
                                }
                            },
                            {
                                "Sid": "ServicePutObject",
                                "Effect": "Allow",
                                "Action": "s3:PutObject",
                                "Resource": "arn:aws:s3:::my-service-bucket/*",
                                "Condition": {
                                    "Bool": {"aws:ViaAWSService": "true"}
                                }
                            }
                        ]
                    },
                    "EndpointConfiguration": ["PRIVATE"]
                }
            }
        }
    }"#;

    let values = PathAwareValue::try_from(resources)?;
    let mut eval = BasicQueryTesting {
        root: Rc::new(values),
        recorder: None,
    };
    let status = eval_rule(&rule, &mut eval, ClauseRole::Assertion)?;
    assert_eq!(status, Status::FAIL);

    Ok(())
}

#[test]
fn testing_iam_role_prov_serve() -> Result<()> {
    let resources = r#"
    {
        "Resources": {
            "CounterTaskDefExecutionRole5959CB2D": {
                "Type": "AWS::IAM::Role",
                "Properties": {
                    "AssumeRolePolicyDocument": {
                        "Statement": [
                        {
                            "Action": "sts:AssumeRole",
                            "Effect": "Allow",
                            "Principal": {
                            "Service": "ecs-tasks.amazonaws.com"
                            }
                        }],
                        "Version": "2012-10-17"
                    },
                    "PermissionBoundary": {"Fn::Sub" : "arn::aws::iam::${AWS::AccountId}:policy/my-permission-boundary"},
                    "Tags": [{ "Key": "TestRole", "Value": ""}]
                },
                "Metadata": {
                    "aws:cdk:path": "foo/Counter/TaskDef/ExecutionRole/Resource"
                }
            }
        }
    }
    "#;

    let rules = r#"
let iam_roles = Resources.*[ Type == "AWS::IAM::Role"  ]
let ecs_tasks = Resources.*[ Type == "AWS::ECS::TaskDefinition" ]

rule deny_permissions_boundary_iam_role when %iam_roles !EMPTY {
    # atleast one Tags contains a Key "TestRole"
    %iam_roles.Properties.Tags[ Key == "TestRole" ] NOT EMPTY
    %iam_roles.Properties.PermissionBoundary !EXISTS
}

rule deny_task_role_no_permission_boundary when %ecs_tasks !EMPTY {
    let task_role = %ecs_tasks.Properties.TaskRoleArn

    when %task_role.'Fn::GetAtt' EXISTS {
        let role_name = %task_role.'Fn::GetAtt'[0]
        let iam_roles_by_name = Resources.*[ KEYS == %role_name ]
        %iam_roles_by_name !EMPTY
        iam_roles_by_name.Properties.Tags !EMPTY
    } or
    %task_role == /aws:arn/ # either a direct string or
}
    "#;

    let rules_file = RulesFile::try_from(rules)?;
    let value = PathAwareValue::try_from(resources)?;
    let mut eval = root_scope(&rules_file, Rc::new(value));
    let status = eval_rules_file(&rules_file, &mut eval, None)?;

    println!("{}", status);
    Ok(())
}

#[test]
fn testing_sg_rules_pro_serve() -> Result<()> {
    let sgs = r#"
    [{
    "Resources": {
    "CounterServiceSecurityGroupF41A3908": {
      "Type": "AWS::EC2::SecurityGroup",
      "Properties": {
        "GroupDescription": "foo/Counter/Service/SecurityGroup",
        "SecurityGroupEgress": [
          {
            "CidrIp": "0.0.0.0/0",
            "Description": "Allow all outbound traffic by default",
            "IpProtocol": "-1"
          }
        ],
        "VpcId": {
          "Ref": "Vpc8378EB38"
        }
      },
      "Metadata": {
        "aws:cdk:path": "foo/Counter/Service/SecurityGroup/Resource"
      }
    }
    }
},
    {
    "Resources": {
    "CounterServiceSecurityGroupF41A3908": {
      "Type": "AWS::EC2::SecurityGroup",
      "Properties": {
        "GroupDescription": "foo/Counter/Service/SecurityGroup",
        "SecurityGroupEgress": [
          {
            "CidrIpv6": "::/0",
            "Description": "Allow all outbound traffic by default",
            "IpProtocol": "-1"
          }
        ],
        "VpcId": {
          "Ref": "Vpc8378EB38"
        }
      },
      "Metadata": {
        "aws:cdk:path": "foo/Counter/Service/SecurityGroup/Resource"
      }
    }
    }
}, {
    "Resources": {
    "CounterServiceSecurityGroupF41A3908": {
      "Type": "AWS::EC2::SecurityGroup",
      "Properties": {
        "GroupDescription": "foo/Counter/Service/SecurityGroup",
        "SecurityGroupEgress": [
          {
            "CidrIp": "10.0.0.0/16",
            "Description": "",
            "IpProtocol": "-1"
          }
        ],
        "VpcId": {
          "Ref": "Vpc8378EB38"
        }
      },
      "Metadata": {
        "aws:cdk:path": "foo/Counter/Service/SecurityGroup/Resource"
      }
    }
    }
},
{    "Resources": {
    "CounterServiceSecurityGroupF41A3908": {
      "Type": "AWS::EC2::SecurityGroup",
      "Properties": {
        "GroupDescription": "foo/Counter/Service/SecurityGroup",
        "VpcId": {
          "Ref": "Vpc8378EB38"
        }
      },
      "Metadata": {
        "aws:cdk:path": "foo/Counter/Service/SecurityGroup/Resource"
      }
    }
    }
}]

    "#;

    let rules = r#"
let sgs = Resources.*[ Type == "AWS::EC2::SecurityGroup" ]

rule deny_egress when %sgs NOT EMPTY {
    # Ensure that none of the security group contain a rule
    # that has Cidr Ip set to any
    %sgs.Properties.SecurityGroupEgress[ CidrIp   == "0.0.0.0/0" or
                                         CidrIpv6 == "::/0" ] EMPTY
}

    "#;

    let rules_file = RulesFile::try_from(rules)?;

    let values = PathAwareValue::try_from(sgs)?;
    let samples = match values {
        PathAwareValue::List((_p, v)) => v,
        _ => unreachable!(),
    };

    for (index, each) in samples.iter().enumerate() {
        let mut root_context = root_scope(&rules_file, Rc::new(each.clone()));
        let status = eval_rules_file(&rules_file, &mut root_context, None)?;
        println!("{}", format!("Status {} = {}", index, status).underline());
    }

    Ok(())
}

#[test]
fn test_s3_bucket_pro_serv() -> Result<()> {
    let values = r#"
    [
{
    "Resources": {
        "S3Bucket": {
            "Type": "AWS::S3::Bucket",
            "Properties": {
                "BlockPublicAcls" : true,
                "BlockPublicPolicy" : true,
                "IgnorePublicAcls" : true,
                "RestrictPublicBuckets" : true
            },
            "Metadata": {
             "aws:cdk:path": "foo/Counter/S3/Resource"
            }
        }
    }
},

{    "Resources": {
        "S3Bucket": {
            "Type": "AWS::S3::Bucket",
            "Properties": {
                "BlockPublicAcls" : false,
                "BlockPublicPolicy" : true,
                "IgnorePublicAcls" : true,
                "RestrictPublicBuckets" : true
            },
            "Metadata": {
             "aws:cdk:path": "foo/Counter/S3/Resource"
            }
        }
    }
},

{    "Resources": {
        "S3Bucket": {
            "Type": "AWS::S3::Bucket",
            "Properties": {
                "BlockPublicAcls" : true,
                "BlockPublicPolicy" : false,
                "IgnorePublicAcls" : true,
                "RestrictPublicBuckets" : true
            },
            "Metadata": {
             "aws:cdk:path": "foo/Counter/S3/Resource"
            }
        }
    }
},

{    "Resources": {
        "S3Bucket": {
            "Type": "AWS::S3::Bucket",
            "Properties": {
                "BlockPublicAcls" : true,
                "BlockPublicPolicy" : true,
                "IgnorePublicAcls" : false,
                "RestrictPublicBuckets" : true
            },
            "Metadata": {
             "aws:cdk:path": "foo/Counter/S3/Resource"
            }
        }
    }
},

{    "Resources": {
        "S3Bucket": {
            "Type": "AWS::S3::Bucket",
            "Properties": {
                "BlockPublicAcls" : true,
                "BlockPublicPolicy" : true,
                "IgnorePublicAcls" : true,
                "RestrictPublicBuckets" : false
            },
            "Metadata": {
             "aws:cdk:path": "foo/Counter/S3/Resource"
            }
        }
    }
},

{    "Resources": {
        "S3Bucket": {
            "Type": "AWS::S3::Bucket",
            "Properties": {
                "BlockPublicAcls" : false,
                "BlockPublicPolicy" : false,
                "IgnorePublicAcls" : false,
                "RestrictPublicBuckets" : false
            },
            "Metadata": {
             "aws:cdk:path": "foo/Counter/S3/Resource"
            }
        }
    }
},

{    "Resources": {
        "S3Bucket": {
            "Type": "AWS::S3::Bucket",
            "Metadata": {
             "aws:cdk:path": "foo/Counter/S3/Resource"
            }
        }
    }
},

{    "Resources": {
        "S3Bucket": {
            "Type": "AWS::S3::Bucket",
            "Properties": {
            "BlockPublicAcls" : true
            },
            "Metadata": {
             "aws:cdk:path": "foo/Counter/S3/Resource"
            }
        }
    }
},

{    "Resources": {
        "S3Bucket": {
            "Type": "AWS::S3::Bucket",
            "Properties": {
            "BlockPublicAcls" : true,
            "BlockPublicPolicy" : true
            },
            "Metadata": {
             "aws:cdk:path": "foo/Counter/S3/Resource"
            }
        }
    }
},

{    "Resources": {
        "S3Bucket": {
            "Type": "AWS::S3::Bucket",
            "Properties": {
            "BlockPublicAcls" : true,
            "BlockPublicPolicy" : true,
            "RestrictPublicBuckets" : true
            },
            "Metadata": {
             "aws:cdk:path": "foo/Counter/S3/Resource"
            }
        }
    }
}]

    "#;

    let parsed_values = match PathAwareValue::try_from(values)? {
        PathAwareValue::List((_, v)) => v,
        _ => unreachable!(),
    };

    let rule = r#"
    rule deny_s3_public_bucket {
    AWS::S3::Bucket {  # this is just a short form notation for Resources.*[ Type == "AWS::S3::Bucket" ]
        Properties.BlockPublicAcls NOT EXISTS or
        Properties.BlockPublicPolicy NOT EXISTS or
        Properties.IgnorePublicAcls NOT EXISTS or
        Properties.RestrictPublicBuckets NOT EXISTS or

        Properties.BlockPublicAcls == false or
        Properties.BlockPublicPolicy == false or
        Properties.IgnorePublicAcls == false or
        Properties.RestrictPublicBuckets == false
    }
}

    "#;

    let s3_rule = RulesFile::try_from(rule)?;
    let expectations = [
        Status::FAIL,
        Status::PASS,
        Status::PASS,
        Status::PASS,
        Status::PASS,
        Status::PASS,
        Status::PASS,
        Status::PASS,
        Status::PASS,
        Status::PASS,
    ];

    for (idx, each) in parsed_values.iter().enumerate() {
        let mut root_scope = root_scope(&s3_rule, Rc::new(each.clone()));
        let status = eval_rules_file(&s3_rule, &mut root_scope, None)?;
        assert_eq!(status, expectations[idx]);
    }
    Ok(())
}

#[test]
fn match_lhs_with_rhs_single_element_pass() -> Result<()> {
    let clause = r#"algorithms == ["KMS"]"#;
    let value = r#"algorithms: KMS"#;
    let path_value = PathAwareValue::try_from(serde_yaml::from_str::<serde_yaml::Value>(value)?)?;
    let guard_clause = GuardClause::try_from(clause)?;
    let mut eval = BasicQueryTesting {
        root: Rc::new(path_value),
        recorder: None,
    };
    let status = eval_guard_clause(&guard_clause, &mut eval, ClauseRole::Assertion)?;
    assert_eq!(status, Status::PASS);

    let clause = r#"algorithms == ["KMS", "SSE"]"#;
    let value = r#"algorithms: KMS"#;
    let path_value = PathAwareValue::try_from(serde_yaml::from_str::<serde_yaml::Value>(value)?)?;
    let guard_clause = GuardClause::try_from(clause)?;
    let mut eval = BasicQueryTesting {
        root: Rc::new(path_value),
        recorder: None,
    };
    let status = eval_guard_clause(&guard_clause, &mut eval, ClauseRole::Assertion)?;
    assert_eq!(status, Status::FAIL);

    Ok(())
}

#[test]
fn parameterized_evaluations() -> Result<()> {
    let parameterized = r###"
    rule check_iam_statements(statements) {
        %statements {
            when Effect == 'Allow' {
                Action != '*'
            }
        }
    }

    rule iam_checks {
        when Resources exists {
            Resources[ Type == /IAM::Role/ ] {
                check_iam_statements(Properties.AssumeRolePolicyDocument.Statement[*])
            }
        }

        when resourceType == /IAM::Role/ {
            check_iam_statements(configuration.assumeRolePolicyDocument.Statement[*])
        }
    }
    "###;

    let rules_files = RulesFile::try_from(parameterized)?;
    let template_value = r###"
    Resources:
      iamRole:
        Type: AWS::IAM::Role
        Properties:
          AssumeRolePolicyDocument:
            Statement:
              - Action: '*'
                Principal: '*'
                Resource: '*'
                Effect: Allow
    "###;
    let template =
        PathAwareValue::try_from(serde_yaml::from_str::<serde_yaml::Value>(template_value)?)?;

    let mut eval = root_scope(&rules_files, Rc::new(template));
    let status = eval_rules_file(&rules_files, &mut eval, None)?;
    let top = eval.reset_recorder().extract();
    let mut writer = Writer::default();
    crate::commands::validate::print_verbose_tree(&top, &mut writer);
    assert_eq!(status, Status::FAIL);

    let aws_config_value = r###"
    version: 1.2
    resourceType: AWS::IAM::Role
    configuration:
      assumeRolePolicyDocument:
        Statement:
          - Action: 'sts:AssumeRole'
            Principal: '*'
            Resource: '*'
            Effect: Allow
    "###;
    let config_value =
        PathAwareValue::try_from(serde_yaml::from_str::<serde_yaml::Value>(aws_config_value)?)?;

    let mut eval = root_scope(&rules_files, Rc::new(config_value));
    let status = eval_rules_file(&rules_files, &mut eval, None)?;
    let top = eval.reset_recorder().extract();
    crate::commands::validate::print_verbose_tree(&top, &mut writer);
    assert_eq!(status, Status::PASS);

    Ok(())
}

#[test]
fn using_resource_names_for_assessment() -> Result<()> {
    let resources = r###"
    Resources:
        s3:
            Type: AWS::S3::Bucket
        s3Policy:
            Type: AWS::S3::BucketPolicy
            Properties:
                BucketName:
                    Ref: s3
        s3Fail:
            Type: AWS::S3::Bucket
    "###;

    let value = PathAwareValue::try_from(serde_yaml::from_str::<serde_yaml::Value>(resources)?)?;

    let rules_file = r###"
    rule check_s3_has_bucket_policy {
        let s3_buckets = Resources[ s3_name | Type == 'AWS::S3::Bucket' ]
        let s3_bucket_policy_associations =
            some Resources[ Type == 'AWS::S3::BucketPolicy' ].Properties.BucketName.Ref
        when %s3_buckets not empty {
            # %s3_name == %s3_bucket_policy_associations
            %s3_bucket_policy_associations == %s3_name
                <<ALL S3 buckets do not have a bucket policy associated>>
        }
    }
    "###;

    let rules = RulesFile::try_from(rules_file)?;
    let mut eval = root_scope(&rules, Rc::new(value));
    let status = eval_rules_file(&rules, &mut eval, None)?;
    assert_eq!(status, Status::FAIL);

    Ok(())
}

/// An upstream failure parked in 2023 (commit `1aca9003`), diagnosed here but not fixed.
///
/// `IN` applies substring semantics only when its right-hand side is a literal string. When
/// the right-hand side comes from a query it falls back to equality, so the same spelling
/// means a different operator depending on where the value came from. Measured on this
/// fixture:
///
///     <query>    in '<literal string>'   PASS   -- substring, via string_in
///     'literal'  in <query>              FAIL   -- equality, via contained_in -> compare_eq
///
/// The mechanism is in `InOperation::compare`. A literal left side against a queried right
/// side takes the `(Some(l), None)` arm, which for two scalars calls `contained_in`, and
/// `contained_in`'s scalar/scalar case ends in `match_value(.., compare_eq)`. `string_in`,
/// which is the function that does `rhs.contains(lhs)`, is only reached from the
/// literal/literal arm and from the `(None, Some(r))` arm where the right side is a literal
/// `PathAwareValue::String`.
///
/// This test writes `some %bucket_names[*] in ...'Fn::Sub'` -- a query on both sides -- so it
/// gets equality and fails. Not the capture syntax and not the intrinsic: probing each
/// separately shows `%s3_buckets`, `%bucket_names` and the
/// `Properties.PolicyDocument.Statement.Resource.'Fn::Sub'` query all resolve, and the failure
/// reproduces with a plain literal `'s3'` in place of the captured variable.
///
/// Left ignored because the fix is a semantics decision for upstream, not a local repair.
/// Making scalar-against-query use `string_in` would turn every `IN` between a scalar and a
/// queried value into a substring test -- `Properties.Name in Resources.*.Tags` would start
/// matching on fragments -- and that is a visible behaviour change for every existing ruleset.
/// docs/CLAUSES.md documents only scalar left-hand sides for `IN`, so it does not settle which
/// reading is intended. Same family of question as the mirrored zero-selection asymmetry.
#[test]
#[ignore = "upstream, 2023: IN uses equality against a queried RHS but substring against a literal one"]
fn test_string_in_comparison() -> Result<()> {
    let resources = r#"
    Resources:
      s3:
        Type: AWS::S3::Bucket
      s3Policy:
        Type: AWS::S3::BucketPolicy
        Properties:
          PolicyDocument:
            Statement:
              Resource:
                Fn::Sub: "aws:arn:s3::${s3}"
    "#;
    let value = PathAwareValue::try_from(serde_yaml::from_str::<serde_yaml::Value>(resources)?)?;

    let rules = r###"
    let s3_buckets = Resources[ bucket_names | Type == 'AWS::S3::Bucket' ]
    rule s3_policies {
        when %s3_buckets not empty {
            Resources[ Type == 'AWS::S3::BucketPolicy' ] {
                some %bucket_names[*] in Properties.PolicyDocument.Statement.Resource.'Fn::Sub'
            }
        }
    }
    "###;

    let rules_files = RulesFile::try_from(rules)?;
    let mut eval = root_scope(&rules_files, Rc::new(value));
    let status = eval_rules_file(&rules_files, &mut eval, None)?;
    assert_eq!(status, Status::PASS);

    Ok(())
}

#[test]
fn test_searcher() -> Result<()> {
    let resources = r#"
    Resources:
      s3:
        Type: AWS::S3::Bucket
      s3Policy:
        Type: AWS::S3::BucketPolicy
        Properties:
          PolicyDocument:
            Statement:
              Resource:
                Fn::Sub: "aws:arn:s3::${s3}"
    "#;

    use grep_matcher::Matcher;
    use grep_regex::RegexMatcher;

    let matcher = RegexMatcher::new("\\s+(s3):$|\\s+(s3Policy):$").unwrap();
    SearcherBuilder::new()
        .line_number(true)
        .build()
        .search_slice(
            &matcher,
            resources.as_bytes(),
            grep_searcher::sinks::UTF8(|_, line| {
                let mut captures = matcher.new_captures()?;
                let _matched = matcher.captures(line.trim_end().as_bytes(), &mut captures)?;
                Ok(true)
            }),
        )?;

    Ok(())
}

#[test]
fn status_combinator() {
    let skip: Status = Status::SKIP;
    let pass: Status = Status::PASS;
    let fail: Status = Status::FAIL;

    assert_eq!(skip.and(skip), Status::SKIP);

    assert_eq!(skip.and(pass), Status::PASS);
    assert_eq!(pass.and(skip), Status::PASS);
    assert_eq!(pass.and(pass), Status::PASS);

    assert_eq!(fail.and(fail), Status::FAIL);
    assert_eq!(fail.and(skip), Status::FAIL);
    assert_eq!(skip.and(fail), Status::FAIL);
    assert_eq!(pass.and(fail), Status::FAIL);
    assert_eq!(fail.and(pass), Status::FAIL);
}

//
// Comparisons whose right-hand side (the reference/allow/deny list) resolves to no
// values. These used to SKIP, which exits 0, so an allowlist that resolved empty
// reported compliance for a violating template.
//
// The answer depends on polarity, and on whether the clause is a body assertion or a
// `when` condition. All four combinations are pinned here because getting any one of
// them wrong reintroduces a wrong PASS or starts failing compliant templates.
//
fn status_of(rules: &str, input: &str) -> Result<Status> {
    let value = PathAwareValue::try_from(input)?;
    let rules_file = RulesFile::try_from(rules)?;
    let mut root = root_scope(&rules_file, Rc::new(value));
    eval_rules_file(&rules_file, &mut root, None)
}

const ONE_BUCKET: &str = r#"
{
    Resources: {
        bucket: {
            Type: 'AWS::S3::Bucket',
            Properties: { BucketName: "PUBLIC-INSECURE" }
        }
    }
}
"#;

#[test]
fn positive_comparison_against_empty_reference_fails() -> Result<()> {
    // "the name must be one of the approved names", where the approved list is
    // derived from a resource type absent from this template. Nothing qualifies, so
    // the clause cannot be satisfied. Before the fix this SKIPped and exited 0.
    let rules = r###"
    let approved = Resources.*[ Type == 'AWS::KMS::Key' ].Properties.KeyId
    rule name_must_be_approved {
        Resources.*[ Type == 'AWS::S3::Bucket' ].Properties.BucketName IN %approved
    }
    "###;
    assert_eq!(status_of(rules, ONE_BUCKET)?, Status::FAIL);
    Ok(())
}

/// A negated comparison whose reference resolved to no values fails as an assertion.
///
/// This asserted SKIP until the semantics were settled in review, on the reading that the
/// clause is vacuously satisfied: there is nothing to collide with, and a denylist is
/// legitimately empty whenever the template contains none of the denied values.
///
/// The reading was rejected because the two error modes are not symmetric. A wrong FAIL is
/// visible and gets investigated; a wrong SKIP exits 0 and is indistinguishable from PASS in
/// CI, so a rule whose only check is `Property != %empty_reference` silently enforced nothing.
/// That is a denylist bypass, and it is the hole this branch exists to close.
///
/// The alternative considered was to keep the SKIP and require authors to declare the
/// expectation with an accompanying `!empty` clause. Rejected: it leaves every existing
/// ruleset without such a guard silently defeatable, which is the state being fixed.
///
/// FAIL specifically, not merely "not SKIP" — a PASS here would short-circuit a disjunction
/// and abandon its sibling disjuncts, which
/// `vacuous_negated_comparison_does_not_satisfy_a_disjunction` covers.
#[test]
fn negated_comparison_against_empty_reference_fails() -> Result<()> {
    let rules = r###"
    let denied = Resources.*[ Type == 'AWS::KMS::Key' ].Properties.KeyId
    rule name_must_not_be_denied {
        Resources.*[ Type == 'AWS::S3::Bucket' ].Properties.BucketName != %denied
    }
    "###;
    assert_eq!(status_of(rules, ONE_BUCKET)?, Status::FAIL);
    Ok(())
}

/// The escape hatch for a reference that is legitimately allowed to be empty.
///
/// Failing closed on an empty reference is only defensible if an author who genuinely expects
/// one has a way to say so. `when <reference> !empty { ... }` is that way, and it needs no new
/// machinery: the gate's own `!empty` check fails when the reference resolved to nothing, so
/// `eval_rule` treats the rule as inapplicable and the guarded comparison never runs.
///
/// Asserted rather than assumed. The claim was made in review as the reason failing closed is
/// safe, and if it were wrong the change would leave no way to express a permissibly-empty
/// denylist at all.
#[test]
fn an_empty_reference_can_be_guarded_with_a_when_not_empty_gate() -> Result<()> {
    let guarded = r###"
    let denied = Resources.*[ Type == 'AWS::KMS::Key' ].Properties.KeyId
    rule name_must_not_be_denied when %denied !empty {
        Resources.*[ Type == 'AWS::S3::Bucket' ].Properties.BucketName != %denied
    }
    "###;

    // The gate closes on the empty reference, so the rule does not apply and the clause that
    // would otherwise fail never runs.
    assert_eq!(
        status_of(guarded, ONE_BUCKET)?,
        Status::SKIP,
        "the `when %denied !empty` guard must make the rule inapplicable rather than failing \
         it -- without this there is no way to express a permissibly-empty reference"
    );

    // Liveness: with a non-empty reference the gate opens and the comparison decides. Without
    // this row the assertion above is satisfied by a rule that never ran for any reason.
    let with_keys = r#"
    {
        Resources: {
            bucket: { Type: 'AWS::S3::Bucket', Properties: { BucketName: "PUBLIC-INSECURE" } },
            key: { Type: 'AWS::KMS::Key', Properties: { KeyId: "PUBLIC-INSECURE" } }
        }
    }
    "#;
    assert_eq!(
        status_of(guarded, with_keys)?,
        Status::FAIL,
        "liveness: with a populated reference the gate must open and the collision be caught"
    );

    Ok(())
}

/// Why the empty-reference arms stay a SKIP for a `when` condition instead of failing closed
/// like an assertion.
///
/// The condition fold in `eval_conjunction_clauses` absorbs a SKIP but counts a FAIL, and it
/// answers FAIL before PASS. A gate that cannot compare therefore has to SKIP: failing it
/// would outrank the sibling conditions that did pass, make the rule inapplicable, and drop a
/// body those siblings would have enforced -- all at exit 0, which is the same wrong-PASS
/// shape this branch exists to close.
///
/// Two conditions joined by AND here. The first compares against an empty reference and cannot be
/// evaluated; the second passes. With the SKIP the second decides, the body runs, and its
/// violation is reported as a FAIL. Under an unconditional FAIL the rule reports SKIP and
/// nothing is enforced.
///
/// This shape is required to observe the difference at all. With a single condition both
/// statuses are indistinguishable, because `eval_rule` maps every non-PASS condition to a
/// rule-level SKIP; a disjunction hides it too, since a passing arm short-circuits either
/// way. An earlier version of this test used one condition and passed no matter which status
/// the arm returned.
///
/// `empty_reference_in_a_when_condition_does_not_disarm_the_block` is the same test for the
/// positive polarity, which reaches the other empty-reference arm.
#[test]
fn negated_empty_reference_in_a_when_condition_does_not_disarm_the_block() -> Result<()> {
    let rules = r###"
    let denied = Resources.*[ Type == 'AWS::KMS::Key' ].Properties.KeyId
    rule name_is_approved when Resources.*[ Type == 'AWS::S3::Bucket' ].Properties.BucketName != %denied
                               Resources.*[ Type == 'AWS::S3::Bucket' ] !empty {
        Resources.*[ Type == 'AWS::S3::Bucket' ].Properties.BucketName == 'approved-name'
    }
    "###;

    assert_eq!(
        status_of(rules, ONE_BUCKET)?,
        Status::FAIL,
        "the unevaluatable condition must be absorbed so the passing condition still applies \
         the rule; a FAIL there reports SKIP and drops the body"
    );
    Ok(())
}

#[test]
fn negation_on_a_parameterized_rule_call_is_honored() -> Result<()> {
    // `not r(...)` used to behave identically to `r(...)`: the parser stores the
    // leading `not` on the call, but eval_parameterized_rule_call returned the
    // invoked rule's status unchanged, discarding it. Same defect class as the
    // dropped clause-level negation on binary comparisons.
    //
    // `inner` PASSes here, so `not inner("x")` must FAIL and `inner("x")` must PASS.
    // Before the fix both PASSed.
    let input = r#"
    {
        Resources: {
            bucket: {
                Type: 'AWS::S3::Bucket',
                Properties: { BucketName: "b" }
            }
        }
    }
    "#;

    let negated = r###"
    rule inner(t) {
        %t == 'AWS::S3::Bucket'
    }
    rule outer {
        not inner(Resources.bucket.Type)
    }
    "###;

    let plain = r###"
    rule inner(t) {
        %t == 'AWS::S3::Bucket'
    }
    rule outer {
        inner(Resources.bucket.Type)
    }
    "###;

    // The two forms must disagree; that they agreed is what proved the bug.
    assert_eq!(status_of(negated, input)?, Status::FAIL);
    assert_eq!(status_of(plain, input)?, Status::PASS);

    Ok(())
}

#[test]
fn parameterized_rule_used_as_a_gate_does_not_disarm_the_block() -> Result<()> {
    // Regression test for a wrong PASS found by review.
    //
    // A parameterized rule invoked from a `when` condition is a gate, so its body
    // must evaluate with gate semantics. eval_when_clause threaded the role into its
    // Clause and NamedRule arms but not ParameterizedNamedRule, and everything
    // downstream defaulted to assertion strictness. The gate therefore FAILed instead
    // of SKIPping, eval_rule read a non-PASS condition as "rule does not apply", and
    // the guarded body -- the real check -- was never evaluated. Exit 0 on a
    // violating template, where base correctly exited 19.
    //
    // `inner` SKIPs (its query selects a resource type not present). `gate` is
    // parameterized and negates it. `must_be_encrypted` is gated on `gate`.
    let rules = r###"
    rule inner {
        Resources.*[ Type == 'AWS::Nonexistent::Thing' ] {
            Properties.Foo == 'bar'
        }
    }

    rule gate(unused) {
        not inner
    }

    rule must_be_encrypted when gate("x") {
        Resources.Bucket.Properties.Encrypted == true
    }
    "###;

    let input = r#"
    {
        Resources: {
            Bucket: {
                Type: 'AWS::S3::Bucket',
                Properties: { BucketName: "mybucket", Encrypted: false }
            }
        }
    }
    "#;

    // The gate must open, so the body runs and its violated check fails the rule.
    // With the role not threaded to the parameterized gate, the gate FAILed, the rule
    // was treated as inapplicable, and this returned SKIP.
    assert_eq!(status_of(rules, input)?, Status::FAIL);

    Ok(())
}

/// A LIVE DEFECT INTRODUCED BY THIS BRANCH: a wrong FAIL across a named-rule gate.
///
/// This is the cost of the empty-collection FAIL, and it was not visible until a reviewer
/// measured the *satisfiable-body* case. Earlier gate fixtures all used a failing body, so
/// both "gate opened, body failed" and "gate closed, body dropped" exited 19 and the exit
/// code could not tell them apart.
///
/// `vac_eq` compares an empty collection, so it FAILs (correctly, as an assertion). `body`
/// gates on it. `eval_rule` treats any non-PASS condition as "rule does not apply", so
/// `body` is dropped even though its own check would pass:
///
/// - v3.2.0: exit 0, both rules compliant — but see below, that PASS is itself the defect.
/// - this branch: exit 19, `not_compliant: [vac_eq]`, `not_applicable: [body]`.
///
/// The regression is the `not_applicable: [body]`, not the exit code, and the distinction is
/// the point: exit 19 cannot tell "gate opened, body failed" from "gate closed, body dropped".
/// v3.2.0's exit 0 is not the target to restore, because it comes from `Tags == 'Owner'`
/// reporting *compliant* against `Tags: []` — the empty-collection wrong PASS this branch
/// removes. Asserting that status would make this test green exactly when the defect is
/// reintroduced.
///
/// Cause was the named-rule boundary: `rule_status` evaluated a named rule's body with
/// `ClauseRole::Assertion` whatever the reference site was, so the `role.is_strict()` guard
/// on the empty-collection arm could not see that the rule was being used as a gate. At a
/// *syntactic* `when` the guard already worked and the gate opened — verified separately —
/// so the defect was specific to the named-rule spelling.
///
/// Fixed by carrying the reference site's role through `rule_status` into `eval_rule`, and
/// keying the `rules_status` cache on `(rule, role)` so a body reference and a gate
/// reference to the same rule do not share a cache slot. Without the key change the fix
/// would be order-dependent: whichever reference ran first would decide the cached answer
/// for the other.
///
/// Reverting was rejected rather than untried: it restores the original wrong PASS, where
/// `Tags == 'Owner'` certifies `Tags: []` as compliant, and a wrong PASS on a policy gate is
/// worse than a wrong FAIL.
#[test]
fn a_named_rule_gate_does_not_drop_a_satisfiable_body() -> Result<()> {
    // `Name` matches what `body` requires, so `body` is satisfiable. Only `Tags: []` is
    // unusual, and it is what makes the gate condition fail.
    let input = r#"
    {
        Resources: {
            b: {
                Type: 'AWS::S3::Bucket',
                Properties: { Name: "publicbucket", Tags: [] }
            }
        }
    }
    "#;

    // The named-rule spelling is the subject, so it has to stay, and `vac_eq` is therefore
    // also a top-level rule subject to the file-level fold.
    //
    // Not because the language makes that unavoidable — an earlier version of this comment
    // claimed "every named rule in Guard is also top-level" and that is false. A
    // *parameterized* rule lands in a separate `parameterized_rules` vec (`exprs.rs:283`)
    // which the fold never iterates (the `for each_rule in &rule.guard_rules` loop in
    // `eval_rules_file`), so `rule vac_eq(unused)` gated by
    // `when vac_eq("x")` escapes the fold entirely.
    //
    // It escapes the defect too, which is the actual reason not to use it here: measured, that
    // shape reports the gated rule as compliant with nothing dropped. The parameterized
    // boundary threads the reference-site role correctly, so a fixture built on it pins the
    // *working* path — the same tell as the inline `when` spelling, and already covered by
    // `parameterized_rule_used_as_a_gate_does_not_disarm_the_block` above.
    let named_gate = r###"
    rule vac_eq {
        Resources.*[ Type == 'AWS::S3::Bucket' ].Properties.Tags == 'Owner'
    }
    rule body when vac_eq {
        Resources.*[ Type == 'AWS::S3::Bucket' ].Properties.Name == 'publicbucket'
    }
    "###;

    // Asserted on `body`'s own applicability, NOT on the file's status. Both halves of that
    // choice are load-bearing.
    //
    // The file status cannot express this regression. `eval_rules_file` evaluates every
    // top-level rule with `ClauseRole::Assertion` unconditionally -- the `eval_rule(..,
    // ClauseRole::Assertion)` call in `eval_rules_file` -- and folds `fails > 0 -> FAIL` in
    // that same function, so a `vac_eq` that fails strictly forces the file to
    // FAIL however the gate behaves. Keying the rule-status cache on `(rule, role)` fixes the
    // gate and cannot touch that fold.
    //
    // And the file status that *would* make this file PASS is itself the defect: on v3.2.0
    // `Tags == 'Owner'` reports `compliant` against `Tags: []`, which is the empty-collection
    // wrong PASS this branch removes. An earlier version of this test asserted exactly that
    // PASS as its target — so it would have gone green precisely when the defect was
    // reintroduced, with a green suite certifying the opposite.
    //
    // What is both reachable and wrong-PASS-free is narrower: `body` is satisfiable, so it
    // must not be reported not-applicable. `vac_eq` failing is correct and stays correct.
    let report_for = |data: &str| -> Result<Vec<String>> {
        let resources = PathAwareValue::try_from(data)?;
        let rules_file = RulesFile::try_from(named_gate)?;
        let mut root = root_scope(&rules_file, Rc::new(resources));
        let _ = eval_rules_file(&rules_file, &mut root, None)?;
        let top = root.reset_recorder().extract();
        Ok(simplified_json_from_root(&top)?
            .not_applicable
            .into_iter()
            .collect())
    };

    // Liveness first, for the reason recorded on the ordering reproduction: an absence claim
    // is satisfied when nothing runs at all, so a query that stopped selecting would let the
    // claim below pass while blind. With populated tags the gate opens on real evidence.
    let populated = r#"
    {
        Resources: {
            b: {
                Type: 'AWS::S3::Bucket',
                Properties: { Name: "publicbucket", Tags: ['Owner'] }
            }
        }
    }
    "#;
    let live = report_for(populated)?;
    assert!(
        live.is_empty(),
        "liveness: with `Tags: ['Owner']` both rules apply and nothing should be \
         not-applicable, got {:?}. Anything else means the query stopped selecting and the \
         claim below is meaningless.",
        live
    );

    // The claim, stated name-independently: with the gate condition failing, NOTHING should be
    // dropped. Not "nothing called `body`".
    //
    // An earlier version asserted `!dropped.iter().any(|name| name == "body")`, which is
    // vacuously true the moment the gated rule is renamed — measured: rename it and the test
    // goes green with the defect fully live, the renamed rule sitting in `not_applicable`
    // where nothing looks for it. The liveness row above cannot catch that, because
    // `live.is_empty()` is itself name-independent and holds under any renaming.
    //
    // Worth being precise about what liveness does buy here, since it is not this. Rotting the
    // query makes `body` not-applicable, which *violates* an absence claim rather than
    // satisfying it — so for the rot mutation liveness gives a clearer diagnostic, not
    // false-green protection. The mutation that passes blind is the rename. The general form:
    // an absence claim needs a mutation probe on its own matching key, and liveness guards the
    // query rather than the key.
    let dropped = report_for(input)?;
    assert!(
        dropped.is_empty(),
        "a rule was reported not-applicable even though its own check passes: the failing \
         gate condition swallowed it. not_applicable = {:?}",
        dropped
    );

    Ok(())
}

/// Carrying the gate role into a named rule's body must not soften a real violation.
///
/// This is the adversarial case for the `(rule, role)` change. That change makes
/// `rule_status` evaluate a referenced rule's body with the *reference site's* role, so a
/// `when` condition now evaluates the body with `ClauseRole::Gate`. The obvious risk is that
/// Gate strictness laundered every failure inside a gate into SKIP, which would open gates
/// that ought to close and run bodies that ought to be dropped -- a wrong PASS, and strictly
/// worse than the wrong FAIL being fixed.
///
/// It does not, and the reason is narrow enough to be worth pinning rather than asserting:
/// `ClauseRole` is consulted for exactly one outcome, the unevaluatable one. `Outcome`'s
/// doc puts it as "role matters for exactly one variant" and `to_status` only branches on
/// `Unevaluatable`. A populated collection that genuinely violates its clause is `Violated`,
/// which maps to FAIL under either role. So the gate still closes here, on real evidence.
///
/// Distinct from `a_named_rule_gate_does_not_drop_a_satisfiable_body`, which covers the
/// *unevaluatable* input (`Tags: []`). This one covers populated, violating input, where the
/// correct behaviour is the opposite: the gate must close.
#[test]
fn a_named_rule_gate_does_not_soften_a_real_violation() -> Result<()> {
    // Populated and genuinely violating: Tags is ['Backup'], the gate wants 'Owner'. Nothing
    // is unevaluatable, so role must not change the answer.
    let violating = r#"
    {
        Resources: {
            b: {
                Type: 'AWS::S3::Bucket',
                Properties: { Name: "publicbucket", Tags: ['Backup'] }
            }
        }
    }
    "#;
    let satisfying = r#"
    {
        Resources: {
            b: {
                Type: 'AWS::S3::Bucket',
                Properties: { Name: "publicbucket", Tags: ['Owner'] }
            }
        }
    }
    "#;

    let rules = r###"
    rule tags_name_owner {
        Resources.*[ Type == 'AWS::S3::Bucket' ].Properties.Tags == 'Owner'
    }
    rule body when tags_name_owner {
        Resources.*[ Type == 'AWS::S3::Bucket' ].Properties.Name == 'publicbucket'
    }
    "###;

    // Liveness: with 'Owner' present the gate opens and both rules apply, so the referenced
    // rule really is being consulted and the row below is not passing on a dead query.
    assert_eq!(
        status_of(rules, satisfying)?,
        Status::PASS,
        "liveness: the gate must open and the body pass when Tags == ['Owner']"
    );

    // The claim: a real violation inside a rule used as a gate is still a FAIL. If Gate
    // strictness had laundered it to SKIP the file would come back SKIP, not FAIL.
    assert_eq!(
        status_of(rules, violating)?,
        Status::FAIL,
        "a populated, violating collection was softened by the gate role -- Gate strictness \
         must only affect unevaluatable clauses, never real violations"
    );

    Ok(())
}

/// The same named rule referenced from a gate and from a body answers each independently.
///
/// This is what the `(rule, role)` cache key buys, and it is the part a role parameter alone
/// would not fix. `rules_status` memoises a rule's status; keyed on the name alone, the first
/// reference to reach a rule decided the cached value and every later reference reused it
/// whatever role it was in. With one reference from a gate and one from a body in the same
/// file, the answer would then depend on evaluation order -- and rule iteration order is not
/// something a rule author controls or can see.
///
/// `vac` is unevaluatable (`Tags: []`), which is the one outcome where the two roles disagree,
/// so this file forces both answers out of the same rule in a single run: `asserts` references
/// it from a body and `gated` references it from a `when`.
///
/// Declaration order is load-bearing. `eval_rules_file` walks top-level rules in order, so
/// `asserts` resolves `vac` first and populates the cache under the assertion role, where the
/// answer is "failure". Keyed on the name alone that entry is what the later gate reference
/// reads, the gate closes, and `gated` is dropped. Swapping the two rules hides it again, which
/// is why the order is called out rather than left to look incidental.
///
/// That this test is the *only* thing pinning the cache key is measured, not assumed. Mutating
/// `rule_status` to keep the role parameter but look up and store under a fixed
/// `ClauseRole::Assertion` -- role threaded, cache keyed on the name -- leaves
/// `a_named_rule_gate_does_not_drop_a_satisfiable_body` and
/// `a_named_rule_gate_does_not_soften_a_real_violation` both green and fails only this one. So
/// the original reproduction would have ratified a half-fix whose answer depends on rule
/// declaration order.
#[test]
fn the_same_named_rule_answers_both_roles_independently() -> Result<()> {
    let input = r#"
    {
        Resources: {
            b: {
                Type: 'AWS::S3::Bucket',
                Properties: { Name: "publicbucket", Tags: [] }
            }
        }
    }
    "#;

    // `asserts` before `gated`, deliberately -- see the note above on declaration order.
    let rules = r###"
    rule vac {
        Resources.*[ Type == 'AWS::S3::Bucket' ].Properties.Tags == 'Owner'
    }
    rule asserts {
        vac
    }
    rule gated when vac {
        Resources.*[ Type == 'AWS::S3::Bucket' ].Properties.Name == 'publicbucket'
    }
    "###;

    let report_for = |data: &str| -> Result<Vec<String>> {
        let resources = PathAwareValue::try_from(data)?;
        let rules_file = RulesFile::try_from(rules)?;
        let mut root = root_scope(&rules_file, Rc::new(resources));
        let _ = eval_rules_file(&rules_file, &mut root, None)?;
        let top = root.reset_recorder().extract();
        Ok(simplified_json_from_root(&top)?
            .not_applicable
            .into_iter()
            .collect())
    };

    // Liveness: with `Tags: ['Owner']` every rule applies, so nothing is not-applicable. An
    // absence claim is satisfied when nothing runs at all, so without this row a query that
    // stopped selecting would let the claim below pass while blind.
    let populated = r#"
    {
        Resources: {
            b: {
                Type: 'AWS::S3::Bucket',
                Properties: { Name: "publicbucket", Tags: ['Owner'] }
            }
        }
    }
    "#;
    let live = report_for(populated)?;
    assert!(
        live.is_empty(),
        "liveness: with `Tags: ['Owner']` nothing should be not-applicable, got {:?}",
        live
    );

    // The claim: the earlier assertion-role answer must not reach the gate. `gated`'s own
    // check passes, so it must not be reported not-applicable.
    let dropped = report_for(input)?;
    assert!(
        dropped.is_empty(),
        "a rule was reported not-applicable: the assertion-role status of `vac` was reused \
         for the gate reference, closing it. not_applicable = {:?}",
        dropped
    );

    Ok(())
}

/// `IN` must not certify an empty collection as compliant. Was pre-existing in v3.2.0.
///
/// `Tags IN ['Owner']` against `Tags: []` used to exit 0 and report `"compliant"` with
/// `not_applicable: []` — an affirmative certification that the policy held. The mechanism
/// was in `contained_in`: `diff` is "elements of the left side absent from the right", so an
/// empty left side produced an empty `diff` and an affirmative `Success`. That made it a
/// wrong Success to suppress rather than a missing entry to supply, so it needed a
/// different fix from the empty-`statues` fold that `2224cb1` addressed for `==`.
///
/// Two measurements say the old behaviour was not merely a defensible reading of universal
/// quantification:
///
/// - The same template under `==` failed (exit 19). One spelling of "the tag must be Owner"
///   blocked the deployment and the other certified it.
/// - `Tags not in ['Owner']` on the same empty list *also* failed (19). A proposition and
///   its negation cannot both be unsatisfied, so the pair was internally inconsistent
///   whatever the intended reading. That spelling now passes, which is the vacuous-truth
///   reading and the consistent one.
///
/// Fixed by routing the empty collection through `elements_or_record_empty` to
/// `EmptyLhsCollection`, which `binary_operation` resolves by role: an assertion fails, a
/// gate contributes nothing and stays decided by its other conditions. Deciding it inside
/// the comparator instead would close a `when` gate and drop the guarded body, which is how
/// two earlier attempts regressed;
/// `an_empty_collection_in_a_when_condition_does_not_disarm_the_guarded_block` pins that.
#[test]
fn in_does_not_certify_an_empty_collection() -> Result<()> {
    let rules = r###"
    rule tags_must_name_owner {
        Resources.*[ Type == 'AWS::S3::Bucket' ].Properties.Tags IN ['Owner']
    }
    "###;

    let input = r#"
    {
        Resources: {
            b: {
                Type: 'AWS::S3::Bucket',
                Properties: { Name: "publicbucket", Tags: [] }
            }
        }
    }
    "#;

    let resources = PathAwareValue::try_from(input)?;
    let rules_file = RulesFile::try_from(rules)?;
    let mut root = root_scope(&rules_file, Rc::new(resources));

    // FAIL, matching what the same rule spelled with `==` already does. SKIP would also be
    // arguable — "no tags, so nothing to check" — but PASS is not: it affirmatively
    // certifies a bucket that carries none of the required tags.
    assert_eq!(eval_rules_file(&rules_file, &mut root, None)?, Status::FAIL);

    Ok(())
}

/// The ordering operators must not certify an empty collection. Was pre-existing in v3.2.0.
///
/// Same class as `in_does_not_certify_an_empty_collection` but a *different impl* —
/// `CommonOperator`, not `contained_in` — so a fix for `IN` alone would not have touched it.
/// Worth its own test for that reason, and both now route through the one shared guard in
/// `elements_or_record_empty`.
///
/// The argument that this is a defect rather than defensible vacuous truth is stronger here
/// than for `IN`. `Ports <= 100` and `Ports > 100` are exact logical negations, and **both**
/// returned PASS on the same `Ports: []`. Universal quantification over an empty set defends
/// "every element satisfies P" for both P and not-P; it cannot defend certifying `x <= 100`
/// and `x > 100` for the same x. `<` and `>=` behaved identically — all four route through
/// the same impl.
///
/// The gate hazard that blocked two earlier attempts is measured rather than inferred:
/// `rule r when ...Ports <= 100 { ...Name == 'safe' }` exits 19 on `Ports: []` *because* the
/// vacuous PASS opens the gate and the body then catches a violating name. Deciding the
/// empty comparison inside the comparator turns that into exit 0 with the body dropped — one
/// unenforced clause traded for a disarmed block. Avoided by leaving the decision to
/// `binary_operation`, which contributes nothing for a gate.
#[test]
fn ordering_operators_do_not_certify_an_empty_collection() -> Result<()> {
    let input = r#"
    {
        Resources: {
            sg: { Type: 'AWS::EC2::SecurityGroup', Properties: { Ports: [] } }
        }
    }
    "#;

    // Exact logical negations. At most one may pass for any given input.
    let le = r###"
    rule ports_must_be_low {
        Resources.*[ Type == 'AWS::EC2::SecurityGroup' ].Properties.Ports <= 100
    }
    "###;
    let gt = r###"
    rule ports_must_be_high {
        Resources.*[ Type == 'AWS::EC2::SecurityGroup' ].Properties.Ports > 100
    }
    "###;

    // Liveness FIRST, and it is not decoration. An earlier version of this test asserted
    // only `passes <= 1` over the two negations, which had two false greens:
    //
    // - Rename one character in the Type filter and both rules SKIP. SKIP is not PASS, so
    //   `passes == 0` satisfied `<= 1` and the test went green while blind.
    // - Add one resource that genuinely violates `<= 100`. The rule aggregates across
    //   resources, so `> 100` becomes FAIL overall and `passes` drops to 1 — green, with the
    //   rule text unchanged and the empty resource still certified under both negations.
    //   Verified from the blame paths that the empty resource is never named.
    //
    // Going absolute per operator kills the second but not the first: rot yields SKIP and
    // `SKIP != PASS`, so an `assert_ne!(PASS)` claim alone still passes blind. The liveness
    // rows are what close that — under rot they fail before the claim is reached.
    //
    // `[9]` rather than `[80]` for the liveness row on purpose: `"9" <= "100"` is false
    // lexicographically and true numerically, so this row also pins numeric comparison. That
    // matters because an earlier measurement of this operator class was confounded by string
    // ordering, and `[8080]` discriminates nothing — it is false under both readings.
    let live_low = r#"
    { Resources: { sg: { Type: 'AWS::EC2::SecurityGroup', Properties: { Ports: [9] } } } }
    "#;
    let live_high = r#"
    { Resources: { sg: { Type: 'AWS::EC2::SecurityGroup', Properties: { Ports: [8080] } } } }
    "#;

    let status_of_pair = |rules: &str, data: &str| -> Result<Status> {
        let resources = PathAwareValue::try_from(data)?;
        let rules_file = RulesFile::try_from(rules)?;
        let mut root = root_scope(&rules_file, Rc::new(resources));
        eval_rules_file(&rules_file, &mut root, None)
    };

    assert_eq!(
        status_of_pair(le, live_low)?,
        Status::PASS,
        "liveness: `Ports <= 100` must pass on [9] — numerically true, lexicographically \
         false, so this also pins numeric semantics. A SKIP here means the query stopped \
         selecting and every other row in this test is meaningless."
    );
    assert_eq!(
        status_of_pair(le, live_high)?,
        Status::FAIL,
        "liveness: `Ports <= 100` must fail on [8080]"
    );
    assert_eq!(
        status_of_pair(gt, live_high)?,
        Status::PASS,
        "liveness: `Ports > 100` must pass on [8080]"
    );
    assert_eq!(
        status_of_pair(gt, live_low)?,
        Status::FAIL,
        "liveness: `Ports > 100` must fail on [9]"
    );

    // The claim, stated absolutely per operator rather than as a count. Each negation is
    // asserted on its own, so no aggregation across resources and no SKIP can launder it.
    assert_ne!(
        status_of_pair(le, input)?,
        Status::PASS,
        "`Ports <= 100` certified an empty collection"
    );
    assert_ne!(
        status_of_pair(gt, input)?,
        Status::PASS,
        "`Ports > 100` certified an empty collection"
    );

    Ok(())
}

/// The control for the ordering operators, which must keep passing.
///
/// Both polarities on populated collections, so a future fix for the empty case cannot
/// quietly break ordinary numeric comparison. These are the rows that make the empty row
/// above interpretable — without them, a wrong answer on `[]` could just mean the rule or
/// the query was malformed.
///
/// It also pins **numeric** rather than lexicographic comparison, which is a stronger
/// guarantee than it looks and is load-bearing here: an earlier measurement of this operator
/// class was discarded as confounded by string ordering. Note which fixtures carry that
/// weight. `[8080]` discriminates nothing — `8080 <= 100` and `"8080" <= "100"` are both
/// false. `[80]` and `[9]` are the discriminating ones: `"80" <= "100"` and `"9" <= "100"` are
/// both false lexicographically (`'8'`/`'9'` > `'1'` at index 0) and true numerically, so a
/// PASS on either can only be numeric.
#[test]
fn ordering_operators_still_decide_populated_collections_correctly() -> Result<()> {
    let rules = r###"
    rule ports_must_be_low {
        Resources.*[ Type == 'AWS::EC2::SecurityGroup' ].Properties.Ports <= 100
    }
    "###;

    // Both discriminate numeric from lexicographic; `[9]` most sharply, since it is a single
    // digit and cannot be read as "shorter string sorts first".
    let low = r#"
    { Resources: { sg: { Type: 'AWS::EC2::SecurityGroup', Properties: { Ports: [80] } } } }
    "#;
    let single_digit = r#"
    { Resources: { sg: { Type: 'AWS::EC2::SecurityGroup', Properties: { Ports: [9] } } } }
    "#;
    let high = r#"
    { Resources: { sg: { Type: 'AWS::EC2::SecurityGroup', Properties: { Ports: [8080] } } } }
    "#;

    let rules_file = RulesFile::try_from(rules)?;

    for (data, want, why) in [
        (
            low,
            Status::PASS,
            "80 <= 100 numerically; false lexicographically",
        ),
        (
            single_digit,
            Status::PASS,
            "9 <= 100 numerically; false lexicographically",
        ),
        (
            high,
            Status::FAIL,
            "8080 <= 100 is false under either reading",
        ),
    ] {
        let resources = PathAwareValue::try_from(data)?;
        let mut root = root_scope(&rules_file, Rc::new(resources));
        assert_eq!(
            eval_rules_file(&rules_file, &mut root, None)?,
            want,
            "{}",
            why
        );
    }

    Ok(())
}

/// The control for the above, which must keep passing: `IN` on a populated collection.
///
/// Pinned separately so that a future fix for the empty case cannot quietly break the
/// ordinary one. Both polarities are exercised — a satisfying list passes, a violating list
/// fails — which is what establishes that the rule and query are well formed and that only
/// the empty case is wrong.
#[test]
fn in_still_decides_populated_collections_correctly() -> Result<()> {
    let rules = r###"
    rule tags_must_name_owner {
        Resources.*[ Type == 'AWS::S3::Bucket' ].Properties.Tags IN ['Owner']
    }
    "###;

    let satisfying = r#"
    { Resources: { b: { Type: 'AWS::S3::Bucket', Properties: { Tags: ['Owner'] } } } }
    "#;
    let violating = r#"
    { Resources: { b: { Type: 'AWS::S3::Bucket', Properties: { Tags: ['Backup'] } } } }
    "#;

    let rules_file = RulesFile::try_from(rules)?;

    let resources = PathAwareValue::try_from(satisfying)?;
    let mut root = root_scope(&rules_file, Rc::new(resources));
    assert_eq!(eval_rules_file(&rules_file, &mut root, None)?, Status::PASS);

    let resources = PathAwareValue::try_from(violating)?;
    let mut root = root_scope(&rules_file, Rc::new(resources));
    assert_eq!(eval_rules_file(&rules_file, &mut root, None)?, Status::FAIL);

    Ok(())
}

/// `NOT IN` over an empty collection is vacuously satisfied, a deliberate change from FAIL.
///
/// Before the shared empty-collection guard, `Tags not in ['Owner']` on `Tags: []` FAILed:
/// `contained_in` built an affirmative Success out of the empty `diff` and the negation
/// inverted it. That rejected a compliant template, since no element of `[]` is in
/// `['Owner']` and there is nothing to collide with.
///
/// It now routes through `EmptyLhsCollection`, where a negated clause contributes no entry
/// because `role.is_strict() && !cmp.1` is false, so the fold sees no failures and reports
/// PASS. Same path `!=` already took, so this inherits that path's known
/// disjunction-absorption hazard rather than introducing one — see
/// `a_vacuous_negated_clause_does_not_absorb_a_disjunction`, still ignored.
#[test]
fn not_in_over_an_empty_collection_is_vacuously_satisfied() -> Result<()> {
    let rules = r###"
    rule tags_must_not_name_owner {
        Resources.*[ Type == 'AWS::S3::Bucket' ].Properties.Tags not in ['Owner']
    }
    "###;

    let colliding = r#"
    { Resources: { b: { Type: 'AWS::S3::Bucket', Properties: { Tags: ['Owner'] } } } }
    "#;
    let clean = r#"
    { Resources: { b: { Type: 'AWS::S3::Bucket', Properties: { Tags: ['Backup'] } } } }
    "#;
    let empty = r#"
    { Resources: { b: { Type: 'AWS::S3::Bucket', Properties: { Tags: [] } } } }
    "#;

    // Liveness first. Without these two rows the empty-collection claim below would go
    // green on a rule that stopped selecting anything at all.
    assert_eq!(
        status_of(rules, colliding)?,
        Status::FAIL,
        "liveness: `not in` must still catch a colliding tag"
    );
    assert_eq!(
        status_of(rules, clean)?,
        Status::PASS,
        "liveness: `not in` must still pass a non-colliding tag"
    );

    // SKIP, not PASS. The old behaviour was FAIL, which rejected a compliant template; this
    // asserted PASS when the comparator guard landed, and became SKIP when the fold moved
    // onto `Outcome`.
    //
    // Either way the clause is not blamed, and the exit code is 0 for both. SKIP is the
    // stronger answer because PASS would let this disjunct absorb an `or` and abandon its
    // siblings -- see `a_vacuous_negated_clause_does_not_absorb_a_disjunction`.
    assert_eq!(
        status_of(rules, empty)?,
        Status::SKIP,
        "`not in` over an empty collection is vacuously satisfied and must not be blamed, but \
         must not report PASS either -- PASS absorbs a disjunction"
    );

    Ok(())
}

/// The mirrored spelling of an ordering comparison must answer for an empty collection too.
///
/// `%limit >= ...Ports` and `...Ports <= %limit` mean the same thing, so certifying
/// `Ports: []` under one and not the other leaves the defect reachable by writing the clause
/// backwards — legal Guard, and a rule author has no reason to think the two differ. This is
/// why `CommonOperator::compare` routes *both* sides through `elements_or_record_empty`; a
/// left-side-only fix satisfies `ordering_operators_do_not_certify_an_empty_collection` while
/// leaving this shape wrong. Same argument `EqOperation` makes for its mirrored empty-RHS
/// guard, pinned by `an_empty_collection_fails_when_it_is_the_right_hand_operand`.
#[test]
fn a_mirrored_empty_collection_fails_an_ordering_comparison() -> Result<()> {
    let rules = r###"
    let limit = 100
    rule ports_must_be_low {
        %limit >= Resources.*[ Type == 'AWS::EC2::SecurityGroup' ].Properties.Ports
    }
    "###;

    let empty = r#"
    { Resources: { sg: { Type: 'AWS::EC2::SecurityGroup', Properties: { Ports: [] } } } }
    "#;
    let live_low = r#"
    { Resources: { sg: { Type: 'AWS::EC2::SecurityGroup', Properties: { Ports: [9] } } } }
    "#;
    let live_high = r#"
    { Resources: { sg: { Type: 'AWS::EC2::SecurityGroup', Properties: { Ports: [8080] } } } }
    "#;

    // Liveness, and `[9]` rather than `[80]` for the same reason as the forward test:
    // `"9" <= "100"` is false lexicographically and true numerically, so this row also pins
    // numeric comparison in the mirrored direction.
    assert_eq!(
        status_of(rules, live_low)?,
        Status::PASS,
        "liveness: `%limit >= Ports` must pass on [9]"
    );
    assert_eq!(
        status_of(rules, live_high)?,
        Status::FAIL,
        "liveness: `%limit >= Ports` must fail on [8080]"
    );

    // FAIL specifically, not merely "not PASS": as an assertion, `EmptyLhsCollection` is
    // resolved to FAIL, so asserting the exact status confirms the mirrored guard fired
    // rather than the clause having been skipped for some unrelated reason.
    assert_eq!(
        status_of(rules, empty)?,
        Status::FAIL,
        "the mirrored spelling certified an empty collection"
    );

    Ok(())
}

/// An empty collection in a `when` condition built on an ordering operator must not disarm
/// the guarded block.
///
/// This is the measured hazard that reverted two earlier attempts, and it is specific to
/// `CommonOperator` — the existing coverage
/// (`an_empty_collection_in_a_when_condition_does_not_disarm_the_guarded_block`) exercises
/// `==` and so goes through `EqOperation`. Here the gate contributes no entry rather than a
/// FAIL, so it stays open, the body runs, and the violating Name is caught. Deciding the
/// empty comparison inside the comparator turns this into exit 0 with the body dropped:
/// one unenforced clause traded for an entire disarmed block.
#[test]
fn an_empty_collection_in_an_ordering_gate_does_not_disarm_the_block() -> Result<()> {
    let rules = r###"
    rule name_must_be_safe when Resources.*[ Type == 'AWS::EC2::SecurityGroup' ].Properties.Ports <= 100 {
        Resources.*[ Type == 'AWS::EC2::SecurityGroup' ].Properties.Name != 'insecure'
    }
    "###;

    let input = r#"
    {
        Resources: {
            sg: {
                Type: 'AWS::EC2::SecurityGroup',
                Properties: { Name: "insecure", Ports: [] }
            }
        }
    }
    "#;

    // FAIL because the body ran and `insecure` violated it. SKIP would mean the gate closed
    // and the violation went unreported.
    assert_eq!(status_of(rules, input)?, Status::FAIL);

    Ok(())
}

/// Every FAIL must state a reason. This asserts report *contents*, not just status.
///
/// The gap this closes: no test in the repository asserted anything about a report's
/// `checks`, so a FAIL that produced an empty report was green in all 355 of them. The
/// empty-collection FAIL added by `2224cb1` did exactly that — `eval_context.rs`'s
/// `QueryResult::Resolved` arm wrapped its whole body in `if let Some(to) = to` with no
/// `else`, and that FAIL is the one construction site pairing a resolved `from` with
/// `to: None`, so it pushed no `ClauseReport` at all.
///
/// The visible result was exit 19 with `checks: []` in JSON and YAML, `results: []` in
/// SARIF, an empty `<failure/>` in JUnit, and "Number of non-compliant resources 0" on the
/// console — a blocked deployment with nothing to act on, and the message built at the
/// construction site never reaching any output.
///
/// Written against the general property rather than the one operator that exposed it: any
/// rule reporting FAIL must carry at least one clause explaining why.
#[test]
fn a_failing_rule_always_reports_at_least_one_reason() -> Result<()> {
    // Three shapes that all FAIL, including both operand orders of the empty-collection
    // case. The third is a plain scalar mismatch, which reported correctly all along and
    // is here so the test would catch a regression in the normal path too.
    let rulesets = [
        r###"
        rule tags_must_name_owner {
            Resources.*[ Type == 'AWS::S3::Bucket' ].Properties.Tags == 'Owner'
        }
        "###,
        r###"
        let expected = 'Owner'
        rule tags_must_name_owner_mirrored {
            %expected == Resources.*[ Type == 'AWS::S3::Bucket' ].Properties.Tags
        }
        "###,
        r###"
        rule name_must_be_private {
            Resources.*[ Type == 'AWS::S3::Bucket' ].Properties.Name == 'privatebucket'
        }
        "###,
    ];

    let input = r#"
    {
        Resources: {
            bucketEmptyTags: {
                Type: 'AWS::S3::Bucket',
                Properties: { Name: "publicbucket", Tags: [] }
            }
        }
    }
    "#;

    // Asserted through the serialized structured output rather than the internal report
    // type, for two reasons: the builder is private to eval_context, and this is the
    // artifact a user or a CI job actually consumes. `checks: []` under a FAIL is exactly
    // what the defect looked like from outside.
    for rules in rulesets {
        let resources = PathAwareValue::try_from(input)?;
        let rules_file = RulesFile::try_from(rules)?;
        let mut root = root_scope(&rules_file, Rc::new(resources));
        let status = eval_rules_file(&rules_file, &mut root, None)?;

        assert_eq!(
            status,
            Status::FAIL,
            "ruleset was expected to fail, ruleset was: {}",
            rules
        );

        // Assert on the CONSTRUCTED report, not on the recorded evaluation tree.
        //
        // This distinction is the whole point. `eval.rs` always wrote a ClauseValueCheck
        // record, so a test that walked the recorder passed even with the defect present --
        // I wrote that test first and the mutation check caught it. The report builder in
        // eval_context is where the record was dropped, so that is the layer to assert at.
        let top = root.reset_recorder().extract();
        let report = simplified_json_from_root(&top)?;

        assert!(
            !report.not_compliant.is_empty(),
            "a failing file reported nothing non-compliant, ruleset was: {}",
            rules
        );
        for clause in &report.not_compliant {
            assert!(
                !clause_report_is_empty(clause),
                "rule reported FAIL with an empty report, so every format renders it as a \
                 blocked deployment with no stated reason. Ruleset was: {}",
                rules
            );
        }
    }

    Ok(())
}

/// True when a report names a failure but carries nothing explaining it.
///
/// Structural rather than message-keyed on purpose: the property is "the report has
/// something in it", and asserting on wording would accept a report that says the wrong
/// thing while rejecting a correct rewording.
fn clause_report_is_empty(report: &ClauseReport<'_>) -> bool {
    match report {
        ClauseReport::Rule(rule) => {
            rule.checks.is_empty() || rule.checks.iter().all(clause_report_is_empty)
        }
        // A block report is a leaf: it carries its own message rather than children.
        ClauseReport::Block(_) => false,
        ClauseReport::Disjunctions(disj) => {
            disj.checks.is_empty() || disj.checks.iter().all(clause_report_is_empty)
        }
        // A leaf clause report is the thing that explains a failure, so reaching one means
        // the report is not empty.
        ClauseReport::Clause(_) => false,
    }
}

/// A denylist written with a parameter did not block the value it named.
///
/// `LetValue::Value` -- a literal argument at the call site -- was bound as
/// `QueryResult::Resolved`, while `resolve_variable` returns `let` literals as
/// `QueryResult::Literal`. `is_literal` recognises only the latter, so the two spellings
/// of the same literal took different comparator arms: `let` reached the element-wise
/// `(None, Some(_))` arm, and a parameter reached `(None, None)`, which compares whole
/// query results through `diff`. A list-valued left side was therefore compared against
/// the scalar *as a list*, never matched, and the negation inverted that into a pass.
///
/// The blast radius is not empty collections. `Tags: ["PublicRead"]` -- a populated list
/// holding exactly the banned value -- passed, while the byte-identical policy with the
/// value inlined or bound with `let` correctly failed.
#[test]
fn a_denylist_passed_through_a_parameter_still_blocks_the_banned_value() -> Result<()> {
    let rules = r###"
    rule no_banned_tag(banned) {
        Resources.*[ Type == 'AWS::S3::Bucket' ].Properties.Tags != %banned
    }

    rule main { no_banned_tag("PublicRead") }
    "###;

    let input = r#"
    {
        Resources: {
            exposedBucket: {
                Type: 'AWS::S3::Bucket',
                Properties: { Name: "customerdata", Tags: ['PublicRead'] }
            }
        }
    }
    "#;

    let resources = PathAwareValue::try_from(input)?;
    let rules_file = RulesFile::try_from(rules)?;
    let mut root = root_scope(&rules_file, Rc::new(resources));

    assert_eq!(eval_rules_file(&rules_file, &mut root, None)?, Status::FAIL);

    Ok(())
}

/// The positive half of the same defect: `==` through a parameter was inverted.
///
/// It failed the template that *did* match and passed the one that did not, for the same
/// reason -- a whole-list-versus-scalar comparison whose result happened to be wrong in
/// both directions. Asserted together with the negated form because a fix that corrects
/// only one of them would leave the arm half-wrong in a way a single-polarity test cannot
/// see.
#[test]
fn a_positive_comparison_through_a_parameter_matches_the_right_template() -> Result<()> {
    let rules = r###"
    rule tag_must_match(wanted) {
        Resources.*[ Type == 'AWS::S3::Bucket' ].Properties.Tags == %wanted
    }

    rule main { tag_must_match("Keeper") }
    "###;

    let matching = r#"
    {
        Resources: {
            b: { Type: 'AWS::S3::Bucket', Properties: { Tags: ['Keeper'] } }
        }
    }
    "#;

    let non_matching = r#"
    {
        Resources: {
            b: { Type: 'AWS::S3::Bucket', Properties: { Tags: ['Other'] } }
        }
    }
    "#;

    let rules_file = RulesFile::try_from(rules)?;

    let resources = PathAwareValue::try_from(matching)?;
    let mut root = root_scope(&rules_file, Rc::new(resources));
    assert_eq!(
        eval_rules_file(&rules_file, &mut root, None)?,
        Status::PASS,
        "the template carrying the wanted tag must pass"
    );

    let resources = PathAwareValue::try_from(non_matching)?;
    let mut root = root_scope(&rules_file, Rc::new(resources));
    assert_eq!(
        eval_rules_file(&rules_file, &mut root, None)?,
        Status::FAIL,
        "the template without the wanted tag must fail"
    );

    Ok(())
}

/// A literal argument must behave identically however it is spelled.
///
/// This is the property the fix restores, and the one whose absence made the defect
/// invisible: every spelling other than the parameter form was already correct, so any
/// fixture that inlined the value or used `let` passed.
#[test]
fn a_literal_argument_agrees_across_parameter_let_and_inline_spellings() -> Result<()> {
    let input = r#"
    {
        Resources: {
            b: { Type: 'AWS::S3::Bucket', Properties: { Tags: ['PublicRead'] } }
        }
    }
    "#;

    let parameterized = r###"
    rule deny(banned) { Resources.*[ Type == 'AWS::S3::Bucket' ].Properties.Tags != %banned }
    rule main { deny("PublicRead") }
    "###;

    let let_bound = r###"
    let banned = 'PublicRead'
    rule main { Resources.*[ Type == 'AWS::S3::Bucket' ].Properties.Tags != %banned }
    "###;

    let inlined = r###"
    rule main { Resources.*[ Type == 'AWS::S3::Bucket' ].Properties.Tags != 'PublicRead' }
    "###;

    let mut statuses = Vec::new();
    for rules in [parameterized, let_bound, inlined] {
        let resources = PathAwareValue::try_from(input)?;
        let rules_file = RulesFile::try_from(rules)?;
        let mut root = root_scope(&rules_file, Rc::new(resources));
        statuses.push(eval_rules_file(&rules_file, &mut root, None)?);
    }

    assert_eq!(
        statuses,
        vec![Status::FAIL, Status::FAIL, Status::FAIL],
        "parameter, let and inline spellings of the same literal disagreed"
    );

    Ok(())
}

#[test]
fn vacuous_negated_comparison_does_not_satisfy_a_disjunction() -> Result<()> {
    // Regression test for a wrong PASS found by review.
    //
    // eval_conjunction_clauses treats PASS as short-circuiting (`continue
    // 'conjunction`) but SKIP as absorbing (`=> {}`). Reporting the vacuous
    // empty-denylist case as PASS therefore satisfied the whole `or` block and
    // abandoned the sibling disjunct unevaluated, so an unencrypted resource passed
    // the gate. Base 57bbdbf failed this ruleset correctly; an intermediate version
    // of this change passed it.
    //
    // Disjunct 1 is vacuously satisfied (empty denylist). Disjunct 2 is the real
    // check and genuinely fails. The rule must fail.
    let rules = r###"
    let denied = Resources[ Type == 'AWS::S3::Bucket' ].Properties.BucketName
    rule gate {
        Resources.V.Properties.Encrypted != %denied
        or
        Resources.V.Properties.Encrypted == true
    }
    "###;

    let input = r#"
    {
        Resources: {
            V: {
                Type: 'AWS::EC2::Volume',
                Properties: { Encrypted: false }
            }
        }
    }
    "#;

    assert_eq!(status_of(rules, input)?, Status::FAIL);
    Ok(())
}

#[test]
fn empty_reference_in_a_when_condition_does_not_disarm_the_block() -> Result<()> {
    // The critical case. If the empty-RHS condition FAILs, the gate is not-PASS and
    // eval_rule treats that as "rule does not apply", skipping the entire body --
    // so the real check silently stops running and the file exits 0. The condition
    // must stay a SKIP so the remaining conditions decide the gate.
    let rules = r###"
    let empt = Resources.*[ Type == 'AWS::EC2::Instance' ].Properties.Foo
    rule gated when Resources.*.Type IN %empt
                    Resources.*.Type == /S3/ {
        Resources.*.Properties.BucketName == /^secure-/
    }
    "###;
    // FAIL specifically. "Not PASS" would also admit SKIP, which is the exact
    // failure mode this test exists to catch: a SKIP here means the gate closed and
    // the body never ran, which is indistinguishable from a pass at the gate because
    // both exit 0. Asserting FAIL proves the body actually executed and rejected the
    // bucket name.
    assert_eq!(status_of(rules, ONE_BUCKET)?, Status::FAIL);
    Ok(())
}

#[test]
fn literal_lhs_against_empty_reference_fails_without_panicking() -> Result<()> {
    // A `let` literal on the left resolves to QueryResult::Literal, which three
    // reporters treat as unreachable inside a comparison record. Emitting a status
    // rather than a per-value comparison keeps this off that path.
    let rules = r###"
    let lit = "foo"
    let empt = Resources.*[ Type == 'AWS::EC2::Instance' ].Properties.Missing
    rule literal_lhs {
        %lit IN %empt
    }
    "###;
    // Must produce a verdict rather than panicking.
    let status = status_of(rules, ONE_BUCKET)?;
    assert_eq!(status, Status::FAIL);
    Ok(())
}

//
// Clause-level negation on a BINARY comparison.
//
// `not <query> == <value>` parses (parser.rs:969 accepts a leading not before the
// query) and is stored as GuardAccessClause::negation, but the binary evaluation
// path used to drop it, so the clause evaluated as its un-negated self -- the exact
// inverse of the author's intent -- while the report still displayed the `not`.
//
// The unary path was never affected; these tests cover the binary path and assert
// that an un-negated clause is unchanged.
//
fn eval_single_rule(rules: &str, resources: &str) -> Result<Status> {
    let value = PathAwareValue::try_from(serde_yaml::from_str::<serde_yaml::Value>(resources)?)?;
    let rules_file = RulesFile::try_from(rules)?;
    let mut eval = root_scope(&rules_file, Rc::new(value));
    eval_rules_file(&rules_file, &mut eval, None)
}

#[test]
fn negated_binary_clause_is_honored() -> Result<()> {
    let encrypted_false = r#"
    Resources:
      bucket:
        Type: AWS::S3::Bucket
        Properties:
          Encrypted: false
    "#;
    let encrypted_true = r#"
    Resources:
      bucket:
        Type: AWS::S3::Bucket
        Properties:
          Encrypted: true
    "#;

    // "It must NOT be the case that Encrypted == false."
    let negated = r###"
    rule encrypted_must_not_be_false {
        not Resources.*[ Type == 'AWS::S3::Bucket' ].Properties.Encrypted == false
    }
    "###;

    // Encrypted: false violates the intent -> FAIL.
    // Before the fix this returned PASS.
    assert_eq!(eval_single_rule(negated, encrypted_false)?, Status::FAIL);

    // Encrypted: true satisfies the intent -> PASS.
    // Before the fix this returned FAIL.
    assert_eq!(eval_single_rule(negated, encrypted_true)?, Status::PASS);

    Ok(())
}

#[test]
fn unnegated_binary_clause_is_unchanged() -> Result<()> {
    let encrypted_false = r#"
    Resources:
      bucket:
        Type: AWS::S3::Bucket
        Properties:
          Encrypted: false
    "#;
    let encrypted_true = r#"
    Resources:
      bucket:
        Type: AWS::S3::Bucket
        Properties:
          Encrypted: true
    "#;

    let plain = r###"
    rule encrypted_equals_false {
        Resources.*[ Type == 'AWS::S3::Bucket' ].Properties.Encrypted == false
    }
    "###;

    // The negated and un-negated forms must now disagree on every input; before the
    // fix they agreed, which is what proved the `not` was being dropped.
    assert_eq!(eval_single_rule(plain, encrypted_false)?, Status::PASS);
    assert_eq!(eval_single_rule(plain, encrypted_true)?, Status::FAIL);

    Ok(())
}

#[test]
fn negation_composes_with_operator_not_flag() -> Result<()> {
    let encrypted_false = r#"
    Resources:
      bucket:
        Type: AWS::S3::Bucket
        Properties:
          Encrypted: false
    "#;

    // Double negation: clause-level `not` plus the operator's own `!=`.
    // `not X != false` is equivalent to `X == false`, which holds here -> PASS.
    let double = r###"
    rule double_negation {
        not Resources.*[ Type == 'AWS::S3::Bucket' ].Properties.Encrypted != false
    }
    "###;
    assert_eq!(eval_single_rule(double, encrypted_false)?, Status::PASS);

    // Single negation via the operator alone is unaffected: `X != false` is false
    // here -> FAIL.
    let op_only = r###"
    rule op_not_only {
        Resources.*[ Type == 'AWS::S3::Bucket' ].Properties.Encrypted != false
    }
    "###;
    assert_eq!(eval_single_rule(op_only, encrypted_false)?, Status::FAIL);

    Ok(())
}

//
// `not <rule>` where the dependent rule SKIPped.
//
// In a rule BODY this is an assertion, and a SKIPped rule is not evidence, so it
// must not report compliance. It previously returned PASS -- and because the
// enclosing rule then reported PASS rather than SKIP, the output gave no hint that
// the check had never run.
//
// In a `when` CONDITION the same shape is intentional ("apply this rule when that
// other rule did not apply") and is covered by cross_rule_clause_when_checks, so
// that behavior is deliberately preserved here.
//
#[test]
fn negated_reference_to_skipped_rule_does_not_pass_in_rule_body() -> Result<()> {
    // `inner` SKIPs: its query filters on a resource type absent from the input.
    let rules = r###"
    rule inner {
        Resources.*[ Type == 'AWS::KMS::Key' ].Properties.KeyId exists
    }

    rule deny when Resources.*.Type exists {
        not inner
    }
    "###;

    let input = r#"
    {
        Resources: {
            bucket: {
                Type: 'AWS::S3::Bucket',
                Properties: { BucketName: "b" }
            }
        }
    }
    "#;

    let resources = PathAwareValue::try_from(input)?;
    let rules_file = RulesFile::try_from(rules)?;
    let mut root = root_scope(&rules_file, Rc::new(resources));
    let status = eval_rules_file(&rules_file, &mut root, None)?;

    // FAIL specifically. Before the fix this was PASS, manufactured from a dependent
    // rule that never ran. "Not PASS" would also admit SKIP, and a SKIP would mean
    // the negated reference had been made merely inert rather than fail-closed --
    // still exit 0, so still a gate bypass. FAIL is the property that matters.
    assert_eq!(status, Status::FAIL);

    Ok(())
}

#[test]
fn negated_reference_to_skipped_rule_still_gates_a_when_condition() -> Result<()> {
    // Same shape, but the negated reference is a `when` condition rather than a body
    // assertion. Gating here is intentional: the guarded block should still run.
    let rules = r###"
    rule inner {
        Resources.*[ Type == 'AWS::KMS::Key' ].Properties.KeyId exists
    }

    rule gated when not inner {
        Resources.*.Type exists
    }
    "###;

    let input = r#"
    {
        Resources: {
            bucket: {
                Type: 'AWS::S3::Bucket',
                Properties: { BucketName: "b" }
            }
        }
    }
    "#;

    let resources = PathAwareValue::try_from(input)?;
    let rules_file = RulesFile::try_from(rules)?;
    let mut root = root_scope(&rules_file, Rc::new(resources));
    let status = eval_rules_file(&rules_file, &mut root, None)?;

    // The gate opens and the body (`Type exists`) holds, so this passes.
    assert_eq!(status, Status::PASS);

    Ok(())
}

//
// Empty collection on the left-hand side of a comparison.
//
// `flattened`/`selected` expand a list into its elements, so an empty list contributes
// none. The comparison loop then pushed zero results and the enclosing fold read an empty
// result vector as "nothing to check" and reported PASS -- so `Tags == 'Owner'` against
// `Tags: []` was reported as *compliant*, not as not-applicable. It claimed to have
// verified a property it never compared, while the same rule against a missing `Tags`
// correctly failed. The weaker input was treated more leniently.
//

#[test]
fn an_empty_collection_fails_a_positive_comparison_as_an_assertion() -> Result<()> {
    let rules = r###"
    rule tags_must_be_owner {
        Resources.*[ Type == 'AWS::S3::Bucket' ].Properties.Tags == 'Owner'
    }
    "###;

    let input = r#"
    {
        Resources: {
            bucket: {
                Type: 'AWS::S3::Bucket',
                Properties: { Name: "publicbucket", Tags: [] }
            }
        }
    }
    "#;

    let resources = PathAwareValue::try_from(input)?;
    let rules_file = RulesFile::try_from(rules)?;
    let mut root = root_scope(&rules_file, Rc::new(resources));
    let status = eval_rules_file(&rules_file, &mut root, None)?;

    // FAIL, not SKIP. SKIP would also exit 0, which is operationally identical to the
    // PASS being fixed: the clause would still go unenforced at the gate.
    assert_eq!(status, Status::FAIL);

    Ok(())
}

/// The cardinality that a previous attempt at this fix silently failed to handle.
///
/// That attempt tested `lhs_flattened.is_empty()` -- the flattening of *all* query
/// results together -- so a single sibling with a non-empty list made it non-empty and the
/// guard never ran. It fired on single-resource templates and did nothing on the mixed
/// templates that are the common real-world shape, while passing every test written for
/// it, all of which had one resource.
///
/// This asserts the per-result behaviour directly: `BucketFull` genuinely satisfies the
/// rule, so the file-level FAIL can only come from `BucketEmpty` being caught.
#[test]
fn an_empty_collection_is_caught_even_when_a_sibling_resource_satisfies_the_rule() -> Result<()> {
    let rules = r###"
    rule tags_must_be_owner {
        Resources.*[ Type == 'AWS::S3::Bucket' ].Properties.Tags == 'Owner'
    }
    "###;

    let input = r#"
    {
        Resources: {
            bucketEmpty: {
                Type: 'AWS::S3::Bucket',
                Properties: { Name: "publicbucket", Tags: [] }
            },
            bucketFull: {
                Type: 'AWS::S3::Bucket',
                Properties: { Name: "privatebucket", Tags: ['Owner'] }
            }
        }
    }
    "#;

    let resources = PathAwareValue::try_from(input)?;
    let rules_file = RulesFile::try_from(rules)?;
    let mut root = root_scope(&rules_file, Rc::new(resources));
    let status = eval_rules_file(&rules_file, &mut root, None)?;

    assert_eq!(status, Status::FAIL);

    Ok(())
}

/// The counterpart, and the reason the fix is resolved by role rather than in the
/// comparator.
///
/// `eval_rule` treats any non-PASS condition as "this rule does not apply" and drops the
/// guarded body (the `if status != Status::PASS` branch in `eval_rule`). A fix that failed
/// the empty comparison unconditionally
/// would therefore turn this blocked template into a passing one -- trading one unenforced
/// clause for an entire disarmed block. That is exactly how an earlier attempt regressed,
/// and it exits 0, so nothing downstream notices.
#[test]
fn an_empty_collection_in_a_when_condition_does_not_disarm_the_guarded_block() -> Result<()> {
    let rules = r###"
    rule name_must_be_safe when Resources.*[ Type == 'AWS::S3::Bucket' ].Properties.Tags == 'Owner' {
        Resources.*[ Type == 'AWS::S3::Bucket' ].Properties.Name != 'publicbucket'
    }
    "###;

    let input = r#"
    {
        Resources: {
            bucket: {
                Type: 'AWS::S3::Bucket',
                Properties: { Name: "publicbucket", Tags: [] }
            }
        }
    }
    "#;

    let resources = PathAwareValue::try_from(input)?;
    let rules_file = RulesFile::try_from(rules)?;
    let mut root = root_scope(&rules_file, Rc::new(resources));
    let status = eval_rules_file(&rules_file, &mut root, None)?;

    // FAIL because the body ran and `publicbucket` violated it. A SKIP here would mean
    // the gate closed and the violation went unreported.
    assert_eq!(status, Status::FAIL);

    Ok(())
}

/// The same defect written backwards, which the first version of this fix left open.
///
/// `'Owner' == Properties.Tags` is legal Guard and means what `Properties.Tags == 'Owner'`
/// means, but it takes a different comparator arm: the literal is the left side, so the
/// empty list arrives as the *right* operand and a separate loop pushes nothing. Fixing
/// only the first spelling left the wrong PASS reachable by anyone who happened to write
/// the operands in the other order -- measured 0 with the first fix in place, 19 now.
///
/// Found by asking which arms of `EqOperation` the fix did not touch, rather than by a
/// failing test. The asymmetry is invisible from the rule author's side.
#[test]
fn an_empty_collection_fails_when_it_is_the_right_hand_operand() -> Result<()> {
    let rules = r###"
    let expected = 'Owner'
    rule tags_must_be_owner {
        %expected == Resources.*[ Type == 'AWS::S3::Bucket' ].Properties.Tags
    }
    "###;

    let input = r#"
    {
        Resources: {
            bucket: {
                Type: 'AWS::S3::Bucket',
                Properties: { Name: "publicbucket", Tags: [] }
            }
        }
    }
    "#;

    let resources = PathAwareValue::try_from(input)?;
    let rules_file = RulesFile::try_from(rules)?;
    let mut root = root_scope(&rules_file, Rc::new(resources));
    let status = eval_rules_file(&rules_file, &mut root, None)?;

    assert_eq!(status, Status::FAIL);

    Ok(())
}

/// The gate counterpart of the mirrored form. Same requirement as the un-mirrored gate
/// test: the guarded body must still run, so the violation is still reported.
#[test]
fn a_mirrored_empty_collection_in_a_when_condition_does_not_disarm_the_block() -> Result<()> {
    let rules = r###"
    let expected = 'Owner'
    rule name_must_be_safe when %expected == Resources.*[ Type == 'AWS::S3::Bucket' ].Properties.Tags {
        Resources.*[ Type == 'AWS::S3::Bucket' ].Properties.Name != 'publicbucket'
    }
    "###;

    let input = r#"
    {
        Resources: {
            bucket: {
                Type: 'AWS::S3::Bucket',
                Properties: { Name: "publicbucket", Tags: [] }
            }
        }
    }
    "#;

    let resources = PathAwareValue::try_from(input)?;
    let rules_file = RulesFile::try_from(rules)?;
    let mut root = root_scope(&rules_file, Rc::new(resources));
    let status = eval_rules_file(&rules_file, &mut root, None)?;

    assert_eq!(status, Status::FAIL);

    Ok(())
}

/// A negated comparison over an empty collection is vacuously true, so it must not fail.
///
/// The empty-LHS record is emitted before the per-value inversion, so a FAIL raised there
/// is one the `not` can never reach. Without the `!cmp.1` guard this reports FAIL, which
/// is a wrong FAIL: `not (Tags == 'Owner')` is satisfied when there are no tags at all.
#[test]
fn a_negated_comparison_over_an_empty_collection_does_not_fail() -> Result<()> {
    let rules = r###"
    rule r {
        Resources.*[ Type == 'AWS::S3::Bucket' ] {
            not Properties.Tags == 'Owner'
        }
    }
    "###;

    let input = r#"
    {
        Resources: {
            bucket: {
                Type: 'AWS::S3::Bucket',
                Properties: { Name: "n", Tags: [] }
            }
        }
    }
    "#;

    let resources = PathAwareValue::try_from(input)?;
    let rules_file = RulesFile::try_from(rules)?;
    let mut root = root_scope(&rules_file, Rc::new(resources));
    let status = eval_rules_file(&rules_file, &mut root, None)?;

    // SKIP. Not FAIL is the property this test guards, and it is real: `not (Tags ==
    // 'Owner')` over an empty collection is vacuously true, so the clause must not be blamed
    // for having nothing to compare.
    //
    // This asserted PASS until the fold moved onto `Outcome`, and the comment here recorded
    // that PASS as "a known defect" pointing at
    // `a_vacuous_negated_clause_does_not_absorb_a_disjunction`. That is why: PASS
    // short-circuits `eval_conjunction_clauses`, so a vacuously-satisfied disjunct satisfied
    // an entire `or` and abandoned its siblings. SKIP is absorbed by the disjunction fold
    // instead, so the siblings still run, and that reproduction now passes.
    //
    // Reporting SKIP was implemented and reverted twice before, both times because it closed
    // a `when` gate -- the second time through a named rule. It holds now because the arm is
    // split by role: an assertion contributes SKIP, a gate still contributes the PASS that
    // keeps it open, and `(rule, role)` keying means the role survives the named-rule
    // boundary.
    //
    // The user-visible effect is a reporting change, not an exit-code one: this rule moves
    // from `compliant` to `not_applicable`, and both exit 0. Arguably the honest answer --
    // nothing was verified.
    //
    // Exact rather than `assert_ne!(FAIL)` so any future change to this status is visible
    // instead of silently admitted.
    assert_eq!(status, Status::SKIP);

    Ok(())
}

/// A vacuously-satisfied disjunct must not absorb an `or`. Pre-existing in v3.2.0.
///
/// `eval_conjunction_clauses` short-circuits on PASS (`continue 'conjunction'`) but absorbs
/// SKIP (`=> {}`), so a vacuously-satisfied first disjunct reported as PASS satisfied the
/// whole `or` and its siblings never ran. Here the sibling is a real failing check, so the
/// file exited 0 while `Name` was `publicbucket`, reporting `"compliant"` rather than
/// `"not_applicable"`.
///
/// Three fixes for it were worse than the defect before this one stuck. Returning SKIP from
/// the empty-collection arm:
///
/// - Unconditionally: closed the direct `when Tags != 'Owner'` gate, 19 -> 0.
/// - Narrowed to `role.is_strict()`: still closed a gate reached through a *named rule*,
///   19 -> 0, because `rule_status` evaluated every named rule's body with
///   `ClauseRole::Assertion` regardless of the reference site and cached the status per rule
///   name, so the poisoned SKIP was reused by later references.
/// - Applied to both polarities while converting the fold: closed five gates at once.
///
/// What made it hold: `rule_status` now carries the reference site's role and keys its cache
/// on `(rule, role)`, so the role survives the named-rule boundary; and the empty-collection
/// arm is split by that role -- an assertion contributes SKIP so it cannot absorb a
/// disjunction, a gate still contributes the PASS that keeps it open. The fold itself runs
/// through `Outcome::all`/`Outcome::any`, whose identity is `NotApplicable`, so a fold over
/// zero elements no longer reports "satisfied".
///
/// Known weakness of this fixture, unchanged: see the note below on `assert_eq!(FAIL)`.
///
/// Known weakness, recorded rather than fixed. `assert_eq!(FAIL)` is satisfied by *any* part
/// of the rule failing, so a one-line template edit reaches green without a fix: change
/// `Tags: []` to `Tags: ['Owner']` and the second disjunct performs a real comparison, really
/// fails, and the test passes while the vacuous-absorption defect is untouched. The rule is
/// named `vacuous_ne_absorbs_or`, so the signal to a reader editing the template is at least
/// present. Left as-is because tightening it would need the same liveness-plus-absolute shape
/// as `ordering_operators_do_not_certify_an_empty_collection`, and the fixture here has only
/// one meaningful data shape to vary — the empty list is the whole point of it.
///
/// Note for anyone running `--ignored`: exactly one test is ignored in this crate now, and it
/// is not ours. `test_string_in_comparison` is an upstream failure parked in 2023 (commit
/// `1aca9003`, verified by `git blame`), failing identically on the pre-branch tree.
///
/// The other four that were ignored here all now pass and are asserted normally: this one,
/// `a_named_rule_gate_does_not_drop_a_satisfiable_body`,
/// `in_does_not_certify_an_empty_collection` and
/// `ordering_operators_do_not_certify_an_empty_collection`. The last two keep companion
/// controls pinning that populated collections still decide correctly, because a fix that
/// stopped certifying empty collections by stopping evaluation altogether would satisfy them
/// otherwise.
#[test]
fn a_vacuous_negated_clause_does_not_absorb_a_disjunction() -> Result<()> {
    let rules = r###"
    rule vacuous_ne_absorbs_or {
        Resources.*[ Type == 'AWS::S3::Bucket' ].Properties.Tags != 'Owner'
        or
        Resources.*[ Type == 'AWS::S3::Bucket' ].Properties.Name == 'safebucket'
    }
    "###;

    let input = r#"
    {
        Resources: {
            bucket: {
                Type: 'AWS::S3::Bucket',
                Properties: { Name: "publicbucket", Tags: [] }
            }
        }
    }
    "#;

    let resources = PathAwareValue::try_from(input)?;
    let rules_file = RulesFile::try_from(rules)?;
    let mut root = root_scope(&rules_file, Rc::new(resources));
    let status = eval_rules_file(&rules_file, &mut root, None)?;

    // FAIL, from the sibling disjunct that actually got evaluated.
    assert_eq!(status, Status::FAIL);

    Ok(())
}

/// A gate reached through a NAMED RULE, which is the shape that caught the reverted fix.
///
/// Every gate fixture in this file spells the condition inline (`rule r when <clause> {}`),
/// where `eval_when_clause` hardcodes `ClauseRole::Gate`. This one references a rule
/// instead, and that path is different in a way no inline fixture can show:
/// `rule_status` evaluates a named rule's body with `ClauseRole::Assertion`
/// whatever the reference site is, and caches the result per rule name.
///
/// So a fix that keys on `role.is_strict()` inside the clause sees "assertion" even though
/// the rule is being used as a gate. The reverted `EmptyQueryResult(SKIP)` did exactly
/// that: `vac_ne` returned SKIP, `eval_rule` read the non-PASS condition as "does not
/// apply", and the guarded body was dropped -- 19 -> 0, with the violating `publicbucket`
/// never examined and both rules reported `not_applicable`.
///
/// Guards the revert. If someone re-lands a SKIP-based fix without threading the
/// reference-site role through rule evaluation, this fails.
#[test]
fn a_vacuous_negation_inside_a_named_rule_does_not_close_the_gate_referencing_it() -> Result<()> {
    let rules = r###"
    rule vac_ne {
        Resources.*[ Type == 'AWS::S3::Bucket' ].Properties.Tags != 'Owner'
    }

    rule body_bad when vac_ne {
        Resources.*[ Type == 'AWS::S3::Bucket' ].Properties.Name == 'privatebucket'
    }
    "###;

    let input = r#"
    {
        Resources: {
            bucketEmptyTags: {
                Type: 'AWS::S3::Bucket',
                Properties: { Name: "publicbucket", Tags: [] }
            }
        }
    }
    "#;

    let resources = PathAwareValue::try_from(input)?;
    let rules_file = RulesFile::try_from(rules)?;
    let mut root = root_scope(&rules_file, Rc::new(resources));
    let status = eval_rules_file(&rules_file, &mut root, None)?;

    // FAIL because the gate opened and the body rejected `publicbucket`. SKIP would mean
    // the gate closed and the violation went unreported at exit 0.
    assert_eq!(status, Status::FAIL);

    Ok(())
}

/// The counterpart that makes the fix above a role split rather than a status change.
///
/// A negated comparison over an empty collection opens its `when` gate *because* it folds
/// to PASS. Reporting SKIP for it unconditionally -- which an earlier version of this fix
/// did -- closes the gate, and `eval_rule` then drops every check in the guarded body
/// (the `if status != Status::PASS` branch in `eval_rule`). Measured at that point: exit
/// 19 -> 0, the rule moved to
/// `not_applicable`, and the violating `publicbucket` was never examined. A worse defect
/// than the wrong PASS being fixed, and it exits 0 so nothing downstream notices.
///
/// So the vacuous PASS is load-bearing in a gate and a defect in a body. `ClauseRole`
/// carries exactly that asymmetry, and `vacuously_satisfied` is only set when
/// `role.is_strict()`.
#[test]
fn a_vacuous_negated_gate_still_opens_and_runs_its_body() -> Result<()> {
    let rules = r###"
    rule gated_by_vacuous_ne when Resources.*[ Type == 'AWS::S3::Bucket' ].Properties.Tags != 'Owner' {
        Resources.*[ Type == 'AWS::S3::Bucket' ].Properties.Name == 'privatebucket'
    }
    "###;

    let input = r#"
    {
        Resources: {
            bucket: {
                Type: 'AWS::S3::Bucket',
                Properties: { Name: "publicbucket", Tags: [] }
            }
        }
    }
    "#;

    let resources = PathAwareValue::try_from(input)?;
    let rules_file = RulesFile::try_from(rules)?;
    let mut root = root_scope(&rules_file, Rc::new(resources));
    let status = eval_rules_file(&rules_file, &mut root, None)?;

    // FAIL because the gate opened and the body ran. SKIP here would mean the gate closed
    // and the violation went unreported.
    assert_eq!(status, Status::FAIL);

    Ok(())
}

/// The SKIP return is guarded on `statues.is_empty()`, and this is why.
///
/// `vacuously_satisfied` is function-scoped: any iteration can set it, and it is read once
/// at the end. When one query result is a vacuous negation and another produces a real
/// comparison, the flag must be ignored -- otherwise one resource with `Tags: []` would
/// suppress a genuine collision on its sibling and take the whole clause to SKIP.
///
/// Here `BucketEmpty` contributes the vacuous case and `BucketOwner` genuinely collides
/// with `!= 'Owner'`. Verified from the JSON that both the real collision on
/// `/Resources/BucketOwner/Properties/Tags/0` *and* the sibling disjunct on
/// `/Resources/BucketEmpty/Properties/Name` were evaluated, so the vacuous result neither
/// suppressed the failure nor absorbed the disjunction.
#[test]
fn a_vacuous_negation_does_not_suppress_a_real_result_from_a_sibling() -> Result<()> {
    let rules = r###"
    rule mixed_ne {
        Resources.*[ Type == 'AWS::S3::Bucket' ].Properties.Tags != 'Owner'
        or
        Resources.*[ Type == 'AWS::S3::Bucket' ].Properties.Name == 'safebucket'
    }
    "###;

    let input = r#"
    {
        Resources: {
            bucketEmpty: {
                Type: 'AWS::S3::Bucket',
                Properties: { Name: "publicbucket", Tags: [] }
            },
            bucketOwner: {
                Type: 'AWS::S3::Bucket',
                Properties: { Name: "alsopublic", Tags: ['Owner'] }
            }
        }
    }
    "#;

    let resources = PathAwareValue::try_from(input)?;
    let rules_file = RulesFile::try_from(rules)?;
    let mut root = root_scope(&rules_file, Rc::new(resources));
    let status = eval_rules_file(&rules_file, &mut root, None)?;

    assert_eq!(status, Status::FAIL);

    Ok(())
}

/// A vacuous negation nested inside a `when` block must not leak the SKIP outward.
///
/// The outer gate is unrelated to the empty collection, and the inner gate is the vacuous
/// negation. If the vacuous SKIP closed the inner gate, `privatebucket` would go unchecked
/// and the file would exit 0 -- the same silent-drop shape the top-level gate test pins,
/// one level down, where a `when` inside a `when` composes two gating decisions.
#[test]
fn a_vacuous_negation_nested_in_a_when_block_still_runs_the_inner_body() -> Result<()> {
    let rules = r###"
    rule nested_ne when Resources.*[ Type == 'AWS::S3::Bucket' ] !empty {
        when Resources.*[ Type == 'AWS::S3::Bucket' ].Properties.Tags != 'Owner' {
            Resources.*[ Type == 'AWS::S3::Bucket' ].Properties.Name == 'privatebucket'
        }
    }
    "###;

    let input = r#"
    {
        Resources: {
            bucket: {
                Type: 'AWS::S3::Bucket',
                Properties: { Name: "publicbucket", Tags: [] }
            }
        }
    }
    "#;

    let resources = PathAwareValue::try_from(input)?;
    let rules_file = RulesFile::try_from(rules)?;
    let mut root = root_scope(&rules_file, Rc::new(resources));
    let status = eval_rules_file(&rules_file, &mut root, None)?;

    assert_eq!(status, Status::FAIL);

    Ok(())
}

/// A non-negated parameterized gate that SKIPs must not poison the rest of the `when`.
///
/// `eval_parameterized_rule_call` returned the invoked rule's status through a `_` arm that
/// converted any non-PASS, non-strict-SKIP result into FAIL for a non-negated call. With a
/// single condition that is invisible: FAIL and SKIP both make `eval_rule` treat the rule as
/// inapplicable and drop the guarded body.
///
/// It becomes visible with two conditions, which is why this test has two.
/// `eval_conjunction_clauses` absorbs SKIP (`Status::SKIP => {}`) but counts a FAIL, so the
/// inapplicable gate returning FAIL dropped a body that the passing sibling condition should
/// have kept enforced. `ClauseRole::Gate` is documented as "the block it guards is still
/// decided by the remaining conditions", so FAIL here defeated the role propagation.
///
/// The fixture: `no_such_type` invokes a parameterized rule whose query selects nothing, so it
/// SKIPs; `bucket_exists` PASSes; and the guarded body requires a Name the template violates.
/// The body must therefore run and the file must FAIL. Before the fix it exited 0 with the
/// body dropped, which is the wrong-PASS shape this whole branch is about.
#[test]
fn a_skipping_parameterized_gate_does_not_drop_a_body_its_sibling_enforces() -> Result<()> {
    // `relevant` must SKIP, not FAIL, or this fixture tests the wrong arm. A binary
    // comparison whose left-hand query selects nothing yields `EvalResult::Skip`
    // (`CmpOperator::compare`'s `lhs.is_empty()` guard), so the rule SKIPs. `!empty` would
    // FAIL instead -- an unresolved query is EMPTY, so `!empty` is false -- which reaches the
    // `_` arm by a different route and would not exercise the SKIP path at all.
    let rules = r###"
    rule relevant(ty) {
        Resources.*[ Type == %ty ].Properties.Name == 'anything'
    }
    rule guarded when relevant('AWS::Nonexistent::Type') Resources.*[ Type == 'AWS::S3::Bucket' ] !empty {
        Resources.*[ Type == 'AWS::S3::Bucket' ].Properties.Name == 'safebucket'
    }
    "###;

    let input = r#"
    {
        Resources: {
            b: {
                Type: 'AWS::S3::Bucket',
                Properties: { Name: "publicbucket" }
            }
        }
    }
    "#;

    let resources = PathAwareValue::try_from(input)?;
    let rules_file = RulesFile::try_from(rules)?;
    let mut root = root_scope(&rules_file, Rc::new(resources));
    let status = eval_rules_file(&rules_file, &mut root, None)?;

    // FAIL: the parameterized gate does not apply, the sibling condition passes, so the body
    // runs and catches `publicbucket`. SKIP here would mean the inapplicable gate closed the
    // whole `when` and the violation went unreported while the process exited 0.
    assert_eq!(
        status,
        Status::FAIL,
        "a parameterized gate that did not apply suppressed a body its sibling condition \
         should have kept enforced"
    );

    Ok(())
}

/// `EMPTY` and `!EMPTY` on a boolean are an incompatible-type error, not a silent pass.
///
/// The Bool arm of `element_empty_operation` computed `(*boolean).to_string().is_empty()`.
/// Neither "true" nor "false" is ever the empty string, so EMPTY on a boolean was
/// unconditionally false and `!EMPTY` unconditionally true: a clause that reads like a check
/// and cannot fail for any input. A rule author writing `Properties.Enabled !EMPTY` got a
/// green check that verified nothing.
///
/// Removing the arm lets a boolean reach the same `IncompatibleError` every other unsupported
/// type already reached, which surfaces the mistake with the offending path instead of
/// certifying it.
///
/// All four combinations are covered because the two axes fail differently. The old code made
/// `!EMPTY` a silent *pass* and `EMPTY` a silent *fail*, so a test on one polarity alone would
/// have left the other spelling unguarded, and `true` versus `false` is exactly the axis the
/// old implementation was insensitive to -- asserting only one value would not have
/// distinguished "handled" from "ignored".
#[test]
fn boolean_empty_is_an_incompatible_type() -> Result<()> {
    for value in ["true", "false"] {
        for comparator in ["EMPTY", "!EMPTY"] {
            let rules = format!(
                r###"
                rule flag_check {{
                    Resources.*[ Type == 'AWS::S3::Bucket' ].Properties.Enabled {comparator}
                }}
                "###
            );
            let input = format!(
                r#"
                {{
                    Resources: {{
                        b: {{
                            Type: 'AWS::S3::Bucket',
                            Properties: {{ Enabled: {value} }}
                        }}
                    }}
                }}
                "#
            );

            let resources = PathAwareValue::try_from(input.as_str())?;
            let rules_file = RulesFile::try_from(rules.as_str())?;
            let mut root = root_scope(&rules_file, Rc::new(resources));
            let result = eval_rules_file(&rules_file, &mut root, None);

            let err = match result {
                Err(e) => e,
                Ok(status) => panic!(
                    "`Enabled {}` on the boolean {} returned {:?} instead of an \
                     incompatible-type error. Before this fix `!EMPTY` passed for every \
                     boolean and `EMPTY` failed for every boolean, in both cases without \
                     comparing anything.",
                    comparator, value, status
                ),
            };

            let message = format!("{err}");
            assert!(
                message.contains("EMPTY"),
                "expected the incompatible-type error to name the EMPTY operation so the \
                 author can find the clause, got: {}",
                message
            );
        }
    }

    Ok(())
}

/// Exhaustive coverage of the empty-collection decision surface, with the count asserted.
///
/// The empty-collection work on this branch is combinatorial: what a clause reports depends on
/// the operator, the polarity, whether the clause is an assertion or a gate, and whether the
/// left-hand collection is empty, satisfying or violating. Individual tests pin individual
/// cells, and every defect found on this branch was a cell nobody had looked at -- `IN` and the
/// four ordering operators certifying an empty collection, the mirrored spelling, the negated
/// assertion absorbing a disjunction, the gate that dropped its body.
///
/// So the surface is enumerated here rather than sampled. The rows are generated by nested
/// loops over the four axes, not written out, which makes a missing row impossible rather than
/// unlikely; and `ROWS_EXPECTED` is asserted against the product of the axis lengths so that
/// dropping an axis value fails instead of quietly shrinking coverage.
///
/// The expected value for each cell comes from `expected_status`, which is written as the
/// *specification* -- role and polarity and emptiness composed from first principles -- rather
/// than from observed behaviour. Two properties of that specification are asserted separately
/// below, because they are the invariants the defects violated:
///
/// - role is irrelevant when the collection is populated. `ClauseRole` exists to decide what an
///   *unevaluatable* clause reports; a real comparison must not depend on it. A fix that made
///   populated comparisons role-sensitive would be a silent behaviour change.
/// - an empty collection never reports PASS as an assertion. That is the wrong-PASS class in
///   one line: `Ports <= 100` and `Ports > 100` are exact logical negations and both used to
///   certify `Ports: []`.
#[test]
fn the_empty_collection_decision_surface_is_covered_exhaustively() -> Result<()> {
    // (label, clause tail, satisfying value, violating value). Numeric fixtures throughout:
    // the ordering operators are the reason this matrix exists and string ordering would
    // confound them -- "8080" < "100" lexicographically but not numerically.
    const OPERATORS: [(&str, &str, &str, &str); 6] = [
        ("Eq", "== 50", "50", "8080"),
        ("In", "IN [50]", "50", "8080"),
        ("Lt", "< 100", "50", "8080"),
        ("Le", "<= 100", "50", "8080"),
        ("Gt", "> 100", "8080", "50"),
        ("Ge", ">= 100", "8080", "50"),
    ];
    const NEGATED: [bool; 2] = [false, true];
    const AS_GATE: [bool; 2] = [false, true];
    const LHS: [&str; 3] = ["empty", "satisfying", "violating"];
    const ROWS_EXPECTED: usize = 6 * 2 * 2 * 3;

    /// The specification, not a transcript of current behaviour.
    ///
    /// An assertion reports the comparison: PASS when it holds, FAIL when it does not. Over an
    /// empty collection there is nothing to compare, so a positive assertion FAILs -- it
    /// claimed a property of every element and cannot establish it over none -- and a negated
    /// one is vacuously true but not evidence, so it reports SKIP and cannot absorb a
    /// disjunction.
    ///
    /// A gate is observed through whether its guarded body ran, so the value here is PASS when
    /// the gate opens and SKIP when it closes. It opens when its condition holds, and over an
    /// empty collection it opens in either polarity: `eval_rule` reads any non-PASS condition
    /// as "rule does not apply" and drops every check inside, so a gate with nothing to
    /// compare must not close.
    fn expected_status(negated: bool, as_gate: bool, lhs: &str) -> Status {
        let condition_holds = match lhs {
            "empty" => {
                return if as_gate {
                    Status::PASS
                } else if negated {
                    Status::SKIP
                } else {
                    Status::FAIL
                }
            }
            "satisfying" => !negated,
            "violating" => negated,
            other => unreachable!("unknown lhs state {other}"),
        };
        if as_gate {
            if condition_holds {
                Status::PASS
            } else {
                Status::SKIP
            }
        } else if condition_holds {
            Status::PASS
        } else {
            Status::FAIL
        }
    }

    let query = "Resources.*[ Type == 'AWS::EC2::SecurityGroup' ].Properties.Ports";
    let mut rows = 0usize;
    let mut failures: Vec<String> = vec![];

    for (op_label, tail, satisfying, violating) in OPERATORS {
        for negated in NEGATED {
            for as_gate in AS_GATE {
                for lhs in LHS {
                    rows += 1;

                    let ports = match lhs {
                        "empty" => "[]".to_string(),
                        "satisfying" => format!("[{satisfying}]"),
                        "violating" => format!("[{violating}]"),
                        other => unreachable!("unknown lhs state {other}"),
                    };
                    let clause = format!("{}{query} {tail}", if negated { "not " } else { "" });

                    // The gate spelling wraps the clause in a `when` whose body is
                    // unconditionally true for the selected resource, so the observable is
                    // purely whether the body ran.
                    let rules = if as_gate {
                        format!(
                            r###"
                            rule gated when {clause} {{
                                Resources.*[ Type == 'AWS::EC2::SecurityGroup' ].Type == 'AWS::EC2::SecurityGroup'
                            }}
                            "###
                        )
                    } else {
                        format!(
                            r###"
                            rule asserted {{
                                {clause}
                            }}
                            "###
                        )
                    };

                    let input = format!(
                        r#"
                        {{
                            Resources: {{
                                sg: {{
                                    Type: 'AWS::EC2::SecurityGroup',
                                    Properties: {{ Ports: {ports} }}
                                }}
                            }}
                        }}
                        "#
                    );

                    let actual = status_of(rules.as_str(), input.as_str())?;
                    let want = expected_status(negated, as_gate, lhs);
                    if actual != want {
                        failures.push(format!(
                            "  {op_label:3} negated={negated:5} gate={as_gate:5} lhs={lhs:10} \
                             want {want:?}, got {actual:?}"
                        ));
                    }
                }
            }
        }
    }

    assert_eq!(
        rows, ROWS_EXPECTED,
        "the matrix enumerated {rows} rows but the axes multiply to {ROWS_EXPECTED}; an axis \
         value was dropped and coverage shrank silently"
    );

    assert!(
        failures.is_empty(),
        "{}/{} cells of the empty-collection decision surface disagree with the \
         specification:\n{}",
        failures.len(),
        rows,
        failures.join("\n")
    );

    Ok(())
}

/// A populated comparison must not depend on whether the clause is an assertion or a gate.
///
/// `ClauseRole` exists to decide what an *unevaluatable* clause reports -- `Outcome::to_status`
/// branches on it for exactly one variant. A real comparison over real values must give the
/// same verdict either way, and this is asserted separately from the matrix because the matrix
/// would still pass if a future change made populated comparisons role-sensitive in a way its
/// specification happened to encode.
#[test]
fn role_does_not_change_a_populated_comparison() -> Result<()> {
    const OPERATORS: [(&str, &str, &str); 6] = [
        ("Eq", "== 50", "50"),
        ("In", "IN [50]", "50"),
        ("Lt", "< 100", "50"),
        ("Le", "<= 100", "50"),
        ("Gt", "> 100", "8080"),
        ("Ge", ">= 100", "8080"),
    ];
    let query = "Resources.*[ Type == 'AWS::EC2::SecurityGroup' ].Properties.Ports";

    for (label, tail, satisfying) in OPERATORS {
        for negated in [false, true] {
            let clause = format!("{}{query} {tail}", if negated { "not " } else { "" });
            let input = format!(
                r#"
                {{
                    Resources: {{
                        sg: {{
                            Type: 'AWS::EC2::SecurityGroup',
                            Properties: {{ Ports: [{satisfying}] }}
                        }}
                    }}
                }}
                "#
            );

            let as_assertion = status_of(
                format!("rule asserted {{ {clause} }}").as_str(),
                input.as_str(),
            )?;
            let as_gate = status_of(
                format!(
                    r###"rule gated when {clause} {{
                        Resources.*[ Type == 'AWS::EC2::SecurityGroup' ].Type == 'AWS::EC2::SecurityGroup'
                    }}"###
                )
                .as_str(),
                input.as_str(),
            )?;

            // A satisfying populated comparison holds, so a positive clause passes in both
            // spellings. A negated one does not hold: as an assertion that is FAIL, and as a
            // gate it closes, which is SKIP. Both are the comparison's verdict rather than a
            // role-dependent reinterpretation of it.
            let (want_assertion, want_gate) = if negated {
                (Status::FAIL, Status::SKIP)
            } else {
                (Status::PASS, Status::PASS)
            };

            assert_eq!(
                as_assertion, want_assertion,
                "{label} negated={negated} as an assertion over populated data"
            );
            assert_eq!(
                as_gate, want_gate,
                "{label} negated={negated} as a gate over populated data"
            );
        }
    }

    Ok(())
}

/// The four cells of zero-selection, pinned so the asymmetry is intentional not incidental.
///
/// Measured on a template containing no S3 bucket at all, so the bucket query selects nothing:
///
///     <query> == %lit     SKIP
///     %lit == <query>      FAIL   <- the asymmetry, and only here
///     <query> != %lit     SKIP
///     %lit != <query>      SKIP
///
/// A comment in `CmpOperator::compare` used to call the FAIL a contradiction of
/// docs/QUERY_AND_FILTERING.md:222, which says a query matching nothing makes block level
/// clauses skip. On the measurements that claim is too strong, twice over: the doc sentence is
/// about clauses whose *subject* is the empty query, and the disagreement is confined to the
/// positive spelling -- both negated forms already agree at SKIP.
///
/// The reading that makes all four cells right is that Guard's comparison is not
/// operand-symmetric even when the operator is. The left side is the subject being checked and
/// the right side is the reference it is checked against:
///
/// - no subject values: there is nothing to assert, so the rule does not apply. SKIP. This is
///   what lets one ruleset run against templates that do not all contain the resource type,
///   which is the case the doc sentence describes.
/// - no reference values: the assertion is that the subject is among the references, and
///   nothing is among zero references, so it cannot hold. FAIL. Making this SKIP instead is
///   how an allowlist that resolved empty used to report compliance, which
///   `positive_comparison_against_empty_reference_fails` exists to prevent.
///
/// So this is pinned rather than fixed, and deliberately: "fix the asymmetry" means picking one
/// of those two to break. Making the mirrored form SKIP reintroduces the empty-allowlist wrong
/// PASS; making the forward form FAIL breaks every ruleset run against a template lacking the
/// resource type. v3.2.0 exits 0 for both spellings, so it had the wrong PASS in both.
#[test]
fn zero_selection_is_asymmetric_by_operand_role() -> Result<()> {
    let no_bucket =
        r#"{ Resources: { q: { Type: 'AWS::SQS::Queue', Properties: { Name: "q" } } } }"#;
    let one_bucket = r#"
    { Resources: { b: { Type: 'AWS::S3::Bucket', Properties: { Tags: 'Owner' } } } }
    "#;
    let query = "Resources.*[ Type == 'AWS::S3::Bucket' ].Properties.Tags";

    for (label, clause, want) in [
        (
            "forward positive",
            format!("{query} == %expected"),
            Status::SKIP,
        ),
        (
            "mirrored positive",
            format!("%expected == {query}"),
            Status::FAIL,
        ),
        (
            "forward negated",
            format!("{query} != %expected"),
            Status::SKIP,
        ),
        (
            "mirrored negated",
            format!("%expected != {query}"),
            Status::SKIP,
        ),
    ] {
        let rules = format!("let expected = 'Owner'\nrule r {{ {clause} }}");

        // Liveness first: with a bucket present the clause must actually decide, or the
        // zero-selection row below is satisfied by a rule that never ran.
        let live = status_of(rules.as_str(), one_bucket)?;
        assert_ne!(
            live,
            Status::SKIP,
            "liveness: `{clause}` must decide when a bucket is present, got SKIP -- the \
             zero-selection assertion below would then prove nothing"
        );

        assert_eq!(
            status_of(rules.as_str(), no_bucket)?,
            want,
            "{label}: `{clause}` against a template with no S3 bucket"
        );
    }

    Ok(())
}
