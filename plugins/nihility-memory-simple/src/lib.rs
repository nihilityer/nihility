mod graph;
mod retrieval;

use anyhow::Result;
use graph::KnowledgeGraph;
use lazy_static::lazy_static;
use nihility_common::idea::{ MemoryIdea};
use nihility_common::inspiration::Inspiration;
use nihility_common::{register_inspiration_plugin, register_memory_idea_plugin};
use retrieval::HippoRAGRetriever;
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
    retriever: HippoRAGRetriever,
    memory_sender: Sender<Inspiration>,
}

impl NihilitySimpleMemory {
    pub async fn init() -> Result<()> {
        info!("Initializing NihilitySimpleMemory");

        let graph = KnowledgeGraph::new();
        let retriever = HippoRAGRetriever::new(0.85);

        let (tx, _) = broadcast::channel(10);
        let (core_rx, mut core_tx) = mpsc::channel::<MemoryIdea>(10);
        let mut core = Self {
            id: None,
            graph,
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
                    MemoryIdea::Remember(text) => {
                        if let Err(e) = core.add_knowledge(text).await {
                            error!("Error adding knowledge: {}", e);
                        }
                    }
                    MemoryIdea::Query(text) => {
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
                if let Err(e) = core_rx.send(idea).await {
                    error!("Failed to send memory query: {:?}", e);
                }
            }
        });

        Ok(())
    }

    pub async fn query_knowledge<T: Into<String>>(&self, text: T) -> Result<Vec<String>> {
        self.graph.query(text, &self.retriever).await
    }

    pub async fn add_knowledge<T: Into<String>>(&mut self, text: T) -> Result<()> {
        self.graph.add_knowledge(text).await?;
        Ok(())
    }
}
