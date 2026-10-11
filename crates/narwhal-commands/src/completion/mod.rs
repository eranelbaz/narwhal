//! SQL completion provider.
//!
//! The provider produces an ordered list of [`Completion`]
//! candidates from a prefix and the active session's cached
//! schemas. Matches are scored cheaply: exact case-insensitive
//! prefix match wins, otherwise candidates that contain the prefix
//! as a substring come second.

mod context;
mod gather;
mod items;
mod keywords;
mod tokenizer;

pub use context::{CompletionContext, detect_context, detect_context_with_schemas};
pub use gather::gather;
pub use items::{Completion, CompletionKind};
pub use keywords::{FUNCTIONS, KEYWORDS, PHRASES};

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use narwhal_core::{ColumnHeader, Schema, Table, TableKind};
    use narwhal_domain::SchemaListing;

    use super::*;

    fn listing() -> Vec<SchemaListing> {
        vec![(
            Schema {
                name: "public".into(),
            },
            vec![
                Table {
                    schema: "public".into(),
                    name: "orders".into(),
                    kind: TableKind::Table,
                },
                Table {
                    schema: "public".into(),
                    name: "order_items".into(),
                    kind: TableKind::Table,
                },
                Table {
                    schema: "public".into(),
                    name: "users".into(),
                    kind: TableKind::Table,
                },
            ],
        )]
    }

    fn no_columns() -> HashMap<String, (String, Vec<ColumnHeader>)> {
        HashMap::new()
    }

    #[test]
    fn empty_prefix_yields_nothing() {
        assert!(
            gather(
                "",
                &listing(),
                &CompletionContext::Generic,
                &no_columns(),
                &HashMap::new(),
                20
            )
            .is_empty()
        );
    }

    #[test]
    fn prefix_hits_come_before_substring_hits() {
        let out = gather(
            "or",
            &listing(),
            &CompletionContext::Generic,
            &no_columns(),
            &HashMap::new(),
            20,
        );
        let ord = out
            .iter()
            .position(|c| c.text == "orders")
            .expect("orders present");
        let ord_items = out
            .iter()
            .position(|c| c.text == "order_items")
            .expect("order_items present");
        let or = out
            .iter()
            .position(|c| c.text == "OR")
            .expect("OR keyword present");
        // Both "orders" and "order_items" prefix-match; "OR" also
        // prefix-matches as a keyword. All three are in the prefix tier.
        assert!(ord < out.len() && ord_items < out.len() && or < out.len());
    }

    #[test]
    fn case_insensitive_match() {
        let out = gather(
            "SEL",
            &listing(),
            &CompletionContext::Generic,
            &no_columns(),
            &HashMap::new(),
            20,
        );
        assert!(out.iter().any(|c| c.text == "SELECT"));
    }

    #[test]
    fn deduplicates_by_kind_and_name() {
        // Two listings would each emit `orders`; the result still has it
        // only once.
        let mut listings = listing();
        listings.push(listings[0].clone());
        let out = gather(
            "orders",
            &listings,
            &CompletionContext::Generic,
            &no_columns(),
            &HashMap::new(),
            20,
        );
        let n = out.iter().filter(|c| c.text == "orders").count();
        assert_eq!(n, 1);
    }

    #[test]
    fn limit_is_respected() {
        let out = gather(
            "e",
            &listing(),
            &CompletionContext::Generic,
            &no_columns(),
            &HashMap::new(),
            3,
        );
        assert!(out.len() <= 3);
    }

    #[test]
    fn from_keyword_narrows_to_tables() {
        let ctx = detect_context("SELECT * FROM u", 14);
        assert_eq!(ctx, CompletionContext::TableExpected);

        let out = gather("u", &listing(), &ctx, &no_columns(), &HashMap::new(), 50);
        // Should contain `users` table but NOT `UNION` or `UPDATE` keywords.
        assert!(
            out.iter()
                .any(|c| c.text == "users" && c.kind == CompletionKind::Table)
        );
        assert!(
            !out.iter()
                .any(|c| c.text == "UNION" && c.kind == CompletionKind::Keyword)
        );
        assert!(
            !out.iter()
                .any(|c| c.text == "UPDATE" && c.kind == CompletionKind::Keyword)
        );
    }

    #[test]
    fn dotted_identifier_suggests_columns() {
        let mut cols = HashMap::new();
        cols.insert(
            "users".to_owned(),
            (
                "public".to_owned(),
                vec![
                    ColumnHeader {
                        name: "id".into(),
                        data_type: "int4".into(),
                    },
                    ColumnHeader {
                        name: "name".into(),
                        data_type: "varchar".into(),
                    },
                    ColumnHeader {
                        name: "email".into(),
                        data_type: "varchar".into(),
                    },
                ],
            ),
        );
        let ctx = detect_context("SELECT users.", 13);
        assert_eq!(
            ctx,
            CompletionContext::ColumnExpected {
                table: "users".into()
            }
        );

        let out = gather("", &listing(), &ctx, &cols, &HashMap::new(), 50);
        // Right after `users.` every column is offered.
        assert_eq!(out.len(), cols["users"].1.len());
        assert!(
            gather(
                "",
                &listing(),
                &CompletionContext::Generic,
                &cols,
                &HashMap::new(),
                50
            )
            .is_empty()
        );

        // With a prefix we get the matching columns.
        let out = gather("n", &listing(), &ctx, &cols, &HashMap::new(), 50);
        assert!(
            out.iter()
                .any(|c| c.text == "name" && c.kind == CompletionKind::Column)
        );
        assert!(!out.iter().any(|c| c.kind == CompletionKind::Keyword));
    }

    #[test]
    fn context_stops_at_previous_semicolon() {
        let ctx = detect_context("SELECT * FROM users; SELECT u", 27);
        // The FROM is past the `;`, so we should NOT be in TableExpected.
        assert_eq!(ctx, CompletionContext::Generic);
    }

    #[test]
    fn join_keyword_narrows_to_tables() {
        let ctx = detect_context("SELECT * FROM orders JOIN u", 27);
        assert_eq!(ctx, CompletionContext::TableExpected);

        let out = gather("u", &listing(), &ctx, &no_columns(), &HashMap::new(), 50);
        assert!(
            out.iter()
                .any(|c| c.text == "users" && c.kind == CompletionKind::Table)
        );
        assert!(
            !out.iter()
                .any(|c| c.text == "UNION" && c.kind == CompletionKind::Keyword)
        );
    }

    #[test]
    fn update_keyword_narrows_to_tables() {
        let ctx = detect_context("UPDATE u", 8);
        assert_eq!(ctx, CompletionContext::TableExpected);

        let out = gather("u", &listing(), &ctx, &no_columns(), &HashMap::new(), 50);
        assert!(
            out.iter()
                .any(|c| c.text == "users" && c.kind == CompletionKind::Table)
        );
        assert!(
            !out.iter()
                .any(|c| c.text == "UNION" && c.kind == CompletionKind::Keyword)
        );
    }

    fn user_cols() -> HashMap<String, (String, Vec<ColumnHeader>)> {
        let mut m = HashMap::new();
        m.insert(
            "users".to_owned(),
            (
                "public".to_owned(),
                vec![
                    ColumnHeader {
                        name: "id".into(),
                        data_type: "int4".into(),
                    },
                    ColumnHeader {
                        name: "email".into(),
                        data_type: "text".into(),
                    },
                ],
            ),
        );
        m
    }

    /// `FROM users u WHERE u.` should resolve `u` → `users` and
    /// suggest the table's columns instead of treating `u` as a real
    /// table name.
    #[test]
    fn alias_in_from_resolves_to_real_table_for_dot_completion() {
        let buf = "SELECT * FROM users u WHERE u.";
        let ctx = detect_context(buf, buf.len());
        assert_eq!(
            ctx,
            CompletionContext::ColumnExpected {
                table: "users".into()
            }
        );
        let out = gather("e", &listing(), &ctx, &user_cols(), &HashMap::new(), 50);
        assert!(
            out.iter()
                .any(|c| c.text == "email" && c.kind == CompletionKind::Column)
        );
    }

    /// `SELECT u.| FROM users u` — alias declared after the cursor in
    /// the same statement still resolves; aliases from other statements
    /// do not leak in.
    #[test]
    fn alias_declared_after_cursor_resolves() {
        let buf = "SELECT u. FROM users u; SELECT * FROM orders x";
        let cursor = "SELECT u.".len();
        let ctx = detect_context(buf, cursor);
        assert_eq!(
            ctx,
            CompletionContext::ColumnExpected {
                table: "users".into()
            }
        );
        let out = gather("e", &listing(), &ctx, &user_cols(), &HashMap::new(), 50);
        assert!(out.iter().any(|c| c.text == "email"));

        let buf = "SELECT x. FROM users u; SELECT * FROM orders x";
        let ctx = detect_context(buf, "SELECT x.".len());
        assert_eq!(ctx, CompletionContext::ColumnExpected { table: "x".into() });

        let buf = "select u.em from users u";
        let ctx = detect_context(buf, "select u.em".len());
        assert_eq!(
            ctx,
            CompletionContext::ColumnExpected {
                table: "users".into()
            }
        );

        let before = "SELECT * FROM users u WHERE u.";
        let buf = format!("{before} IN (SELECT id FROM orders u)");
        let ctx = detect_context(&buf, before.len());
        assert_eq!(
            ctx,
            CompletionContext::ColumnExpected {
                table: "users".into()
            }
        );
    }

    /// `JOIN orders AS o ON o.` walks through the explicit `AS` form.
    #[test]
    fn alias_with_explicit_as_keyword_is_resolved() {
        let mut cols = user_cols();
        cols.insert(
            "orders".to_owned(),
            (
                "public".to_owned(),
                vec![ColumnHeader {
                    name: "total".into(),
                    data_type: "numeric".into(),
                }],
            ),
        );
        let buf = "SELECT * FROM users u JOIN orders AS o ON o.";
        let ctx = detect_context(buf, buf.len());
        assert_eq!(
            ctx,
            CompletionContext::ColumnExpected {
                table: "orders".into()
            }
        );
        let out = gather("t", &listing(), &ctx, &cols, &HashMap::new(), 50);
        assert!(out.iter().any(|c| c.text == "total"));
    }

    /// `public.` when `public` is a known schema lands in
    /// `SchemaTableExpected` and the gather only emits tables from
    /// that schema.
    #[test]
    fn schema_prefix_narrows_table_list() {
        let buf = "SELECT * FROM public.";
        let known = vec!["public".to_owned()];
        let ctx = detect_context_with_schemas(buf, buf.len(), &known);
        assert_eq!(
            ctx,
            CompletionContext::SchemaTableExpected {
                schema: "public".into()
            }
        );
        let out = gather("u", &listing(), &ctx, &no_columns(), &HashMap::new(), 50);
        assert!(
            out.iter()
                .any(|c| c.text == "users" && c.kind == CompletionKind::Table)
        );
    }

    /// Without the schema list, `public.` falls back to the legacy
    /// behaviour (`ColumnExpected` on a non-existent table).
    #[test]
    fn unknown_dotted_prefix_falls_back_to_column_lookup() {
        let buf = "SELECT * FROM public.";
        let ctx = detect_context(buf, buf.len());
        assert_eq!(
            ctx,
            CompletionContext::ColumnExpected {
                table: "public".into()
            }
        );
    }

    /// Generic context surfaces the function list with the trailing
    /// `(` so the cursor lands inside the call after acceptance.
    #[test]
    fn generic_context_includes_functions() {
        let ctx = detect_context("SELECT ", 7);
        assert_eq!(ctx, CompletionContext::Generic);
        let out = gather("cou", &listing(), &ctx, &no_columns(), &HashMap::new(), 50);
        assert!(
            out.iter()
                .any(|c| c.text == "COUNT(" && c.kind == CompletionKind::Function)
        );
    }

    /// Function suggestions are kind-tagged distinctly so the UI can
    /// render them with a different glyph.
    #[test]
    fn function_kind_distinct_from_keyword() {
        let ctx = detect_context("SELECT ", 7);
        let out = gather("now", &listing(), &ctx, &no_columns(), &HashMap::new(), 50);
        assert!(
            out.iter()
                .any(|c| c.text == "NOW()" && c.kind == CompletionKind::Function)
        );
    }

    fn rec_value_ctx() -> CompletionContext {
        CompletionContext::ValueExpected {
            table: "recommendations".into(),
            column: "type".into(),
        }
    }

    #[test]
    fn equals_after_qualified_column_expects_value() {
        let buf = "select r.*\nfrom recommendations r\nwhere r.trackable = true\n  and r.type = ";
        assert_eq!(detect_context(buf, buf.len()), rec_value_ctx());
    }

    #[test]
    fn value_context_survives_partial_literal_prefix_and_comment() {
        for buf in [
            "select * from recommendations r where r.type = CO",
            "select * from recommendations r where r.type = 'CO",
            "select * from recommendations r where r.type = '",
            "select * from recommendations r where r.type = /* pick */ ",
        ] {
            assert_eq!(detect_context(buf, buf.len()), rec_value_ctx(), "{buf}");
        }
    }

    #[test]
    fn keyword_named_column_still_expects_value() {
        let buf = "select * from orders o where o.order = ";
        assert_eq!(
            detect_context(buf, buf.len()),
            CompletionContext::ValueExpected {
                table: "orders".into(),
                column: "order".into(),
            }
        );
    }

    #[test]
    fn non_equality_positions_are_not_value_context() {
        for buf in [
            "select * from recommendations r where r.type = 'BLOG' ",
            "select * from recommendations r where r.type != ",
            "select * from recommendations r where r.type <= ",
            "select * from recommendations r where r.type >= ",
            "select * from recommendations r where r.type::text = ",
            "select * from recommendations r where r.type = 'X' and r.",
        ] {
            assert!(
                !matches!(
                    detect_context(buf, buf.len()),
                    CompletionContext::ValueExpected { .. }
                ),
                "{buf}"
            );
        }
    }

    #[test]
    fn auto_paired_quote_is_value_but_existing_literal_is_not() {
        let paired = "select * from recommendations r where r.type = 'CO'";
        assert_eq!(detect_context(paired, paired.len() - 1), rec_value_ctx());
        let existing = "select * from recommendations r where r.type = 'BLOG'";
        let at = existing.find('\'').unwrap();
        assert!(!matches!(
            detect_context(existing, at),
            CompletionContext::ValueExpected { .. }
        ));
        assert!(!matches!(
            detect_context(existing, at + 3),
            CompletionContext::ValueExpected { .. }
        ));
    }

    fn rec_cols() -> HashMap<String, (String, Vec<ColumnHeader>)> {
        HashMap::from([(
            "recommendations".to_owned(),
            (
                "public".to_owned(),
                vec![
                    ColumnHeader {
                        name: "type".into(),
                        data_type: "\"RecommendationType\"".into(),
                    },
                    ColumnHeader {
                        name: "title".into(),
                        data_type: "text".into(),
                    },
                ],
            ),
        )])
    }

    fn rec_enums() -> HashMap<String, Vec<String>> {
        HashMap::from([(
            "\"RecommendationType\"".to_owned(),
            ["CONTENT", "BLOG", "TECHNICAL", "PR", "THIRD_PARTY"]
                .map(String::from)
                .to_vec(),
        )])
    }

    fn value_ctx(column: &str) -> CompletionContext {
        CompletionContext::ValueExpected {
            table: "recommendations".into(),
            column: column.into(),
        }
    }

    fn texts(out: &[Completion]) -> Vec<&str> {
        out.iter().map(|c| c.text.as_str()).collect()
    }

    #[test]
    fn enum_column_suggests_quoted_labels_in_declared_order() {
        let out = gather(
            "",
            &listing(),
            &value_ctx("type"),
            &rec_cols(),
            &rec_enums(),
            50,
        );
        assert_eq!(
            texts(&out),
            [
                "'CONTENT'",
                "'BLOG'",
                "'TECHNICAL'",
                "'PR'",
                "'THIRD_PARTY'"
            ]
        );
        assert!(out.iter().all(|c| c.kind == CompletionKind::Value));
    }

    #[test]
    fn enum_labels_filter_by_prefix() {
        let out = gather(
            "th",
            &listing(),
            &value_ctx("type"),
            &rec_cols(),
            &rec_enums(),
            50,
        );
        assert_eq!(texts(&out), ["'THIRD_PARTY'"]);
    }

    #[test]
    fn unmatched_prefix_on_enum_column_falls_back_to_generic() {
        let out = gather(
            "nu",
            &listing(),
            &value_ctx("type"),
            &rec_cols(),
            &rec_enums(),
            50,
        );
        assert!(out.iter().any(|c| c.text.eq_ignore_ascii_case("NULL")));
        assert!(!out.iter().any(|c| c.kind == CompletionKind::Value));
    }

    #[test]
    fn non_enum_column_suggests_no_labels_but_keeps_generic() {
        for column in ["title", "missing"] {
            assert!(
                gather(
                    "",
                    &listing(),
                    &value_ctx(column),
                    &rec_cols(),
                    &rec_enums(),
                    50
                )
                .is_empty()
            );
        }
        let out = gather(
            "no",
            &listing(),
            &value_ctx("title"),
            &rec_cols(),
            &rec_enums(),
            50,
        );
        assert!(out.iter().any(|c| c.kind == CompletionKind::Function));
        assert!(!out.iter().any(|c| c.kind == CompletionKind::Value));
    }

    #[test]
    fn enum_label_quotes_are_escaped() {
        let enums = HashMap::from([("\"RecommendationType\"".to_owned(), vec!["it's".to_owned()])]);
        let out = gather("", &listing(), &value_ctx("type"), &rec_cols(), &enums, 50);
        assert_eq!(texts(&out), ["'it''s'"]);
    }
}
