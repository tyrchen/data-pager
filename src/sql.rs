use crate::{
    error::*,
    utils::{decode_u64, encode_u64},
    Id, PageInfo, Pager, Paginator,
};
use derive_builder::Builder;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use snafu::ensure;
use std::{borrow::Cow, collections::VecDeque};

const MAX_PAGE_SIZE: u64 = 100;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, Builder)]
#[builder(build_fn(name = "private_build"), setter(into, strip_option), default)]
pub struct SqlQuery<'a> {
    /// source table or view
    pub source: Cow<'a, str>,
    /// fields to include in the result
    pub projection: Vec<Cow<'a, str>>,
    /// filter condition (the WHERE clause)
    pub filter: Option<Cow<'a, str>>,
    /// sort order (the ORDER BY clause)
    pub order: Option<Cow<'a, str>>,
    /// previous page cursor, in base64 (right now this is just the number of items to skip)
    pub cursor: Option<Cow<'a, str>>,
    /// page size
    pub page_size: u64,
}

impl<'a> SqlQueryBuilder<'a> {
    pub fn build(&self) -> Result<SqlQuery<'a>, Error> {
        let mut data = self
            .private_build()
            .expect("failed to build SqlQuery struct");
        data.normalize();
        data.validate()?;

        Ok(data)
    }
}

impl<'a> SqlQuery<'a> {
    pub fn to_sql(&self) -> String {
        let middle_plus = if self.cursor.is_none() { 0 } else { 1 };
        let limit = self.page_size + 1 + middle_plus;
        let offset = self.get_cursor().unwrap_or_default();

        let where_clause = if let Some(filter) = &self.filter {
            Cow::Owned(format!("WHERE {filter}"))
        } else {
            Cow::Borrowed("")
        };

        let order_clause = if let Some(order) = &self.order {
            Cow::Owned(format!("ORDER BY {order}"))
        } else {
            Cow::Borrowed("")
        };

        [
            "SELECT",
            &self.projection(),
            "FROM",
            &self.source,
            &where_clause,
            &order_clause,
            "LIMIT",
            &limit.to_string(),
            "OFFSET",
            &offset.to_string(),
        ]
        .iter()
        .filter(|s| !s.is_empty())
        .join(" ")
    }

    pub fn get_pager<T: Id>(&self, data: &mut VecDeque<T>) -> Pager {
        let page_info = self.page_info();
        page_info.get_pager(data)
    }

    pub fn get_cursor(&self) -> Option<u64> {
        self.cursor.as_deref().and_then(|c| decode_u64(c).ok())
    }

    pub fn next_page(&self, pager: &Pager) -> Option<Self> {
        let page_info = self.page_info();
        let page_info = page_info.next_page(pager);
        page_info.map(|page_info| Self {
            source: self.source.clone(),
            projection: self.projection.clone(),
            filter: self.filter.clone(),
            order: self.order.clone(),
            cursor: page_info.cursor.map(|c| encode_u64(c).into()),
            page_size: page_info.page_size,
        })
    }

    fn page_info(&self) -> PageInfo {
        PageInfo {
            cursor: self.get_cursor(),
            page_size: self.page_size,
        }
    }

    fn projection(&self) -> Cow<'a, str> {
        if self.projection.is_empty() {
            return "*".into();
        }

        self.projection.iter().join(", ").into()
    }

    fn validate(&self) -> Result<(), Error> {
        ensure!(
            self.page_size > 0 && self.page_size < MAX_PAGE_SIZE,
            InvalidPageSizeSnafu {
                size: self.page_size
            }
        );
        ensure!(!self.source.is_empty(), InvalidSourceSnafu);

        Ok(())
    }

    fn normalize(&mut self) {
        if self.page_size == 0 {
            self.page_size = 10;
        } else if self.page_size > MAX_PAGE_SIZE {
            self.page_size = MAX_PAGE_SIZE;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pager_test_utils::generate_test_ids;
    use anyhow::{Context, Result};

    #[test]
    fn sql_query_should_generate_right_sql() -> Result<()> {
        let query = SqlQuery {
            source: "users".into(),
            projection: vec!["id".into(), "name".into()],
            filter: Some("id > 10".into()),
            order: Some("id DESC".into()),
            cursor: Some(encode_u64(10).into()),
            page_size: 10,
        };

        let sql = query.to_sql();
        assert_eq!(
            sql,
            "SELECT id, name FROM users WHERE id > 10 ORDER BY id DESC LIMIT 12 OFFSET 10"
        );

        Ok(())
    }

    #[test]
    fn sql_builder_should_get_correct_page_info() -> Result<()> {
        let query = SqlQueryBuilder::default().source("users").build()?;

        let mut data = generate_test_ids(1, 11);
        let pager = query.get_pager(&mut data);
        assert_eq!(pager.prev, None);
        assert_eq!(pager.next, Some(10));

        let query = query.next_page(&pager).context("no next page")?;
        let sql = query.to_sql();
        assert_eq!(sql, "SELECT * FROM users LIMIT 12 OFFSET 10");
        Ok(())
    }
}
