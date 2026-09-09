// Copyright Amazon Web Services, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
#[cfg(test)]
mod functional_tests {
    use pretty_assertions::assert_eq;
    #[test]
    fn test_run_check() {
        let data = String::from(
            r#"
                {
                    "Resources": {
                        "VPC" : {
                            "Type" : "AWS::ApiGateway::Method",
                            "Properties" : {
                                "AuthorizationType" : "10.0.0.0/24"
                            }
                        }
                    }
                }
            "#,
        );
        let rule = "AWS::ApiGateway::Method { Properties.AuthorizationType == \"NONE\"}";
        let expected = r#"{
                  "context": "File(rules=1)",
                  "container": {
                    "FileCheck": {
                      "name": "functional_test.json",
                      "status": "FAIL",
                      "message": null
                    }
                  },
                  "children": [
                    {
                      "context": "functional_test.rule/default",
                      "container": {
                        "RuleCheck": {
                          "name": "functional_test.rule/default",
                          "status": "FAIL",
                          "message": null
                        }
                      },
                      "children": [
                        {
                          "context": "TypeBlock#AWS::ApiGateway::Method",
                          "container": {
                            "TypeCheck": {
                              "type_name": "AWS::ApiGateway::Method",
                              "block": {
                                "at_least_one_matches": false,
                                "status": "FAIL",
                                "message": null
                              }
                            }
                          },
                          "children": [
                            {
                              "context": "Filter/Map#1",
                              "container": {
                                "Filter": "PASS"
                              },
                              "children": [
                                {
                                  "context": "GuardAccessClause#block Type EQUALS  \"AWS::ApiGateway::Method\"",
                                  "container": {
                                    "GuardClauseBlockCheck": {
                                      "at_least_one_matches": false,
                                      "status": "PASS",
                                      "message": null
                                    }
                                  },
                                  "children": [
                                    {
                                      "context": " Type EQUALS  \"AWS::ApiGateway::Method\"",
                                      "container": {
                                        "ClauseValueCheck": "Success"
                                      },
                                      "children": []
                                    }
                                  ]
                                }
                              ]
                            },
                            {
                              "context": "TypeBlock#AWS::ApiGateway::Method/0",
                              "container": {
                                "TypeBlock": "FAIL"
                              },
                              "children": [
                                {
                                  "context": "GuardAccessClause#block Properties.AuthorizationType EQUALS  \"NONE\"",
                                  "container": {
                                    "GuardClauseBlockCheck": {
                                      "at_least_one_matches": false,
                                      "status": "FAIL",
                                      "message": null
                                    }
                                  },
                                  "children": [
                                    {
                                      "context": " Properties.AuthorizationType EQUALS  \"NONE\"",
                                      "container": {
                                        "ClauseValueCheck": {
                                          "Comparison": {
                                            "comparison": [
                                              "Eq",
                                              false
                                            ],
                                            "from": {
                                              "Resolved": {
                                                "path": "/Resources/VPC/Properties/AuthorizationType",
                                                "value": "10.0.0.0/24"
                                              }
                                            },
                                            "to": {
                                              "Resolved": {
                                                "path": "",
                                                "value": "NONE"
                                              }
                                            },
                                            "message": null,
                                            "custom_message": null,
                                            "status": "FAIL"
                                          }
                                        }
                                      },
                                      "children": []
                                    }
                                  ]
                                }
                              ]
                            }
                          ]
                        }
                      ]
                    }
                  ]
                }"#;
        let verbose = true;
        use cfn_guard::*;
        let serialized = run_checks(
            ValidateInput {
                content: &data,
                file_name: "functional_test.json",
            },
            ValidateInput {
                content: rule,
                file_name: "functional_test.rule",
            },
            verbose,
        )
        .unwrap();
        let result = serde_json::from_str::<serde_json::Value>(&serialized)
            .ok()
            .unwrap();
        let expected = serde_json::from_str::<serde_json::Value>(expected)
            .ok()
            .unwrap();
        assert_eq!(expected, result);
    }

    /// The non-verbose `run_checks` output carries the reason a comparison had no answer.
    ///
    /// This is the surface `guard-ffi/src/lib.rs` and `guard-lambda/src/main.rs` both call, and both
    /// default `verbose` to false. Neither has a stderr channel, so the string this returns is the
    /// operator's only record of what happened.
    ///
    /// `test_run_check` above also calls `run_checks`, but with `verbose = true`, which returns the
    /// event-record tree early and never reaches `GenericSummary::report_eval`. So the non-verbose
    /// formatter had no coverage at all, which is why a missing explanation in it went unnoticed:
    /// nothing here would have failed.
    ///
    /// Four comparators over one eighteen-character map key, which is one byte past where
    /// `(?!x)((a+)+)b` exhausts `fancy-regex`'s backtracking budget. The pattern compiles -- the engine
    /// backtracks and accepts the lookahead -- and `is_match` is what gives up. All four are FAIL and
    /// all four return the same status either way, so the assertion is on the text.
    #[test]
    fn non_verbose_run_checks_reports_an_undecided_map_key_comparison() {
        use cfn_guard::*;

        const REASON: &str = "could not be evaluated";
        let data = r#"{ "Cfg": { "aaaaaaaaaaaaaaaaaa": 1, "other": 2 } }"#;

        for (operator, rhs) in [
            ("==", "/(?!x)((a+)+)b/"),
            ("!=", "/(?!x)((a+)+)b/"),
            ("in", "[/(?!x)((a+)+)b/]"),
            ("not in", "[/(?!x)((a+)+)b/]"),
        ] {
            let rule = format!("rule r {{ Cfg[ keys {operator} {rhs} ] !empty }}");
            let serialized = run_checks(
                ValidateInput {
                    content: data,
                    file_name: "functional_test.json",
                },
                ValidateInput {
                    content: &rule,
                    file_name: "functional_test.rule",
                },
                false,
            )
            .unwrap();

            assert!(
                serialized.contains(REASON),
                "`keys {}` must say the comparison had no answer on the non-verbose surface that the \
                 FFI and Lambda entry points use, where the report is the only record; got:\n{}",
                operator,
                serialized
            );
        }
    }

    /// A JSON integer above `i64::MAX` does not answer a numeric guard from a flipped sign.
    ///
    /// `run_checks` tries `serde_json::from_str` first and only falls back to `serde_yaml` when that
    /// fails (`commands/helper.rs:51`). `18446744073709551615` is valid JSON, so it takes the JSON
    /// branch and reaches `TryFrom<&serde_json::Value> for Value`, which is a separate arm from the
    /// `serde_yaml` one. Fixing only the YAML arm left this entry point reading `u64::MAX` as `-1`
    /// through `as i64` -- so `Size < 0` reported PASS on a document whose `Size` is the largest
    /// unsigned 64-bit integer, at exit 0 with nothing on either channel.
    ///
    /// This is the surface `cfn_guard_run_checks` in `guard-ffi/src/lib.rs` and the Lambda handler in
    /// `guard-lambda/src/main.rs` call, so the sign flip was reachable by every caller that is not the
    /// CLI. Neither has a stderr channel, which is why the assertion is on the returned report.
    ///
    /// `< 0` rather than a comparison against the number itself: it is the shortest clause whose
    /// correct answer is unambiguous for every value the input could hold. No unsigned 64-bit integer
    /// is negative, so FAIL is right whether the digits are kept or the comparison refuses -- and only
    /// a sign flip can produce PASS.
    #[test]
    fn a_json_integer_above_i64_max_does_not_pass_a_negative_comparison() {
        use cfn_guard::*;

        let data = r#"{ "Size": 18446744073709551615 }"#;

        let serialized = run_checks(
            ValidateInput {
                content: data,
                file_name: "functional_test.json",
            },
            ValidateInput {
                content: "rule r { Size < 0 }",
                file_name: "functional_test.rule",
            },
            false,
        )
        .unwrap();

        assert!(
            !serialized.contains("PASS"),
            "`Size < 0` must not pass for 18446744073709551615 -- a PASS here is the `as i64` sign \
             flip reinterpreting u64::MAX as -1 on the JSON branch of `run_checks`, which is what \
             the FFI and Lambda entry points call; got:\n{}",
            serialized
        );
        assert!(
            serialized.contains("FAIL"),
            "`Size < 0` must report FAIL for 18446744073709551615; got:\n{}",
            serialized
        );
    }
}
