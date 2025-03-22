mod embedding;
mod graph;
mod openie;
mod retrieval;

use crate::openie::OpenIEProcessor;
use anyhow::Result;
use async_openai::config::OpenAIConfig;
use embedding::EmbeddingClient;
use graph::KnowledgeGraph;
use lazy_static::lazy_static;
use nihility_common::idea::{ChatIdea, Idea};
use nihility_common::inspiration::Inspiration;
use nihility_common::{register_inspiration_plugin, register_memory_idea_plugin};
use retrieval::HippoRAGRetriever;
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast::Sender;
use tokio::sync::{Mutex, broadcast, mpsc};
use tracing::{error, info};
use uuid::Uuid;

lazy_static! {
    static ref CORE: Mutex<Option<NihilitySimpleMemory>> = Mutex::new(None);
}

#[derive(Clone)]
pub struct NihilitySimpleMemory {
    id: Option<Uuid>,
    graph: KnowledgeGraph,
    openie: OpenIEProcessor,
    embed_client: EmbeddingClient,
    retriever: HippoRAGRetriever,
    memory_sender: Sender<Inspiration>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct NihilitySimpleMemoryConfig {
    pub embedding_model: String,
    pub openie_model: String,
    pub api_base_url: String,
    pub api_key: String,
}

impl NihilitySimpleMemory {
    pub async fn init(config: &NihilitySimpleMemoryConfig) -> Result<()> {
        info!("Initializing NihilitySimpleMemory");
        let openai_config = OpenAIConfig::new()
            .with_api_base(config.api_base_url.to_string())
            .with_api_key(config.api_key.to_string());

        let graph = KnowledgeGraph::new();
        let embed_client =
            EmbeddingClient::new(openai_config.clone(), config.embedding_model.to_string());
        let openie = OpenIEProcessor::new(openai_config, config.openie_model.to_string());
        let retriever = HippoRAGRetriever::new(0.85);

        let (tx, _) = broadcast::channel(10);
        let (core_rx, mut core_tx) = mpsc::channel::<ChatIdea>(10);
        let mut core = Self {
            id: None,
            graph,
            openie,
            embed_client,
            retriever,
            memory_sender: tx,
        };
        core.id
            .replace(register_inspiration_plugin(core.memory_sender.subscribe()).await?);
        CORE.lock().await.replace(core);

        tokio::spawn(async move {
            while let Some(idea) = core_tx.recv().await {
                let mut core = CORE.lock().await.clone().unwrap();
                match idea {
                    ChatIdea::Remember(text) => {
                        if let Err(e) = core.add_knowledge(text).await {
                            error!("Error adding knowledge: {}", e);
                        }
                    }
                    ChatIdea::Query(text) => {
                        let result = core.query_knowledge(text).await;
                        match result {
                            Ok(knowledge) => {
                                if let Err(e) =
                                    core.memory_sender.send(Inspiration::Memory(knowledge))
                                {
                                    error!("Failed to send memory inspiration: {:?}", e);
                                }
                            }
                            Err(e) => {
                                error!("Failed to query knowledge: {:?}", e);
                            }
                        }
                    }
                }
            }
        });

        tokio::spawn(async move {
            let mut idea_receiver = register_memory_idea_plugin().await;
            while let Ok(idea) = idea_receiver.recv().await {
                if let Idea::Memory(text) = idea {
                    if let Err(e) = core_rx.send(text).await {
                        error!("Failed to send memory query: {:?}", e);
                    }
                }
            }
        });

        Ok(())
    }

    pub async fn query_knowledge<T: Into<String>>(&self, text: T) -> Result<Vec<String>> {
        self.graph
            .query(text, &self.retriever, &self.openie, &self.embed_client)
            .await
    }

    pub async fn add_knowledge<T: Into<String>>(&mut self, text: T) -> Result<()> {
        self.graph
            .add_knowledge(text, &self.openie, &self.embed_client)
            .await?;
        Ok(())
    }
}
