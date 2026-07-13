use std::{collections::HashMap, sync::Arc};

use super::ScheduledTaskDefinition;
use crate::application::{SchedulerError, SchedulerResult};

pub trait TaskCatalog: Send + Sync + 'static {
    fn all(&self) -> Vec<ScheduledTaskDefinition>;
    fn get(&self, task_key: &str) -> Option<ScheduledTaskDefinition>;
}

pub struct StaticTaskCatalog {
    definitions: HashMap<&'static str, ScheduledTaskDefinition>,
}

impl StaticTaskCatalog {
    pub fn try_new(definitions: impl IntoIterator<Item = ScheduledTaskDefinition>) -> SchedulerResult<Arc<Self>> {
        let mut entries = HashMap::new();
        for definition in definitions {
            if entries.insert(definition.task_key, definition).is_some() {
                return Err(SchedulerError::Infrastructure(format!("duplicate scheduled task key: {}", definition.task_key)));
            }
        }
        Ok(Arc::new(Self { definitions: entries }))
    }
}

impl TaskCatalog for StaticTaskCatalog {
    fn all(&self) -> Vec<ScheduledTaskDefinition> {
        let mut values = self.definitions.values().copied().collect::<Vec<_>>();
        values.sort_by_key(|definition| definition.task_key);
        values
    }

    fn get(&self, task_key: &str) -> Option<ScheduledTaskDefinition> {
        self.definitions.get(task_key).copied()
    }
}
