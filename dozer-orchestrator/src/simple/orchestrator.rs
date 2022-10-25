use std::{sync::Arc, thread};

use dozer_api::api_server::ApiServer;
use dozer_cache::cache::LmdbCache;
use dozer_schema::registry::SchemaRegistryClient;
use tokio::runtime::Runtime;

use super::executor::Executor;
use crate::Orchestrator;
use dozer_types::models::{api_endpoint::ApiEndpoint, source::Source};

pub struct SimpleOrchestrator {
    pub sources: Vec<Source>,
    pub api_endpoints: Vec<ApiEndpoint>,
    pub schema_client: Arc<SchemaRegistryClient>,
}

impl Orchestrator for SimpleOrchestrator {
    fn add_sources(&mut self, sources: Vec<Source>) -> &mut Self {
        for source in sources.iter() {
            self.sources.push(source.to_owned());
        }
        self
    }

    fn add_endpoints(&mut self, endpoints: Vec<ApiEndpoint>) -> &mut Self {
        self.api_endpoints = endpoints;
        self
    }

    fn run(&mut self) -> anyhow::Result<()> {
        let cache = Arc::new(LmdbCache::new(true));
        let cache_2 = cache.clone();
        let endpoints = self.api_endpoints.clone();
        let endpoints2 = self.api_endpoints.get(0).unwrap().clone();
        let sources = self.sources.clone();

        let thread = thread::spawn(move || {
            let api_server = ApiServer::default();
            api_server.run(endpoints, cache_2).unwrap()
        });
        let _thread2 = thread::spawn(move || {
            // TODO: Refactor add endpoint method to support multiple endpoints
            Executor::run(sources, endpoints2, cache).unwrap();
        });

        let _thread3 = thread::spawn(move || {
            Runtime::new()
                .unwrap()
                .block_on(async { dozer_schema::run().await })
        });
        thread.join().unwrap();

        Ok(())
    }
}

impl SimpleOrchestrator {
    pub fn new(schema_client: Arc<SchemaRegistryClient>) -> Self {
        Self {
            sources: vec![],
            api_endpoints: vec![],
            schema_client,
        }
    }
}
