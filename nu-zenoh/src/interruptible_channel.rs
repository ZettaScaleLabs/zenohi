//
// Copyright (c) 2025 ZettaScale Technology
//
// This program and the accompanying materials are made available under the
// terms of the Eclipse Public License 2.0 which is available at
// http://www.eclipse.org/legal/epl-2.0, or the Apache License, Version 2.0
// which is available at https://www.apache.org/licenses/LICENSE-2.0.
//
// SPDX-License-Identifier: EPL-2.0 OR Apache-2.0
//
// Contributors:
//   ZettaScale Zenoh Team, <zenoh@zettascale.tech>
//
use std::time::Duration;

use nu_protocol::Signals;

// TODO(fuzzypixelz): interupt on session drop?
pub(crate) struct InterruptibleChannel<T, D = ()> {
    receiver: flume::Receiver<T>,
    signals: Signals,
    _data: D,
}

impl<T> InterruptibleChannel<T> {
    pub(crate) fn new(receiver: flume::Receiver<T>, signals: Signals) -> InterruptibleChannel<T> {
        InterruptibleChannel {
            receiver,
            signals,
            _data: (),
        }
    }
}

impl<T, D> InterruptibleChannel<T, D> {
    pub(crate) fn with_data(
        receiver: flume::Receiver<T>,
        signals: Signals,
        data: D,
    ) -> InterruptibleChannel<T, D> {
        InterruptibleChannel {
            receiver,
            signals,
            _data: data,
        }
    }
}

impl<T, D> InterruptibleChannel<T, D> {
    // REVIEW(fuzzypixelz): is this a sane value?
    const TIMEOUT: Duration = Duration::from_millis(50);
}

impl<T, D> Iterator for InterruptibleChannel<T, D> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.signals.interrupted() {
                return None;
            } else {
                match self.receiver.recv_timeout(Self::TIMEOUT) {
                    Ok(item) => return Some(item),
                    Err(flume::RecvTimeoutError::Timeout) => continue,
                    Err(flume::RecvTimeoutError::Disconnected) => return None,
                }
            }
        }
    }
}
