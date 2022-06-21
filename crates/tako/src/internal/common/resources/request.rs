use serde::{Deserialize, Serialize};

use crate::internal::common::error::DsError;
use crate::internal::common::resources::{
    GenericResourceAmount, GenericResourceId, NumOfCpus, NumOfNodes,
};
use crate::internal::worker::pool::ResourcePool;
use smallvec::{smallvec, SmallVec};
use std::time::Duration;

#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq)]
pub enum CpuRequest {
    Compact(NumOfCpus),
    ForceCompact(NumOfCpus),
    Scatter(NumOfCpus),
    All,
}

impl Default for CpuRequest {
    fn default() -> Self {
        CpuRequest::Compact(1)
    }
}

impl CpuRequest {
    pub fn validate(&self) -> crate::Result<()> {
        match &self {
            CpuRequest::Scatter(n_cpus)
            | CpuRequest::ForceCompact(n_cpus)
            | CpuRequest::Compact(n_cpus) => {
                if *n_cpus == 0 {
                    Err(DsError::GenericError(
                        "Zero cpus cannot be requested".to_string(),
                    ))
                } else {
                    Ok(())
                }
            }
            CpuRequest::All => Ok(()),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq)]
pub struct GenericResourceRequest {
    pub resource: GenericResourceId,
    pub amount: GenericResourceAmount,
}

pub type GenericResourceRequests = SmallVec<[GenericResourceRequest; 2]>;
pub type TimeRequest = Duration;

#[derive(Default, Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq)]
pub struct ResourceRequestVariant {
    cpus: CpuRequest,

    // After normalization, this array is sorted by resource id
    #[serde(default)]
    generic: GenericResourceRequests,
}

#[derive(Default, Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq)]
pub struct ResourceRequest {
    n_nodes: NumOfNodes,

    variants: SmallVec<[ResourceRequestVariant; 1]>,

    /// Minimal remaining time of the worker life time needed to START the task
    /// !!! Do not confuse with time_limit.
    /// If task is started and task is running, it is not stopped if
    /// it consumes more. If you need this, see time_limit in task configuration
    /// On worker with not defined life time, this resource is always satisfied.
    #[serde(default)]
    min_time: TimeRequest,
}

impl ResourceRequestVariant {
    pub fn generic_requests(&self) -> &GenericResourceRequests {
        &self.generic
    }
    pub fn cpus(&self) -> &CpuRequest {
        &self.cpus
    }

    pub fn validate(&self) -> crate::Result<()> {
        self.cpus.validate()?;
        for pair in self.generic.windows(2) {
            if pair[0].resource >= pair[1].resource {
                return Err("Generic request are not sorted or unique".into());
            }
        }
        Ok(())
    }
}

impl ResourceRequest {
    pub fn new(
        n_nodes: NumOfNodes,
        cpu_request: CpuRequest,
        time: TimeRequest,
        mut generic_resources: GenericResourceRequests,
    ) -> ResourceRequest {
        generic_resources.sort_unstable_by_key(|r| r.resource);
        ResourceRequest {
            n_nodes,
            variants: smallvec![ResourceRequestVariant {
                cpus: cpu_request,
                generic: generic_resources
            }],
            min_time: time,
        }
    }

    pub fn is_multi_node(&self) -> bool {
        self.n_nodes > 0
    }

    pub fn n_nodes(&self) -> NumOfNodes {
        self.n_nodes
    }

    /*pub fn generic_requests(&self) -> &GenericResourceRequests {
        &self.generic
    }*/

    pub fn min_time(&self) -> TimeRequest {
        self.min_time
    }

    /*pub fn cpus(&self) -> &CpuRequest {
        &self.cpus
    }*/

    pub fn sort_key(
        &self,
        resource_pool: &ResourcePool,
    ) -> (NumOfCpus, NumOfCpus, TimeRequest, f32) {
        todo!()
        // let generic_resources_portion = self
        //     .generic
        //     .iter()
        //     .map(|gr| resource_pool.fraction_of_resource(gr))
        //     .sum();
        //
        // match &self.cpus {
        //     CpuRequest::Compact(n_cpus) => (*n_cpus, 1, self.min_time, generic_resources_portion),
        //     CpuRequest::ForceCompact(n_cpus) => {
        //         (*n_cpus, 2, self.min_time, generic_resources_portion)
        //     }
        //     CpuRequest::Scatter(n_cpus) => (*n_cpus, 0, self.min_time, generic_resources_portion),
        //     CpuRequest::All => (
        //         NumOfCpus::MAX,
        //         NumOfCpus::MAX,
        //         self.min_time,
        //         generic_resources_portion,
        //     ),
        // }
    }

    pub fn validate(&self) -> crate::Result<()> {
        for variant in &self.variants {
            variant.validate()?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::internal::common::resources::CpuRequest;
    use crate::internal::tests::utils::resources::ResBuilder;

    #[test]
    fn test_resource_request_validate() {
        let rq = ResBuilder::default()
            .cpus(CpuRequest::All)
            .add_generic(10, 4)
            .add_generic(7, 6)
            .add_generic(10, 6)
            .finish();
        assert!(rq.validate().is_err())
    }
}
