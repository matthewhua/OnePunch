use anyhow::{anyhow, Result};
use polars::prelude::*;
use sqlparser::parser::Parser;
use std::ops::{Deref, DerefMut};
use polars::io::csv;
use tracing::info;

#[derive(Debug)]
pub struct DataSet(DataFrame);

// 让 DataSet 用起来和 DataFrame 一致
impl Deref for DataSet {
    type Target = DataFrame;
 
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// 让 DataSet 用起来和 DataFrame 一致
impl DerefMut for DataSet {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl DataSet {

    /// 从 DataSet 转换成 csv
    pub fn to_csv(&self) -> Result<String> {
        let mut buffer = Vec::new();
        let mut writer = CsvWriter::new(&mut buffer);
        writer.finish(self)?;
        Ok(String::from_utf8(buffer)?)
    }
}