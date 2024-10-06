use iced::futures::{
    channel::mpsc::{self, unbounded, UnboundedReceiver, UnboundedSender},
    Sink, SinkExt, Stream, StreamExt,
};
use std::{
    ops::{Deref, DerefMut},
    sync::{atomic::AtomicU64, Arc},
};
use thiserror::Error;
use tracing::info;

type EndpointId = u64;
type InletId = u64;

static NEXT_ENDPOINT_ID: AtomicU64 = AtomicU64::new(0);

#[derive(Error, Debug)]
pub enum ConnectorError {}

pub struct EndpointMessage<Message> {
    from: InletId,
    msg: Message,
}

impl<Message> EndpointMessage<Message> {
    pub fn from(&self) -> &InletId {
        &self.from
    }
    pub fn into_inner(self) -> Message {
        self.msg
    }
}

#[derive(Debug)]
pub struct Inlet<M> {
    id: InletId,
    next_id: Arc<AtomicU64>,
    endpoint_id: EndpointId,
    tx: UnboundedSender<EndpointMessage<M>>,
}

impl<M> Clone for Inlet<M> {
    fn clone(&self) -> Self {
        let inlet = Inlet {
            id: self
                .next_id
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed),
            next_id: self.next_id.clone(),
            endpoint_id: self.endpoint_id,
            tx: self.tx.clone(),
        };

        tracing::info!(
            "Inlet {} created for endpoint {}",
            inlet.id,
            inlet.endpoint_id
        );
        inlet
    }
}

impl<M> Sink<M> for Inlet<M> {
    type Error = mpsc::SendError;

    fn poll_ready(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.tx.poll_ready(cx)
    }

    fn start_send(mut self: std::pin::Pin<&mut Self>, item: M) -> Result<(), Self::Error> {
        let id = self.id;
        tracing::info!("STARTING SEND INLET ID {id}");
        self.tx.start_send(EndpointMessage {
            from: id,
            msg: item,
        })
    }

    fn poll_flush(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.tx.poll_flush_unpin(cx)
    }

    fn poll_close(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.tx.poll_close_unpin(cx)
    }
}

/*
impl<M> Deref for Inlet<M> {
    type Target = UnboundedSender<EndpointMessage<M>>;

    fn deref(&self) -> &Self::Target {
        &self.tx
    }
}

impl<M> DerefMut for Inlet<M> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.tx
    }
}
*/

#[derive(Debug)]
pub(crate) struct Outlet<M> {
    rx: UnboundedReceiver<EndpointMessage<M>>,
}

impl<M> Stream for Outlet<M> {
    type Item = EndpointMessage<M>;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        self.rx.poll_next_unpin(cx)
        /*
        self.rx.poll_next_unpin(cx).map(|m| match m {
            Some(msg) => {
                tracing::error!("Received message from {}", msg.from);
                Some(msg.msg)
            }
            None => None,
        })
        */
    }
}

/*
impl<M> Outlet<M> {
    pub fn into_stream(self) -> UnboundedReceiver<EndpointMessage<M>> {
        self.rx
    }
}
*/

impl<M> Deref for Outlet<M> {
    type Target = UnboundedReceiver<EndpointMessage<M>>;

    fn deref(&self) -> &Self::Target {
        &self.rx
    }
}

impl<M> DerefMut for Outlet<M> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.rx
    }
}

#[derive(Debug)]
pub struct Endpoint<M> {
    id: EndpointId,
    outlet: Option<Outlet<M>>,
    inlet: Inlet<M>,
    //plug: Plug<M>,
}

impl<M> Endpoint<M> {
    pub fn new() -> Self {
        let id = NEXT_ENDPOINT_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let (tx, rx) = unbounded();
        let outlet = Some(Outlet { rx });

        let inlet = Inlet {
            id: 0,
            next_id: Arc::new(AtomicU64::new(1)),
            tx,
            endpoint_id: id,
        };
        info!("Endpoint ID={} Created", id);
        Self { id, outlet, inlet }
    }

    pub fn take_outlet(&mut self) -> Outlet<M> {
        self.outlet.take().expect("Outlet already taken")
    }

    pub fn get_inlet(&self) -> Inlet<M> {
        self.inlet.clone()
    }

    pub fn id(&self) -> EndpointId {
        self.id
    }
}
