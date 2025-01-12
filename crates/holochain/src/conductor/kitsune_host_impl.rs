//! Implementation of the Kitsune Host API

use std::sync::Arc;

use futures::FutureExt;
use holo_hash::DnaHash;
use holochain_p2p::DnaHashExt;
use kitsune_p2p::{
    agent_store::AgentInfoSigned, event::GetAgentInfoSignedEvt, KitsuneHost, KitsuneHostResult,
};

use super::space::Spaces;
use holochain_types::db::PermittedConn;

/// Implementation of the Kitsune Host API.
/// Lets Kitsune make requests of Holochain
pub struct KitsuneHostImpl {
    spaces: Spaces,
}

impl KitsuneHostImpl {
    /// Constructor
    pub fn new(spaces: Spaces) -> Arc<Self> {
        Arc::new(Self { spaces })
    }
}

impl KitsuneHost for KitsuneHostImpl {
    fn peer_extrapolated_coverage(
        &self,
        space: std::sync::Arc<kitsune_p2p::KitsuneSpace>,
        dht_arc_set: holochain_p2p::dht_arc::DhtArcSet,
    ) -> KitsuneHostResult<Vec<f64>> {
        async move {
            let db = self.spaces.p2p_agents_db(&DnaHash::from_kitsune(&space))?;
            use holochain_sqlite::db::AsP2pAgentStoreConExt;
            let permit = db.conn_permit().await;
            let task = tokio::task::spawn_blocking(move || {
                let mut conn = db.with_permit(permit)?;
                conn.p2p_extrapolated_coverage(dht_arc_set)
            })
            .await;
            Ok(task??)
        }
        .boxed()
        .into()
    }

    fn record_metrics(
        &self,
        space: std::sync::Arc<kitsune_p2p::KitsuneSpace>,
        records: Vec<kitsune_p2p::event::MetricRecord>,
    ) -> KitsuneHostResult<()> {
        async move {
            let db = self.spaces.p2p_metrics_db(&DnaHash::from_kitsune(&space))?;
            use holochain_sqlite::db::AsP2pMetricStoreConExt;
            let permit = db.conn_permit().await;
            let task = tokio::task::spawn_blocking(move || {
                let mut conn = db.with_permit(permit)?;
                conn.p2p_log_metrics(records)
            })
            .await;
            Ok(task??)
        }
        .boxed()
        .into()
    }

    fn get_agent_info_signed(
        &self,
        GetAgentInfoSignedEvt { space, agent }: GetAgentInfoSignedEvt,
    ) -> KitsuneHostResult<Option<AgentInfoSigned>> {
        let dna_hash = DnaHash::from_kitsune(&space);
        let db = self.spaces.p2p_agents_db(&dna_hash);
        async move {
            Ok(super::p2p_agent_store::get_agent_info_signed(db?.into(), space, agent).await?)
        }
        .boxed()
        .into()
    }
}
