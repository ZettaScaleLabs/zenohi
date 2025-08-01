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
use std::path::PathBuf;

use nu_engine::CallExt;
use nu_protocol::{
    engine::{Call, Command, EngineState, Stack},
    PipelineData, ShellError, Signature, SyntaxShape, Type, Value,
};
use zenoh::Wait;

use crate::{call_ext2::CallExt2, conv, signature_ext::SignatureExt, State};

#[derive(Clone)]
pub(crate) struct Open {
    state: State,
}

impl Open {
    pub(crate) fn new(state: State) -> Self {
        Self { state }
    }
}

impl Command for Open {
    fn name(&self) -> &str {
        "zenoh session open"
    }

    fn signature(&self) -> Signature {
        Signature::build(self.name())
            .session()
            .zenoh_category()
            .input_output_type(Type::Nothing, Type::Nothing)
            .named(
                "config-file",
                SyntaxShape::Filepath,
                "Path to a Zenoh configuration file",
                None,
            )
            .optional(
                "config",
                SyntaxShape::Record(vec![]),
                "Zenoh configuration object (see https://github.com/eclipse-zenoh/zenoh/blob/main/DEFAULT_CONFIG.json5)",
            )
    }

    fn description(&self) -> &str {
        "(re)Open a session"
    }

    fn run(
        &self,
        engine_state: &EngineState,
        stack: &mut Stack,
        call: &Call,
        _input: PipelineData,
    ) -> Result<PipelineData, ShellError> {
        let file_path = call.get_flag::<PathBuf>(engine_state, stack, "file")?;
        let config_record = call.opt::<Value>(engine_state, stack, 0)?;

        let config = match (file_path.as_ref(), config_record.as_ref()) {
            (Some(file_path), None) => zenoh::Config::from_file(file_path).map_err(|e| {
                nu_protocol::LabeledError::new("Failed to load config file").with_label(
                    format!("Could not read config from {}: {}", file_path.display(), e),
                    call.head,
                )
            })?,
            (None, Some(config_record)) => match config_record {
                val @ Value::Record { .. } => {
                    let json_value =
                        conv::value_to_json_value(engine_state, val, call.head, false)?;
                    zenoh::Config::from_json5(&json_value.to_string()).map_err(|e| {
                        nu_protocol::LabeledError::new("Failed to parse config record")
                            .with_label(format!("Could not parse config record: {e}"), call.head)
                    })?
                }
                _ => {
                    return Err(ShellError::GenericError {
                        error: "Invalid config type".to_string(),
                        msg: "Config must be a record".to_string(),
                        span: Some(call.head),
                        help: Some("Provide a record with Zenoh configuration options".to_string()),
                        inner: vec![],
                    });
                }
            },
            (Some(_), Some(_)) => {
                return Err(ShellError::GenericError {
                    error: "Conflicting arguments".to_string(),
                    msg: "Cannot specify both --file and config record".to_string(),
                    span: Some(call.head),
                    help: Some(
                        "Use either --file <path> or provide a config record, not both".to_string(),
                    ),
                    inner: vec![],
                });
            }
            (None, None) => zenoh::Config::default(),
        };

        let session_name = call.session(engine_state, stack)?;
        let mut sessions = self.state.sessions.write().unwrap();
        if let Some(sess) = sessions.remove(&session_name) {
            sess.close().wait().map_err(|e| {
                nu_protocol::LabeledError::new("Failed to reopen Zenoh session '{session_name}'")
                    .with_label(format!("Could not close Zenoh session: {e}"), call.head)
            })?
        }
        let new_session = zenoh::open(config).wait().map_err(|e| {
            nu_protocol::LabeledError::new("Failed to open Zenoh session")
                .with_label(format!("Could not establish Zenoh session: {e}"), call.head)
        })?;
        sessions.insert(session_name, new_session);

        Ok(PipelineData::Value(Value::nothing(call.head), None))
    }
}
