use crate::{
    bytes::ToBytes,
    config::Config,
    l2cap::{L2CAPState, L2CAPStateTx},
    link::{
        data::{Llid, Pdu},
        llcp::{self, ConnectionParamRequest, ControlPdu},
        queue::{Consume, Consumer, Producer},
        Connection,
    },
    utils::HexSlice,
    Error,
};

/// Data channel packet processor.
///
/// This hooks up to the Real-Time part of the LE Link Layer via a packet queue. This part can run
/// at a lower priority (eg. being driven in the apps idle loop) and receives and transmits packets
/// using the packet queue.
///
/// Some *LL Control PDUs* sent as part of the Link Layer Control Protocol (LLCP) are answered by
/// the responder directly, and all L2CAP data is forwarded to an `L2CAPState<M>`. Note that most
/// LLCPDUs are handled directly by the real-time code.
pub struct Responder<C: Config> {
    tx: C::PacketProducer,
    rx: Option<C::PacketConsumer>,
    l2cap: L2CAPState<C::ChannelMapper>,
}

impl<C: Config> Responder<C> {
    /// Creates a new packet processor hooked up to data channel packet queues.
    pub fn new(
        tx: C::PacketProducer,
        rx: C::PacketConsumer,
        l2cap: L2CAPState<C::ChannelMapper>,
    ) -> Self {
        Self {
            tx,
            rx: Some(rx),
            l2cap,
        }
    }

    /// Returns `true` when this responder has work to do.
    ///
    /// If this returns `true`, `process` may be called to process incoming packets and send
    /// outgoing ones.
    pub fn has_work(&mut self) -> bool {
        self.with_rx(|rx, _| rx.has_data())
    }

    /// Processes a single incoming packet in the packet queue.
    ///
    /// Returns `Error::Eof` if there are no incoming packets in the RX queue.
    pub fn process_one(&mut self) -> Result<(), Error> {
        self.with_rx(|rx, this| {
            rx.consume_pdu_with(|_, pdu| match pdu {
                Pdu::Control { data } => {
                    // Also see:
                    // https://github.com/jonas-schievink/rubble/issues/26

                    let pdu = data.read();
                    info!("<- LL Control PDU: {:?}", pdu);
                    let response = match pdu {
                        // These PDUs are handled by the real-time code:
                        ControlPdu::FeatureReq { .. } | ControlPdu::VersionInd { .. } => {
                            unreachable!("LLCPDU not handled by LL");
                        }
                        _ => ControlPdu::UnknownRsp {
                            unknown_type: pdu.opcode(),
                        },
                    };
                    info!("-> Response: {:?}", response);

                    // Consume the LL Control PDU iff we can fit the response in the TX buffer:
                    Consume::on_success(this.tx.produce_with(
                        response.encoded_size().into(),
                        |writer| {
                            response.to_bytes(writer)?;
                            Ok(Llid::Control)
                        },
                    ))
                }
                Pdu::DataStart { message } => {
                    info!("L2start: {:?}", HexSlice(message));
                    this.l2cap().process_start(message)
                }
                Pdu::DataCont { message } => {
                    info!("L2cont {:?}", HexSlice(message));
                    this.l2cap().process_cont(message)
                }
            })
        })
    }

    /// Obtains access to the L2CAP instance.
    pub fn l2cap(&mut self) -> L2CAPStateTx<'_, C::ChannelMapper, C::PacketProducer> {
        self.l2cap.tx(&mut self.tx)
    }

    /// Provides raw access to the Link Layer Control Protocol (LLCP).
    ///
    /// If the link layer has already initiated an LLCP procedure and is waiting for the response,
    /// an error will be returned.
    pub fn llcp<'a>(&'a mut self, conn: &'a mut Connection<C>) -> Result<LLCPTx<'a, C>, Error> {
        if conn.llcp_initiated {
            Err(Error::InvalidState)
        } else {
            Ok(LLCPTx {
                producer: &mut self.tx,
                link: conn,
            })
        }
    }

    /// A helper method that splits `self` into the `rx` and the remaining `Self`.
    ///
    /// This can possibly be removed after *RFC 2229 (Closures Capture Disjoint Fields)* is
    /// implemented in stable Rust.
    fn with_rx<R>(&mut self, f: impl FnOnce(&mut C::PacketConsumer, &mut Self) -> R) -> R {
        let mut rx = self.rx.take().unwrap();
        let result = f(&mut rx, self);
        self.rx = Some(rx);
        result
    }
}

pub struct LLCPTx<'a, C: Config> {
    producer: &'a mut C::PacketProducer,
    link: &'a mut Connection<C>,
}

impl<'a, C: Config> LLCPTx<'a, C> {
    /// Returns a reference to the link layer connection state.
    pub fn connection(&self) -> &Connection<C> {
        self.link
    }

    /// Start a *Connection Parameters Request Procedure*, requesting a change in connection
    /// parameters.
    pub fn request_conn_params(self, params: ConnectionParamRequest) -> Result<(), Error> {
        self.link.llcp_initiated = true;

        let cpdu = llcp::ControlPdu::ConnectionParamReq(params);
        self.producer.produce_with(cpdu.encoded_size(), |writer| {
            cpdu.to_bytes(writer)?;
            Ok(Llid::Control)
        })?;
        Ok(())
    }
}
