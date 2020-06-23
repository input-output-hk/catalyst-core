use async_graphql::{InputValueError, InputValueResult, ScalarType, Value};

// Wrapper around `i64` to force graphql to format it as a number (JSON) instead of a string
pub struct ScalarI64(pub i64);

#[async_graphql::Scalar]
impl ScalarType for ScalarI64 {
    fn parse(value: Value) -> InputValueResult<Self> {
        match value {
            Value::Int(n) => Ok(ScalarI64(n)),
            Value::String(s) => Ok(ScalarI64(s.parse()?)),
            _ => Err(InputValueError::ExpectedType(value)),
        }
    }

    fn to_json(&self) -> Result<serde_json::Value, async_graphql::Error> {
        Ok(serde_json::Value::Number(self.0.into()))
    }
}
