// Copyright 2020 Adam Greig
// Dual licensed under the Apache 2.0 and MIT licenses.

use stm32ral::usb;
use stm32ral::{read_reg, write_reg};

use super::{USBStackRequest, Endpoint, stat_disabled, stat_stall, stat_nak, stat_valid};
use super::buffers::*;

/// USB handling code for DAP endpoint
pub(super) struct DAPEndpoint {
    epbuf: &'static mut EPBuf,
    btable: &'static mut BTableRow,
}

impl DAPEndpoint {
    fn process_tx_complete(&self, _usb: &usb::Instance) -> Option<USBStackRequest> {
        None
    }

    fn process_rx_complete(&self, _usb: &usb::Instance) -> Option<USBStackRequest> {
        None
    }

    /// Indicate a packet has been loaded into the buffer and is ready for transmission
    fn tx_valid(&self, usb: &usb::Instance) {
        let (stat_tx, ep_type, ea) = read_reg!(usb, usb, EP2R, STAT_TX, EP_TYPE, EA);
        write_reg!(usb, usb, EP2R, CTR_RX: 1, EP_TYPE: ep_type, CTR_TX: 1, EA: ea,
                   STAT_TX: stat_valid(stat_tx));
    }
}

impl Endpoint for DAPEndpoint {
    fn new(epbuf: &'static mut EPBuf, btable: &'static mut BTableRow) -> Self {
        DAPEndpoint { epbuf, btable }
    }

    fn write_btable(&mut self) {
        self.btable.write(&self.epbuf);
    }

    fn reset_endpoint(&self, usb: &usb::Instance) {
        let (stat_tx, stat_rx) = read_reg!(usb, usb, EP2R, STAT_TX, STAT_RX);
        write_reg!(usb, usb, EP2R,
                   STAT_TX: stat_disabled(stat_tx), STAT_RX: stat_disabled(stat_rx));
    }

    fn configure_endpoint(&self, usb: &usb::Instance) {
        // Set up EP2R to be a didirectional interrupt endpoint,
        // with STAT_TX to NAK=10 and STAT_RX to Stall=11,
        // and DTOG_TX and DTOG_RX both set to 0.
        let (stat_tx, stat_rx, dtog_rx, dtog_tx) =
            read_reg!(usb, usb, EP2R, STAT_TX, STAT_RX, DTOG_RX, DTOG_TX);
        write_reg!(usb, usb, EP2R,
                   CTR_RX: 1, EP_TYPE: Interrupt, EP_KIND: 0, CTR_TX: 1, EA: 2,
                   DTOG_RX: dtog_rx, DTOG_TX: dtog_tx,
                   STAT_TX: stat_nak(stat_tx), STAT_RX: stat_stall(stat_rx));

    }

    fn process_transfer(&mut self, usb: &usb::Instance) -> Option<USBStackRequest> {
        let mut req = None;
        let (ctr_tx, ctr_rx, ep_type, ea) =
            read_reg!(usb, usb, EP2R, CTR_TX, CTR_RX, EP_TYPE, EA);
        if ctr_tx == 1 {
            req = self.process_tx_complete(usb);
            // Clear CTR_TX
            write_reg!(usb, usb, EP2R,
                       CTR_RX: 1, EP_TYPE: ep_type, CTR_TX: 0, EA: ea);
        }
        if ctr_rx == 1 {
            req = self.process_rx_complete(usb);
            // Clear CTR_RX
            write_reg!(usb, usb, EP2R,
                       CTR_RX: 0, EP_TYPE: ep_type, CTR_TX: 1, EA: ea);
        }
        req
    }

    fn transmit_slice(&mut self, usb: &usb::Instance, data: &[u8]) {
        assert!(data.len() <= 64);
        self.epbuf.write_tx(data);
        self.btable.tx_count(data.len());
        self.tx_valid(usb);
    }

    /// Resume reception of new HID requests
    fn rx_valid(&mut self, usb: &usb::Instance) {
        let (stat_rx, ep_type, ea) = read_reg!(usb, usb, EP2R, STAT_RX, EP_TYPE, EA);
        write_reg!(usb, usb, EP2R, CTR_RX: 1, EP_TYPE: ep_type, CTR_TX: 1, EA: ea,
                   STAT_RX: stat_valid(stat_rx));
    }

    /// Cancel reception of new HID requests
    fn rx_stall(&mut self, usb: &usb::Instance) {
        let (stat_rx, ep_type, ea) = read_reg!(usb, usb, EP2R, STAT_RX, EP_TYPE, EA);
        write_reg!(usb, usb, EP2R, CTR_RX: 1, EP_TYPE: ep_type, CTR_TX: 1, EA: ea,
                   STAT_RX: stat_stall(stat_rx));
    }
}