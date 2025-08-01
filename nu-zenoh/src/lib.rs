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
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use nu_protocol::{
    engine::{EngineState, StateWorkingSet},
    LabeledError,
};
use zenoh::{Config, Session, Wait};

mod call_ext2;
mod cmd;
mod conv;
mod interruptible_channel;
mod signature_ext;

/// Adds extra context (e.g. aliases) as Nu source code
///
/// This should be called after [`crate::add_zenoh_context`].
pub const ZENOH_CONTEXT_EXTRAS: &[u8] = include_bytes!("nu/extras.nu");

/// Adds all `zenoh *` commands to the given [`nu_protocol::engine::EngineState`].
pub fn add_zenoh_context(mut engine_state: EngineState) -> EngineState {
    let delta = {
        let mut working_set = StateWorkingSet::new(&engine_state);

        let state = State::new();

        working_set.add_decl(Box::new(cmd::put::Put::new(state.clone())));
        working_set.add_decl(Box::new(cmd::delete::Delete::new(state.clone())));
        working_set.add_decl(Box::new(cmd::get::Get::new(state.clone())));
        working_set.add_decl(Box::new(cmd::sub::Sub::new(state.clone())));
        working_set.add_decl(Box::new(cmd::zid::Zid::new(state.clone())));

        working_set.add_decl(Box::new(cmd::session::list::List::new(state.clone())));
        working_set.add_decl(Box::new(cmd::session::open::Open::new(state.clone())));
        working_set.add_decl(Box::new(cmd::session::close::Close::new(state.clone())));

        working_set.add_decl(Box::new(cmd::log_path::LogPath::new(state.clone())));
        working_set.add_decl(Box::new(cmd::reply::Reply::new(state.clone())));
        working_set.add_decl(Box::new(cmd::scout::Scout::new(state.clone())));
        working_set.add_decl(Box::new(cmd::info::Info::new(state.clone())));
        working_set.add_decl(Box::new(cmd::config::Config::new(state)));

        working_set.add_decl(Box::new(cmd::keyexpr::Includes));
        working_set.add_decl(Box::new(cmd::keyexpr::Intersects));

        working_set.render()
    };

    if let Err(err) = engine_state.merge_delta(delta) {
        eprintln!("Error creating Zenoh command context: {err:?}");
    }

    engine_state
}

#[derive(Clone)]
struct State {
    sessions: Arc<RwLock<HashMap<String, Session>>>,
}

impl State {
    const DEFAULT_SESSION_NAME: &str = "default";

    fn new() -> Self {
        let mut sessions = HashMap::new();
        let default_session = zenoh::open(Config::default())
            .wait()
            .expect("could not open default session");
        sessions.insert(Self::DEFAULT_SESSION_NAME.to_string(), default_session);

        Self {
            sessions: Arc::new(RwLock::new(sessions)),
        }
    }
}

impl State {
    pub(crate) fn with_session<F, T>(&self, name: &str, f: F) -> Result<T, LabeledError>
    where
        F: FnOnce(&Session) -> T,
    {
        let sessions = self.sessions.read().unwrap();
        let session = sessions
            .get(name)
            .ok_or_else(|| LabeledError::new(format!("session '{name}' not found")))?;
        Ok(f(session))
    }
}
