#![deny(missing_docs)]

//! The target is the thing that manages containers

use std::{fs, process::Command};

use anyhow::Result;

/// The container engine to use
#[allow(missing_docs)]
pub enum ContainerEngine {
    #[cfg(feature = "docker")]
    Docker,
    #[cfg(feature = "podman")]
    Podman,
}

#[cfg(feature = "docker")]
const DOCKER_SOCKET_PATH: &str = "/var/run/docker.sock";
#[cfg(feature = "podman")]
const PODMAN_BIN_PATH: &str = "/usr/bin/podman";

const CMD_NOT_FOUND_STATUS: i32 = 127;

/// Errors arising from container engine communication
#[derive(thiserror::Error, Debug)]
pub enum ContainerError {
    /// Something's gone wrong either while dialing the socket or while sending information to it
    #[cfg(feature = "docker")]
    #[error("failed to connect to socket")]
    SocketError(#[from] ureq::Error),

    /// The container engine is unreachable - for example, missing Podman command, etc.
    #[error("the container engine couldn't be reached")]
    EngineUnreachable,

    /// The resource that the caller requested could not be found
    #[error("requested resource was not found")]
    ResourceNotFound,

    /// Some other error
    #[error("unknown engine error")]
    Unknown,
}

impl ContainerEngine {
    /// Try to detect the container engine available on the target
    // TODO: handle multiple runtimes (I know, niche)
    pub fn try_detect() -> Option<Self> {
        // docker
        #[cfg(feature = "docker")]
        if fs::File::open(DOCKER_SOCKET_PATH).is_ok() {
            return Some(Self::Docker);
        }

        // podman
        #[cfg(feature = "podman")]
        {
            // TODO: suppress the output of this command

            let status = Command::new(PODMAN_BIN_PATH).status().ok()?.code();
            if let Some(code) = status
                && code != CMD_NOT_FOUND_STATUS
            {
                return Some(Self::Podman);
            }
        }

        None
    }

    /// Starts the resource specified by `resource_id`
    pub fn start_container(&self, resource_id: &str) -> Result<(), ContainerError> {
        match self {
            #[cfg(feature = "docker")]
            ContainerEngine::Docker => {
                use ureq::{Agent, http::StatusCode};

                let uri = format!("{DOCKER_SOCKET_PATH}/containers/{resource_id}/start");
                let res = Agent::new_with_defaults().post(uri).send(&[])?;

                // this match looks weird, but 404 and 500 are the only documented status codes
                match res.status() {
                    StatusCode::NOT_FOUND => return Err(ContainerError::ResourceNotFound),
                    StatusCode::INTERNAL_SERVER_ERROR => return Err(ContainerError::Unknown),

                    _ => return Err(ContainerError::Unknown),
                }
            }
            #[cfg(feature = "podman")]
            ContainerEngine::Podman => {
                use std::process::Stdio;

                let status = Command::new(PODMAN_BIN_PATH)
                    .args(["start", resource_id])
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .status()
                    .map_err(|_| ContainerError::EngineUnreachable)?;

                if !status.success() {
                    let status_code = status.code().ok_or(ContainerError::EngineUnreachable)?;

                    match status_code {
                        CMD_NOT_FOUND_STATUS => return Err(ContainerError::EngineUnreachable),

                        _ => return Err(ContainerError::Unknown),
                    }
                }
            }
        }

        Err(ContainerError::EngineUnreachable)
    }

    /// Stops the resource specified by `resource_id`
    pub fn stop_container(&self, resource_id: &str) -> Result<(), ContainerError> {
        match self {
            #[cfg(feature = "docker")]
            ContainerEngine::Docker => {
                use ureq::{Agent, http::StatusCode};

                let uri = format!("{DOCKER_SOCKET_PATH}/containers/{resource_id}/stop");
                let res = Agent::new_with_defaults().post(uri).send(&[])?;

                // this match looks weird, but 404 and 500 are the only documented status codes
                match res.status() {
                    StatusCode::NOT_FOUND => return Err(ContainerError::ResourceNotFound),
                    StatusCode::INTERNAL_SERVER_ERROR => return Err(ContainerError::Unknown),

                    _ => return Err(ContainerError::Unknown),
                }
            }
            #[cfg(feature = "podman")]
            ContainerEngine::Podman => {
                use std::process::Stdio;

                let status = Command::new(PODMAN_BIN_PATH)
                    .args(["stop", resource_id])
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .status()
                    .map_err(|_| ContainerError::EngineUnreachable)?;

                if !status.success() {
                    let status_code = status.code().ok_or(ContainerError::EngineUnreachable)?;

                    match status_code {
                        CMD_NOT_FOUND_STATUS => return Err(ContainerError::EngineUnreachable),

                        _ => return Err(ContainerError::Unknown),
                    }
                }
            }
        }

        Ok(())
    }
}
