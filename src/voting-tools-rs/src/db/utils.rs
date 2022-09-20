//! General purpose utilities for working with the db-sync database

use diesel::sql_types::SqlType;
use diesel::{
    expression::AsExpression,
    pg::Pg,
    sql_types::{Nullable, Text},
    Expression,
};

// Allows us to use postgres built-in regex matching in the diesel dsl
infix_operator!(RegexMatches, " ~ ", backend: Pg);

/// Extension trait for working with `~` in a more idiomatic diesel way
pub trait RegexExpressionMethods: Expression<SqlType = Nullable<Text>> + Sized {
    fn regex_matches<T>(self, other: T) -> RegexMatches<Self, T::Expression>
    where
        Self::SqlType: SqlType,
        T: AsExpression<Nullable<Text>>,
    {
        RegexMatches::new(self, other.as_expression())
    }
}

impl<T: Expression<SqlType = Nullable<Text>>> RegexExpressionMethods for T {}
