use std::borrow::Cow;

use serde::de::{Deserialize, Deserializer, Error};

pub fn spdx_expression_opt<'de, D>(deserializer: D) -> Result<Option<spdx::Expression>, D::Error>
where
    D: Deserializer<'de>,
    D::Error: Error,
{
    let raw = Option::<Cow<'static, str>>::deserialize(deserializer)?;
    raw.map(|value| spdx::Expression::parse(&value).map_err(D::Error::custom))
        .transpose()
}
