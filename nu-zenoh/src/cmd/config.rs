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
use nu_protocol::{
    engine::{Call, Command, EngineState, Stack},
    PipelineData, ShellError, Signature, Type, Value,
};

use crate::{call_ext2::CallExt2, signature_ext::SignatureExt, State};

#[derive(Clone)]
pub(crate) struct Config {
    state: State,
}

impl Config {
    pub(crate) fn new(state: State) -> Self {
        Self { state }
    }
}

impl Command for Config {
    fn name(&self) -> &str {
        "zenoh config"
    }

    fn signature(&self) -> Signature {
        Signature::build(self.name())
            .session()
            .zenoh_category()
            .input_output_type(Type::Nothing, Type::String)
    }

    fn description(&self) -> &str {
        "Zenoh Configuration"
    }

    fn run(
        &self,
        engine_state: &EngineState,
        stack: &mut Stack,
        call: &Call,
        _input: PipelineData,
    ) -> Result<PipelineData, ShellError> {
        let config = self
            .state
            .with_session(&call.session(engine_state, stack)?, |sess| {
                sess.config().lock().to_string()
            })?;

        Ok(PipelineData::Value(Value::string(config, call.head), None))
    }
}
