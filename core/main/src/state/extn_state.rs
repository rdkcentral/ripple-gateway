use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use ripple_sdk::{
    api::{
        manifest::extn_manifest::{ExtnManifestEntry, ExtnSymbol},
        status_update::ExtnStatus,
    },
    crossbeam::channel::Sender as CSender,
    extn::{
        client::extn_sender::ExtnSender,
        extn_id::ExtnId,
        ffi::{ffi_channel::ExtnChannel, ffi_library::ExtnMetadata, ffi_message::CExtnMessage},
    },
    libloading::Library,
    tokio::{self, sync::mpsc},
    utils::error::RippleError,
};

use crate::service::extn::ripple_client::RippleClient;

use super::bootstrap_state::ChannelsState;

#[derive(Debug)]
pub struct LoadedLibrary {
    pub library: Library,
    metadata: Box<ExtnMetadata>,
    pub entry: ExtnManifestEntry,
}

impl LoadedLibrary {
    pub fn new(
        library: Library,
        metadata: Box<ExtnMetadata>,
        entry: ExtnManifestEntry,
    ) -> LoadedLibrary {
        LoadedLibrary {
            library,
            metadata,
            entry,
        }
    }

    pub fn get_channels(&self) -> Vec<ExtnSymbol> {
        let extn_ids: Vec<String> = self
            .metadata
            .symbols
            .iter()
            .filter(|x| x.id.is_channel())
            .map(|x| x.id.clone().to_string())
            .collect();
        self.entry
            .clone()
            .symbols
            .into_iter()
            .filter(|x| extn_ids.contains(&x.id))
            .collect()
    }

    pub fn get_extns(&mut self) -> Vec<ExtnId> {
        self.metadata
            .symbols
            .iter()
            .filter(|x| x.id.is_extn())
            .map(|x| x.id.clone())
            .collect()
    }

    pub fn get_symbols(&self) {}

    pub fn get_metadata(&self) -> Box<ExtnMetadata> {
        self.metadata.clone()
    }
}

#[derive(Debug)]
pub struct PreLoadedExtnChannel {
    pub channel: Box<ExtnChannel>,
    pub extn_id: ExtnId,
    pub symbol: ExtnSymbol,
}

/// Bootstrap state which is used to store transient extension information used while bootstrapping.
/// Content within state is related to extension symbols and Libraries.
#[derive(Debug, Clone)]
pub struct ExtnState {
    sender: CSender<CExtnMessage>,
    pub loaded_libraries: Arc<RwLock<Vec<LoadedLibrary>>>,
    pub device_channels: Arc<RwLock<Vec<PreLoadedExtnChannel>>>,
    pub deferred_channels: Arc<RwLock<Vec<PreLoadedExtnChannel>>>,
    pub launcher_channel: Arc<RwLock<Option<Box<ExtnChannel>>>>,
    extn_status_map: Arc<RwLock<HashMap<String, ExtnStatus>>>,
    extn_status_listeners: Arc<RwLock<HashMap<String, mpsc::Sender<ExtnStatus>>>>,
}

impl ExtnState {
    pub fn new(channels_state: ChannelsState) -> ExtnState {
        ExtnState {
            sender: channels_state.get_extn_sender(),
            loaded_libraries: Arc::new(RwLock::new(Vec::new())),
            device_channels: Arc::new(RwLock::new(Vec::new())),
            deferred_channels: Arc::new(RwLock::new(Vec::new())),
            launcher_channel: Arc::new(RwLock::new(None)),
            extn_status_map: Arc::new(RwLock::new(HashMap::new())),
            extn_status_listeners: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn update_extn_status(&self, id: ExtnId, status: ExtnStatus) {
        let mut extn_status_map = self.extn_status_map.write().unwrap();
        let _ = extn_status_map.insert(id.to_string(), status);
    }

    pub fn is_extn_ready(&self, extn_id: ExtnId) -> bool {
        if let Some(v) = self
            .extn_status_map
            .read()
            .unwrap()
            .get(extn_id.to_string().as_str())
        {
            match v {
                ExtnStatus::Ready => return true,
                _ => {}
            }
        }
        false
    }

    pub fn add_extn_status_listener(&self, id: ExtnId, sender: mpsc::Sender<ExtnStatus>) -> bool {
        {
            if self.is_extn_ready(id.clone()) {
                return true;
            }
        }
        let mut extn_status_listeners = self.extn_status_listeners.write().unwrap();
        let _ = extn_status_listeners.insert(id.to_string(), sender);
        false
    }

    pub fn get_extn_status_listener(&self, id: ExtnId) -> Option<mpsc::Sender<ExtnStatus>> {
        let extn_status_listeners = self.extn_status_listeners.read().unwrap();
        extn_status_listeners.get(id.to_string().as_str()).cloned()
    }

    pub fn clear_status_listener(&self, extn_id: ExtnId) {
        let mut extn_status_listeners = self.extn_status_listeners.write().unwrap();
        let _ = extn_status_listeners.remove(extn_id.to_string().as_str());
    }

    pub fn get_sender(self) -> CSender<CExtnMessage> {
        self.sender.clone()
    }

    pub fn start_channel(
        &mut self,
        channel: PreLoadedExtnChannel,
        client: RippleClient,
    ) -> Result<(), RippleError> {
        let sender = self.clone().get_sender();
        let symbol = channel.symbol.clone();
        let extn_id = channel.extn_id.clone();
        let extn_sender = ExtnSender::new(sender, extn_id.clone(), symbol.clone().uses);
        let (extn_tx, extn_rx) = ChannelsState::get_crossbeam_channel();
        let extn_channel = channel.channel;
        tokio::spawn(async move {
            (extn_channel.start)(extn_sender, extn_rx);
        });
        client.add_extn_sender(extn_id, symbol, extn_tx);
        return Ok(());
    }
}
