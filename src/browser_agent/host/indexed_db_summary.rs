//! Native IndexedDB record serialization for browser capabilities.

use crate::browser_agent::IndexedDbRecord;
use crate::value::Value;

use super::super::state::HostState;

#[cfg(test)]
#[path = "indexed_db_summary_tests.rs"]
mod tests;

pub(super) fn invoke(state: &HostState) -> Result<Value, String> {
    Ok(super::super::value::list(
        state
            .page
            .indexed_db_records()?
            .into_iter()
            .map(record)
            .collect(),
    ))
}

fn record(record: IndexedDbRecord) -> Value {
    super::super::value::map(vec![
        ("origin", super::super::value::string(record.origin)),
        ("database", super::super::value::string(record.database)),
        (
            "object_store",
            super::super::value::string(record.object_store),
        ),
        ("key", super::super::value::string(record.key)),
        ("value", super::super::value::string(record.value)),
    ])
}
